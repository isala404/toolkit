use sqlx::SqlitePool;

pub mod handler;


pub async fn health_checks(_pool: SqlitePool) -> handler::HealthCheck {
    handler::HealthCheck::default()
}
