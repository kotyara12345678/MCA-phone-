"""Tool-calling probe: verify a fast non-reasoning model still emits tool_calls
through the Polza OpenAI-compatible gateway. The MCA pipeline depends on in-call
tools (mca_get_requirements, mca_save_facts, ...), so a low-latency model is only
usable if it calls tools reliably.

Key is read from the POLZA_API_KEY env var and never printed.
"""
import json
import os
import ssl
import time
import urllib.request

KEY = os.environ["POLZA_API_KEY"]
BASE = os.getenv("POLZA_BASE_URL", "https://polza.ai/api/v1").rstrip("/")

TOOLS = [{
    "type": "function",
    "function": {
        "name": "mca_save_facts",
        "description": "Сохранить подтверждённые клиентом факты по перевозке в CRM.",
        "parameters": {
            "type": "object",
            "properties": {
                "facts": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "field": {"type": "string"},
                            "value": {"type": "string"},
                        },
                        "required": ["field", "value"],
                    },
                }
            },
            "required": ["facts"],
        },
    },
}]

PROMPTS = [
    "Клиент: Перевезём 3 паллеты стройматериалов из Москвы в Казань, 800 кг. Ассистент:",
    "Клиент: Отправляем хрупкий груз, 2 коробки, забор завтра. Ассистент:",
]


def probe(model, n=2):
    ctx = ssl.create_default_context()
    ok = 0
    ttft = []
    for i in range(n):
        payload = {
            "model": model,
            "stream": False,
            "temperature": 0.4,
            "max_tokens": 200,
            "tools": TOOLS,
            "tool_choice": "auto",
            "messages": [
                {"role": "system", "content": "Ты — голосовой ассистент MCA. Сохрани факты через mca_save_facts."},
                {"role": "user", "content": PROMPTS[i % len(PROMPTS)]},
            ],
        }
        req = urllib.request.Request(
            BASE + "/chat/completions",
            data=json.dumps(payload).encode(),
            headers={"Authorization": "Bearer " + KEY, "Content-Type": "application/json"},
            method="POST",
        )
        t0 = time.perf_counter()
        try:
            with urllib.request.urlopen(req, timeout=45, context=ctx) as r:
                data = json.loads(r.read().decode())
            dt = time.perf_counter() - t0
            msg = (data.get("choices") or [{}])[0].get("message", {})
            tcs = msg.get("tool_calls") or []
            if tcs:
                ok += 1
                ttft.append(dt)
                names = [t.get("function", {}).get("name") for t in tcs]
                print(f"  {model} run{i}: TOOL_CALL {names} in {dt:.2f}s")
            else:
                print(f"  {model} run{i}: no tool_call (content={str(msg.get('content'))[:60]!r}) in {dt:.2f}s")
        except Exception as e:  # noqa: BLE001
            print(f"  {model} run{i}: ERROR {type(e).__name__} {str(e)[:80]}")
    print(f"==> {model}: tool_call {ok}/{n}")


for m in ["google/gemini-2.5-flash-lite", "openai/gpt-4.1-mini"]:
    print(f"probing {m} ...")
    probe(m)
