// src/services/messaging_service.rs
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tracing;
use thiserror::Error;
use chrono::Utc;

use crate::{
    errors::SparrowError as AppError,
    models::{user::User, driver::Driver, job::Job},
    services::cache_service::{CacheService, CacheKeys},
};

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("FCM send failed: {0}")]
    FcmError(String),
    
    #[error("Device token not found")]
    NoDeviceToken,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone)]
pub struct FcmConfig {
    pub fcm_server_key: String,
    pub fcm_url: String,
}

impl Default for FcmConfig {
    fn default() -> Self {
        Self {
            fcm_server_key: std::env::var("FCM_SERVER_KEY")
                .unwrap_or_else(|_| "".to_string()),
            fcm_url: "https://fcm.googleapis.com/fcm/send".to_string(),
        }
    }
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_to_device(&self, device_token: &str, message: NotificationMessage) -> Result<(), AppError>;
    async fn send_to_driver(&self, driver_id: &str, message: NotificationMessage) -> Result<(), AppError>;
    async fn send_to_user(&self, user_id: &str, message: NotificationMessage) -> Result<(), AppError>;
    async fn notify_driver_assigned(&self, job: &Job, driver: &Driver) -> Result<(), AppError>;
    async fn notify_package_picked_up(&self, job: &Job) -> Result<(), AppError>;
    async fn notify_delivery_completed(&self, job: &Job) -> Result<(), AppError>;
    async fn notify_ride_status_update(&self, job: &Job, status: &str) -> Result<(), AppError>;
}

#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub title: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
    pub priority: NotificationPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationPriority {
    Normal,
    High,    // Will wake sleeping devices
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::High
    }
}

pub struct FcmNotificationService {
    config: FcmConfig,
    client: reqwest::Client,
    cache_service: Arc<CacheService>,
}

impl FcmNotificationService {
    pub fn new(config: FcmConfig, cache_service: Arc<CacheService>) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            cache_service,
        }
    }
    
    pub fn with_server_key(server_key: String, cache_service: Arc<CacheService>) -> Self {
        Self::new(
            FcmConfig {
                fcm_server_key: server_key,
                ..Default::default()
            },
            cache_service,
        )
    }
    
    async fn get_driver_device_token(&self, driver_id: &str) -> Result<String, AppError> {
        // TODO: implement proper driver token retrieval
        // For now, try to get from user instead since we can't convert User to Driver
        if let Some(user) = self.cache_service.get_user(&CacheKeys::user_by_id(driver_id)).await? {
            user.device_tokens.first()
                .cloned()
                .ok_or_else(|| AppError::FcmInvalidToken("Driver has no device token".to_string()))
        } else {
            Err(AppError::DriverNotFound(driver_id.to_string()))
        }
    }
    
    async fn get_user_device_token(&self, user_id: &str) -> Result<String, AppError> {
        // This would typically come from your user service
        // For now, we'll use a placeholder
        if let Some(user) = self.cache_service.get_user(&CacheKeys::user_by_id(user_id)).await? {
            user.device_tokens.first()
                .cloned()
                .ok_or_else(|| AppError::FcmInvalidToken("User has no device token".to_string()))
        } else {
            Err(AppError::UserNotFound(user_id.to_string()))
        }
    }
}

#[async_trait]
impl NotificationService for FcmNotificationService {
    async fn send_to_device(&self, device_token: &str, message: NotificationMessage) -> Result<(), AppError> {
        if device_token.is_empty() {
            return Err(AppError::FcmInvalidToken("Empty device token".to_string()));
        }
        
        tracing::info!("Sending FCM notification to device: {}", device_token);
        
        let mut fcm_message = json!({
            "to": device_token,
            "notification": {
                "title": message.title,
                "body": message.body,
                "sound": "default"
            },
            "priority": match message.priority {
                NotificationPriority::High => "high",
                NotificationPriority::Normal => "normal",
            }
        });
        
        if let Some(data) = message.data {
            fcm_message["data"] = data;
        }
        
        let response = self.client
            .post(&self.config.fcm_url)
            .header("Authorization", format!("key={}", self.config.fcm_server_key))
            .header("Content-Type", "application/json")
            .json(&fcm_message)
            .send()
            .await
            .map_err(|e| AppError::NetworkConnection(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("FCM request failed: {}", error_text);
            return Err(AppError::FcmDelivery(error_text));
        }
        
        tracing::debug!("FCM notification sent successfully");
        Ok(())
    }
    
    async fn send_to_driver(&self, driver_id: &str, message: NotificationMessage) -> Result<(), AppError> {
        let device_token = self.get_driver_device_token(driver_id).await?;
        self.send_to_device(&device_token, message).await
    }
    
    async fn send_to_user(&self, user_id: &str, message: NotificationMessage) -> Result<(), AppError> {
        let device_token = self.get_user_device_token(user_id).await?;
        self.send_to_device(&device_token, message).await
    }
    
    async fn notify_driver_assigned(&self, job: &Job, driver: &Driver) -> Result<(), AppError> {
        let message = NotificationMessage {
            title: "🚗 New Delivery Assignment".to_string(),
            body: format!("Delivery from {} to {} - {} GHS", 
                job.pickup_location.city, 
                job.dropoff_location.city,
                job.pricing.total
            ),
            data: Some(json!({
                "type": "driver_assigned",
                "job_id": job.id,
                "amount": job.pricing.total,
                "pickup_address": job.pickup_location.address,
                "dropoff_address": job.dropoff_location.address,
                "customer_name": "Customer", // Would get from user service
                "priority": job.priority.to_string(),
            })),
            priority: NotificationPriority::High,
        };
        
        self.send_to_driver(&driver.id, message).await
    }
    
    async fn notify_package_picked_up(&self, job: &Job) -> Result<(), AppError> {
        let message = NotificationMessage {
            title: "📦 Package Picked Up".to_string(),
            body: format!("Your package has been collected and is on the way!"),
            data: Some(json!({
                "type": "package_picked_up",
                "job_id": job.id,
                "driver_name": "Driver", // Would get from driver service
                "estimated_arrival": "30 minutes", // Would calculate ETA
            })),
            priority: NotificationPriority::Normal,
        };
        
        self.send_to_user(&job.customer_id, message).await
    }
    
    async fn notify_delivery_completed(&self, job: &Job) -> Result<(), AppError> {
        let message = NotificationMessage {
            title: "✅ Delivery Completed".to_string(),
            body: format!("Your package has been delivered successfully!"),
            data: Some(json!({
                "type": "delivery_completed",
                "job_id": job.id,
                "amount": job.pricing.total,
                "completion_time": Utc::now().to_rfc3339(),
            })),
            priority: NotificationPriority::Normal,
        };
        
        self.send_to_user(&job.customer_id, message).await
    }
    
    async fn notify_ride_status_update(&self, job: &Job, status: &str) -> Result<(), AppError> {
        let (title, body) = match status {
            "driver_en_route" => (
                "🚗 Driver On The Way".to_string(),
                "Your driver is coming to pickup location".to_string()
            ),
            "driver_arrived" => (
                "📍 Driver Arrived".to_string(),
                "Your driver has arrived at pickup location".to_string()
            ),
            "in_progress" => (
                "📦 Package In Transit".to_string(),
                "Your package is on the way to destination".to_string()
            ),
            _ => (
                "📋 Status Updated".to_string(),
                format!("Delivery status: {}", status)
            ),
        };
        
        let message = NotificationMessage {
            title,
            body,
            data: Some(json!({
                "type": "status_update",
                "job_id": job.id,
                "status": status,
                "timestamp": Utc::now().to_rfc3339(),
            })),
            priority: NotificationPriority::Normal,
        };
        
        self.send_to_user(&job.customer_id, message).await
    }
}

// Mock service for development and testing
#[derive(Debug)]
pub struct MockNotificationService;

#[async_trait]
impl NotificationService for MockNotificationService {
    async fn send_to_device(&self, device_token: &str, message: NotificationMessage) -> Result<(), AppError> {
        tracing::info!("[MOCK] Would send FCM to {}: {} - {}", 
            device_token, message.title, message.body);
        Ok(())
    }
    
    async fn send_to_driver(&self, driver_id: &str, message: NotificationMessage) -> Result<(), AppError> {
        tracing::info!("[MOCK] Would send to driver {}: {} - {}", 
            driver_id, message.title, message.body);
        Ok(())
    }
    
    async fn send_to_user(&self, user_id: &str, message: NotificationMessage) -> Result<(), AppError> {
        tracing::info!("[MOCK] Would send to user {}: {} - {}", 
            user_id, message.title, message.body);
        Ok(())
    }
    
    async fn notify_driver_assigned(&self, job: &Job, driver: &Driver) -> Result<(), AppError> {
        tracing::info!("[MOCK] Driver assigned: {} to job {}", driver.id, job.id);
        Ok(())
    }
    
    async fn notify_package_picked_up(&self, job: &Job) -> Result<(), AppError> {
        tracing::info!("[MOCK] Package picked up for job: {}", job.id);
        Ok(())
    }
    
    async fn notify_delivery_completed(&self, job: &Job) -> Result<(), AppError> {
        tracing::info!("[MOCK] Delivery completed for job: {}", job.id);
        Ok(())
    }
    
    async fn notify_ride_status_update(&self, job: &Job, status: &str) -> Result<(), AppError> {
        tracing::info!("[MOCK] Status update for job {}: {}", job.id, status);
        Ok(())
    }
}

// Helper functions for creating notifications
impl NotificationMessage {
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
            data: None,
            priority: NotificationPriority::default(),
        }
    }
    
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
    
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }
}
