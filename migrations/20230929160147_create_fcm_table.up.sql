CREATE TABLE fcm_schedule (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    fb_user_id TEXT NOT NULL,
    fb_project_id TEXT NOT NULL,
    push_token TEXT NOT NULL,
    cron_pattern TEXT NOT NULL,
    payload JSONB NOT NULL,
    last_execution TIMESTAMP NOT NULL,
    next_execution TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
