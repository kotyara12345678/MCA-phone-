# MCA-phone voice-channel latency instrumentation.
#
# Purpose-built for the TTFA (time-to-first-audio) optimisation pass. The module
# records monotonic timestamps for the discrete events of one conversational turn
# (end of customer speech -> first played audio frame) and turns them into the
# latency breakdown needed to reason about the critical path:
#
#   stt_latency_ms             end-of-speech -> final STT transcript
#   llm_request_to_ttft_ms     LLM request start -> first streamed LLM token
#   llm_ttft_ms                end-of-speech -> first streamed LLM token
#   first_sentence_ms          end-of-speech -> first complete sentence handed to TTS
#   tts_ttfa_ms                TTS start -> first synthesized audio byte
#   end_of_speech_to_first_audio_ms  end-of-speech -> first frame pushed to playback
#   end_of_speech_to_first_played_ms end-of-speech -> first frame confirmed played
#   full_turn_ms               end-of-speech -> end-of-turn (last audio handed off)
#
# It also tracks rolling p50/p95 per metric, error/timeout counters, and the set
# of model names observed per stage (stt/llm/tts).
#
# PRIVACY: this module deliberately records only timings, counts, model names and
# bounded numeric metadata. It NEVER stores raw transcripts, caller phone numbers,
# API keys or free-form text. `note_transcript` records only its length.
#
# The module is stdlib-only so it imports and unit-tests on any platform (the AVA
# engine itself is Linux-only because of fcntl/audioop, but this must not be).

from __future__ import annotations

import threading
import time
from collections import defaultdict, deque
from dataclasses import dataclass, field
from typing import Any, Deque, Dict, Iterable, List, Optional

# Rolling window size per metric (kept modest: these are per-turn values).
_WINDOW = 256

# Canonical event names. Using constants keeps call-sites honest and lets the
# report/tests refer to stages without stringly-typed drift.
EV_SPEECH_START = "speech_start"
EV_SPEECH_END = "speech_end"                  # end-of-speech / endpoint detected
EV_ENDPOINT_DETECT = "endpoint_detect"        # engine noticed the utterance ended
EV_STT_SENT = "stt_request_sent"
EV_STT_FIRST = "stt_first_result"
EV_STT_FINAL = "stt_final_result"
EV_LLM_START = "llm_request_start"
EV_LLM_FIRST_TOKEN = "llm_first_token"
EV_FIRST_SENTENCE = "first_sentence_ready"    # first complete sentence handed to TTS
EV_TTS_START = "tts_start"
EV_TTS_FIRST_BYTE = "tts_first_audio_byte"
EV_FIRST_FRAME_SENT = "first_audio_frame_sent"     # handed to playback stream
EV_FIRST_FRAME_PLAYED = "first_audio_frame_played" # confirmed played (if measurable)
EV_TURN_END = "turn_end"

# Derived latency metrics: (name, start_event, end_event).
_DERIVED = [
    # STT round-trip: request sent -> final transcript (provider time). The wider
    # "end of speech -> transcript" view is covered by
    # end_of_speech_to_first_audio_ms and llm_ttft_ms, which start at EV_SPEECH_END.
    ("stt_latency_ms", EV_STT_SENT, EV_STT_FINAL),
    ("llm_request_to_ttft_ms", EV_LLM_START, EV_LLM_FIRST_TOKEN),
    ("llm_ttft_ms", EV_SPEECH_END, EV_LLM_FIRST_TOKEN),
    ("first_sentence_ms", EV_SPEECH_END, EV_FIRST_SENTENCE),
    ("tts_ttfa_ms", EV_TTS_START, EV_TTS_FIRST_BYTE),
    ("end_of_speech_to_first_audio_ms", EV_SPEECH_END, EV_FIRST_FRAME_SENT),
    ("end_of_speech_to_first_played_ms", EV_SPEECH_END, EV_FIRST_FRAME_PLAYED),
    ("full_turn_ms", EV_SPEECH_END, EV_TURN_END),
]


def _pct(sorted_vals: List[float], q: float) -> float:
    """Linear-interpolated percentile on an already-sorted list."""
    if not sorted_vals:
        return float("nan")
    if len(sorted_vals) == 1:
        return sorted_vals[0]
    k = (len(sorted_vals) - 1) * q
    lo = int(k)
    hi = min(lo + 1, len(sorted_vals) - 1)
    frac = k - lo
    return sorted_vals[lo] * (1.0 - frac) + sorted_vals[hi] * frac


@dataclass
class _Turn:
    """Per-(call_id, turn) monotonic event timestamps plus stage metadata."""

    events: Dict[str, float] = field(default_factory=dict)
    models: Dict[str, str] = field(default_factory=dict)   # stage -> model name
    meta: Dict[str, Any] = field(default_factory=dict)     # bounded numeric meta only

    def mark(self, event: str, ts: float) -> None:
        # Keep the FIRST occurrence of endpoint/speech markers so later re-marks
        # (e.g. barge-in) do not corrupt the critical-path deltas.
        if event in self.events:
            return
        self.events[event] = ts


class LatencyMetrics:
    """Thread-safe, async-friendly latency collector.

    Usage from the engine:
        metrics = get_latency_metrics()          # process-wide singleton
        metrics.mark(call_id, EV_SPEECH_END)
        ...
        metrics.finish_turn(call_id, models={...})  # flushes the turn -> rolling windows
    """

    def __init__(self, window: int = _WINDOW) -> None:
        self._lock = threading.Lock()
        self._window = int(window)
        self._turns: Dict[str, _Turn] = {}
        self._samples: Dict[str, Deque[float]] = defaultdict(lambda: deque(maxlen=self._window))
        self._errors: Dict[str, int] = defaultdict(int)
        self._timeouts: Dict[str, int] = defaultdict(int)
        self._models: Dict[str, int] = defaultdict(int)   # model name -> count of turns
        self._turn_count = 0

    # -- per-turn recording -------------------------------------------------

    def _turn(self, call_id: str) -> _Turn:
        turn = self._turns.get(call_id)
        if turn is None:
            turn = _Turn()
            self._turns[call_id] = turn
        return turn

    def mark(self, call_id: str, event: str, ts: Optional[float] = None) -> None:
        """Record a monotonic event timestamp for this call's current turn."""
        if not call_id:
            return
        t = ts if ts is not None else time.monotonic()
        with self._lock:
            self._turn(call_id).mark(event, t)

    def set_model(self, call_id: str, stage: str, model: Optional[str]) -> None:
        """Attach a (bounded, non-secret) model name to a stage for this turn."""
        if not call_id or not model:
            return
        with self._lock:
            self._turn(call_id).models[stage] = str(model)[:120]

    def set_meta(self, call_id: str, key: str, value: Any) -> None:
        """Attach bounded numeric metadata (e.g. audio buffer ms, chunk size)."""
        if not call_id:
            return
        with self._lock:
            self._turn(call_id).meta[key] = value

    def note_transcript(self, call_id: str, text: Optional[str]) -> None:
        """Record only the LENGTH of a transcript. Never the content (PII-safe)."""
        if not call_id:
            return
        with self._lock:
            self._turn(call_id).meta["transcript_len"] = len(text or "")

    def record_error(self, stage: str) -> None:
        with self._lock:
            self._errors[stage] += 1

    def record_timeout(self, stage: str) -> None:
        with self._lock:
            self._timeouts[stage] += 1

    # -- derived deltas -----------------------------------------------------

    @staticmethod
    def _delta(turn: _Turn, start_ev: str, end_ev: str) -> Optional[float]:
        s = turn.events.get(start_ev)
        e = turn.events.get(end_ev)
        if s is None or e is None or e < s:
            return None
        return (e - s) * 1000.0

    def finish_turn(self, call_id: str, models: Optional[Dict[str, str]] = None) -> Dict[str, Any]:
        """Compute this turn's latency breakdown, fold samples into the rolling
        windows, and drop the per-turn state. Returns the turn breakdown."""
        with self._lock:
            turn = self._turns.pop(call_id, None)
            if turn is None:
                return {}
            if models:
                for stage, name in models.items():
                    if name:
                        turn.models[stage] = str(name)[:120]

            breakdown: Dict[str, Any] = {"models": dict(turn.models)}
            for name, start_ev, end_ev in _DERIVED:
                d = self._delta(turn, start_ev, end_ev)
                breakdown[name] = None if d is None else round(d, 2)
                if d is not None:
                    self._samples[name].append(d)

            for stage, name in turn.models.items():
                if name:
                    self._models[name] += 1
            self._turn_count += 1
            return breakdown

    def abandon_turn(self, call_id: str) -> None:
        """Drop per-turn state without folding samples (cancelled/failed turns)."""
        with self._lock:
            self._turns.pop(call_id, None)

    # -- aggregation / export ----------------------------------------------

    def _stat(self, name: str) -> Dict[str, Any]:
        vals = sorted(self._samples.get(name, ()))
        if not vals:
            return {"count": 0}
        return {
            "count": len(vals),
            "p50_ms": round(_pct(vals, 0.50), 1),
            "p95_ms": round(_pct(vals, 0.95), 1),
            "min_ms": round(vals[0], 1),
            "max_ms": round(vals[-1], 1),
            "last_ms": round(vals[-1], 1),
        }

    def snapshot(self) -> Dict[str, Any]:
        """Operator-facing snapshot: per-metric p50/p95 + errors/timeouts/models."""
        with self._lock:
            metrics = {name: self._stat(name) for name, _, _ in _DERIVED}
            return {
                "turns": self._turn_count,
                "metrics": metrics,
                "errors": dict(self._errors),
                "timeouts": dict(self._timeouts),
                "models": dict(self._models),
            }

    def reset(self) -> None:
        with self._lock:
            self._turns.clear()
            self._samples.clear()
            self._errors.clear()
            self._timeouts.clear()
            self._models.clear()
            self._turn_count = 0


# Process-wide singleton so engine + adapters share one collector without
# threading an instance through every call-signature.
_singleton = LatencyMetrics()
_singleton_lock = threading.Lock()


def get_latency_metrics() -> LatencyMetrics:
    return _singleton


def reset_latency_metrics() -> None:
    """Test helper: reset the process-wide singleton."""
    _singleton.reset()
