// src/services/driver_service.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing;

use crate::{
    errors::SparrowError as AppError,
    models::driver::{
        Driver, DriverRegistration, DriverStatus, DriverStatusUpdate, DriverLocationUpdate,
        DriverResponse, Vehicle,
    },
    models::user::User,
    services::cache_service::{CacheService, CacheKeys},
    services::messaging_service::NotificationService,
    utils::id_generator::{IdGenerator, IdType, WithGeneratedId},
};

#[async_trait]
pub trait DriverOperations: Send + Sync {
    async fn register_driver(&self, registration: DriverRegistration) -> Result<DriverResponse, AppError>;
    async fn get_driver(&self, driver_id: &str) -> Result<Option<DriverResponse>, AppError>;
    async fn get_driver_by_user_id(&self, user_id: &str) -> Result<Option<DriverResponse>, AppError>;
    async fn update_driver_status(&self, update: DriverStatusUpdate) -> Result<DriverResponse, AppError>;
    async fn update_driver_location(&self, update: DriverLocationUpdate) -> Result<DriverResponse, AppError>;
    async fn find_nearby_drivers(&self, latitude: f64, longitude: f64, radius_km: f64, limit: usize) -> Result<Vec<DriverResponse>, AppError>;
    async fn get_online_drivers(&self) -> Result<Vec<DriverResponse>, AppError>;
    async fn get_driver_stats(&self, driver_id: &str) -> Result<User, AppError>;
    async fn delete_driver(&self, driver_id: &str) -> Result<(), AppError>;
}

pub struct DriverService {
    notification_service: Arc<dyn NotificationService>,
    cache_service: Arc<CacheService>,
}

impl DriverService {
    pub fn new(
        cache_service: Arc<CacheService>,
        notification_service: Arc<dyn NotificationService>
    ) -> Self {
        Self { cache_service, notification_service }
    }
    
    fn to_response(&self, driver: Driver) -> DriverResponse {
        DriverResponse {
            id: driver.id,
            first_name: driver.first_name,
            last_name: driver.last_name,
            phone_number: driver.phone_number,
            status: driver.status,
            current_location: driver.current_location,
            vehicle: driver.vehicle,
            rating: driver.rating,
            total_rides: driver.total_rides,
            is_verified: driver.is_verified,
            current_ride_id: driver.current_ride_id,
        }
    }
}

#[async_trait]
impl DriverOperations for DriverService {
    async fn register_driver(&self, registration: DriverRegistration) -> Result<DriverResponse, AppError> {
        tracing::info!("Registering driver for user: {}", registration.user_id);
        
        // Check if driver already exists for this user
        if let Some(existing) = self.get_driver_by_user_id(&registration.user_id).await? {
            return Err(AppError::validation_error("user_id", "Driver already exists for this user"));
        }
        
        // Create vehicle with generated ID
        let vehicle = Vehicle {
            id: IdGenerator::generate(IdType::Vehicle), // Using our ID generator!
            license_plate: registration.license_plate,
            vehicle_type: registration.vehicle_type,
            make: registration.vehicle_make,
            model: registration.vehicle_model,
            year: registration.vehicle_year,
            color: registration.vehicle_color,
            capacity_kg: registration.capacity_kg,
        };
        
        // Create driver with our ID generator
        let mut driver = Driver {
            id: String::new(), // Will be set by with_generated_id
            user_id: registration.user_id,
            first_name: registration.first_name,
            last_name: registration.last_name,
            phone_number: registration.phone_number,
            email: registration.email,
            status: DriverStatus::Offline,
            current_location: None,
            vehicle,
            rating: 0.0,
            total_rides: 0,
            is_verified: false,
            is_active: true,
            current_ride_id: None,
            device_token: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Use our WithGeneratedId trait to set the ID
        driver.set_generated_id(IdType::Driver);
        
        // Cache the driver - note: we need to implement driver caching separately
        // from user caching since Driver and User are different models
        // For now, we'll skip the caching and return the response
        // TODO: Implement proper driver caching
        
        tracing::info!("Driver registered successfully: {}", driver.id);
        
        Ok(self.to_response(driver))
    }
    
    async fn get_driver(&self, driver_id: &str) -> Result<Option<DriverResponse>, AppError> {
        // Validate ID format first
        if !IdGenerator::validate_id(driver_id, Some(IdType::Driver)) {
            tracing::warn!("Invalid driver ID format: {}", driver_id);
            return Ok(None);
        }
        
        tracing::debug!("Getting driver: {}", driver_id);
        
        // Try cache first - TODO: implement proper driver caching
        // For now, return None since we can't convert User to Driver
        
        Ok(None)
    }

    async fn get_driver_by_user_id(&self, user_id: &str) -> Result<Option<DriverResponse>, AppError> {
        // Implementation needed
        Ok(None)
    }
    
    // ... rest of the methods remain the same but with ID validation
    async fn update_driver_status(&self, update: DriverStatusUpdate) -> Result<DriverResponse, AppError> {
        // Validate driver ID format
        if !IdGenerator::validate_id(&update.driver_id, Some(IdType::Driver)) {
            return Err(AppError::validation_error("driver_id", "Invalid driver ID format"));
        }
        
        tracing::info!("Updating driver status: {} to {:?}", update.driver_id, update.status);
        
        // TODO: implement proper driver retrieval from cache
        // For now, return an error since we can't properly retrieve drivers
        return Err(AppError::NotFound("Driver service not fully implemented".to_string()));
    }
    
    async fn update_driver_location(&self, update: DriverLocationUpdate) -> Result<DriverResponse, AppError> {
        // Validate driver ID format
        if !IdGenerator::validate_id(&update.driver_id, Some(IdType::Driver)) {
            return Err(AppError::validation_error("driver_id", "Invalid driver ID format"));
        }
        
        tracing::debug!("Updating driver location: {}", update.driver_id);
        
        // TODO: implement proper driver location updates
        // For now, return an error since we can't properly retrieve/update drivers
        return Err(AppError::NotFound("Driver location update not fully implemented".to_string()));
    }
    
    async fn find_nearby_drivers(&self, _: f64, _: f64, _: f64, _: usize) -> Result<Vec<DriverResponse>, AppError> {
        Ok(vec![])
    }

    async fn get_online_drivers(&self) -> Result<Vec<DriverResponse>, AppError> {
        Ok(vec![])
    }

    async fn get_driver_stats(&self, _: &str) -> Result<User, AppError> {
        unimplemented!()
    }

    async fn delete_driver(&self, _: &str) -> Result<(), AppError> {
        unimplemented!()
    }
}
