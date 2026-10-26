"""Speak-first turn tests: one fast LLM round for speech, tools in parallel.

Pins the fix for the "LLM takes ~7s" regression measured on the live Polza
gateway (bench/bench_prod_shape.py, 2026-10-26): with the five mca_* tool
schemas attached, the conversational model answers with a bare ``mca_save_facts``
tool call and NO text in 6/6 runs, so every turn paid two LLM round-trips
(p50 4.3s for the whole tool loop) before the first spoken word.

With llm option ``speak_first`` the engine now:

1. strips ``tools`` from the CRITICAL streaming round, so the model always
   answers with text and TTS starts after a single round-trip;
2. issues the tool-enabled round in PARALLEL (same transcript, non-streaming)
   and executes its tool calls once the reply has been streamed;
3. cancels the parallel round when the stream produced nothing, leaving the
   serial path the sole owner of the tools (no double execution);
4. keeps the spoken reply if the parallel round fails — a CRM write must never
   cost the caller dead air.

Without the flag the previous behavior is preserved: tools stay on the
streaming request.

PY3.13 NOTE: same import-only ``audioop`` shim as the other pipeline tests.
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
from src.engine import Engine  # noqa: E402
from src.pipelines.base import LLMComponent, LLMResponse, STTComponent, TTSComponent  # noqa: E402
from src.tools.base import Tool, ToolCategory, ToolDefinition  # noqa: E402
from src.tools.registry import ToolRegistry  # noqa: E402

CALL_ID = "call-speak-first"
TOKENS = ["Здравствуйте! ", "Чем помочь?"]
TOOL_CALL = {"id": "turn-tool-1", "name": "transcript_tool", "parameters": {}}


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
    """One active stream per call, mirroring StreamingPlaybackManager's contract."""

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


class _TextTTS(TTSComponent):
    def __init__(self, events):
        self.events = events

    async def synthesize(self, call_id, text, options):
        self.events.append(("response_synth", text))
        yield b"R1"
        yield b"R2"


class _SpeakFirstLLM(LLMComponent):
    """Records the options each round received and its start order."""

    supports_streaming = True

    def __init__(self, events, tokens=None, tool_calls=None, tool_round_fails=False,
                 tool_round_delay=0.05, ttft_sec=0.15):
        self.events = events
        self._tokens = TOKENS if tokens is None else tokens
        self._tool_calls = [dict(tc) for tc in (tool_calls or [TOOL_CALL])]
        self._tool_round_fails = tool_round_fails
        self._tool_round_delay = tool_round_delay
        self.ttft_sec = ttft_sec
        self._pending_tool_calls_by_call = {}
        self.stream_options_seen = []
        self.generate_options_seen = []
        self.generate_calls = 0
        self.cancelled = False

    async def generate(self, call_id, transcript, context, options):
        self.generate_calls += 1
        self.generate_options_seen.append(dict(options or {}))
        self.events.append(("tool_round_start", transcript))
        try:
            await asyncio.sleep(self._tool_round_delay)
        except asyncio.CancelledError:
            self.cancelled = True
            raise
        if self._tool_round_fails:
            raise RuntimeError("polza gateway unavailable")
        # One-shot: a real model stops calling tools once they were executed and
        # the result is in context, which keeps the continuation round clean.
        calls = self._tool_calls
        self._tool_calls = []
        return LLMResponse(text="", tool_calls=[dict(tc) for tc in calls])

    async def generate_stream(self, call_id, transcript, context, options):
        self.stream_options_seen.append(dict(options or {}))
        self.events.append(("llm_stream_start", transcript))
        if self.ttft_sec:
            await asyncio.sleep(self.ttft_sec)
        self.events.append(("llm_first_token", None))
        for token in self._tokens:
            yield token
        self.events.append(("llm_stream_end", None))


class _TranscriptTool(Tool):
    def __init__(self):
        self.executed = asyncio.Event()
        self.runs = 0

    @property
    def definition(self):
        return ToolDefinition(
            name="transcript_tool",
            description="Test tool",
            category=ToolCategory.BUSINESS,
        )

    async def execute(self, parameters, context):
        self.runs += 1
        self.executed.set()
        return {"status": "success", "message": "Tool completed"}


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


def _event_index(events, pred):
    for idx, event in enumerate(events):
        if pred(event):
            return idx
    return -1


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
            tools=["transcript_tool"],
            in_call_http_tools={},
            disable_global_in_call_tools=[],
        ),
    )
    return engine


async def _prepare_session(engine):
    session = CallSession(call_id=CALL_ID, caller_channel_id=CALL_ID)
    session.pipeline_name = "speed"
    session.context_name = "speed-context"
    session.allowed_tools = ["transcript_tool"]
    registry = ToolRegistry.isolated()
    tool = _TranscriptTool()
    registry.register_instance(tool)
    session.tool_runtime_registry = registry
    await engine.session_store.upsert_call(session)
    return session, tool


async def _run_turn(engine, stt, resolution, transcript="это тест фраза"):
    engine.pipeline_orchestrator.get_pipeline = lambda *a, **k: resolution
    await engine._ensure_pipeline_runner(
        await engine.session_store.get_by_call_id(CALL_ID), forced=True
    )
    await asyncio.wait_for(stt.started.wait(), timeout=2)
    await stt.results.put(transcript)


def _spoken_sentences(events):
    return [e[1] for e in events if e[0] == "response_synth"]


# --- 1. tools stripped from the critical round ------------------------------


@pytest.mark.asyncio
async def test_speak_first_runs_tool_round_in_parallel(monkeypatch):
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _SpeakFirstLLM(events)
    resolution = _Resolution(stt, llm, _TextTTS(events), llm_options={
        "tools": ["transcript_tool"],
        "speak_first": True,
        "skip_continuation_when_spoken": True,
        "max_history_messages": 16,
    })
    session, tool = await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    await asyncio.wait_for(tool.executed.wait(), timeout=3)
    await asyncio.sleep(0.05)

    # The spoken round carried NO tool schemas...
    assert llm.stream_options_seen, "streaming round never ran"
    assert "tools" not in llm.stream_options_seen[0]
    # ...while the parallel round did.
    assert llm.generate_options_seen, "parallel tool round never ran"
    assert llm.generate_options_seen[0].get("tools") == ["transcript_tool"]

    # Parallel: the tool round started before the first streamed token landed.
    start_tool = _event_index(events, lambda e: e[0] == "tool_round_start")
    first_token = _event_index(events, lambda e: e[0] == "llm_first_token")
    assert start_tool >= 0 and first_token >= 0, events
    assert start_tool < first_token, events

    # Exactly one non-streaming call: the parallel round. No serial fallback and
    # no tool continuation round (reply already spoken).
    assert llm.generate_calls == 1

    # The reply reached the caller, in order, and ended with the EOS sentinel.
    info = engine.streaming_playback_manager.active.get(CALL_ID)
    delivered = []
    for sid, payload in engine.streaming_playback_manager.captured.items():
        if b"R1" in payload:
            delivered = payload
    if not delivered and info is not None:
        delivered = engine.streaming_playback_manager._drain(
            info["stream_id"], info["queue"]
        )
    assert delivered == [b"R1", b"R2", b"R1", b"R2", None], delivered
    assert "".join(_spoken_sentences(events)) == "".join(t.strip() for t in TOKENS)

    # Tool executed exactly once, and its result is in the call record.
    assert tool.runs == 1
    stored = await engine.session_store.get_by_call_id(CALL_ID)
    assert [m["role"] for m in stored.conversation_history] == ["user", "assistant"]

    await engine._cleanup_call(CALL_ID)


@pytest.mark.asyncio
async def test_without_flag_tools_stay_on_the_streaming_round(monkeypatch):
    """Default behavior preserved: no speak_first -> tools stay in the request."""
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _SpeakFirstLLM(events)
    resolution = _Resolution(stt, llm, _TextTTS(events), llm_options={
        "tools": ["transcript_tool"],
        "max_history_messages": 16,
    })
    await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_until(lambda: bool(_spoken_sentences(events))), events
    assert llm.stream_options_seen
    assert llm.stream_options_seen[0].get("tools") == ["transcript_tool"]
    # No parallel round was scheduled.
    assert llm.generate_calls == 0
    assert not any(e[0] == "tool_round_start" for e in events), events

    await engine._cleanup_call(CALL_ID)


# --- 2. tool round failure must not cost the reply --------------------------


@pytest.mark.asyncio
async def test_speak_first_model_applies_to_spoken_round_only(monkeypatch):
    """The fast model owns the spoken round; the tool round keeps its own model."""
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _SpeakFirstLLM(events)
    resolution = _Resolution(stt, llm, _TextTTS(events), llm_options={
        "tools": ["transcript_tool"],
        "speak_first": True,
        "speak_first_model": "openai/gpt-4.1-mini",
        "skip_continuation_when_spoken": True,
        "max_history_messages": 16,
    })
    session, tool = await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    await asyncio.wait_for(tool.executed.wait(), timeout=3)

    stream_opts = llm.stream_options_seen[0]
    tool_opts = llm.generate_options_seen[0]
    assert stream_opts.get("chat_model") == "openai/gpt-4.1-mini"
    assert "tools" not in stream_opts
    # The tool round is untouched: it still gets the tool schemas and no model
    # override, so it stays on the provider's own (reliable) model.
    assert tool_opts.get("tools") == ["transcript_tool"]
    assert "chat_model" not in tool_opts

    await engine._cleanup_call(CALL_ID)


@pytest.mark.asyncio
async def test_tool_round_failure_keeps_spoken_reply(monkeypatch):
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _SpeakFirstLLM(events, tool_round_fails=True)
    resolution = _Resolution(stt, llm, _TextTTS(events), llm_options={
        "tools": ["transcript_tool"],
        "speak_first": True,
        "skip_continuation_when_spoken": True,
        "max_history_messages": 16,
    })
    session, tool = await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_until(lambda: bool(_spoken_sentences(events))), events
    await asyncio.sleep(0.2)
    # Reply was spoken in full despite the gateway error on the tool round.
    assert "".join(_spoken_sentences(events)) == "".join(t.strip() for t in TOKENS)
    assert not tool.executed.is_set()
    assert llm.generate_calls == 1  # the failed round, no retry loop

    await engine._cleanup_call(CALL_ID)


# --- 3. no text from the stream: tools must not run twice --------------------


@pytest.mark.asyncio
async def test_no_text_stream_cancels_parallel_round_and_uses_serial_path(monkeypatch):
    events = []
    engine = await _new_engine(monkeypatch, events)
    stt = _ResultStreamingStubSTT()
    llm = _SpeakFirstLLM(events, tokens=[], tool_round_delay=0.3)
    resolution = _Resolution(stt, llm, _TextTTS(events), llm_options={
        "tools": ["transcript_tool"],
        "speak_first": True,
        "skip_continuation_when_spoken": True,
        "max_history_messages": 16,
    })
    session, tool = await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    await asyncio.wait_for(tool.executed.wait(), timeout=3)
    await asyncio.sleep(0.5)
    # The parallel round was cancelled (it was still sleeping) and the serial
    # path executed the tool exactly once. The follow-up round after the tool
    # returns no text, so nothing is spoken twice.
    assert tool.runs == 1, tool.runs
    assert llm.cancelled, "parallel tool round must be cancelled when the stream is empty"
    assert not _spoken_sentences(events), events

    await engine._cleanup_call(CALL_ID)


async def _wait_until(predicate, timeout=3.0):
    deadline = asyncio.get_running_loop().time() + timeout
    while asyncio.get_running_loop().time() < deadline:
        if predicate():
            return True
        await asyncio.sleep(0.01)
    return predicate()
