use chrono::{NaiveDateTime, Utc};
use cron_parser::parse;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use poem::{error::InternalServerError, web::Data, Result};
use poem_openapi::{
    auth::Bearer,
    param::Path,
    payload::Json,
    types::{ParseFromJSON, ToJSON},
    ApiResponse, Object, OpenApi, SecurityScheme,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

/// Firebase Auth
#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
struct FirebaseAuth(Bearer);

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    aud: String,
    user_id: String,
}

/// Success response
#[derive(Object, Serialize)]
struct SuccessResponse<T: ParseFromJSON + ToJSON> {
    /// Success
    data: T,
}

/// Error response
#[derive(Object, Serialize)]
struct ErrorResponse {
    /// Error
    error: String,
}

/// Create FCMSchedule schema
#[derive(Debug, Object, Clone, Eq, PartialEq, Serialize)]
struct FCMSchedule {
    #[oai(read_only)]
    id: i64,
    #[oai(validator(min_length = 3, max_length = 64))]
    name: Option<String>,
    #[oai(read_only)]
    fb_user_id: Option<String>,
    #[oai(validator(min_length = 32, max_length = 128))]
    push_token: Option<String>,
    #[oai(read_only)]
    fb_project_id: Option<String>,
    #[oai(validator(min_length = 3, max_length = 64))]
    cron_pattern: Option<String>,
    payload: Value,
    #[oai(read_only)]
    last_execution: Option<NaiveDateTime>,
    #[oai(read_only)]
    next_execution: Option<NaiveDateTime>,
    #[oai(read_only)]
    created_at: Option<NaiveDateTime>,
    #[oai(read_only)]
    updated_at: Option<NaiveDateTime>,
}

/// Update Schedule schema
#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct UpdateSchedule {
    /// Name
    name: Option<String>,
    /// firebase user id
    fb_user_id: Option<String>,
    /// firebase project id
    fb_project_id: Option<String>,
    /// firebase push token
    push_token: Option<String>,
    /// cron pattern
    cron_pattern: Option<String>,
    /// payload
    payload: Option<Value>,
}

#[derive(ApiResponse)]
enum CreateScheduleResponse {
    #[oai(status = 200)]
    Ok(Json<SuccessResponse<FCMSchedule>>),
    #[oai(status = 401)]
    Unauthorized(Json<ErrorResponse>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
}

#[derive(ApiResponse)]
enum FindScheduleResponse {
    #[oai(status = 200)]
    Ok(Json<SuccessResponse<Vec<FCMSchedule>>>),
    #[oai(status = 401)]
    Unauthorized(Json<ErrorResponse>),
}

#[derive(ApiResponse)]
enum DeleteScheduleResponse {
    #[oai(status = 200)]
    Ok(Json<SuccessResponse<FCMSchedule>>),
    #[oai(status = 401)]
    Unauthorized(Json<ErrorResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

#[derive(ApiResponse)]
enum UpdateScheduleResponse {
    #[oai(status = 200)]
    Ok(Json<SuccessResponse<FCMSchedule>>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 401)]
    Unauthorized(Json<ErrorResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

#[derive(Default)]
pub struct FCMAPI;
#[OpenApi]
impl FCMAPI {
    // create schedule
    #[oai(path = "/fcm/", method = "post")]
    async fn create_schedule(
        &self,
        pool: Data<&SqlitePool>,
        auth: FirebaseAuth,
        payload: Json<FCMSchedule>,
    ) -> Result<CreateScheduleResponse> {
        // extract user id from token
        let data = match extract_claims(&auth.0.token) {
            Ok(data) => data,
            Err(e) => {
                return Ok(CreateScheduleResponse::Unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;
        let fb_project_id = data.aud;

        let next_execution = match decode_cron(&payload.cron_pattern.as_ref().unwrap()) {
            Ok(next) => next,
            Err(e) => {
                return Ok(CreateScheduleResponse::BadRequest(e));
            }
        };

        let current_time = Utc::now().naive_local();

        let result = sqlx::query!(
            "INSERT INTO fcm_schedule (
                name, fb_user_id, push_token, fb_project_id, cron_pattern, payload, last_execution, next_execution, created_at, updated_at
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        ).execute(pool.0).await.map_err(InternalServerError)?.last_insert_rowid();

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE id = ?",
            result
        )
        .fetch_one(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(CreateScheduleResponse::Ok(Json(SuccessResponse {
            data: schedule,
        })))
    }

    // find all schedules for the user
    #[oai(path = "/fcm/", method = "get")]
    async fn find_all_schedules(
        &self,
        pool: Data<&SqlitePool>,
        auth: FirebaseAuth,
    ) -> Result<FindScheduleResponse> {
        // extract user id from token
        let data = match extract_claims(&auth.0.token) {
            Ok(data) => data,
            Err(e) => {
                return Ok(FindScheduleResponse::Unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;

        let schedules = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE fb_user_id = ?",
            fb_user_id
        )
        .fetch_all(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(FindScheduleResponse::Ok(Json(SuccessResponse {
            data: schedules,
        })))
    }

    // Delete schedule by id (only if it belongs to the user)
    #[oai(path = "/fcm/:id", method = "delete")]
    async fn delete_schedule(
        &self,
        pool: Data<&SqlitePool>,
        auth: FirebaseAuth,
        id: Path<i64>,
    ) -> Result<DeleteScheduleResponse> {
        // extract user id from token
        let data = match extract_claims(&auth.0.token) {
            Ok(data) => data,
            Err(e) => {
                return Ok(DeleteScheduleResponse::Unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE id = ? AND fb_user_id = ?",
            id.0,
            fb_user_id
        )
        .fetch_one(pool.0)
        .await;

        let schedule = match schedule {
            Ok(schedule) => schedule,
            Err(_) => {
                return Ok(DeleteScheduleResponse::NotFound(Json(ErrorResponse {
                    error: "Schedule not found".to_string(),
                })));
            }
        };

        let result = sqlx::query!(
            "DELETE FROM fcm_schedule WHERE id = ? AND fb_user_id = ?",
            id.0,
            fb_user_id
        )
        .execute(pool.0)
        .await
        .map_err(InternalServerError)?;

        if result.rows_affected() == 0 {
            return Ok(DeleteScheduleResponse::NotFound(Json(ErrorResponse {
                error: "Schedule not found".to_string(),
            })));
        }

        Ok(DeleteScheduleResponse::Ok(Json(SuccessResponse {
            data: schedule,
        })))
    }

    // Update schedule by id (only if it belongs to the user)
    #[oai(path = "/fcm/:id", method = "put")]
    async fn update_schedule(
        &self,
        pool: Data<&SqlitePool>,
        auth: FirebaseAuth,
        id: Path<i64>,
        payload: Json<UpdateSchedule>,
    ) -> Result<UpdateScheduleResponse> {
        // extract user id from token
        let data = match extract_claims(&auth.0.token) {
            Ok(data) => data,
            Err(e) => {
                return Ok(UpdateScheduleResponse::Unauthorized(e));
            }
        };

        let fb_user_id = data.user_id;

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE id = ? AND fb_user_id = ?",
            id.0,
            fb_user_id
        )
        .fetch_one(pool.0)
        .await;

        let _ = match schedule {
            Ok(schedule) => schedule,
            Err(_) => {
                return Ok(UpdateScheduleResponse::NotFound(Json(ErrorResponse {
                    error: "Schedule not found".to_string(),
                })));
            }
        };

        let next_execution = match decode_cron(&payload.cron_pattern.as_ref().unwrap()) {
            Ok(next) => next,
            Err(e) => {
                return Ok(UpdateScheduleResponse::BadRequest(e));
            }
        };

        let current_time = Utc::now().naive_local();

        let result = sqlx::query!(
            "UPDATE fcm_schedule SET name = ?, push_token = ?, cron_pattern = ?, payload = ?, next_execution = ?, updated_at = ? WHERE id = ? AND fb_user_id = ?",
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
        .await
        .map_err(InternalServerError)?;

        if result.rows_affected() == 0 {
            return Ok(UpdateScheduleResponse::NotFound(Json(ErrorResponse {
                error: "Schedule not found".to_string(),
            })));
        }

        let schedule = sqlx::query_as!(
            FCMSchedule,
            "SELECT * FROM fcm_schedule WHERE id = ? AND fb_user_id = ?",
            id.0,
            fb_user_id
        )
        .fetch_one(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(UpdateScheduleResponse::Ok(Json(SuccessResponse {
            data: schedule,
        })))
    }
}

fn extract_claims(token: &str) -> Result<Claims, Json<ErrorResponse>> {
    // extract user id from token
    let key = DecodingKey::from_secret(&[]);
    let mut validation = Validation::new(Algorithm::HS256);
    validation.insecure_disable_signature_validation();

    match decode::<Claims>(&token, &key, &validation) {
        Ok(data) => Ok(data.claims), // Return the data in the Ok variant
        Err(_) => Err(Json(ErrorResponse {
            error: "Unauthorized".to_string(),
        })), // Return an ErrorResponse in the Err variant
    }
}

fn decode_cron(cron_pattern: &str) -> Result<NaiveDateTime, Json<ErrorResponse>> {
    let next = std::panic::catch_unwind(|| parse(cron_pattern, &Utc::now()));

    let next = match next {
        Ok(next) => next,
        Err(_) => {
            return Err(Json(ErrorResponse {
                error: "Invalid cron pattern".to_string(),
            }))
        }
    };
    let next = match next {
        Ok(next) => next,
        Err(_) => {
            return Err(Json(ErrorResponse {
                error: "Invalid cron pattern".to_string(),
            }))
        }
    };
    Ok(next.naive_utc())
}
