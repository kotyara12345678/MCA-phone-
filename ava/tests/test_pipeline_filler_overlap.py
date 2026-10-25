"""Filler / tool-continuation latency tests for the pipeline turn path.

Three speed changes in ``Engine.run_turn`` are pinned here:

1. **Parallel filler.** The acknowledgement phrase used to be synthesized and
   *fully played back* before the LLM request even started, so every turn paid
   the filler duration on top of the LLM time-to-first-token. The filler now
   runs as a background task while the LLM stream starts immediately, and the
   playback slot is released (phrase cut) at the first real audio chunk.

2. **Lazy playback stream.** The real TTS stream is opened at the first
   synthesized chunk, not before the LLM call, because the call owns a single
   streaming slot that the filler still holds.

3. **skip_continuation_when_spoken.** When a turn already streamed a spoken
   reply, the follow-up LLM round that fact-save tools used to trigger is
   skipped (it cost ~1.5-2s plus a redundant utterance). Without the flag the
   continuation must still run, so the business flow is not silently removed.

The playback manager is stubbed to model the real one-slot-per-call contract:
``start_streaming_playback`` returns the live stream while its producer is the
same queue, a new producer (new queue) replaces the finished stream, and
``is_stream_active`` gates every queue put.

PY3.13 NOTE: same import-only ``audioop`` shim as test_latency_metrics_ttfa.py.
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

CALL_ID = "call-filler-overlap"
FILLER_TEXT = "Угу."


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
                # Same producer: the real manager hands back the live stream.
                # This is why the filler must be stopped before the response
                # stream opens, otherwise its audio would go unheard.
                return existing["stream_id"]
            # A different queue means a new producer: the previous stream has
            # already finished in the real manager, so settle it and replace.
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


class _FillerAwareTTS(TTSComponent):
    def __init__(self, events):
        self.events = events

    async def synthesize(self, call_id, text, options):
        if text.strip() == FILLER_TEXT:
            self.events.append(("filler_synth", text))
            yield b"F1"
            await asyncio.sleep(0.4)  # natural playback time of the phrase
            yield b"F2"
            return
        self.events.append(("response_synth", text))
        yield b"R1"
        yield b"R2"


class _StreamingTextLLM(LLMComponent):
    supports_streaming = True

    def __init__(self, events, ttft_sec=0.15, tokens=None, pending_tools=None):
        self.events = events
        self.ttft_sec = ttft_sec
        self._tokens = tokens if tokens is not None else ["Готово. ", "Еще фраза."]
        self._pending_tools = pending_tools or []
        self._pending_tool_calls_by_call = {}
        self.generate_calls = 0

    async def generate(self, call_id, transcript, context, options):
        self.generate_calls += 1
        if not self._tokens:
            # Model a turn where the model produced nothing at all.
            return LLMResponse(text="", tool_calls=[])
        return LLMResponse(text="non-streaming fallback", tool_calls=[])

    async def generate_stream(self, call_id, transcript, context, options):
        self.events.append(("llm_stream_start", transcript))
        if self.ttft_sec:
            await asyncio.sleep(self.ttft_sec)
        self.events.append(("llm_first_token", None))
        if self._pending_tools:
            self._pending_tool_calls_by_call[call_id] = list(self._pending_tools)
        for token in self._tokens:
            yield token


class _TranscriptTool(Tool):
    def __init__(self):
        self.executed = asyncio.Event()

    @property
    def definition(self):
        return ToolDefinition(
            name="transcript_tool",
            description="Test tool",
            category=ToolCategory.BUSINESS,
        )

    async def execute(self, parameters, context):
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


def _base_config(streaming=None):
    data = {
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
    if streaming is not None:
        data["streaming"] = streaming
    return data


async def _new_engine(monkeypatch, events, *, filler=False, tools=False):
    streaming_cfg = (
        {"pipeline_filler_enabled": True, "pipeline_filler_phrases": [FILLER_TEXT]}
        if filler
        else None
    )
    engine = Engine(AppConfig(**_base_config(streaming_cfg)))
    engine.pipeline_orchestrator._started = True
    monkeypatch.setattr(engine, "streaming_playback_manager", _SlotStubPlayback(events))
    engine.ari_client.set_channel_var = AsyncMock(return_value=True)
    if tools:
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


async def _prepare_session(engine, *, tools=False):
    session = CallSession(call_id=CALL_ID, caller_channel_id=CALL_ID)
    session.pipeline_name = "speed"
    session.context_name = "speed-context"
    if tools:
        session.allowed_tools = ["transcript_tool"]
        registry = ToolRegistry.isolated()
        registry.register_instance(_TranscriptTool())
        session.tool_runtime_registry = registry
    await engine.session_store.upsert_call(session)
    return session


async def _run_turn(engine, stt, resolution, transcript="это тест фраза"):
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


async def _wait_until(predicate, timeout=3.0):
    deadline = asyncio.get_running_loop().time() + timeout
    while asyncio.get_running_loop().time() < deadline:
        if predicate():
            return True
        await asyncio.sleep(0.01)
    return predicate()


async def _stored_session(engine, predicate, timeout=3.0):
    """Poll the session store until ``predicate`` holds on the fresh session."""
    deadline = asyncio.get_running_loop().time() + timeout
    while True:
        session = await engine.session_store.get_by_call_id(CALL_ID)
        if session is not None and predicate(session):
            return session
        if asyncio.get_running_loop().time() >= deadline:
            pytest.fail(f"condition not met within {timeout}s: {session}")
        await asyncio.sleep(0.01)


# --- 1. parallel filler -----------------------------------------------------


@pytest.mark.asyncio
async def test_llm_stream_starts_while_filler_is_playing(monkeypatch):
    """The LLM request must overlap the filler, not wait for its playback."""
    events = []
    engine = await _new_engine(monkeypatch, events, filler=True)
    stt = _ResultStreamingStubSTT()
    llm = _StreamingTextLLM(events, ttft_sec=0.15)
    resolution = _Resolution(stt, llm, _FillerAwareTTS(events))
    await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_for(
        events, lambda ev: ("response_synth", "Еще фраза.") in ev
    ), events
    # Let the final sentence flush and the EOS sentinel land.
    await asyncio.sleep(0.05)

    start_llm = _event_index(events, lambda e: e[0] == "llm_stream_start")
    stop_filler = _event_index(
        events, lambda e: e[0] == "stop" and e[1] == "pipeline-tts-filler"
    )
    start_response = _event_index(
        events, lambda e: e[0] == "start" and e[1] == "pipeline-tts"
    )

    assert start_llm >= 0, "LLM stream never started"
    assert stop_filler >= 0, "filler stream was never stopped"
    assert start_response >= 0, "response stream never opened"
    # Core assertion: the LLM request went out before the phrase finished.
    assert start_llm < stop_filler, events
    # The phrase is cut (no drain) so the response can take the slot at once.
    assert events[stop_filler][2] is False, events[stop_filler]
    # The slot is released before the response stream opens.
    assert stop_filler < start_response, events

    # Response audio reached the response stream in order, ending with EOS,
    # and no filler audio leaked into the response stream.
    info = engine.streaming_playback_manager.active.get(CALL_ID)
    assert info is None or info["playback_type"] == "pipeline-tts", info
    delivered = []
    for sid, payload in engine.streaming_playback_manager.captured.items():
        if b"R1" in payload:
            delivered = payload
    if not delivered and info is not None:
        delivered = engine.streaming_playback_manager._drain(
            info["stream_id"], info["queue"]
        )
    assert delivered == [b"R1", b"R2", b"R1", b"R2", None], delivered
    assert not any(c and c.startswith(b"F") for c in delivered), delivered

    await engine._cleanup_call(CALL_ID)


@pytest.mark.asyncio
async def test_filler_drains_when_turn_produces_no_audio(monkeypatch):
    """No response audio -> the phrase finishes naturally and frees the slot,
    then the guaranteed-response fallback speaks an acknowledgement instead of
    leaving the caller in dead air."""
    events = []
    engine = await _new_engine(monkeypatch, events, filler=True)
    stt = _ResultStreamingStubSTT()
    llm = _StreamingTextLLM(events, tokens=[])  # no tokens -> no model audio
    resolution = _Resolution(stt, llm, _FillerAwareTTS(events))
    await _prepare_session(engine)
    await _run_turn(engine, stt, resolution)

    assert await _wait_for(
        events, lambda ev: any(e[0] == "stop" and e[1] == "pipeline-tts-filler" for e in ev)
    ), events
    await asyncio.sleep(0.05)

    filler_stops = [e for e in events if e[0] == "stop" and e[1] == "pipeline-tts-filler"]
    assert len(filler_stops) == 1, events
    assert filler_stops[0][2] is True, filler_stops
    # The slot is freed by the drained phrase BEFORE the fallback opens the
    # response stream — the caller hears the phrase, then the acknowledgement.
    assert await _wait_for(
        events,
        lambda ev: any(
            e[0] == "response_synth" and "Извините" in e[1] for e in ev
        ),
    ), events
    start_response = _event_index(
        events, lambda e: e[0] == "start" and e[1] == "pipeline-tts"
    )
    assert start_response > events.index(filler_stops[0]), events

    await engine._cleanup_call(CALL_ID)


# --- 2/3. tool continuation round ------------------------------------------


def _tool_llm(events):
    return _StreamingTextLLM(
        events,
        ttft_sec=0.0,
        tokens=["Данные приняты. ", "Спасибо."],
        pending_tools=[
            {"id": "turn-tool-1", "name": "transcript_tool", "parameters": {}}
        ],
    )


async def _run_tool_turn(monkeypatch, llm_options):
    events = []
    engine = await _new_engine(monkeypatch, events, tools=True)
    stt = _ResultStreamingStubSTT()
    llm = _tool_llm(events)
    resolution = _Resolution(stt, llm, _FillerAwareTTS(events), llm_options=llm_options)
    session = await _prepare_session(engine, tools=True)
    await _run_turn(engine, stt, resolution, transcript="выполни проверку сейчас")

    tool = session.tool_runtime_registry.get("transcript_tool")
    await asyncio.wait_for(tool.executed.wait(), timeout=2)
    # Post-tool bookkeeping (history persistence, continuation round) runs
    # after the tool returns; give it a bounded chance to settle.
    return engine, session, llm


@pytest.mark.asyncio
async def test_skip_continuation_when_reply_already_spoken(monkeypatch):
    engine, session, llm = await _run_tool_turn(
        monkeypatch, {"tools": ["transcript_tool"], "skip_continuation_when_spoken": True}
    )

    # The tool result must be persisted for the call record first, then the
    # continuation decision is final: no second LLM round-trip either way.
    session = await _stored_session(engine, lambda s: bool(s.tool_calls))
    await asyncio.sleep(0.2)
    assert llm.generate_calls == 0
    assert [m["role"] for m in session.conversation_history] == ["user", "assistant"]
    assert session.conversation_history[1]["content"].startswith("Данные приняты")
    assert session.tool_calls[0]["tool_call_id"] == "turn-tool-1"

    await engine._cleanup_call(CALL_ID)


@pytest.mark.asyncio
async def test_continuation_still_runs_without_the_flag(monkeypatch):
    engine, session, llm = await _run_tool_turn(
        monkeypatch, {"tools": ["transcript_tool"]}
    )

    # Default behavior preserved: the follow-up round still happens.
    assert await _wait_until(lambda: llm.generate_calls >= 1), "continuation never ran"
    assert llm.generate_calls == 1
    session = await _stored_session(engine, lambda s: bool(s.tool_calls))
    assert session.tool_calls[0]["tool_call_id"] == "turn-tool-1"

    await engine._cleanup_call(CALL_ID)
