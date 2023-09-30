CREATE TABLE fcm_schedule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    fb_user_id TEXT,
    fb_project_id TEXT,
    push_token TEXT,
    cron_pattern TEXT,
    payload TEXT,
    last_execution DATETIME,
    next_execution DATETIME,
    created_at DATETIME,
    updated_at DATETIME
);
