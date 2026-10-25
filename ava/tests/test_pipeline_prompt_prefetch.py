"""Pipeline prompt prefetch tests.

``Engine._prefetch_pipeline_prompt_tools`` executes the in-call read-only tools
listed in the pipeline llm option ``prefetch_tools`` once, before the system
prompt is composed, and stores each successful payload in
``session.pre_call_results`` as ``<tool_name>_data``. The normal prompt
substitution then exposes it as ``{<tool_name>_data}``, which removes the
mandatory "call the CRM lookup at the start of the call" LLM round-trip from
the conversational path (~1.5-2.5s of dead air per first turn on a cloud
gateway).

Failure semantics matter as much as the happy path: a down CRM, a bad URL or
a timeout must leave the session untouched so the prompt keeps its explicit
"if the data block is empty, call the tool yourself" fallback.

PY3.13 NOTE: same import-only ``audioop`` shim as test_latency_metrics_ttfa.py
(needed because ``src.engine`` transitively imports ``src.audio.resampler``).
"""

import asyncio
import sys
import types

import pytest

try:  # pragma: no cover - exercised by platform availability
    import audioop  # noqa: F401
except ModuleNotFoundError:  # Python >= 3.13
    sys.modules.setdefault("audioop", types.ModuleType("audioop"))

from src.config import AppConfig  # noqa: E402
from src.core.models import CallSession  # noqa: E402
from src.engine import Engine  # noqa: E402


class _StubDefinition:
    def __init__(self, timeout_ms=500):
        self.timeout_ms = timeout_ms


class _StubTool:
    def __init__(self, result=None, error=None, delay_sec=0.0):
        self._result = result
        self._error = error
        self._delay_sec = delay_sec
        self.definition = _StubDefinition()
        self.calls = 0

    async def execute(self, parameters, ctx):
        self.calls += 1
        if self._delay_sec:
            await asyncio.sleep(self._delay_sec)
        if self._error is not None:
            raise self._error
        return self._result


class _StubRegistry:
    def __init__(self, tools):
        self._tools = tools

    def get(self, name):
        return self._tools.get(name)


def _make_engine():
    engine = Engine(
        AppConfig(
            default_provider="local",
            providers={"local": {"enabled": True}},
            asterisk={"host": "127.0.0.1", "username": "u", "password": "p"},
            llm={"prompt": "You are helpful"},
            pipelines={"mca": {}},
            active_pipeline="mca",
        )
    )
    engine.pipeline_orchestrator._started = True
    return engine


def _session(call_id="prefetch-call"):
    return CallSession(call_id=call_id, caller_channel_id=call_id)


@pytest.mark.asyncio
async def test_prefetch_injects_tool_data_into_pre_call_results(monkeypatch):
    engine = _make_engine()
    tool = _StubTool(result={"status": "success", "data": {"blocking_gaps": ["cargo"]}})
    monkeypatch.setattr(
        engine, "_tool_registry_for_session", lambda session: _StubRegistry({"mca_get_requirements": tool})
    )
    session = _session()
    session.pre_call_results = {"mca_lead_id": "lead-1"}

    out = await engine._prefetch_pipeline_prompt_tools(
        session.call_id, session, {"prefetch_tools": ["mca_get_requirements"]}
    )

    assert out == {"prefetch_tools": ["mca_get_requirements"]}
    assert tool.calls == 1
    # Existing pre-call variables are preserved, the payload is compact JSON.
    assert session.pre_call_results["mca_lead_id"] == "lead-1"
    assert session.pre_call_results["mca_get_requirements_data"] == '{"blocking_gaps":["cargo"]}'
    # Persisted so later prompt substitutions (and session reloads) see it.
    stored = await engine.session_store.get_by_call_id(session.call_id)
    assert stored.pre_call_results["mca_get_requirements_data"] == '{"blocking_gaps":["cargo"]}'


@pytest.mark.asyncio
async def test_prefetch_result_feeds_prompt_template_substitution(monkeypatch):
    engine = _make_engine()
    tool = _StubTool(result={"status": "success", "data": {"asked_questions": []}})
    monkeypatch.setattr(
        engine, "_tool_registry_for_session", lambda session: _StubRegistry({"mca_get_requirements": tool})
    )
    session = _session("prefetch-template")

    await engine._prefetch_pipeline_prompt_tools(
        session.call_id, session, {"prefetch_tools": ["mca_get_requirements"]}
    )

    prompt = engine._apply_prompt_template_substitution(
        "ДАННЫЕ ЗАЯВКИ: {mca_get_requirements_data}. Отвечай коротко.", session
    )
    assert '{"asked_questions":[]}' in prompt
    assert "{mca_get_requirements_data}" not in prompt


@pytest.mark.asyncio
async def test_prefetch_tool_error_leaves_session_untouched(monkeypatch):
    engine = _make_engine()
    tool = _StubTool(error=RuntimeError("crm down"))
    monkeypatch.setattr(
        engine, "_tool_registry_for_session", lambda session: _StubRegistry({"mca_get_requirements": tool})
    )
    session = _session("prefetch-error")

    # Must not raise: the prompt keeps its in-call tool fallback.
    await engine._prefetch_pipeline_prompt_tools(
        session.call_id, session, {"prefetch_tools": ["mca_get_requirements"]}
    )

    assert "mca_get_requirements_data" not in (session.pre_call_results or {})


@pytest.mark.asyncio
async def test_prefetch_non_success_and_empty_payload_are_skipped(monkeypatch):
    engine = _make_engine()
    registry = _StubRegistry(
        {
            "failed_tool": _StubTool(result={"status": "error", "message": "nope"}),
            "empty_tool": _StubTool(result={"status": "success", "data": {}}),
        }
    )
    monkeypatch.setattr(engine, "_tool_registry_for_session", lambda session: registry)
    session = _session("prefetch-empty")

    await engine._prefetch_pipeline_prompt_tools(
        session.call_id, session, {"prefetch_tools": ["failed_tool", "empty_tool"]}
    )

    assert "failed_tool_data" not in (session.pre_call_results or {})
    assert "empty_tool_data" not in (session.pre_call_results or {})


@pytest.mark.asyncio
async def test_prefetch_is_noop_without_option_or_registry(monkeypatch):
    engine = _make_engine()
    session = _session("prefetch-noop")

    out = await engine._prefetch_pipeline_prompt_tools(session.call_id, session, {})
    assert out == {}
    out = await engine._prefetch_pipeline_prompt_tools(session.call_id, session, {"prefetch_tools": []})
    assert out == {"prefetch_tools": []}
    assert not (session.pre_call_results or {})


@pytest.mark.asyncio
async def test_prefetch_truncates_oversized_payload(monkeypatch):
    engine = _make_engine()
    big = "x" * (engine._PIPELINE_PREFETCH_MAX_CHARS + 500)
    tool = _StubTool(result={"status": "success", "data": big})
    monkeypatch.setattr(engine, "_tool_registry_for_session", lambda session: _StubRegistry({"big_tool": tool}))
    session = _session("prefetch-big")

    await engine._prefetch_pipeline_prompt_tools(session.call_id, session, {"prefetch_tools": ["big_tool"]})

    value = session.pre_call_results["big_tool_data"]
    assert value.endswith("…")
    assert len(value) == engine._PIPELINE_PREFETCH_MAX_CHARS + 1


@pytest.mark.asyncio
async def test_prefetch_timeout_is_non_fatal(monkeypatch):
    engine = _make_engine()
    slow = _StubTool(result={"status": "success", "data": {"a": 1}}, delay_sec=5.0)
    slow.definition = _StubDefinition(timeout_ms=50)
    monkeypatch.setattr(engine, "_tool_registry_for_session", lambda session: _StubRegistry({"slow_tool": slow}))
    session = _session("prefetch-timeout")

    await engine._prefetch_pipeline_prompt_tools(session.call_id, session, {"prefetch_tools": ["slow_tool"]})

    assert "slow_tool_data" not in (session.pre_call_results or {})
