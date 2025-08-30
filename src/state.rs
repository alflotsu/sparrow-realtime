// src/state.rs
use std::sync::Arc;


use crate::services::{
    cache_service::CacheService, 
    driver_service::DriverService, 
    job_service::JobService, 
    user_service::UserService, 
    messaging_service::{FcmNotificationService, MockNotificationService, NotificationService}
};

pub struct AppState {
    pub user_service: Arc<UserService>,
    pub driver_service: Arc<DriverService>,
    pub job_service: Arc<JobService>,
    pub cache_service: Arc<CacheService>,
    pub notification_service: Arc<dyn NotificationService>,
    pub config: AppConfig,
}

#[derive(Clone)]
pub struct AppConfig {
    pub dynamo_url: String,
    pub postgres_url: String,
    pub redis_url: String,
    pub fcm_server_key: Option<String>,  // Changed from fcm_api_key to fcm_server_key
    pub ably_api_key: String,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let cache_service = Arc::new(CacheService::new(&config.redis_url).await?);
        
        // Initialize notification service first since other services might need it
        let notification_service: Arc<dyn NotificationService> = 
            match config.fcm_server_key.clone() {
                Some(server_key) => {
                    tracing::info!("Using FCM notification service with server key");
                    Arc::new(FcmNotificationService::with_server_key(
                        server_key, 
                        cache_service.clone()
                    ))
                }
                None => {
                    tracing::warn!("FCM_SERVER_KEY not set, using mock notification service");
                    Arc::new(MockNotificationService)
                }
            };

        let user_service = Arc::new(UserService::new(
            cache_service.clone(),
            notification_service.clone(),
        ));

        let driver_service = Arc::new(DriverService::new(
            cache_service.clone(),
            notification_service.clone(),
        ));

        let job_service = Arc::new(JobService::new(
            cache_service.clone(),
            driver_service.clone(),
            notification_service.clone(),
        ));

        Ok(Self {
            user_service,
            driver_service,
            job_service,
            cache_service,
            notification_service,
            config,
        })
    }
}