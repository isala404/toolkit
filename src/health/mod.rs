use crate::browser::handler::Selenium;
use sqlx::postgres::PgPool;

pub mod handler;

pub async fn health_checks(pool: PgPool, browser_api: Selenium) -> handler::HealthCheck {
    handler::HealthCheck::new(pool, browser_api)
}
