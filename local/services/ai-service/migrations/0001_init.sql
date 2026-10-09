-- ai-service bootstrap schema.
-- The dialogue engine owns typed aggregates; each row stores that aggregate as
-- a JSON document (`payload`) plus the minimal scalar columns the API actually
-- queries on. Column drift is impossible: the engine never splits an entity
-- across columns it does not read back.

create table if not exists sessions (
    id uuid primary key,
    call_id uuid not null,
    external_call_id text,
    caller_phone text,
    payload jsonb not null default '{}'::jsonb,
    started_at timestamptz not null default now(),
    ended_at timestamptz
);

create table if not exists messages (
    id uuid primary key,
    session_id uuid not null references sessions (id) on delete cascade,
    payload jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create table if not exists applications (
    id uuid primary key,
    session_id uuid not null references sessions (id) on delete cascade,
    call_id uuid not null,
    payload jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table if not exists emails (
    id uuid primary key,
    application_id uuid not null references applications (id) on delete cascade,
    to_address text not null,
    subject text not null,
    payload jsonb not null default '{}'::jsonb,
    generated_at timestamptz not null default now()
);

create index if not exists idx_messages_session on messages (session_id, created_at);
create index if not exists idx_sessions_external on sessions (external_call_id);
create index if not exists idx_applications_session on applications (session_id);
create index if not exists idx_emails_application on emails (application_id);