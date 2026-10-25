"""Turn resilience tests: the agent must always answer.

Covers the "sometimes the agent stays silent after a big request" failure and
its latency-related fixes:

1. **Guaranteed response.** When the LLM stream fails and the serial retry
   also fails (provider error, timeout, malformed reply), the engine speaks a
   short fallback acknowledgement instead of returning silently. Same for a
   response that produced neither text nor tool calls.
2. **Partial stream is kept.** If the stream dies *after* some tokens were
   already synthesized, the spoken part is kept as the turn's reply and the
   serial path must NOT replay the whole answer (duplicate audio).
3. **History trimming.** ``_trim_history_for_llm`` bounds the prompt sent each
   turn (the "big request" slowdown) and never splits an assistant-tool_calls
   / ``tool`` pair.

Deterministic mock providers only — no real Polza calls here.

PY3.13 NOTE: same import-only ``audioop`` shim as test_pipeline_filler_overlap.py.
"""

import asyncio
import sys
import types
from types import SimpleNamespace
from unittest.mock import AsyncMock

import pytest

try:  # pragma: no cover - exercised by platform availability
    import audioop  # noqa: F401
except ModuleNotFoundError:  # Python >= 3.13
    sys.modules.setdefault("audioop", types.ModuleType("audioop"))

from src.config import AppConfig  # noqa: E402
from src.core.models import CallSession  # noqa: E402
from src.engine import Engine, _trim_history_for_llm  # noqa: E402
from src.pipelines.base import LLMComponent, LLMResponse, STTComponent, TTSComponent  # noqa: E402
from src.tools.registry import ToolRegistry  # noqa: E402

CALL_ID = "call-turn-fallback"
FALLBACK_TEXT = "Извините, я вас не расслышала. Повторите, пожалуйста, ваш вопрос."


# --- stubs -----------------------------------------------------------------


class _ResultStreamingStubSTT(STTComponent):
    supports_streaming = True

    def __init__(self):
        self.started = asyncio.Event()
        self.results = asyncio.Queue()

    async def open_call(self, call_id, options):
        pass

    async def transcribe(self, call_id, audio_pcm16, sample_rate_hz, options):
        raise AssertionError("streaming adapter must not use buffered transcription")

    async def start_stream(self, call_id, options, *, sample_rate_hz, fmt):
        self.started.set()

    async def send_audio(self, call_id, audio, *, fmt="pcm16_16k"):
        pass

    async def iter_results(self, call_id):
        while True:
            value = await self.results.get()
            if value is None:
                return
            yield value

    async def stop_stream(self, call_id):
        self.results.put_nowait(None)


class _SlotStubPlayback:
    """One active stream per call, mirroring StreamingPlaybackManager."""

    def __init__(self, events):
        self.events = events
        self.active = {}
        self.captured = {}
        self._n = 0

    def _drain(self, stream_id, queue):
        captured = self.captured.setdefault(stream_id, [])
        while True:
            try:
                item = queue.get_nowait()
            except asyncio.QueueEmpty:
                return captured
            captured.append(item)
            if item is None:
                return captured

    async def start_streaming_playback(self, call_id, queue, **kwargs):
        playback_type = kwargs.get("playback_type")
        existing = self.active.get(call_id)
        if existing:
            if existing["queue"] is queue:
                return existing["stream_id"]
            self._drain(existing["stream_id"], existing["queue"])
            self.events.append(("stop", existing["playback_type"], False))
            del self.active[call_id]
        self._n += 1
        stream_id = f"stream-{self._n}"
        self.active[call_id] = {
            "stream_id": stream_id,
            "playback_type": playback_type,
            "queue": queue,
        }
        self.events.append(("start", playback_type))
        return stream_id

    async def stop_streaming_playback(self, call_id, *, drain=False, **kwargs):
        info = self.active.get(call_id)
        if not info:
            return False
        self._drain(info["stream_id"], info["queue"])
        self.events.append(("stop", info["playback_type"], bool(drain)))
        del self.active[call_id]
        return True

    def is_stream_active(self, call_id, stream_id=None):
        info = self.active.get(call_id)
        if not info:
            return False
        return stream_id is None or info["stream_id"] == stream_id


class _RecordingTTS(TTSComponent):
    def __init__(self, events):
        self.events = events

    async def synthesize(self, call_id, text, options):
        self.events.append(("synth", text))
        yield b"A1"
        yield b"A2"


class _ExplodingLLM(LLMComponent):
    """Stream fails and the serial retry fails too — the historic silent turn."""

    supports_streaming = True

    def __init__(self, events):
        self.events = events
        self.generate_calls = 0
        self._pending_tool_calls_by_call = {}

    async def generate(self, call_id, transcript, context, options):
        self.generate_calls += 1
        raise RuntimeError("gateway 500")

    async def generate_stream(self, call_id, transcript, context, options):
        self.events.append(("llm_stream_start", transcript))
        raise RuntimeError("gateway 500 mid-stream")


class _EmptyLLM(LLMComponent):
    """Stream yields nothing and the serial path returns an empty reply."""

    supports_streaming = True

    def __init__(self, events):
        self.events = events
        self.generate_calls = 0
        self._pending_tool_calls_by_call = {}

    async def generate(self, call_id, transcript, context, options):
        self.generate_calls += 1
        return LLMResponse(text="", tool_calls=[])

    async def generate_stream(self, call_id, transcript, context, options):
        self.events.append(("llm_stream_start", transcript))
        if False:
            yield ""


class _PartialThenExplodingLLM(LLMComponent):
    """Dies after the first sentence already reached the caller."""

    supports_streaming = True

    def __init__(self, events):
        self.events = events
        self.generate_calls = 0
        self._pending_tool_calls_by_call = {}

    async def generate(self, call_id, transcript, context, options):
        self.generate_calls += 1
        return LLMResponse(text="полный ответ из serial-пути", tool_calls=[])

    async def generate_stream(self, call_id, transcript, context, options):
        self.events.append(("llm_stream_start", transcript))
        yield "Первая фраза. "
        yield "Вторая фраза."
        raise ConnectionError("stream reset")


class _Resolution:
    def __init__(self, stt, llm, tts, llm_options=None):
        self.pipeline_name = "stub"
        self.stt_key = "stub_stt"
        self.stt_adapter = stt
        self.llm_adapter = llm
        self.tts_adapter = tts
        self.stt_options = {"streaming": True, "chunk_ms": 80}
        self.llm_options = llm_options or {}
        self.tts_options = {}
        self.prepared = True

    def component_summary(self):
        return {"stt": "stub", "llm": "stub", "tts": "stub"}


def _base_config():
    return {
        "default_provider": "local",
        "providers": {"local": {"enabled": True}},
        "asterisk": {
            "host": "127.0.0.1",
            "port": 8088,
            "username": "u",
            "password": "p",
            "app_name": "ai-voice-agent",
        },
        "llm": {"initial_greeting": "", "prompt": "You are helpful", "model": "gpt-4o"},
        "pipelines": {"speed": {}},
        "active_pipeline": "speed",
        "audio_transport": "externalmedia",
        "downstream_mode": "stream",
    }


async def _new_engine(monkeypatch, events):
    engine = Engine(AppConfig(**_base_config()))
    engine.pipeline_orchestrator._started = True
    monkeypatch.setattr(engine, "streaming_playback_manager", _SlotStubPlayback(events))
    engine.ari_client.set_channel_var = AsyncMock(return_value=True)
    monkeypatch.setattr(
        engine.transport_orchestrator,
        "get_context_config",
        lambda *a, **k: SimpleNamespace(
            prompt=None,
            greeting=None,
            tools=[],
            in_call_http_tools={},
            disable_global_in_call_tools=[],
        ),
    )
    return engine


async def _prepare_session(engine):
    session = CallSession(call_id=CALL_ID, caller_channel_id=CALL_ID)
    session.pipeline_name = "speed"
    session.context_name = "speed-context"
    session.tool_runtime_registry = ToolRegistry.isolated()
    await engine.session_store.upsert_call(session)
    return session


async def _run_turn(engine, stt, resolution, transcript="это тестовая фраза"):
    engine.pipeline_orchestrator.get_pipeline = lambda *a, **k: resolution
    await engine._ensure_pipeline_runner(
        await engine.session_store.get_by_call_id(CALL_ID), forced=True
    )
    await asyncio.wait_for(stt.started.wait(), timeout=2)
    await stt.results.put(transcript)


async def _wait_for(events, predicate, timeout=3.0):
    deadline = asyncio.get_running_loop().time() + timeout
    while asyncio.get_running_loop().time() < deadline:
        if predicate(events):
            return True
        await asyncio.sleep(0.01)
    return predicate(events)


# --- 1. guaranteed response ------------------------------------------------


@pytest.mark.asyncio
async def test_llm_double_failure_speaks_fallback(monkeypatch):
    """Stream error + serial error must end in spoken audio, never silence."""
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _ExplodingLLM(events)
    resolution = _Resolution(stt, llm, _RecordingTTS(events))
    await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_for(
        events,
        lambda ev: any(e[0] == "synth" and e[1] == FALLBACK_TEXT for e in ev),
    ), f"fallback never synthesized: {events}"

    await engine._cleanup_call(CALL_ID)


@pytest.mark.asyncio
async def test_empty_llm_response_speaks_fallback(monkeypatch):
    """No text and no tools -> the caller still hears something."""
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _EmptyLLM(events)
    resolution = _Resolution(stt, llm, _RecordingTTS(events))
    await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_for(
        events,
        lambda ev: any(e[0] == "synth" and e[1] == FALLBACK_TEXT for e in ev),
    ), f"fallback never synthesized: {events}"
    # The serial path was actually attempted before falling back.
    assert llm.generate_calls == 1

    await engine._cleanup_call(CALL_ID)


@pytest.mark.asyncio
async def test_fallback_message_is_configurable(monkeypatch):
    """llm option turn_fallback_message overrides the default phrase."""
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _ExplodingLLM(events)
    resolution = _Resolution(
        stt,
        llm,
        _RecordingTTS(events),
        llm_options={"turn_fallback_message": "Один момент, повторю вопрос."},
    )
    await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_for(
        events,
        lambda ev: any(
            e[0] == "synth" and e[1] == "Один момент, повторю вопрос." for e in ev
        ),
    ), f"custom fallback never synthesized: {events}"
    assert not any(e[0] == "synth" and e[1] == FALLBACK_TEXT for e in events), events

    await engine._cleanup_call(CALL_ID)


# --- 2. partial stream is kept ---------------------------------------------


@pytest.mark.asyncio
async def test_partial_stream_keeps_reply_without_serial_replay(monkeypatch):
    """Audio already played once must never be spoken a second time."""
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _PartialThenExplodingLLM(events)
    resolution = _Resolution(stt, llm, _RecordingTTS(events))
    session = await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_for(
        events, lambda ev: ("synth", "Первая фраза.") in ev
    ), events
    # Let the turn settle: history persistence + EOS sentinel.
    await asyncio.sleep(0.2)

    # Serial path never replayed the answer and no fallback apology followed.
    assert llm.generate_calls == 0, "serial path must not replay a partial answer"
    assert not any(e[0] == "synth" and "Извините" in e[1] for e in events), events

    # The partial text is the turn's reply in the persisted transcript.
    stored = await engine.session_store.get_by_call_id(CALL_ID)
    assistant = [m for m in stored.conversation_history if m.get("role") == "assistant"]
    assert assistant, stored.conversation_history
    assert "Первая фраза." in assistant[-1]["content"]

    # Response audio was delivered exactly once onto the playback stream,
    # terminated by the EOS sentinel the failure path queued.
    info = engine.streaming_playback_manager.active.get(CALL_ID)
    assert info is not None, engine.streaming_playback_manager.active
    delivered = engine.streaming_playback_manager._drain(
        info["stream_id"], info["queue"]
    )
    assert delivered, "no response audio reached the playback stream"
    assert delivered[-1] is None, delivered
    assert all(c and c.startswith(b"A") for c in delivered[:-1]), delivered

    await engine._cleanup_call(CALL_ID)


# --- 3. history trimming ---------------------------------------------------


def test_trim_history_keeps_tail_and_stops_on_user():
    messages = [
        {"role": "user", "content": "u0"},
        {"role": "assistant", "content": "a0"},
        {"role": "user", "content": "u1"},
        {"role": "assistant", "content": "a1"},
        {"role": "user", "content": "u2"},
        {"role": "assistant", "content": "a2"},
        {"role": "user", "content": "last"},
    ]
    trimmed = _trim_history_for_llm(messages, 3)
    assert len(trimmed) == 3
    # The cut lands on a user message so no tool pair can be orphaned.
    assert trimmed[0]["role"] == "user"
    assert trimmed[0]["content"] == "u2"
    assert trimmed[-1]["content"] == "last"


def test_trim_history_never_splits_tool_sequence():
    messages = [
        {"role": "user", "content": "первый вопрос"},
        {"role": "assistant", "content": "ответ", "tool_calls": [{"id": "t1"}]},
        {"role": "tool", "content": "результат", "tool_call_id": "t1"},
        {"role": "user", "content": "второй вопрос"},
        {"role": "assistant", "content": "второй ответ"},
        {"role": "user", "content": "третий вопрос"},
    ]
    trimmed = _trim_history_for_llm(messages, 3)
    # A blind tail cut would start at role=tool (rejected by strict gateways);
    # the helper advances to the next user message instead.
    assert trimmed[0]["role"] == "user", trimmed
    assert trimmed[0]["content"] == "второй вопрос"
    assert all(m["role"] != "tool" for m in trimmed), trimmed


def test_trim_history_disabled_or_small_history():
    messages = [{"role": "user", "content": "hi"}] * 3
    assert _trim_history_for_llm(messages, None) is messages
    assert _trim_history_for_llm(messages, 0) is messages
    assert _trim_history_for_llm(messages, 10) is messages
    assert _trim_history_for_llm([], 5) == []
    assert _trim_history_for_llm(messages, "не число") is messages
