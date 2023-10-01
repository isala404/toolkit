CREATE TABLE fcm_schedule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    fb_user_id TEXT NOT NULL,
    fb_project_id TEXT NOT NULL,
    push_token TEXT NOT NULL,
    cron_pattern TEXT NOT NULL,
    payload TEXT NOT NULL,
    last_execution DATETIME NOT NULL,
    next_execution DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
