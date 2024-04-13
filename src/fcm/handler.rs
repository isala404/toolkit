use super::model::{FCMSchedule, UpdateSchedule};
use super::utils::{decode_cron, extract_claims};
use crate::utils::{ApiTags, JsonError, JsonSuccess, ResponseObject};
use chrono::Utc;
use poem::{web::Data, Request};
use poem_openapi::param::Path;
use poem_openapi::{payload::Json, OpenApi};
use serde_json::Value;
use sqlx::postgres::PgPool;

pub struct FirebaseMessaging {
    pub projects: Vec<String>,
}

#[OpenApi(
    prefix_path = "/fcm/",
    request_header(
        name = "firebase-auth",
        ty = "String",
        description = "Bearer token generated from firebase project (example: <code>Bearer {token}</code>)"
    ),
    tag = "ApiTags::FirebaseMessaging"
)]
impl FirebaseMessaging {
    // create new instance
    pub fn new(projects: Vec<String>) -> Self {
        Self { projects }
    }

    // create schedule
    #[oai(path = "/", method = "post", operation_id = "fcm::create_schedule")]
    async fn create_schedule(
        &self,
        req: &Request,
        pool: Data<&PgPool>,
        payload: Json<FCMSchedule>,
    ) -> Result<JsonSuccess<FCMSchedule>, JsonError<String>> {
        // extract user id from token
        let data = match extract_claims(req.header("firebase-auth")) {
            Ok(data) => data,
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;
        let fb_project_id = data.aud;

        if !self.projects.contains(&fb_project_id) {
            return Err(ResponseObject::unauthorized("Invalid project id"));
        }

        // validate payload
        match payload.payload {
            Value::Object(_) => {}
            _ => {
                return Err(ResponseObject::bad_request("Invalid payload"));
            }
        }

        let next_execution = match decode_cron(&payload.cron_pattern.as_ref()) {
            Ok(next) => next,
            Err(e) => {
                return Err(ResponseObject::bad_request(e));
            }
        };

        let current_time = Utc::now().naive_local();

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "INSERT INTO fcm_schedule (
                name, fb_user_id, push_token, fb_project_id, cron_pattern, payload, last_execution, next_execution, created_at, updated_at
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *",
            payload.name,
            fb_user_id,
            payload.push_token,
            fb_project_id,
            payload.cron_pattern,
            payload.payload,
            current_time,
            next_execution,
            current_time,
            current_time
        )
        .fetch_one(pool.0)
        .await;

        let schedule = match schedule {
            Ok(schedule) => schedule,
            Err(e) => {
                return Err(ResponseObject::internal_server_error(e));
            }
        };

        Ok(ResponseObject::created(schedule))
    }

    // find all schedules for the user
    #[oai(path = "/", method = "get", operation_id = "fcm::find_all_schedules")]
    async fn find_all_schedules(
        &self,
        req: &Request,
        pool: Data<&PgPool>,
    ) -> Result<JsonSuccess<Vec<FCMSchedule>>, JsonError<String>> {
        // extract user id from token
        let data = match extract_claims(req.header("firebase-auth")) {
            Ok(data) => data,
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;

        let schedules = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE fb_user_id = $1",
            fb_user_id
        )
        .fetch_all(pool.0)
        .await;

        let schedules = match schedules {
            Ok(schedules) => schedules,
            Err(e) => {
                return Err(ResponseObject::internal_server_error(e));
            }
        };

        Ok(ResponseObject::ok(schedules))
    }

    // Delete schedule by id (only if it belongs to the user)
    #[oai(
        path = "/:id",
        method = "delete",
        operation_id = "fcm::delete_schedule"
    )]
    async fn delete_schedule(
        &self,
        req: &Request,
        pool: Data<&PgPool>,
        id: Path<i32>,
    ) -> Result<JsonSuccess<FCMSchedule>, JsonError<String>> {
        // extract user id from token
        let data = match extract_claims(req.header("firebase-auth")) {
            Ok(data) => data,
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE id = $1 AND fb_user_id = $2",
            id.0,
            fb_user_id
        )
        .fetch_one(pool.0)
        .await;

        let schedule = match schedule {
            Ok(schedule) => schedule,
            Err(_) => {
                return Err(ResponseObject::not_found("Schedule not found"));
            }
        };

        let result = sqlx::query!(
            "DELETE FROM fcm_schedule WHERE id = $1 AND fb_user_id = $2",
            id.0,
            fb_user_id
        )
        .execute(pool.0)
        .await;

        let result = match result {
            Ok(result) => result,
            Err(e) => {
                return Err(ResponseObject::internal_server_error(e));
            }
        };

        if result.rows_affected() == 0 {
            return Err(ResponseObject::not_found("Schedule not found"));
        }

        Ok(ResponseObject::ok(schedule))
    }

    // Update schedule by id (only if it belongs to the user)
    #[oai(path = "/:id", method = "put", operation_id = "fcm::update_schedule")]
    async fn update_schedule(
        &self,
        req: &Request,
        pool: Data<&PgPool>,
        id: Path<i32>,
        payload: Json<UpdateSchedule>,
    ) -> Result<JsonSuccess<FCMSchedule>, JsonError<String>> {
        // extract user id from token
        let data = match extract_claims(req.header("firebase-auth")) {
            Ok(data) => data,
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE id = $1 AND fb_user_id = $2",
            id.0,
            fb_user_id
        )
        .fetch_one(pool.0)
        .await;

        let _ = match schedule {
            Ok(schedule) => schedule,
            Err(_) => {
                return Err(ResponseObject::not_found("Schedule not found"));
            }
        };

        match payload.payload {
            Value::Object(_) => {}
            _ => {
                return Err(ResponseObject::bad_request("Invalid payload"));
            }
        }

        let next_execution = match decode_cron(&payload.cron_pattern) {
            Ok(next) => next,
            Err(e) => {
                return Err(ResponseObject::bad_request(e));
            }
        };

        let current_time = Utc::now().naive_local();

        let result = sqlx::query!(
            "UPDATE fcm_schedule SET name = $1, push_token = $2, cron_pattern = $3, payload = $4, next_execution = $5, updated_at = $6 WHERE id = $7 AND fb_user_id = $8",
            payload.name,
            payload.push_token,
            payload.cron_pattern,
            payload.payload,
            next_execution,
            current_time,
            id.0,
            fb_user_id
        )
        .execute(pool.0)
        .await;

        let result = match result {
            Ok(result) => result,
            Err(e) => {
                return Err(ResponseObject::internal_server_error(e));
            }
        };

        if result.rows_affected() == 0 {
            return Err(ResponseObject::not_found("Schedule not found"));
        }

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE id = $1 AND fb_user_id = $2",
            id.0,
            fb_user_id
        )
        .fetch_one(pool.0)
        .await;

        let schedule = match schedule {
            Ok(schedule) => schedule,
            Err(e) => {
                return Err(ResponseObject::internal_server_error(e));
            }
        };

        Ok(ResponseObject::ok(schedule))
    }
}
