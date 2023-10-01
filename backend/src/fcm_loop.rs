use crate::fcm_api::FCMSchedule;
use chrono::Utc;
use cron_parser::parse;
use gcp_auth::{AuthenticationManager, CustomServiceAccount, Error};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Map, Value};
use sqlx::SqlitePool;
use std::{collections::HashMap, fs, path::PathBuf, time::Duration};
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
struct FCM {
    message: FCMBody,
}

#[derive(Debug, Serialize, Deserialize)]
struct FCMBody {
    // name: String,
    data: Map<String, Value>,
    token: String,
}

const SCOPES: &[&str; 1] = &["https://www.googleapis.com/auth/firebase.messaging"];

pub async fn read_in_serivce_accounts() -> Result<HashMap<String, AuthenticationManager>, Error> {
    let dir_entries = fs::read_dir("service_accounts")?;

    let mut service_accounts = HashMap::new();

    for entry in dir_entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_path = entry.path();

        if file_path.is_file() {
            // check if file is a json file
            if file_path.extension() != Some("json".as_ref()) {
                continue;
            }

            // Do something with the file
            println!("Processing file: {:?}", file_name);

            let credentials_path = PathBuf::from(file_path);
            let service_account = CustomServiceAccount::from_file(credentials_path)?;
            let authentication_manager = AuthenticationManager::from(service_account);
            let project_name = authentication_manager.project_id().await?;
            service_accounts.insert(project_name, authentication_manager);
        }
    }

    Ok(service_accounts)
}

pub async fn run_every_minute(
    auth_managers: HashMap<String, AuthenticationManager>,
    pool: &SqlitePool,
) {
    loop {
        let current_time = Utc::now().naive_local();
        println!("Polling for messages to send at: {:?}", current_time);
    
        let messages = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE next_execution < datetime('now')"
        )
        .fetch_all(pool)
        .await
        .unwrap_or_else(|_| vec![]);

        for message in messages {
            println!("Processing message: {:?}", message.clone());

            let project_id = message.fb_project_id.to_owned();

            let auth_manager = match auth_managers.get(&project_id) {
                Some(auth_manager) => auth_manager,
                None => {
                    println!("No auth manager found for project id: {}", project_id);
                    continue;
                }
            };

            let token = match auth_manager.get_token(SCOPES).await {
                Ok(token) => token,
                Err(e) => {
                    println!("Error getting token: {}", e);
                    continue;
                }
            };

            let mut payload = Map::new();
            match message.payload {
                Value::Object(map) => {
                    for (key, value) in map {
                        payload.insert(key, value);
                    }
                }
                Value::String(s) => {
                    payload = from_str::<Map<String, Value>>(&s).unwrap_or_default();
                }
                _ => {}
            }

            let firebase_message = FCM {
                message: FCMBody {
                    // name: message.name.to_owned(),
                    data: payload,
                    token: message.push_token.to_owned(),
                },
            };

            let header = match format!("Bearer {}", token.as_str()).parse() {
                Ok(header) => header,
                Err(e) => {
                    println!("Error parsing header: {}", e);
                    continue;
                }
            };

            // Create the authorization header with the token
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                header,
            );

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
                        println!("Successfully sent request");
                    } else {
                        println!("Error sending request, response: {:?}", response.text().await);
                    }
                }
                Err(e) => {
                    println!("Error sending request: {}", e);
                    continue;
                }
            }

            // Update the next execution time
            let next = match parse(&message.cron_pattern.to_owned(), &Utc::now()) {
                Ok(next) => next.naive_utc(),
                Err(e) => {
                    println!("Error parsing cron pattern: {}", e);
                    continue;
                }
            };

            // Update database
            let result = sqlx::query!(
                r#"UPDATE fcm_schedule SET next_execution = ?, last_execution = ?, updated_at = ? WHERE id = ?"#,
                next,
                current_time,
                current_time,
                message.id,
            ).execute(pool).await;

            match result {
                Ok(data) => println!("Successfully updated next execution time: {:?}, rows affected: {}", next, data.rows_affected()),
                Err(e) => println!("Error updating next execution time: {}", e),
            }
        }

        // Sleep for 1 minute
        sleep(Duration::from_secs(60)).await;
    }
}
