use crate::{browser::handler::Selenium, utils::ApiTags};
use poem::{error::InternalServerError, http::StatusCode, Error, Result};
use poem_openapi::{payload::PlainText, OpenApi};
use sqlx::postgres::PgPool;

pub struct HealthCheck {
    pool: PgPool,
    browser_api: Selenium,
}

#[OpenApi(prefix_path = "/health/", tag = "ApiTags::HealthCheck")]
impl HealthCheck {
    pub fn new(pool: PgPool, browser_api: Selenium) -> Self {
        Self { pool, browser_api }
    }

    #[oai(path = "/liveness", method = "get")]
    async fn liveness(&self) -> PlainText<String> {
        PlainText("OK".to_string())
    }
    #[oai(path = "/readiness", method = "get")]
    async fn readiness(&self) -> Result<PlainText<String>> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(InternalServerError)?;

        match self.browser_api.health().await {
            Ok(_) => "OK",
            Err(err) => {
                return Err(Error::from_string(
                    err.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };

        Ok(PlainText("OK".to_string()))
    }
}
