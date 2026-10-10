# MCA-phone: голосовой канал через AVA + MCA Mail — итоговый отчёт

Дата: 2026-10-07; обновлено 2026-10-09 (этап 3: email-карточка заказа —
§5; `asked_questions` + продолжение лида между звонками — §6; корневой
`MCA-Workspace/docker-compose.yml` с общим PostgreSQL — §2/§7; 20
обязательных acceptance-сценариев — §8; perf: `statement_timeout` через
`after_connect` — §9). Статус: реализовано и прогнано гейтами;
развёртывание требует операционных шагов (см. раздел «Осталось для прода»).

## 1. Цель и итог

Подключён готовый голосовой агент **AVA AI Voice Agent for Asterisk** (используется
как есть, только конфигурация) к существующей бизнес-логике **MCA Mail** через
минимальный REST-слой. Цепочка: телефон → Asterisk → AVA (STT/LLM/TTS/VAD/трансфер)
→ новые HTTP-инструменты → API MCA Mail (`/api/v1/voice/*`) → существующие
сервисы/репозитории → PostgreSQL.

Не создавался новый голосовой шлюз, STT/TTS/voice-core; никакого нового стека
(Kafka/Redis/MQ). Новый код минимален и ограничен голосовым каналом.

### 1.1. AI-модели: шлюз Polza (проверено живыми запросами)

Один OpenAI-совместимый шлюз закрывает все три этапа (`https://polza.ai/api/v1`,
ключ `POLZA_API_KEY` — только в `prod/.env`):

| Этап | Компонент AVA | Endpoint | Модель | Проба |
|---|---|---|---|---|
| STT | `openai_stt` | `POST /v1/audio/transcriptions` | `openai/whisper-large-v3-turbo` | ✅ 200, `language=ru` |
| LLM | `openai_llm` | `POST /v1/chat/completions` | `openai/gpt-6-luna` | ✅ 200; `temperature=0.4` ✓, `tool_calls` ✓ |
| TTS | `openai_tts` | `POST /v1/audio/speech` | `google/gemini-3.8-flash-tts` | ✅ 200, WAV, `voice=alloy` |

Все id моделей найдены в `GET /v1/models` шлюза (424 позиции). Ключевые
решения при интеграции (по исходникам AVA):

- **Компоненты обязаны называться `openai_stt/openai_llm/openai_tts`** — AVA
  регистрирует openai-адаптеры только под этими каноническими ключами
  (`_hydrate_openai_component_configs`, orchestrator.py:1599); произвольные
  имена вроде `polza_llm` разрешились бы к placeholder-у и упали бы на
  Phase-1 валидации старта.
- **Слэш в id моделей безопасен**: guard-ы адаптеров (openai.py:187 STT,
  :973 TTS) при `"/" в модели` откатывают значение на провайдерский
  default — а мы задаём id именно там, так что запрос уходит с исходным
  `openai/…`/`google/…`.
- **`max_tokens: 1024`** — `gpt-6-luna` reasoning-модель: на пробе
  `max_tokens=50` reasoning съедал весь бюджет и ответ был пустым; 1024
  даёт запас (замер: 23 reasoning-tokens, ~2.6 с на короткий ответ).
- **STT не-стриминговый**: openai-адаптер транскрибирует завершённую
  реплику REST-ом (VAD всё равно отсекает), `language=ru` уходит в форму.
- **`summary_provider: openai_llm`** у `mca_finish_call`: без него AVA
  падает в legacy-путь (`OPENAI_API_KEY` + жёстко `gpt-4o-mini`); с ним
  саммаризация идёт через зарегистрированный компонент `openai_llm`
  (post_call_summary.py → `orchestrator.generate_once`), при ошибке —
  пустая саммария, webhook всё равно доставляется.
- **Ожидаемый шум на старте**: автопроверка connectivy делает GET на три
  endpoint'а Polza — все отвечают 404 на GET → логи
  `Active pipeline validation FAILED` и «unhealthy» в `/health`. Это
  **не блокирует** звонки (orchestrator.py:361-376 явно не выключает
  pipeline), боевые вызовы — POST и они проверены; переключателя отключения
  проверки в AVA нет.

## 2. Изменённые файлы — СОБСТВЕННЫЙ КОД (crates/mca-mail)

| Файл | Что |
|---|---|
| `src/voice/mod.rs` | Новый модуль канала: `InputRequirement`/`ValidatedRequirement`, `validate()`, `DEFAULT_SCOPE`, `parse_scope` |
| `src/voice/phone.rs` | `normalize_phone` (7–15 цифр), `voice_key = voice:<digits>` |
| `src/voice/fields.rs` | Русские подписи `label()` для всех 32 `RequirementField`, `sanitize()`/`sanitize_unit()` (пусто/длина/неотрицательность числовых) |
| `src/voice/handoff.rs` | `MANAGER_KEY_ALLOWLIST=["mca_manager"]`, `ensure_manager_key`, `build_handoff` → `Handoff` (cargo/route summary, missing labels, digest) |
| `src/persistence/voice_repo.rs` | `ensure_voice_lead`, `seed_scope`, `update_scope`, `save_requirements` (Known/1.0 → `requirement_repo::upsert_many`), `append_conversation` (идемпотентность), `conversation`, `status_of` |
| `src/api/routes/voice.rs` | Все 10 эндпоинтов `/api/v1/voice/*`; `transcript_direction()` маппит роли AVA `user/assistant` на `inbound/outbound` и пропускает служебные строки |
| `src/api/auth.rs` | `impl From<AuthError> for AppError` (UNAUTHORIZED/FORBIDDEN) |
| `src/lib.rs`, `src/api/mod.rs`, `src/api/routes/mod.rs`, `src/persistence/mod.rs` | Подключение модулей и `voice::routes()` |

Новый код вне коробки MCA Mail (развёртывание):

| Файл | Что |
|---|---|
| `MCA-phone/ava/` | Локальная копия AVA (18.3 MB, 989 файлов, без `.git`/`__pycache__`) — сборка идёт отсюда, код не правился |
| `MCA-phone/prod/ai-agent.yaml` | Персона MCA (RU), pipeline `mca_hybrid` → `openai_stt/openai_llm/openai_tts` (Polza), инструменты pre/post/in-call, `summary_provider: openai_llm` |
| `MCA-phone/prod/docker-compose.yml` | postgres + mca-mail + ai_engine + local_ai_server; host-network для AVA; `AVA_REPO` по умолчанию `../../ava`, `MCA_MAIL_REPO` |
| `MCA-phone/prod/.env.example` | Секреты и окружение (только плейсхолдеры), включая блок `POLZA_*` |
| `MCA-phone/prod/.env` | Локальный файл секретов (создан; реальный `POLZA_API_KEY` — **только здесь**, в git не попадает) |

Правка в несвязанном коде: **отсутствуют** — email-поток и бизнес-логика не тронуты.

## 3. Карта переиспользования

Не использован вслепую ни один кусок IMA/голоса — всё проверено чтением исходников.

**ПЕРЕИСПОЛЬЗОВАНО (AVA, без изменений; локальная копия `MCA-phone/ava/`):** `src/engine.py`, `src/tools/http/{in_call_lookup,generic_lookup,generic_webhook}.py`, `src/post_call_summary.py`, `src/pipelines/{orchestrator,openai}.py`, конфиг `config/ai-agent.*.yaml`, `Dockerfile`/`docker-compose.yml`. AVA загружает инструменты из `in_call_tools`/`tools` (pre_call/post_call) и подставляет `{caller_number}`, `pre_call_results` (оттуда `{mca_lead_id}`), плейсхолдеры payload (включая `{transcript_json}`, `{summary}`, `{mca_lead_id}` — flatten pre_call_results). Эти факты подтверждены по исходникам (см. приложение).

**ПЕРЕИСПОЛЬЗОВАНО (MCA Mail, без изменений):**
- `persistence/lead_repo.rs` — get/find_by_key/update_identity/update_status/update_summary/lock_automation/automation_locked;
- `persistence/requirement_repo.rs` — all/upsert_many; `persistence/conversation_repo.rs` — upsert; `persistence/handoff_repo.rs` — upsert_open; `persistence/api_key_repo.rs` — verify + Role;
- `domain/{lead,handoff}.rs` — Lead, LeadStatus(+automation_locked), ConversationDirection, OutboundState, HandoffReason, Priority, RequirementField/Scope (`applies_to`, `is_quote_blocking`), blocking/open gaps;
- `api/auth.rs` (AuthenticatedRequest, `X-API-Key`), `error/AppError`, `persistence/voice_repo` → те же таблицы (новая таблица не создана).

## 4. Поток звонка

1. **Входящий вызов** → Asterisk (dialplan) → ARI → `ai_engine` (AudioSocket, ulaw/8k).
2. **Pre-call** `mca_ensure_lead`: `POST /api/v1/voice/leads` `{caller_number}` — сервер идемпотентно создаёт/возвращает лид (source=api, status=new, scope=full_import, conversation_key=voice:<digits>). `mca_lead_id` сохраняется в `pre_call_results`.
3. **В разговоре** AI вызывает in-call-инструменты:
   - `mca_get_requirements` (`GET .../requirements`) — видит blocking/open gaps;
   - `mca_save_facts` (`PUT .../requirements`, JSON-массив фактов) — сервер валидирует поля/значения и upsert'ит (Known, confidence 1.0);
   - `mca_qualify` (`POST .../qualify`) — сервер решает qualified/awaiting_customer по `blocking_gaps`;
   - `mca_transfer_to_manager` (`POST .../handoff`, `manager_key` из allowlist) — сервер создаёт `Handoff` через `upsert_open` и блокирует автоматизацию (`Conflict` при следующих записях).
4. **Post-call** `mca_finish_call`: `POST /api/v1/voice/calls/finish` с `{lead_id, caller_number, call_id, outcome, summary, transcript}`. Сервер маппит роли user/assistant на inbound/outbound, пишет транскрипт идемпотентно (`voice:<call_key>:<index>`), обновляет summary.

## 5. Map инструмент → эндпоинт

| Инструмент AVA | Фаза | Эндпоинт MCA |
|---|---|---|
| `mca_ensure_lead` | pre_call | `POST /api/v1/voice/leads` |
| `mca_get_requirements` | in_call | `GET /api/v1/voice/leads/{id}/requirements` |
| `mca_save_facts` | in_call | `PUT /api/v1/voice/leads/{id}/requirements` |
| `mca_qualify` | in_call | `POST /api/v1/voice/leads/{id}/qualify` |
| `mca_transfer_to_manager` | in_call | `POST /api/v1/voice/leads/{id}/handoff` |
| `mca_finish_call` | post_call | `POST /api/v1/voice/calls/finish` |
| (операторы) | — | `PATCH /leads/{id}`, `GET/POST /leads/{id}/conversation`, `GET /calls/lookup?phone=` |

Все конечные точки требуют `X-API-Key` и роль не ниже **operator**.

## 6. Жёсткие правила и authority

- Lead ID назначает только сервер; повторные вызовы идемпотентны (get_or_create, upsert, ключи идемпотентности).
- Квалификация и полнота данных — только на сервере (`blocking_gaps`). AI только предлагает.
- После handoff автоматизация заблокирована (`automation_locked()`, ответ `409 Conflict`).
- Менеджер-адресат выбирается только allowlist-ключом `mca_manager`, а не LLM.
- Валидация: вес/объём/кол-во/стоимость неотрицательны; неизвестные scope/status/reason/priority → ошибка; некорректный lead id → 404; пустой/невалидный ключ API → 401.
- Никаких выдуманных цен/тарифов/техники/таможни/юруслуг (запрет в прoмпте и в описаниях инструментов).

## 7. Тесты и гейты

| Гейт | Результат |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `cargo check --workspace --all-targets` | ✅ |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ |
| `cargo test --workspace` (с `MCA_TEST_DATABASE_URL`, живой Postgres) | ✅ **368 тестов**: 280 unit + 88 интеграционных, включая 20 обязательных сценариев `tests/scenarios.rs` (§8) и `tests/voice_contract.rs` (§5+§6) |
| `docker compose config --quiet` (корень `MCA-Workspace`, MCA-Mail prod+dev, MCA-phone prod) | ✅ |
| YAML `prod/ai-agent.yaml` (PyYAML) | ✅ 19 секций |
| `docker compose config --quiet` (prod, с реальным `prod/.env`) | ✅ |
| GATE1: env-экспansion + YAML `prod/ai-agent.yaml` собственным лоадером AVA | ✅ (нет нераскрытых `${…}`; pipeline/providers/tools сверены, включая `summary_provider`) |
| GATE2: гидрация `OpenAIProviderConfig(**payload)` для трёх компонентов + роль-парсинг ключей | ✅ (pydantic-гидрация, base_url/модели/голос) |
| Живые пробы Polza (GET/POST с ключом) | ✅ models-list, chat (+tools, +temperature), transcriptions (форма wav, `language=ru`), speech (wav) |
| AVA engine `load_config` на Windows | ⛔ невозможно: `fcntl` (Unix-only); AVA `src.pipelines` additionally требует `audioop` (удалён в py3.13) — обе зависимости есть в Linux-контейнере; полная валидация произойдёт при старте контейнера |
| AVA `pytest` | не запускался: требует Linux + окружение; AVA используется копией без правок кода |

## 8. Осталось для прода (операционные шаги)

Голосовой стек **развёрнут и проверен сквозными SIP-звонками** (мини-SIP-клиент с сервера: INVITE 9999 → Stasis → pre-call `mca_ensure_lead` 200 → TTS-приветствие через Polza дошло абоненту (≈7 с RTP) → post-call `mca_finish_call` 200 → транскрипт/summary в PostgreSQL):

- Asterisk 20.6 (apt) на сервере: ARI 127.0.0.1:8088, SIP 0.0.0.0:5060, RTP 10000-20000; dialplan `from-internal`: 9999 = Stasis(mca-phone-voice-agent) c `Set(AI_AGENT=mca)`, 6000 = менеджер; `from-trunk` готов для боевого транка. Тестовые ext: 7000/6000.
- ai_engine (host network) подключён к ARI (`ari_connected: true`); `audiosocket.format: slin` (серверный AudioSocket шлёт только signed-linear; `ulaw`-outbound отвергается — было исправлено); контекст `mca` в `contexts` yaml обязателен — без него движок пропускает pre-call tools и не резолвит in-call tools (`context_name` пустой → `ctx_config is None`).
- `MCA_API_KEY` = `voice-engine` (операторский ключ в БД mca-mail; `MINT: docker exec mca-phone-mca-mail-1 /app/mca-mail api-key create --name voice-engine --role operator`). Значение держать синхронизированным: локальный `prod/.env` → секрет `PROD_ENV` (оба репо) → сервер (деплой перезаписывает `.env` из PROD_ENV!). `gh secret set` с `-f` может молча не обновляться — проверять `gh secret list` по timestamp.
- mca-mail принимает AVA-transcript в формате `role`/`content` (serde-алиасы на `direction`/`body`; маппинг `user→inbound`, `assistant→outbound` на сервере).
- Входящие RTP от «абонента» в тестах были тишиной (`media_rx_confirmed: false` — ожидаемо для тишины); STT/диалог проверяется реальной речью.

Осталось:

1. **Тест реальной речью**: софтфон (Zoiper и т.п.) → регистрация ext 7000 на `178.212.15.6:5060` (пароль в `.env`/pjsip.conf), дозвон 9999; открыть/настроить на файрволе UDP 5060 + RTP 10000-20000 (риск SIP-фрод — ограничить по IP). Проверить: распознавание речи, инструменты LLM (`mca_get_requirements` и др.), квалификацию, handoff.
2. **SIP-транк для боевых входящих номеров**: контекст `from-trunk` и `Set(AI_AGENT=mca)` уже в dialplan; нужен аккаунт телепровайдера (операционный шаг).
3. (Опционально) Admin UI AVA для мониторинга; reverse-proxy/TLS перед 8080, если доступ не через SSH-туннель. Ожидаемо: warning `Active pipeline validation FAILED` на старте ai_engine (GET-пробы Polza отдают 404) — звонки это не блокирует, см. §1.1.

## 9. Учётные данные и секреты

- Никаких секретов в yaml/исходниках: `prod/.env.example` — только плейсхолдеры. **`prod/.env` содержит реальные секреты** — создаётся/держится локально, в git не попадает (`.gitignore`); зеркало для деплоя — GitHub-секрет `PROD_ENV` (оба репо).
- API-ключ mca-mail (роль operator/manager) задаётся в `MCA_API_KEY`; `MCA_MANAGER_KEY=mca_manager` (allowlist на сервере); `POSTGRES_PASSWORD` и `JWT_SECRET` — свои.
- Порт 8080 опубликован на 127.0.0.1; Postgres — только внутри стека (127.0.0.1:5432).
- `X-API-Key` в инструментах AVA подставляется из `.env` на этапе загрузки конфига (`${ENV}`-экспansion до YAML-парсинга); ключи Polza в yaml только как `${POLZA_API_KEY:-}`.

## 10. СВОЙ КОД / ПЕРЕИСПОЛЬЗОВАНО / ПЕРЕИСПОЛЬЗОВАНО MCA

- **СВОЙ КОД (минимальный):** `src/voice/{mod,phone,fields,handoff}.rs`, `src/persistence/voice_repo.rs`, `src/api/routes/voice.rs` (+ `From<AuthError> for AppError` и 5 строк на подключение модулей); `prod/{ai-agent.yaml,docker-compose.yml,.env.example}` — конфигурация, не код AVA.
- **ПЕРЕИСПОЛЬЗОВАНО (без изменений):** AVA целиком — локальная копия `MCA-phone/ava/` (скопирована из временного чекаута, содержимое не редактировалось, `Dockerfile` AVA собирается как есть); Python `aiohttp/websockets/pydantic` — зависимости AVA.
- **ПЕРЕИСПОЛЬЗОВАНО MCA (без изменений):** `lead_repo`, `requirement_repo`, `conversation_repo`, `handoff_repo`, `api_key_repo`, `domain::{lead,handoff}`, `api/auth`, `error/AppError`; существующая схема БД (миграций на голосовой канал не добавлялось).

## Приложение: подтверждённые факты о AVA (по исходникам)

- `src/config/loaders.py::resolve_config_path/load_yaml_with_env_expansion` — путь `config/ai-agent.yaml` (в контейнере `/app/config/ai-agent.yaml`), `${ENV}`-подстановка до YAML-парсинга; `src/config.py::load_config` (line 1623) — ключ `in_call_tools` (deprecated `in_call_http_tools` — мигрируется).
- `src/tools/registry.py::initialize_in_call_http_tools_from_config` (line 702) — тип `kind: in_call_http_lookup`; `generic_http_lookup`/`generic_webhook` на pre/post (line 662-691).
- `src/tools/http/in_call_lookup.py` — sub-контекст: `{caller_number,called_number,caller_name,context_name,call_id}` + `pre_call_results` + AI-параметры (lines 442-502); выходные `output_variables` — dot-path. `body_template` → json.loads → `json=`; `X-API-Key` в заголовках.
- `src/tools/http/generic_lookup.py` — pre-call, `method POST` + `body_template` (received `data=body`; Content-Type задаётся header'ом), выход в `session.pre_call_results`.
- `src/tools/http/generic_webhook.py` — `phase: post_call`, `_build_payload` (line 463): подстановка `{...}` из `to_payload_dict()` → `src/tools/context.py` (lines 311-358): flatten `pre_call_results` в плейсхолдеры (`{mca_lead_id}`), built-in не перезаписываются; `${ENV}` — «json-escaped»; `generate_summary`: при `summary_provider` — `context.summary_generator` (engine.py:22800 → `PostCallSummaryService.generate` → `orchestrator.generate_once(component_key=…)`), без него — legacy `OPENAI_API_KEY`/gpt-4o-mini (generic_webhook.py:591-601).
- Регистрация компонентов: `_extract_role` требует ключ вида `<provider>_<role>` (orchestrator.py:164); openai-адаптеры регистрируются **только** под каноническими `openai_{stt,llm,tts}` (`_hydrate_openai_component_configs`, :1599-1623) и по имени роли для `*_llm` (`_register_configured_llm_factories`, :900); wildcard `*_{role}` — placeholder → Phase-1 ошибка старта (:1997-2010, :329-345).
- Приоритет опций: runtime → pipeline `options` → provider (`_compose_options` каждого адаптера) — поэтому в `pipelines.mca_hybrid.options.llm` убраны `base_url`/`model` (иначе перебили бы Polza).
- Guard-ы адаптеров openai.py: STT :187-201 (`"/"`/`whisper-large-` → откат на provider `stt_model`), TTS :973-982 (`"/"` → откат на provider `tts_model`), LLM :833-847 (только groq-host/`llama-`/`mixtral`); `validate_connectivity` LLM :438-440 принимает модели с префиксом `o`/`gpt-`/`chatgpt` — `openai/gpt-6-luna` проходит.
- Стартовая проверка connectivy (`_validate_pipeline_connectivity`): GET на endpoint'ы с Bearer; 404 → `healthy: false`, но **pipeline не выключается** (комментарий orchestrator.py:361-376); гейт готовности используется только в health/status-снимках (engine.py:23038-23049).