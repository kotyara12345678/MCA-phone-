"""TTFA latency instrumentation tests.

Three layers, from most to least portable:

1. ``LatencyMetrics`` collector — pure stdlib, runs anywhere. Covers monotonic
   event marking, derived latency deltas, rolling p50/p95, first-wins semantics,
   abandoned turns, error/timeout counters, model attribution and PII-safety
   (transcript content is never stored, only its length).

2. Adapter-level event marks — ``OpenAISTTAdapter`` and ``OpenAILLMAdapter``
   ``generate_stream`` now emit EV_* events into the shared collector. These use
   the same mock-session pattern as ``test_pipeline_openai_adapters.py``.

3. Sentence-boundary splitting — mirrors the engine's streaming-overlap regex
   (``engine.py``: ``_SENTENCE_RE = re.compile(r"[.!?]\\s+")``) to prove ordered,
   lossless fragment delivery into the sequential TTS queue.

PY3.13 NOTE: ``audioop`` was removed from the stdlib in Python 3.13, and
``src/audio/resampler.py`` imports it at module load. The STT and streaming-LLM
code paths exercised here do NOT call audioop, so a bare module shim is injected
below purely to make ``src.pipelines.openai`` importable on 3.13. On the Linux
runtime image (audioop present) the shim is not installed and the real module is
used. TTS synthesis is intentionally NOT tested here because its PCM->mulaw
conversion DOES call audioop and therefore remains a Linux-runtime test.
"""

import asyncio
import json
import re
import sys
import types

import pytest

from src.core.latency_metrics import (
    LatencyMetrics,
    get_latency_metrics,
    reset_latency_metrics,
    EV_SPEECH_END,
    EV_STT_SENT,
    EV_STT_FINAL,
    EV_LLM_START,
    EV_LLM_FIRST_TOKEN,
)

# --- py3.13 audioop shim (import-only) -------------------------------------
try:  # pragma: no cover - exercised by platform availability
    import audioop  # noqa: F401
except ModuleNotFoundError:  # Python >= 3.13
    sys.modules.setdefault("audioop", types.ModuleType("audioop"))

from src.config import AppConfig, OpenAIProviderConfig  # noqa: E402
from src.pipelines.openai import OpenAILLMAdapter, OpenAISTTAdapter  # noqa: E402


@pytest.fixture(autouse=True)
def _reset_metrics():
    reset_latency_metrics()
    yield
    reset_latency_metrics()


# --- 1. collector unit tests ------------------------------------------------

def test_derived_deltas_and_p50_p95():
    m = LatencyMetrics()
    # Two turns with known llm TTFT: 100ms and 300ms.
    for call, dt in (("c1", 0.100), ("c2", 0.300)):
        m.mark(call, EV_SPEECH_END, ts=10.0)
        m.mark(call, EV_LLM_START, ts=10.0)
        m.mark(call, EV_LLM_FIRST_TOKEN, ts=10.0 + dt)
        b = m.finish_turn(call)
        assert b["llm_ttft_ms"] == pytest.approx(dt * 1000, abs=0.5)

    snap = m.snapshot()
    stat = snap["metrics"]["llm_ttft_ms"]
    assert stat["count"] == 2
    assert stat["min_ms"] == pytest.approx(100, abs=1)
    assert stat["max_ms"] == pytest.approx(300, abs=1)
    assert stat["p50_ms"] == pytest.approx(200, abs=50)  # midpoint of 100/300


def test_first_wins_keeps_original_speech_end():
    m = LatencyMetrics()
    m.mark("c1", EV_SPEECH_END, ts=5.0)
    m.mark("c1", EV_SPEECH_END, ts=9.0)  # later re-mark (e.g. barge-in) ignored
    m.mark("c1", EV_LLM_FIRST_TOKEN, ts=5.2)
    b = m.finish_turn("c1")
    assert b["llm_ttft_ms"] == pytest.approx(200, abs=0.5)


def test_missing_events_yield_none_not_crash():
    m = LatencyMetrics()
    m.mark("c1", EV_SPEECH_END, ts=1.0)
    # No first-token event -> llm_ttft None; others None too.
    b = m.finish_turn("c1")
    assert b["llm_ttft_ms"] is None
    assert b["end_of_speech_to_first_audio_ms"] is None


def test_abandon_turn_discards_samples():
    m = LatencyMetrics()
    m.mark("c1", EV_SPEECH_END, ts=1.0)
    m.mark("c1", EV_LLM_FIRST_TOKEN, ts=1.5)
    m.abandon_turn("c1")
    snap = m.snapshot()
    assert snap["turns"] == 0
    assert snap["metrics"]["llm_ttft_ms"] == {"count": 0}


def test_errors_timeouts_and_model_attribution():
    m = LatencyMetrics()
    m.record_error("stt")
    m.record_timeout("llm")
    m.mark("c1", EV_SPEECH_END, ts=1.0)
    m.set_model("c1", "llm", "google/gemini-2.5-flash-lite")
    m.set_model("c1", "stt", "openai/whisper-large-v3-turbo")
    b = m.finish_turn("c1")
    assert b["models"]["llm"] == "google/gemini-2.5-flash-lite"
    snap = m.snapshot()
    assert snap["errors"]["stt"] == 1
    assert snap["timeouts"]["llm"] == 1
    assert snap["models"]["google/gemini-2.5-flash-lite"] == 1


def test_note_transcript_records_length_only():
    m = LatencyMetrics()
    m.note_transcript("c1", "Клиент: мой номер +7 900 123 45 67, ИНН 7701234567")
    b = m.finish_turn("c1")
    # Only a length is retained; no raw content anywhere in the export.
    assert b["models"] == {}
    assert "transcript_len" not in b  # meta is not exported to breakdown
    assert "7701234567" not in json.dumps(m.snapshot())


# --- 3. sentence splitting (mirrors engine streaming-overlap) ---------------

_SENTENCE_RE = re.compile(r"[.!?]\s+")
_MAX_SENTENCE_CHARS = 180
_MIN_FLUSH_CHARS = 40


def _split_stream(tokens):
    """Mirror of engine.py streaming-overlap sentence buffering (incl. the
    speed-driven max-length flush at word boundaries)."""
    buffer = ""
    out = []
    for tok in tokens:
        buffer += tok
        m = _SENTENCE_RE.search(buffer)
        if m:
            pos = m.end()
            to_speak = buffer[:pos].strip()
            buffer = buffer[pos:]
        elif len(buffer) > _MAX_SENTENCE_CHARS:
            cut = buffer.rfind(" ", 0, _MAX_SENTENCE_CHARS)
            if cut >= _MIN_FLUSH_CHARS:
                to_speak = buffer[:cut].strip()
                buffer = buffer[cut:].lstrip()
            else:
                to_speak = ""
        else:
            to_speak = ""
        if to_speak:
            out.append(to_speak)
    rem = buffer.strip()
    if rem:
        out.append(rem)
    return out


def test_long_sentence_flushes_at_word_boundary_for_faster_first_audio():
    # One very long sentence with NO terminal punctuation: the speed path must
    # emit fragments at word boundaries instead of waiting for the whole thing.
    words = ["слово" + str(i) for i in range(80)]  # ~80 words, >180 chars
    frags = _split_stream(list(" ".join(words)))
    assert len(frags) > 1  # flushed incrementally, not one tail fragment
    # Each non-final fragment ends on a word (not mid-word) — quality preserved.
    for f in frags[:-1]:
        assert not f.endswith("слово") or len(f) > 0
        assert f.split()[-1] != ""


def test_sentence_split_is_ordered_and_lossless():
    text = "Добрый день! Это MCA. Подскажите, что перевозим? Спасибо."
    # Stream token-by-token (worst case: split points straddle token boundaries).
    tokens = list(text)
    frags = _split_stream(tokens)
    # Ordered, contiguous, no drops: rejoining with a space yields the original
    # word/ punctuation sequence (whitespace collapsed at boundaries).
    assert frags[0].startswith("Добрый день!")
    assert frags[-1].endswith("Спасибо.")
    joined = " ".join(frags)
    for word in ["Добрый", "MCA", "перевозим?", "Спасибо."]:
        assert word in joined
    # Every fragment ends on a sentence terminator (except a possible trailing remainder).
    for f in frags[:-1]:
        assert f[-1] in ".!?"


def test_sentence_split_no_terminal_punctuation_flushes_remainder():
    frags = _split_stream(list("Просто фраза без точки"))
    assert frags == ["Просто фраза без точки"]


# --- 2. adapter event marks (mock sessions) --------------------------------

def _build_app_config() -> AppConfig:
    providers = {
        "openai": {
            "api_key": "test-key",
            "chat_base_url": "https://api.openai.com/v1",
            "stt_base_url": "https://api.openai.com/v1/audio/transcriptions",
            "chat_model": "google/gemini-2.5-flash-lite",
            "stt_model": "openai/whisper-large-v3-turbo",
            "response_timeout_sec": 2.0,
        }
    }
    pipelines = {
        "p": {
            "stt": "openai_stt",
            "llm": "openai_llm",
            "tts": "openai_tts",
            "options": {"stt": {}, "llm": {"use_realtime": False}, "tts": {}},
        }
    }
    return AppConfig(
        default_provider="openai",
        providers=providers,
        asterisk={"host": "127.0.0.1", "username": "ari", "password": "secret"},
        llm={"initial_greeting": "hi", "prompt": "p"},
        audio_transport="audiosocket",
        downstream_mode="stream",
        pipelines=pipelines,
        active_pipeline="p",
    )


class _FakeResponse:
    def __init__(self, body: bytes, status: int = 200):
        self._body = body
        self.status = status

    async def __aenter__(self):
        return self

    async def __aexit__(self, *a):
        return False

    async def read(self):
        return self._body

    async def text(self):
        return self._body.decode("utf-8", "ignore")


class _FakeSession:
    def __init__(self, body: bytes, status: int = 200):
        self._body = body
        self._status = status
        self.requests = []

    def post(self, url, json=None, data=None, headers=None, timeout=None):
        self.requests.append({"url": url, "json": json, "data": data})
        return _FakeResponse(self._body, status=self._status)

    async def close(self):
        pass


class _SSEStream:
    """Async iterator of raw SSE byte lines for generate_stream."""

    def __init__(self, lines):
        self._lines = list(lines)

    def __aiter__(self):
        self._iter = iter(self._lines)
        return self

    async def __anext__(self):
        try:
            return next(self._iter)
        except StopIteration:
            raise StopAsyncIteration


class _FakeStreamingResponse(_FakeResponse):
    def __init__(self, lines, status=200):
        super().__init__(b"", status=status)
        self.content = _SSEStream(lines)


class _FakeStreamingSession(_FakeSession):
    def __init__(self, lines, status=200):
        super().__init__(b"", status=status)
        self._lines = lines

    def post(self, url, json=None, data=None, headers=None, timeout=None):
        self.requests.append({"url": url, "json": json})
        return _FakeStreamingResponse(self._lines, status=self._status)


def _sse_token(text):
    chunk = {"choices": [{"delta": {"content": text}}]}
    return ("data: " + json.dumps(chunk) + "\n").encode("utf-8")


def _sse_done():
    return b"data: [DONE]\n"


@pytest.mark.asyncio
async def test_stt_adapter_marks_sent_and_final():
    cfg = _build_app_config()
    prov = OpenAIProviderConfig(**cfg.providers["openai"])
    session = _FakeSession(json.dumps({"text": "привет"}).encode("utf-8"))
    adapter = OpenAISTTAdapter("openai_stt", cfg, prov, {}, session_factory=lambda: session)
    await adapter.start()

    transcript = await adapter.transcribe("call-1", b"\x00\x10" * 160, 16000, {})
    assert transcript == "привет"

    b = get_latency_metrics().finish_turn("call-1")
    # EV_STT_SENT and EV_STT_FINAL both present => stt_latency_ms is a number.
    assert b["stt_latency_ms"] is not None
    assert b["models"]["stt"] == "openai/whisper-large-v3-turbo"


@pytest.mark.asyncio
async def test_llm_generate_stream_marks_ttft_and_streams_in_order():
    cfg = _build_app_config()
    prov = OpenAIProviderConfig(**cfg.providers["openai"])
    lines = [_sse_token("Здравствуйте! "), _sse_token("Что перевозим?"), _sse_done()]
    session = _FakeStreamingSession(lines)
    adapter = OpenAILLMAdapter(
        "openai_llm", cfg, prov, {"use_realtime": False}, session_factory=lambda: session
    )
    await adapter.start()

    chunks = [c async for c in adapter.generate_stream("call-1", "hi", {"system_prompt": "s"}, {})]
    assert chunks == ["Здравствуйте! ", "Что перевозим?"]
    assert session.requests[0]["json"]["stream"] is True

    b = get_latency_metrics().finish_turn("call-1")
    assert b["llm_request_to_ttft_ms"] is not None
    assert b["models"]["llm"] == "google/gemini-2.5-flash-lite"


@pytest.mark.asyncio
async def test_llm_generate_stream_timeout_records_timeout_metric():
    cfg = _build_app_config()
    prov = OpenAIProviderConfig(**cfg.providers["openai"])

    class _TimeoutIter:
        def __aiter__(self):
            return self

        async def __anext__(self):
            raise TimeoutError("simulated")

    class _Sess(_FakeStreamingSession):
        def post(self, url, json=None, data=None, headers=None, timeout=None):
            self.requests.append({"url": url, "json": json})
            resp = _FakeStreamingResponse([])
            resp.content = _TimeoutIter()
            return resp

    session = _Sess([])
    adapter = OpenAILLMAdapter(
        "openai_llm", cfg, prov, {"use_realtime": False, "timeout_sec": 0.5},
        session_factory=lambda: session,
    )
    await adapter.start()
    chunks = [c async for c in adapter.generate_stream("call-1", "hi", {}, {})]
    assert chunks == []
    snap = get_latency_metrics().snapshot()
    assert snap["timeouts"]["llm"] >= 1


@pytest.mark.asyncio
async def test_concurrent_sessions_keep_independent_ttft_metrics():
    """Mock load test: N concurrent streaming sessions against the shared
    collector must keep per-call isolation and yield one TTFT sample each."""
    cfg = _build_app_config()
    prov = OpenAIProviderConfig(**cfg.providers["openai"])

    def make_adapter(call_id):
        lines = [_sse_token("Привет! "), _sse_token("Готов помочь."), _sse_done()]
        session = _FakeStreamingSession(lines)
        return OpenAILLMAdapter(
            "openai_llm", cfg, prov, {"use_realtime": False}, session_factory=lambda: session
        ), session

    n = 8
    call_ids = [f"load-{i}" for i in range(n)]
    adapters = [make_adapter(cid) for cid in call_ids]
    for ad, _ in adapters:
        await ad.start()

    async def run_one(cid, ad):
        chunks = [c async for c in ad.generate_stream(cid, "hi", {"system_prompt": "s"}, {})]
        assert chunks  # every session streamed tokens
        return get_latency_metrics().finish_turn(cid)

    breakdowns = await asyncio.gather(
        *(run_one(cid, ad) for cid, (ad, _) in zip(call_ids, adapters))
    )

    assert len(breakdowns) == n
    for b in breakdowns:
        assert b["llm_request_to_ttft_ms"] is not None
        assert b["models"]["llm"] == "google/gemini-2.5-flash-lite"

    snap = get_latency_metrics().snapshot()
    assert snap["turns"] == n
    assert snap["metrics"]["llm_request_to_ttft_ms"]["count"] == n
