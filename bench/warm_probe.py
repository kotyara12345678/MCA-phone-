import json, time, ssl, http.client, os
key = os.environ["POLZA_API_KEY"]
host = "polza.ai"; base = "/api/v1"
ctx = ssl.create_default_context()
conn = http.client.HTTPSConnection(host, 443, context=ctx, timeout=40)
SYS = "Ты — голосовой ассистент компании MCA (грузоперевозки). Отвечай по-русски, кратко, одной фразой."
prompts = ["Клиент: Здравствуйте, нужно перевезти груз в Казань. Ассистент:",
           "Клиент: Сколько стоит паллета 500 кг? Ассистент:",
           "Клиент: У меня хрупкий груз. Ассистент:"]


def run(model, n):
    ttfts = []; totals = []
    for i in range(n):
        body = json.dumps({"model": model, "stream": True, "max_tokens": 48, "temperature": 0.4,
                           "messages": [{"role": "system", "content": SYS},
                                        {"role": "user", "content": prompts[i % len(prompts)]}]}).encode()
        headers = {"Authorization": "Bearer " + key, "Content-Type": "application/json",
                   "Connection": "keep-alive", "User-Agent": "mca-bench/1.0"}
        t0 = time.perf_counter(); conn.request("POST", base + "/chat/completions", body=body, headers=headers)
        resp = conn.getresponse(); ttft = None; buf = b""
        while True:
            chunk = resp.read(128)
            if not chunk: break
            buf += chunk
            while b"\n" in buf:
                line, buf = buf.split(b"\n", 1); line = line.strip()
                if not line.startswith(b"data:"): continue
                ds = line[5:].strip().decode("utf-8", "ignore")
                if ds == "[DONE]": continue
                try: obj = json.loads(ds)
                except Exception: continue
                for ch in obj.get("choices", []):
                    if ch.get("delta", {}).get("content") and ttft is None:
                        ttft = time.perf_counter() - t0
        total = time.perf_counter() - t0
        if ttft is not None: ttfts.append(ttft)
        totals.append(total)
    p = lambda v, q: round(sorted(v)[int((len(v) - 1) * q)] * 1000, 1) if v else float('nan')
    return {"model": model, "ttft_p50": p(ttfts, .5), "ttft_p95": p(ttfts, .95),
            "tot_p50": p(totals, .5), "tot_p95": p(totals, .95)}


out = [run(m, 6) for m in ["google/gemini-2.5-flash-lite", "openai/gpt-4.1-mini", "openai/gpt-4o-mini"]]
print(json.dumps(out, ensure_ascii=False))
