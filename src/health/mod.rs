use sqlx::postgres::PgPool;

pub mod handler;

pub async fn health_checks(_pool: PgPool) -> handler::HealthCheck {
    handler::HealthCheck::default()
}
