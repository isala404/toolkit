use super::model::FCMSchedule;
use chrono::Utc;
use cron_parser::parse;
use gcp_auth::{AuthenticationManager, CustomServiceAccount, Error};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use sqlx::postgres::PgPool;
use std::{collections::HashMap, fs, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

// https://firebase.google.com/docs/cloud-messaging/concept-options#notification-messages-with-optional-data-payload
#[derive(Debug, Serialize, Deserialize)]
struct Fcm {
    message: FCMBody,
}

#[derive(Debug, Serialize, Deserialize)]
struct Notification {
    title: Option<String>,
    body: Option<String>,
}

impl Notification {
    fn is_empty(&self) -> bool {
        self.title.is_none() && self.body.is_none()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FCMBody {
    #[serde(skip_serializing_if = "Notification::is_empty")]
    notification: Notification,
    data: HashMap<String, String>,
    token: String,
}

const SCOPES: &[&str; 1] = &["https://www.googleapis.com/auth/firebase.messaging"];

pub async fn read_in_serivce_accounts() -> Result<HashMap<String, AuthenticationManager>, Error> {
    info!("Reading in service accounts");

    let dir_entries = fs::read_dir("service_accounts")?;
    let mut service_accounts = HashMap::new();
    for entry in dir_entries {
        let entry = entry?;
        // let file_name = entry.file_name();
        let file_path = entry.path();

        if file_path.is_file() {
            // check if file is a json file
            if file_path.extension() != Some("json".as_ref()) {
                continue;
            }

            // Do something with the file
            debug!(file_path = ?file_path, "Found service account file");

            let credentials_path = file_path;
            let service_account = CustomServiceAccount::from_file(credentials_path)?;
            let authentication_manager = AuthenticationManager::try_from(service_account)
                .expect("Error creating authentication manager");
            let project_name = authentication_manager.project_id().await?;
            service_accounts.insert(project_name, authentication_manager);
        }
    }

    Ok(service_accounts)
}

pub async fn run_every_minute(
    auth_managers: HashMap<String, AuthenticationManager>,
    pool: &PgPool,
) {
    loop {
        let current_time = Utc::now().naive_local();

        let messages = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE next_execution < NOW()"
        )
        .fetch_all(pool)
        .await
        .unwrap_or_else(|_| vec![]);

        info!(message_count = messages.len(), "Found messages to process");

        for message in messages {
            debug!(message = ?message, "Processing message");

            let project_id = message.fb_project_id.to_owned();

            let auth_manager = match auth_managers.get(&project_id) {
                Some(auth_manager) => auth_manager,
                None => {
                    warn!(project_id = ?project_id, message_id=?message.id, "No auth manager found for project id");
                    continue;
                }
            };

            let token = match auth_manager.get_token(SCOPES).await {
                Ok(token) => token,
                Err(e) => {
                    error!(project_id = ?project_id, message_id=?message.id, error=?e, "Error getting token");
                    continue;
                }
            };

            let mut payload: HashMap<String, String> = HashMap::new();
            match message.payload {
                Value::Object(map) => {
                    for (key, value) in map {
                        payload.insert(key, value.to_string());
                    }
                }
                Value::String(s) => {
                    payload = from_str::<HashMap<String, String>>(&s).unwrap_or({
                        warn!(project_id = ?project_id, message_id=?message.id, payload=&s, "Error parsing payload, defaulting to empty hashmap");
                        HashMap::new()
                    });
                }
                _ => {}
            }

            let notification = Notification {
                title: payload.remove("title"),
                body: payload.remove("body"),
            };

            let firebase_message = Fcm {
                message: FCMBody {
                    notification,
                    data: payload,
                    token: message.push_token.to_owned(),
                },
            };

            let header = match format!("Bearer {}", token.as_str()).parse() {
                Ok(header) => header,
                Err(e) => {
                    error!(project_id = ?project_id, message_id=?message.id, error=?e, "Error parsing header");
                    continue;
                }
            };

            // Create the authorization header with the token
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, header);

            let endpoint = format!(
                "https://fcm.googleapis.com/v1/projects/{}/messages:send",
                project_id
            );

            // Send the HTTP POST request
            let client = reqwest::Client::new();
            let response = client
                .post(endpoint)
                .headers(headers)
                .json(&firebase_message)
                .send()
                .await;

            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!(project_id = ?project_id, message_id=?message.id, "Successfully sent request")
                    } else {
                        let resp = response.text().await;
                        warn!(project_id = ?project_id, message_id=?message.id, response=?resp, "Error sending request");
                    }
                }
                Err(e) => {
                    error!(project_id = ?project_id, message_id=?message.id, error=?e, "Error sending request");
                    continue;
                }
            }

            // Update the next execution time
            let next = match parse(&message.cron_pattern.to_owned(), &Utc::now()) {
                Ok(next) => next.naive_utc(),
                Err(e) => {
                    println!("Error parsing cron pattern: {}", e);
                    error!(project_id = ?project_id, message_id=?message.id, error=?e, "Error parsing cron pattern");
                    continue;
                }
            };

            // Update database
            let result = sqlx::query!(
                r#"UPDATE fcm_schedule SET next_execution = $1, last_execution = $2, updated_at = $3 WHERE id = $4"#,
                next,
                current_time,
                current_time,
                message.id,
            ).execute(pool).await;

            match result {
                Ok(data) => println!(
                    "Successfully updated next execution time: {:?}, rows affected: {}",
                    next,
                    data.rows_affected()
                ),
                Err(e) => println!("Error updating next execution time: {}", e),
            }
        }

        // Sleep for 1 minute
        sleep(Duration::from_secs(60)).await;
    }
}
