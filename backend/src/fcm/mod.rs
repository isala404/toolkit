use sqlx::SqlitePool;

mod handler;
mod model;
mod utils;
mod worker;

pub async fn fcm_api(pool: SqlitePool) -> handler::FirebaseMessaging {
    let service_accounts = worker::read_in_serivce_accounts().await.unwrap();

    let fcm_api = handler::FirebaseMessaging::new(service_accounts.keys().cloned().collect());

    tokio::spawn(async move {
        worker::run_every_minute(service_accounts, &pool).await;
    });

    return fcm_api;
}
