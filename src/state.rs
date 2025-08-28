// src/state.rs
use std::sync::Arc;

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
    pub fcm_api_key: Option<String>,
    pub ably_api_key: String,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let cache_service = Arc::new(CacheService::new(&config.redis_url).await?);
        let user_service = Arc::new(UserService::new(cache_service.clone()));
        let driver_service = Arc::new(DriverService::new(cache_service.clone()));
        
        let notification_service: Arc<dyn NotificationService> = match config.fcm_api_key {
            Some(api_key) => Arc::new(FcmService::new(api_key)),
            None => {
                tracing::warn!("FCM_API_KEY not set, using mock notification service");
                Arc::new(MockNotificationService)
            }
        };

        let job_service = Arc::new(JobService::new(
            cache_service.clone(),
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