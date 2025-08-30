use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
};
use sparrow_realtime::{
    state::{AppState, AppConfig},
    handlers::{user_handler, driver_handler, job_handler},
};

#[tokio::main]
async fn main() {
    let config = AppConfig {
        dynamo_url: "http://localhost:8000".to_string(),
        postgres_url: "postgres://user:password@localhost/database".to_string(),
        redis_url: "redis://127.0.0.1/".to_string(),
        fcm_server_key: Some("your_fcm_server_key".to_string()),
        ably_api_key: "your_ably_api_key".to_string(),
    };

    let app_state = AppState::new(config).await.unwrap();

    let app = Router::new()
        .route("/users", get(user_handler::get_user).post(user_handler::create_user))
        .route("/drivers", get(driver_handler::get_driver).post(driver_handler::create_driver))
        .route("/jobs", get(job_handler::get_job).post(job_handler::create_job))
        .with_state(Arc::new(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
