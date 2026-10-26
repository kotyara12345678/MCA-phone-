#!/usr/bin/env python3
"""Production-shaped latency benchmark for the MCA-phone voice LLM path.

The short bench (bench_ttfa.py) uses a 2-line system prompt, which is NOT what
the engine sends. In production every turn carries:

  * the full persona/system prompt from prod/ai-agent.yaml (llm.prompt) with the
    CRM block inlined into {mca_get_requirements_data},
  * the five in_call_tools JSON schemas from the same file,
  * up to max_history_messages conversation messages.

This script replays exactly that shape against the live Polza gateway and
reports, per model, streaming:

  ttft_p50/p95            first streamed content token (ms)
  total_p50/p95           full completion (ms)
  first_sentence_p50/p95  first complete sentence, i.e. what reaches TTS
  usage                   prompt/completion tokens when the gateway reports them

If ttft ~= total, the gateway is buffering the stream (or not honouring
stream=true for that model) - that alone explains multi-second dead air.
If ttft is small but the first sentence is late, the model is slow to produce a
speakable sentence (reasoning or a long preamble).

Stdlib + PyYAML only. The API key is read from prod/.env or POLZA_API_KEY and
is never printed.

Usage:
  python bench/bench_prod_shape.py --runs 5
  python bench/bench_prod_shape.py --runs 5 --profile short
"""
from __future__ import annotations

import argparse
import json
import os
import re
import ssl
import socket
import sys
import time
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional

BASE_URL = os.getenv("POLZA_BASE_URL", "https://polza.ai/api/v1").rstrip("/")
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENV_FILE_CANDIDATES = [os.path.join(ROOT, "prod", ".env"), "prod/.env", ".env"]
CONFIG_CANDIDATES = [os.path.join(ROOT, "prod", "ai-agent.yaml"), "prod/ai-agent.yaml"]

# Conversation history: 16 messages, cut on a user message exactly like the
# engine's max_history_messages trimming. Realistic freight-forwarding turns.
HISTORY: List[Dict[str, str]] = [
    {"role": "user", "content": "Здравствуйте, нужно перевезти груз."},
    {"role": "assistant", "content": "Добрый день! Помогу. Откуда и куда везём груз?"},
    {"role": "user", "content": "Из Москвы в Казань."},
    {"role": "assistant", "content": "Понял. Что за груз и какой вес?"},
    {"role": "user", "content": "Паллеты с бумагой, примерно 800 килограмм."},
    {"role": "assistant", "content": "Записал. Сколько паллет и какой размер?"},
    {"role": "user", "content": "Четыре паллеты, стандартные, метр двадцать."},
    {"role": "assistant", "content": "Принято. Когда нужен забор груза?"},
    {"role": "user", "content": "Завтра, утром, с девяти."},
    {"role": "assistant", "content": "Хорошо. Машина нужна тентованная или рефрижератор?"},
    {"role": "user", "content": "Тент, температура не важна."},
    {"role": "assistant", "content": "Понятно. Есть ли особые условия по разгрузке?"},
    {"role": "user", "content": "Разгрузка со стороны дока, есть погрузчик."},
    {"role": "assistant", "content": "Отлично. Нужна страховка груза?"},
    {"role": "user", "content": "Да, на всякий случай."},
    {"role": "assistant", "content": "Хорошо, уточню ещё пару деталей по документам."},
]

SHORT_SYSTEM = (
    "Ты — голосовой ассистент компании MCA (грузоперевозки). Отвечай по-русски, "
    "кратко, одной фразой (до 20 слов), речью."
)
SHORT_PROMPT = "Клиент: Здравствуйте, нужно перевезти груз из Москвы в Казань. Ассистент:"

SAMPLE_CRM = json.dumps(
    {
        "lead_id": "lead-bench",
        "blocking_gaps": ["origin_city", "destination_city", "goods_weight"],
        "open_gaps": ["desired_deadline", "transport_mode"],
        "asked_questions": [
            {"question": "Откуда и куда везём груз?", "answered": True},
            {"question": "Что за груз и какой вес?", "answered": True},
            {"question": "Когда нужен забор груза?", "answered": False},
        ],
        "facts": [
            {"field": "goods_name", "value": "Бумага на паллетах"},
            {"field": "goods_weight", "value": "800", "unit": "kg"},
        ],
    },
    ensure_ascii=False,
)


def load_api_key() -> str:
    key = os.getenv("POLZA_API_KEY", "").strip()
    if key:
        return key
    for cand in ENV_FILE_CANDIDATES:
        try:
            with open(cand, "r", encoding="utf-8") as fh:
                for line in fh:
                    line = line.strip()
                    if line.startswith("POLZA_API_KEY="):
                        return line.split("=", 1)[1].strip().strip('"').strip("'")
        except OSError:
            continue
    return ""


def load_config() -> Dict[str, Any]:
    try:
        import yaml  # type: ignore
    except ImportError:
        return {}
    for cand in CONFIG_CANDIDATES:
        try:
            with open(cand, "r", encoding="utf-8") as fh:
                return yaml.safe_load(fh) or {}
        except OSError:
            continue
    return {}


def build_tools(cfg: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Replay the prod in_call_tools block as OpenAI function schemas."""
    tools: List[Dict[str, Any]] = []
    for name, spec in (cfg.get("in_call_tools") or {}).items():
        if not isinstance(spec, dict) or not spec.get("enabled", True):
            continue
        params: Dict[str, Any] = {"type": "object", "properties": {}}
        required: List[str] = []
        for param in spec.get("parameters") or []:
            pname = param.get("name")
            if not pname:
                continue
            params["properties"][pname] = {
                "type": param.get("type", "string"),
                "description": str(param.get("description", ""))[:600],
            }
            if param.get("required"):
                required.append(pname)
        if required:
            params["required"] = required
        tools.append(
            {
                "type": "function",
                "function": {
                    "name": name,
                    "description": str(spec.get("description", "")),
                    "parameters": params,
                },
            }
        )
    return tools


def build_messages(
    cfg: Dict[str, Any], profile: str, extra_instruction: str = ""
) -> List[Dict[str, str]]:
    if profile == "short":
        return [
            {"role": "system", "content": SHORT_SYSTEM},
            {"role": "user", "content": SHORT_PROMPT},
        ]
    prompt = str((cfg.get("llm") or {}).get("prompt") or SHORT_SYSTEM)
    prompt = prompt.replace("{mca_get_requirements_data}", SAMPLE_CRM)
    if extra_instruction:
        prompt = f"{extra_instruction}\n\n{prompt}"
    messages: List[Dict[str, str]] = [{"role": "system", "content": prompt}]
    messages.extend(HISTORY)
    messages.append(
        {"role": "user", "content": "Когда нужен забор груза? Можно ли раньше девяти?"}
    )
    return messages


def _has_sentence(text: str) -> bool:
    return re.search(r"[^.!?]*[.!?]", text) is not None


def _accumulate_tool_calls(delta_tool_calls: List[Dict[str, Any]], acc: Dict[int, Dict[str, str]]) -> None:
    for tc in delta_tool_calls:
        idx = int(tc.get("index") or 0)
        slot = acc.setdefault(idx, {"id": "", "name": "", "arguments": ""})
        if tc.get("id"):
            slot["id"] = str(tc["id"])
        fn = tc.get("function") or {}
        if fn.get("name"):
            slot["name"] = str(fn["name"])
        if fn.get("arguments"):
            slot["arguments"] += str(fn["arguments"])


def bench_model(
    key: str,
    model: str,
    messages: List[Dict[str, str]],
    tools: List[Dict[str, Any]],
    runs: int,
    max_tokens: int,
    temperature: float,
    timeout: float,
    simulate_rounds: int = 1,
) -> Dict[str, Any]:
    url = f"{BASE_URL}/chat/completions"
    headers = {
        "Authorization": f"Bearer {key}",
        "Content-Type": "application/json",
        "User-Agent": "mca-bench-prod/1.0",
    }
    payload: Dict[str, Any] = {
        "model": model,
        "stream": True,
        "max_tokens": max_tokens,
        "temperature": temperature,
        "messages": messages,
    }
    if tools:
        payload["tools"] = tools
        payload["tool_choice"] = "auto"

    ttfts: List[float] = []
    totals: List[float] = []
    first_sent: List[float] = []
    errors: List[str] = []
    usage: Optional[Dict[str, Any]] = None
    tool_rounds = 0
    tool_names: List[str] = []
    tool_only_runs = 0
    tool_only_round1 = 0
    text_only_runs = 0
    mixed_runs = 0
    tool_only_totals: List[float] = []
    sample_text = ""
    rounds_seen: List[int] = []
    text_wait: List[float] = []
    phase_total: List[float] = []
    tool_names_by_round: List[List[str]] = []

    def _post(messages_now: List[Dict[str, Any]]) -> Dict[str, Any]:
        """One streaming request; returns raw parse state for the caller loop."""
        req_payload = dict(payload)
        req_payload["messages"] = messages_now
        req = urllib.request.Request(
            url,
            data=json.dumps(req_payload).encode("utf-8"),
            headers=headers,
            method="POST",
        )
        t0 = time.perf_counter()
        state: Dict[str, Any] = {
            "t0": t0,
            "ttft": None,
            "first_sentence_ts": None,
            "first_token_at": None,
            "first_sentence_at": None,
            "text": [],
            "tool_acc": {},
            "usage": None,
        }
        ctx = ssl.create_default_context()
        with urllib.request.urlopen(req, timeout=timeout, context=ctx) as resp:
            if resp.status >= 400:
                raise RuntimeError(f"HTTP {resp.status}")
            buf = b""
            while True:
                chunk = resp.read(256)
                if not chunk:
                    break
                buf += chunk
                while b"\n" in buf:
                    line, buf = buf.split(b"\n", 1)
                    line = line.strip()
                    if not line.startswith(b"data:"):
                        continue
                    raw = line[5:].strip().decode("utf-8", "ignore")
                    if raw == "[DONE]":
                        continue
                    try:
                        obj = json.loads(raw)
                    except json.JSONDecodeError:
                        continue
                    if obj.get("usage"):
                        state["usage"] = obj["usage"]
                        usage = obj["usage"]
                    for ch in obj.get("choices", []):
                        delta = ch.get("delta", {})
                        if delta.get("tool_calls"):
                            _accumulate_tool_calls(delta["tool_calls"], state["tool_acc"])
                        content = delta.get("content")
                        if content:
                            state["text"].append(content)
                            now = time.perf_counter() - t0
                            if state["ttft"] is None:
                                state["ttft"] = now
                                state["first_token_at"] = t0 + now
                            if state["first_sentence_ts"] is None and _has_sentence(
                                "".join(state["text"])
                            ):
                                state["first_sentence_ts"] = now
                                state["first_sentence_at"] = t0 + now
        return state

    for _ in range(runs):
        convo: List[Dict[str, Any]] = [dict(m) for m in messages]
        run_start = time.perf_counter()
        rounds = 0
        run_tool_names: List[str] = []
        ttft: Optional[float] = None
        first_sentence_ts: Optional[float] = None
        text: List[str] = []
        got_tool_call = False
        tool_only_first_round = False
        try:
            for rnd in range(max(1, simulate_rounds)):
                state = _post(convo)
                rounds = rnd + 1
                if state.get("usage"):
                    usage = state["usage"]
                round_text = "".join(state["text"])
                text.extend(state["text"])
                if state.get("first_token_at") and ttft is None:
                    ttft = state["first_token_at"] - run_start
                if state.get("first_sentence_at") and first_sentence_ts is None:
                    first_sentence_ts = state["first_sentence_at"] - run_start

                acc = state["tool_acc"]
                calls = [acc[k] for k in sorted(acc)] if acc else []
                names = [c["name"] for c in calls if c.get("name")]
                if calls:
                    got_tool_call = True
                    run_tool_names.extend(names)
                    if rnd == 0 and not round_text:
                        tool_only_first_round = True
                    convo.append(
                        {
                            "role": "assistant",
                            "content": round_text or None,
                            "tool_calls": [
                                {
                                    "id": c.get("id") or f"call_{i}",
                                    "type": "function",
                                    "function": {
                                        "name": c.get("name") or "",
                                        "arguments": c.get("arguments") or "{}",
                                    },
                                }
                                for i, c in enumerate(calls)
                            ],
                        }
                    )
                    for i, c in enumerate(calls):
                        convo.append(
                            {
                                "role": "tool",
                                "tool_call_id": c.get("id") or f"call_{i}",
                                "content": json.dumps(
                                    {"ok": True, "tool": c.get("name")}, ensure_ascii=False
                                ),
                            }
                        )
                    # Real engine: tool HTTP round-trip (MCA API) between LLM calls.
                    time.sleep(0.15)
                    continue
                break

            total = time.perf_counter() - run_start
            if ttft is not None:
                ttfts.append(ttft)
                totals.append(total)
                if first_sentence_ts is not None:
                    first_sent.append(first_sentence_ts)
                sample_text = "".join(text)[:160]
                text_wait.append(ttft)
            phase_total.append(total)
            rounds_seen.append(rounds)
            if got_tool_call:
                tool_rounds += 1
                tool_names.extend(run_tool_names)
                if not text:
                    tool_only_runs += 1
                    tool_only_totals.append(total)
            if text:
                if got_tool_call:
                    mixed_runs += 1
                else:
                    text_only_runs += 1
            if tool_only_first_round:
                tool_only_round1 += 1
        except (urllib.error.URLError, socket.timeout, TimeoutError) as exc:
            errors.append(f"{type(exc).__name__}: {str(exc)[:80]}")
        except Exception as exc:  # noqa: BLE001
            errors.append(f"{type(exc).__name__}: {str(exc)[:80]}")

    def pct(vals: List[float], p: float) -> float:
        if not vals:
            return float("nan")
        vals = sorted(vals)
        k = (len(vals) - 1) * p
        lo = int(k)
        hi = min(lo + 1, len(vals) - 1)
        frac = k - lo
        return (vals[lo] * (1 - frac) + vals[hi] * frac) * 1000

    return {
        "model": model,
        "runs": runs,
        "errors": len(errors),
        "error_samples": errors[:3],
        "ttft_p50_ms": round(pct(ttfts, 0.50), 1),
        "ttft_p95_ms": round(pct(ttfts, 0.95), 1),
        "first_sentence_p50_ms": round(pct(first_sent, 0.50), 1),
        "total_p50_ms": round(pct(totals, 0.50), 1),
        "total_p95_ms": round(pct(totals, 0.95), 1),
        "tool_only_total_p50_ms": round(pct(tool_only_totals, 0.50), 1),
        "tool_call_runs": tool_rounds,
        "tool_only_runs": tool_only_runs,
        "tool_only_round1_runs": tool_only_round1,
        "rounds_p50": round(pct([float(r) for r in rounds_seen], 0.50) / 1000.0, 2)
        if rounds_seen
        else None,
        "llm_phase_p50_ms": round(pct(phase_total, 0.50), 1),
        "llm_phase_p95_ms": round(pct(phase_total, 0.95), 1),
        "mixed_runs": mixed_runs,
        "text_only_runs": text_only_runs,
        "tool_names": tool_names,
        "usage": usage,
        "text_sample": sample_text,
    }


def main(argv: List[str]) -> int:
    ap = argparse.ArgumentParser(description="Production-shaped Polza LLM benchmark")
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--max-tokens", type=int, default=160)
    ap.add_argument("--temperature", type=float, default=0.3)
    ap.add_argument("--timeout", type=float, default=45.0)
    ap.add_argument("--profile", choices=["prod", "short"], default="prod")
    ap.add_argument(
        "--no-tools",
        action="store_true",
        help="drop the tool schemas: models the speak-first critical round",
    )
    ap.add_argument(
        "--extra-instruction",
        type=str,
        default="",
        help="prepended to the system prompt (prompt-variant experiments)",
    )
    ap.add_argument(
        "--simulate-rounds",
        type=int,
        default=1,
        help="replay the full tool loop (round -> tool HTTP -> round ...) like the engine does",
    )
    ap.add_argument(
        "--user-message",
        type=str,
        default="",
        help="override the final user utterance (tool-trigger scenarios)",
    )
    ap.add_argument(
        "--models",
        type=str,
        default="google/gemini-2.5-flash,google/gemini-2.5-flash-lite",
    )
    args = ap.parse_args(argv)

    key = load_api_key()
    if not key:
        print(json.dumps({"error": "POLZA_API_KEY not found"}), file=sys.stderr)
        return 2
    socket.setdefaulttimeout(max(args.timeout, 30))

    cfg = load_config()
    messages = build_messages(cfg, args.profile, args.extra_instruction)
    if args.user_message:
        messages[-1] = {"role": "user", "content": args.user_message}
    tools = build_tools(cfg) if args.profile == "prod" else []
    if args.no_tools:
        tools = []
    meta = {
        "profile": args.profile,
        "no_tools": args.no_tools,
        "messages": len(messages),
        "prompt_chars": sum(len(m["content"]) for m in messages),
        "tools": len(tools),
        "tools_chars": sum(len(json.dumps(t, ensure_ascii=False)) for t in tools),
        "max_tokens": args.max_tokens,
    }

    results = []
    for model in [m.strip() for m in args.models.split(",") if m.strip()]:
        print(f"benching {model} ({args.profile}) ...", file=sys.stderr)
        results.append(
            bench_model(
                key, model, messages, tools, args.runs,
                args.max_tokens, args.temperature, args.timeout,
                args.simulate_rounds,
            )
        )
    print(
        json.dumps(
            {"base_url": BASE_URL, "meta": meta, "results": results},
            ensure_ascii=False,
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
