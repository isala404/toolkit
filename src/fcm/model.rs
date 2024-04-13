use chrono::NaiveDateTime;
use poem_openapi::Object;
use serde::Serialize;
use serde_json::Value;

fn payload_example() -> Value {
    serde_json::from_str("{\"title\": \"Reminder\", \"body\": \"Drink water\", \"foo\": \"bar\"}")
        .unwrap()
}

fn cron_example() -> String {
    "*/1 * * * *".to_string()
}

fn name_example() -> String {
    "Remind me to drink water every 45 minutes".to_string()
}

/// Create FCM Schedule schema
#[derive(Debug, Object, Clone, Eq, PartialEq, Serialize)]
pub struct FCMSchedule {
    #[oai(read_only)]
    /// ID of the schedule
    pub id: i32,

    #[oai(validator(min_length = 3, max_length = 64), default = "name_example")]
    /// Friendly name of the schedule
    pub name: String,

    #[oai(read_only)]
    /// firebase user id (decoded from token)
    pub fb_user_id: String,

    #[oai(validator(min_length = 32, max_length = 512))]
    /// device registration token to send the FCM (https://firebase.google.com/docs/cloud-messaging/manage-tokens)
    pub push_token: String,

    #[oai(read_only)]
    /// firebase project id (decoded from token)
    pub fb_project_id: String,

    #[oai(validator(min_length = 3, max_length = 64), default = "cron_example")]
    /// cron pattern to schedule the FCM (support multiple cron patterns separated by comma)
    pub cron_pattern: String,

    /// payload to send to the FCM (JSON) e.g. {"some": "data", "another": "data"}
    /// If title and body are present, they will be used as notification
    #[oai(default = "payload_example")]
    pub payload: Value,

    #[oai(read_only)]
    /// last time the FCM was sent
    pub last_execution: NaiveDateTime,

    #[oai(read_only)]
    /// next time the FCM will be sent
    pub next_execution: NaiveDateTime,

    #[oai(read_only)]
    /// created time of the schedule
    pub created_at: NaiveDateTime,

    #[oai(read_only)]
    /// last time the schedule was updated
    pub updated_at: NaiveDateTime,
}

/// Update FCM Schedule schema
#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct UpdateSchedule {
    #[oai(validator(min_length = 3, max_length = 64), default = "name_example")]
    /// Friendly name of the schedule
    pub name: String,

    #[oai(validator(min_length = 32, max_length = 512))]
    /// device registration token to send the FCM (https://firebase.google.com/docs/cloud-messaging/manage-tokens)
    pub push_token: String,

    #[oai(validator(min_length = 3, max_length = 64), default = "cron_example")]
    /// cron pattern to schedule the FCM (support multiple cron patterns separated by comma)
    pub cron_pattern: String,

    /// payload to send to the FCM (JSON) e.g. {"some": "data", "another": "data"}
    /// If title and body are present, they will be used as notification
    #[oai(default = "payload_example")]
    pub payload: Value,
}
