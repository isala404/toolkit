use lazy_static::lazy_static;
use poem::{
    http::StatusCode,
    web::headers::authorization::Basic,
    web::headers::{self, HeaderMapExt},
    Endpoint, Error as PoemError, Middleware, Request, Response, Result as PoemResult,
};
use poem_openapi::{
    error::ParseRequestPayloadError,
    payload::{Attachment, Json},
    types::{ParseFromJSON, ToJSON},
    {ApiResponse, Object, Tags},
};
use sqlx::{sqlite::SqliteConnectOptions, Error, SqlitePool};
use std::env;
use std::{future::Future, path::Path};

lazy_static! {
    static ref API_KEY: String = env::var("API_KEY").expect("API_KEY must be set");
}

#[derive(Tags)]
pub enum ApiTags {
    /// Scheduled firebase messageing service
    FirebaseMessaging,
    /// Health check endpoints
    HealthCheck,
    /// Browser automation
    Selenium,
    /// Youtube-dl service
    YoutubeDL,
}

async fn connect(filename: impl AsRef<Path>) -> impl Future<Output = Result<SqlitePool, Error>> {
    let filename = filename
        .as_ref()
        .to_str()
        .unwrap()
        .trim_start_matches("sqlite:");

    let options = SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(true);

    SqlitePool::connect_with(options)
}

pub async fn get_db_pool() -> SqlitePool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = connect(database_url).await.await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    return pool;
}

pub fn get_host() -> String {
    let host = env::var("HOST").expect("HOST must be set");
    return host;
}

pub fn get_port() -> String {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    return port;
}

#[derive(Object)]
pub struct ResponseObject<T: ParseFromJSON + ToJSON + Send + Sync> {
    data: Option<T>,
    error: Option<String>,
}

impl<T: ParseFromJSON + ToJSON + Send + Sync> ResponseObject<T> {
    pub fn ok(data: T) -> MyResponse<T> {
        MyResponse::Ok(Json(ResponseObject {
            data: Some(data),
            error: None,
        }))
    }

    pub fn created(data: T) -> MyResponse<T> {
        MyResponse::Created(Json(ResponseObject {
            data: Some(data),
            error: None,
        }))
    }

    pub fn file_response(data: Attachment<Vec<u8>>) -> MyResponse<T> {
        MyResponse::FileResponse(data)
    }

    pub fn bad_request(error: impl ToString) -> MyResponse<T> {
        MyResponse::BadRequest(Json(ResponseObject {
            data: None,
            error: Some(error.to_string()),
        }))
    }

    pub fn unauthorized(error: impl ToString) -> MyResponse<T> {
        MyResponse::Unauthorized(Json(ResponseObject {
            data: None,
            error: Some(error.to_string()),
        }))
    }

    pub fn not_found(error: impl ToString) -> MyResponse<T> {
        MyResponse::NotFound(Json(ResponseObject {
            data: None,
            error: Some(error.to_string()),
        }))
    }

    pub fn internal_server_error(error: impl ToString) -> MyResponse<T> {
        MyResponse::InternalServerError(Json(ResponseObject {
            data: None,
            error: Some(error.to_string()),
        }))
    }
}

#[derive(ApiResponse)]
#[oai(bad_request_handler = "bad_request_handler")]
pub enum MyResponse<T: ParseFromJSON + ToJSON + Send + Sync> {
    #[oai(status = 200)]
    Ok(Json<ResponseObject<T>>),
    #[oai(status = 200)]
    FileResponse(Attachment<Vec<u8>>),
    #[oai(status = 201)]
    Created(Json<ResponseObject<T>>),
    #[oai(status = 400)]
    BadRequest(Json<ResponseObject<T>>),
    #[oai(status = 401)]
    Unauthorized(Json<ResponseObject<T>>),
    #[oai(status = 404)]
    NotFound(Json<ResponseObject<T>>),
    #[oai(status = 500)]
    InternalServerError(Json<ResponseObject<T>>),
}

fn bad_request_handler<T: ParseFromJSON + ToJSON + Send + Sync>(err: PoemError) -> MyResponse<T> {
    if err.is::<ParseRequestPayloadError>() {
        MyResponse::BadRequest(Json(ResponseObject {
            data: None,
            error: Some(err.to_string()),
        }))
    } else {
        MyResponse::InternalServerError(Json(ResponseObject {
            data: None,
            error: Some(err.to_string()),
        }))
    }
}

pub struct BasicAuth {
    username: String,
    password: String,
}

impl Default for BasicAuth {
    fn default() -> Self {
        let username = env::var("BASIC_AUTH_USERNAME").expect("BASIC_AUTH_USERNAME must be set");
        let password = env::var("BASIC_AUTH_PASSWORD").expect("BASIC_AUTH_PASSWORD must be set");
        BasicAuth { username, password }
    }
}

impl<E: Endpoint> Middleware<E> for BasicAuth {
    // new is a constructor for BasicAuth

    type Output = BasicAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        BasicAuthEndpoint {
            ep,
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

pub struct BasicAuthEndpoint<E> {
    ep: E,
    username: String,
    password: String,
}

#[poem::async_trait]
impl<E: Endpoint> Endpoint for BasicAuthEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, req: Request) -> PoemResult<Self::Output> {
        if let Some(auth) = req.headers().typed_get::<headers::Authorization<Basic>>() {
            if auth.0.username() == self.username && auth.0.password() == self.password {
                return self.ep.call(req).await;
            }
        }

        let res = Response::builder()
            .header("WWW-Authenticate", "Basic")
            .status(StatusCode::UNAUTHORIZED)
            .body(());

        Err(PoemError::from_response(res))
    }
}

pub async fn verify_apikey(req: &Request) -> Result<(), String> {
    // extract user id from token
    let api_key = match req.header("API-Key") {
        Some(key) => key,
        None => {
            return Err("API-Key header is missing".to_string());
        }
    };
    if !API_KEY.eq(api_key) {
        return Err("Invalid API-Key".to_string());
    }

    return Ok(());
}
