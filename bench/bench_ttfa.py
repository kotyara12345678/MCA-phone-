#!/usr/bin/env python3
"""Reproducible latency benchmark for the MCA-phone voice LLM path (Polza gateway).

Measures, per model, on short Russian conversational prompts:
  - TTFT  : time to first streamed content token (stream=true SSE)
  - total : full completion time
  - prompt/completion token usage (when the gateway reports it)

Everything is reported as p50/p95 over N runs. No secrets are ever printed:
the API key is read from prod/.env (or the POLZA_API_KEY env var) and only the
Authorization header is constructed in memory.

Usage:
  python bench/bench_ttfa.py list                # dump available model ids (JSON)
  python bench/bench_ttfa.py bench [--runs N] [--models a,b,c]

This tool is intentionally dependency-free (stdlib only) so it runs on the
voice host and in CI without installing anything.
"""
from __future__ import annotations

import argparse
import json
import os
import socket
import ssl
import sys
import time
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional, Tuple

BASE_URL = os.getenv("POLZA_BASE_URL", "https://polza.ai/api/v1").rstrip("/")
ENV_FILE_CANDIDATES = [
    os.path.join(os.path.dirname(__file__), "..", "prod", ".env"),
    "prod/.env",
    ".env",
]

# Short, realistic Russian opening turns for a freight-forwarding qualification
# line. Kept terse so we measure the *responsiveness* floor, not long-form prose.
RUSSIAN_PROMPTS = [
    "Клиент: Здравствуйте, нужно перевезти груз из Москвы в Казань. Ассистент:",
    "Клиент: Сколько стоит доставка паллеты до 500 кг? Ассистент:",
    "Клиент: У меня хрупкий груз, 3 коробки, заберите завтра. Ассистент:",
    "Клиент: Хотим отправить рефрижератор, +5 градусов, два города. Ассистент:",
    "Клиент: Передайте меня менеджеру, пожалуйста. Ассистент:",
]

SYSTEM_PROMPT = (
    "Ты — голосовой ассистент компании MCA (грузоперевозки). Отвечай по-русски, "
    "кратко, одной фразой (до 20 слов), речью."
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


def _ctx() -> ssl.SSLContext:
    return ssl.create_default_context()


def list_models(key: str, timeout: float) -> Dict[str, Any]:
    req = urllib.request.Request(
        f"{BASE_URL}/models",
        headers={"Authorization": f"Bearer {key}", "User-Agent": "mca-bench/1.0"},
    )
    started = time.perf_counter()
    with urllib.request.urlopen(req, timeout=timeout, context=_ctx()) as resp:
        raw = resp.read()
    elapsed = time.perf_counter() - started
    data = json.loads(raw.decode("utf-8"))
    ids = sorted({m.get("id") for m in data.get("data", []) if m.get("id")})
    return {"elapsed_s": round(elapsed, 3), "count": len(ids), "ids": ids}


def _chat_payload(model: str, prompt: str, max_tokens: int) -> Dict[str, Any]:
    return {
        "model": model,
        "stream": True,
        "max_tokens": max_tokens,
        "temperature": 0.4,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": prompt},
        ],
    }


def bench_model(
    key: str,
    model: str,
    runs: int,
    max_tokens: int,
    timeout: float,
) -> Dict[str, Any]:
    url = f"{BASE_URL}/chat/completions"
    headers = {
        "Authorization": f"Bearer {key}",
        "Content-Type": "application/json",
        "User-Agent": "mca-bench/1.0",
    }
    ttfts: List[float] = []
    totals: List[float] = []
    errors: List[str] = []
    usage_seen: List[Dict[str, int]] = []

    for i in range(runs):
        prompt = RUSSIAN_PROMPTS[i % len(RUSSIAN_PROMPTS)]
        payload = _chat_payload(model, prompt, max_tokens)
        req = urllib.request.Request(
            url, data=json.dumps(payload).encode("utf-8"), headers=headers, method="POST"
        )
        t0 = time.perf_counter()
        ttft: Optional[float] = None
        try:
            with urllib.request.urlopen(req, timeout=timeout, context=_ctx()) as resp:
                if resp.status >= 400:
                    errors.append(f"HTTP {resp.status}")
                    continue
                buf = b""
                while True:
                    chunk = resp.read(256)  # small reads to catch first token ASAP
                    if not chunk:
                        break
                    buf += chunk
                    # process complete SSE lines only
                    while b"\n" in buf:
                        line, buf = buf.split(b"\n", 1)
                        line = line.strip()
                        if not line.startswith(b"data:"):
                            continue
                        data_str = line[5:].strip().decode("utf-8", "ignore")
                        if data_str == "[DONE]":
                            continue
                        try:
                            obj = json.loads(data_str)
                        except json.JSONDecodeError:
                            continue
                        if obj.get("usage"):
                            usage_seen.append(obj["usage"])
                        for ch in obj.get("choices", []):
                            delta = ch.get("delta", {})
                            if delta.get("content") and ttft is None:
                                ttft = time.perf_counter() - t0
            total = time.perf_counter() - t0
            if ttft is not None:
                ttfts.append(ttft)
            totals.append(total)
        except (urllib.error.URLError, socket.timeout, TimeoutError) as exc:
            errors.append(f"{type(exc).__name__}: {str(exc)[:80]}")
        except Exception as exc:  # noqa: BLE001 - benchmark must never crash
            errors.append(f"{type(exc).__name__}: {str(exc)[:80]}")

    def _pct(vals: List[float], p: float) -> float:
        if not vals:
            return float("nan")
        vals = sorted(vals)
        k = (len(vals) - 1) * p
        lo = int(k)
        hi = min(lo + 1, len(vals) - 1)
        frac = k - lo
        return vals[lo] * (1 - frac) + vals[hi] * frac

    return {
        "model": model,
        "runs": runs,
        "errors": len(errors),
        "error_samples": errors[:3],
        "ttft_p50_ms": round(_pct(ttfts, 0.50) * 1000, 1),
        "ttft_p95_ms": round(_pct(ttfts, 0.95) * 1000, 1),
        "total_p50_ms": round(_pct(totals, 0.50) * 1000, 1),
        "total_p95_ms": round(_pct(totals, 0.95) * 1000, 1),
        "usage_sample": usage_seen[-1] if usage_seen else None,
    }


def main(argv: List[str]) -> int:
    ap = argparse.ArgumentParser(description="MCA-phone Polza LLM latency benchmark")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p_list = sub.add_parser("list", help="list available model ids")
    p_list.add_argument("--timeout", type=float, default=120.0)

    p_bench = sub.add_parser("bench", help="benchmark TTFT/total latency")
    p_bench.add_argument("--runs", type=int, default=5)
    p_bench.add_argument("--max-tokens", type=int, default=64)
    p_bench.add_argument("--timeout", type=float, default=45.0)
    p_bench.add_argument(
        "--models",
        type=str,
        default=(
            "openai/gpt-4.1-mini,openai/gpt-4o-mini,openai/gpt-4o,"
            "openai/gpt-5-mini,openai/gpt-5-nano,openai/gpt-4.1"
        ),
        help="comma-separated model ids to benchmark",
    )

    args = ap.parse_args(argv)
    key = load_api_key()
    if not key:
        print(json.dumps({"error": "POLZA_API_KEY not found"}), file=sys.stderr)
        return 2

    socket.setdefaulttimeout(max(args.timeout, 30))

    if args.cmd == "list":
        try:
            print(json.dumps(list_models(key, args.timeout), ensure_ascii=False))
            return 0
        except Exception as exc:  # noqa: BLE001
            print(json.dumps({"error": f"{type(exc).__name__}: {str(exc)[:200]}"}))
            return 1

    models = [m.strip() for m in args.models.split(",") if m.strip()]
    results = []
    for model in models:
        print(f"benching {model} ...", file=sys.stderr)
        results.append(
            bench_model(key, model, args.runs, args.max_tokens, args.timeout)
        )
    print(json.dumps({"base_url": BASE_URL, "results": results}, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
