// src/services/job_service.rs
use async_trait::async_trait;
use chrono::{Utc};
use std::sync::Arc;
use tracing;

use crate::{
    errors::SparrowError as AppError,
    models::{job::{
        Job, JobEstimateRequest, JobPriority, JobRequest, JobResponse, JobStatus, JobStatusUpdate, Location, PackageType, Pricing
    }, user::User},
    services::{cache_service::{CacheKey, CacheKeys, CacheService}, driver_service::{DriverOperations, DriverService}, messaging_service::NotificationService},
    utils::id_generator::{IdGenerator, IdType, WithGeneratedId}, ValidationError,
};

#[async_trait]
pub trait JobOperations: Send + Sync {
    async fn create_job(&self, request: JobRequest) -> Result<JobResponse, AppError>;
    async fn get_job(&self, job_id: &str) -> Result<Option<JobResponse>, AppError>;
    async fn get_jobs_by_customer(&self, customer_id: &str) -> Result<Vec<JobResponse>, AppError>;
    async fn get_jobs_by_driver(&self, driver_id: &str) -> Result<Vec<JobResponse>, AppError>;
    async fn update_job_status(&self, update: JobStatusUpdate) -> Result<JobResponse, AppError>;
    async fn assign_driver_to_job(&self, job_id: &str, driver_id: &str) -> Result<JobResponse, AppError>;
    async fn calculate_estimate(&self, request: JobEstimateRequest) -> Result<Pricing, AppError>;
    async fn find_available_drivers(&self, job_id: &str) -> Result<Vec<String>, AppError>;
    async fn cancel_job(&self, job_id: &str, reason: Option<String>) -> Result<JobResponse, AppError>;
    async fn complete_job(&self, job_id: &str) -> Result<JobResponse, AppError>;
}

pub struct JobService {
    cache_service: Arc<CacheService>,
    driver_service: Arc<DriverService>,
    notification_service: Arc<dyn NotificationService>,
}

impl JobService {
    pub fn new(
        cache_service: Arc<CacheService>,
        driver_service: Arc<DriverService>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            cache_service,
            driver_service,
            notification_service,
        }
    }
    
    fn to_response(&self, job: Job) -> JobResponse {
        JobResponse {
            id: job.id,
            customer_id: job.customer_id,
            driver_id: job.driver_id,
            status: job.status,
            priority: job.priority,
            pickup_location: job.pickup_location,
            dropoff_location: job.dropoff_location,
            estimated_distance_km: job.estimated_distance_km,
            estimated_duration_min: job.estimated_duration_min,
            package: job.package,
            created_at: job.created_at,
            pickup_time: job.pickup_time,
            dropoff_time: job.dropoff_time,
            pricing: job.pricing,
            payment_status: job.payment_status,
            tracking_code: job.tracking_code,
            notes: job.notes,
            rating: job.rating,
        }
    }
    
    async fn calculate_distance_km(&self, loc1: &Location, loc2: &Location) -> f64 {
        // Simple haversine formula implementation
        // In production, you'd use a proper geocoding service
        let earth_radius_km = 6371.0;
        let lat1_rad = loc1.latitude.to_radians();
        let lat2_rad = loc2.latitude.to_radians();
        let delta_lat = (loc2.latitude - loc1.latitude).to_radians();
        let delta_lon = (loc2.longitude - loc1.longitude).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2) +
               lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        earth_radius_km * c
    }
    
    async fn calculate_duration_min(&self, distance_km: f64) -> i32 {
        // Estimate duration based on Ghana traffic patterns
        // Average speed: 25 km/h in urban areas, 40 km/h rural
        let average_speed_kmh = 30.0; // Conservative estimate for Ghana
        ((distance_km / average_speed_kmh) * 60.0) as i32
    }
    
    async fn calculate_pricing(&self, request: &JobEstimateRequest) -> Pricing {
        let distance_km = self.calculate_distance_km(&request.pickup_location, &request.dropoff_location).await;
        let duration_min = self.calculate_duration_min(distance_km).await;
        
        // Ghana-specific pricing model
        let base_fare = match request.priority {
            JobPriority::Standard => 15.0,  // 15 GHS base
            JobPriority::Express => 25.0,   // 25 GHS base
            JobPriority::SameDay => 40.0,   // 40 GHS base  
            JobPriority::Emergency => 60.0, // 60 GHS base
        };
        
        let distance_fare = distance_km * 2.5; // 2.5 GHS per km
        let time_fare = (duration_min as f64) * 0.2; // 0.2 GHS per minute
        
        let package_surcharge = match request.package.package_type {
            PackageType::Document => 0.0,
            PackageType::SmallPackage => 5.0,
            PackageType::MediumPackage => 10.0,
            PackageType::LargePackage => 20.0,
            PackageType::ExtraLarge => 40.0,
            PackageType::Food => 8.0,
            PackageType::Grocery => 15.0,
            PackageType::Pharmacy => 5.0,
            PackageType::Electronics => 15.0,
            PackageType::Fragile => 12.0,
        };
        
        let priority_surcharge = match request.priority {
            JobPriority::Standard => 0.0,
            JobPriority::Express => 10.0,
            JobPriority::SameDay => 25.0,
            JobPriority::Emergency => 50.0,
        };
        
        let subtotal = base_fare + distance_fare + time_fare + package_surcharge + priority_surcharge;
        let service_fee = subtotal * 0.1; // 10% service fee
        let tax = subtotal * 0.03; // 3% VAT for Ghana
        let total = subtotal + service_fee + tax;
        
        Pricing {
            base_fare,
            distance_fare,
            time_fare,
            package_surcharge,
            priority_surcharge,
            service_fee,
            tax,
            total,
            currency: "GHS".to_string(),
            estimated_cost: true,
        }
    }
}

#[async_trait]
impl JobOperations for JobService {
    async fn create_job(&self, request: JobRequest) -> Result<JobResponse, AppError> {
        tracing::info!("Creating job for customer: {}", request.customer_id);
        
        // Validate customer exists (would come from user service)
        // if !self.user_service.user_exists(&request.customer_id).await? {
        //     return Err(AppError::ValidationError("Customer not found".to_string()));
        // }
        
        // Calculate estimate
        let estimate_request = JobEstimateRequest {
            pickup_location: request.pickup_location.clone(),
            dropoff_location: request.dropoff_location.clone(),
            package: request.package.clone(),
            priority: request.priority.clone(),
        };
        
        let pricing = self.calculate_pricing(&estimate_request).await;
        
        // Calculate distance and duration
        let distance_km = self.calculate_distance_km(&request.pickup_location, &request.dropoff_location).await;
        let duration_min = self.calculate_duration_min(distance_km).await;
        
        // Create job with our ID generator
        let mut job = Job {
            id: String::new(), // Will be set by with_generated_id
            customer_id: request.customer_id,
            driver_id: None,
            status: JobStatus::Pending,
            priority: request.priority,
            pickup_location: request.pickup_location,
            dropoff_location: request.dropoff_location,
            estimated_distance_km: distance_km,
            estimated_duration_min: duration_min,
            package: request.package,
            created_at: Utc::now(),
            accepted_at: None,
            pickup_time: None,
            dropoff_time: None,
            cancelled_at: None,
            expires_at: Utc::now() + chrono::Duration::hours(2),
            pricing,
            payment_method_id: request.payment_method_id,
            payment_status: crate::models::job::PaymentStatus::Pending,
            tracking_code: IdGenerator::generate(IdType::Job).replace("job-", "GH"), // Clean tracking code
            notes: request.notes,
            rating: None,
            feedback: None,
            offered_to_drivers: Vec::new(),
            rejected_by_drivers: Vec::new(),
            updated_at: Utc::now(),
        };
        
        // Use our WithGeneratedId trait to set the ID
        job.set_generated_id(IdType::Job);
        
        // Cache the job
        self.cache_service.cache_job(&job).await?;
        
        // Add to customer's job list
        self.cache_service.cache_customer_job(&job.customer_id, &job.id).await?;
        
        tracing::info!("Job created successfully: {} - {} GHS", job.id, job.pricing.total);
        
        Ok(self.to_response(job))
    }
    
    async fn get_job(&self, job_id: &str) -> Result<Option<JobResponse>, AppError> {
        // Validate ID format first
        if !IdGenerator::validate_id(job_id, Some(IdType::Job)) {
            tracing::warn!("Invalid job ID format: {}", job_id);
            return Ok(None);
        }
        
        tracing::debug!("Getting job: {}", job_id);
        let key = CacheKey::Simple(job_id.to_string());
        // Try cache first
        if let Some(job) = self.cache_service.get_job(&key).await? {
            return Ok(Some(self.to_response(job)));
        }
        
        Ok(None)
    }
    
    async fn get_jobs_by_customer(&self, customer_id: &str) -> Result<Vec<JobResponse>, AppError> {
        tracing::debug!("Getting jobs for customer: {}", customer_id);
        
        let job_ids = self.cache_service.get_customer_jobs(customer_id).await?;
        let mut jobs = Vec::new();
        
        for job_id in job_ids {
            if let Some(job) = self.get_job(&job_id).await? {
                jobs.push(job);
            }
        }
        
        // Sort by creation date (newest first)
        jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(jobs)
    }
    
    async fn get_jobs_by_driver(&self, driver_id: &str) -> Result<Vec<JobResponse>, AppError> {
        tracing::debug!("Getting jobs for driver: {}", driver_id);
        
        // Validate driver ID format
        if !IdGenerator::validate_id(driver_id, Some(IdType::Driver)) {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                field: "driver_id".to_string(),
                message: "Invalid driver ID format".to_string(),
            }]));
        }
        
        let job_ids = self.cache_service.get_driver_jobs(driver_id).await?;
        let mut jobs = Vec::new();
        
        for job_id in job_ids {
            if let Some(job) = self.get_job(&job_id).await? {
                jobs.push(job);
            }
        }
        
        // Sort by creation date (newest first)
        jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(jobs)
    }
    
    async fn update_job_status(&self, update: JobStatusUpdate) -> Result<JobResponse, AppError> {
        // Validate job ID format
        if !IdGenerator::validate_id(&update.job_id, Some(IdType::Job)) {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                field: "job_id".to_string(),
                message: "Invalid job ID format".to_string(),
            }]));
        }
        
        tracing::info!("Updating job status: {} to {:?}", update.job_id, update.status);
        
        let mut job: Job = self.cache_service.get_job(&CacheKey::Simple(update.job_id.clone())).await?
            .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;
        
        // Update status and timestamp
        job.status = update.status;
        job.updated_at = Utc::now();
        
        // Set timestamps based on status
        match job.status {
            JobStatus::DriverAssigned => {
                job.accepted_at = Some(Utc::now());
            }
            JobStatus::PackagePickedUp => {
                job.pickup_time = Some(Utc::now());
            }
            JobStatus::DeliveryCompleted => {
                job.dropoff_time = Some(Utc::now());
            }
            JobStatus::Cancelled => {
                job.cancelled_at = Some(Utc::now());
            }
            _ => {}
        }
        
        // Update driver if provided
        if let Some(driver_id) = update.driver_id {
            if !IdGenerator::validate_id(&driver_id, Some(IdType::Driver)) {
                return Err(AppError::ValidationFailed(vec![ValidationError {
                    field: "driver_id".to_string(),
                    message: "Invalid driver ID format".to_string(),
                }]));
            }
            job.driver_id = Some(driver_id);
        }
        
        // Update cache
        self.cache_service.cache_job(&job).await?;
        
        tracing::debug!("Job status updated successfully: {}", job.id);
        
        Ok(self.to_response(job))
    }
    
    async fn assign_driver_to_job(&self, job_id: &str, driver_id: &str) -> Result<JobResponse, AppError> {
        // Validate IDs
        if !IdGenerator::validate_id(job_id, Some(IdType::Job)) {
            return Err(AppError::validation_error("job_id", "Invalid job ID format"));
        }
        if !IdGenerator::validate_id(driver_id, Some(IdType::Driver)) {
            return Err(AppError::validation_error("driver_id", "Invalid driver ID format"));
        }
        
        tracing::info!("Assigning driver {} to job {}", driver_id, job_id);
        
        let mut job: Job = self.cache_service.get_job(&CacheKey::Simple(job_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;
        
        let driver: User = self.cache_service.get_user(&CacheKeys::driver_by_id(driver_id)).await?
            .ok_or_else(|| AppError::NotFound("Driver not found".to_string()))?;
        
        // Check if driver is available
        // if driver.status != crate::models::driver::DriverStatus::Online {
        //     return Err(AppError::ValidationError("Driver is not available".to_string()));
        // }
        
        // Update job
        job.driver_id = Some(driver_id.to_string());
        job.status = JobStatus::DriverAssigned;
        job.accepted_at = Some(Utc::now());
        job.updated_at = Utc::now();
        
        // Update driver
        // In production, you'd update driver's current job
        
        // Update cache
        self.cache_service.cache_job(&job).await?;
        self.cache_service.cache_driver_job(driver_id, job_id).await?;
        
        tracing::info!("Driver {} assigned to job {}", driver_id, job_id);
        
        Ok(self.to_response(job))
    }
    
    async fn calculate_estimate(&self, request: JobEstimateRequest) -> Result<Pricing, AppError> {
        tracing::debug!("Calculating estimate for delivery request");
        
        let pricing = self.calculate_pricing(&request).await;
        
        Ok(pricing)
    }
    
    async fn find_available_drivers(&self, job_id: &str) -> Result<Vec<String>, AppError> {
        tracing::debug!("Finding available drivers for job: {}", job_id);
        
        let job: Job = self.cache_service.get_job(&CacheKey::Simple(job_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;
        
        // Find nearby online drivers
        let nearby_drivers = self.driver_service.find_nearby_drivers(
            job.pickup_location.latitude,
            job.pickup_location.longitude,
            10.0, // 10km radius
            10,   // max 10 drivers
        ).await?;
        
        let driver_ids = nearby_drivers.into_iter()
            .map(|driver| driver.id)
            .collect();
        
        Ok(driver_ids)
    }
    
    async fn cancel_job(&self, job_id: &str, reason: Option<String>) -> Result<JobResponse, AppError> {
        if !IdGenerator::validate_id(job_id, Some(IdType::Job)) {
            return Err(AppError::validation_error("job_id", "Invalid job ID format"));
        }
        
        tracing::info!("Cancelling job: {}", job_id);
        
        let mut job: Job = self.cache_service.get_job(&CacheKey::Simple(job_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;
        
        job.status = JobStatus::Cancelled;
        job.cancelled_at = Some(Utc::now());
        job.updated_at = Utc::now();
        job.notes = reason;
        
        // Update cache
        self.cache_service.cache_job(&job).await?;
        
        // If job had a driver assigned, update driver status
        if let Some(driver_id) = &job.driver_id {
            self.cache_service.remove_driver_job(driver_id, job_id).await?;
        }
        
        tracing::info!("Job cancelled: {}", job_id);
        
        Ok(self.to_response(job))
    }
    
    async fn complete_job(&self, job_id: &str) -> Result<JobResponse, AppError> {
        if !IdGenerator::validate_id(job_id, Some(IdType::Job)) {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                field: "job_id".to_string(),
                message: "Invalid job ID format".to_string(),
            }]));
        }
        
        tracing::info!("Completing job: {}", job_id);
        
        let mut job: Job = self.cache_service.get_job(&CacheKey::Simple(job_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;
        
        job.status = JobStatus::DeliveryCompleted;
        job.dropoff_time = Some(Utc::now());
        job.updated_at = Utc::now();
        job.payment_status = crate::models::job::PaymentStatus::Paid;
        
        // Update cache
        self.cache_service.cache_job(&job).await?;
        
        // Update driver stats
        if let Some(driver_id) = &job.driver_id {
            // if let Some(mut driver) = self.cache_service.get_driver(driver_id).await? {
            //     driver.total_rides += 1;
            //     self.cache_service.cache_driver(&driver).await?;
            // }
        }
        
        tracing::info!("Job completed: {}", job_id);
        
        Ok(self.to_response(job))
    }
}
