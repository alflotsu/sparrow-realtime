// src/models/job.rs
// Created on 28-08-2025 by Alfred Lotsu
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JobStatus {
    Pending,           // Job created, waiting for driver acceptance
    Searching,         // Looking for available drivers
    DriverAssigned,    // Driver assigned but not yet en route
    DriverEnRoute,     // Driver is on the way to pickup
    ArrivedAtPickup,   // Driver arrived at pickup location
    PackagePickedUp,   // Package collected from sender
    InTransit,         // Package is being delivered
    ArrivedAtDropoff,  // Driver arrived at destination
    DeliveryCompleted, // Package successfully delivered
    Cancelled,         // Job was cancelled
    Failed,            // Delivery failed (recipient not available, etc.)
    Expired,           // No drivers accepted the job
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JobPriority {
    Standard,    // Normal delivery (within 24 hours)
    Express,     // Fast delivery (within 4 hours)
    SameDay,     // Same day delivery
    Emergency,   // Immediate delivery (within 1 hour)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PackageType {
    Document,    // Letters, documents, small envelopes
    SmallPackage,// Small boxes (up to 5kg)
    MediumPackage, // Medium boxes (5-15kg)
    LargePackage, // Large boxes (15-30kg)
    ExtraLarge,  // Very large items (30kg+)
    Food,        // Food delivery
    Grocery,     // Grocery delivery
    Pharmacy,    // Medicine delivery
    Electronics, // Sensitive electronics
    Fragile,     // Fragile items requiring special care
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub postal_code: Option<String>,
    pub contact_name: String,
    pub contact_phone: String,
    pub instructions: Option<String>, // Special instructions for driver
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageDetails {
    pub package_type: PackageType,
    pub description: String,
    pub weight_kg: f32,
    pub dimensions: Dimensions, // Length, width, height
    pub estimated_value: Option<f64>, // For insurance purposes
    pub is_fragile: bool,
    pub requires_signature: bool,
    pub contains: Option<String>, // What's inside the package
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dimensions {
    pub length_cm: f32,
    pub width_cm: f32,
    pub height_cm: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pricing {
    pub base_fare: f64,
    pub distance_fare: f64,
    pub time_fare: f64,
    pub package_surcharge: f64,
    pub priority_surcharge: f64,
    pub service_fee: f64,
    pub tax: f64,
    pub total: f64,
    pub currency: String, // "GHS" for Ghana Cedis
    pub estimated_cost: bool, // Whether this is an estimate or final price
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Job {
    pub id: String,
    pub customer_id: String,
    pub driver_id: Option<String>,
    pub status: JobStatus,
    pub priority: JobPriority,
    
    // Location information
    pub pickup_location: Location,
    pub dropoff_location: Location,
    pub estimated_distance_km: f64,
    pub estimated_duration_min: i32,
    
    // Package information
    pub package: PackageDetails,
    
    // Timing information
    pub created_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub pickup_time: Option<DateTime<Utc>>,
    pub dropoff_time: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>, // When job will expire if not accepted
    
    // Pricing information
    pub pricing: Pricing,
    pub payment_method_id: String,
    pub payment_status: PaymentStatus,
    
    // Tracking and metadata
    pub tracking_code: String, // Unique code for customer tracking
    pub notes: Option<String>, // Special instructions from customer
    pub rating: Option<f32>,   // Customer rating (1-5)
    pub feedback: Option<String>,
    
    // Driver assignment history
    pub offered_to_drivers: Vec<String>, // Driver IDs who were offered this job
    pub rejected_by_drivers: Vec<String>, // Driver IDs who rejected this job
    
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Authorized,
    Paid,
    Failed,
    Refunded,
    PartiallyRefunded,
}

// Request/Response Models
#[derive(Debug, Serialize, Deserialize)]
pub struct JobRequest {
    pub customer_id: String,
    pub pickup_location: Location,
    pub dropoff_location: Location,
    pub package: PackageDetails,
    pub priority: JobPriority,
    pub payment_method_id: String,
    pub notes: Option<String>,
    pub desired_pickup_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobEstimateRequest {
    pub pickup_location: Location,
    pub dropoff_location: Location,
    pub package: PackageDetails,
    pub priority: JobPriority,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub customer_id: String,
    pub driver_id: Option<String>,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub pickup_location: Location,
    pub dropoff_location: Location,
    pub estimated_distance_km: f64,
    pub estimated_duration_min: i32,
    pub package: PackageDetails,
    pub created_at: DateTime<Utc>,
    pub pickup_time: Option<DateTime<Utc>>,
    pub dropoff_time: Option<DateTime<Utc>>,
    pub pricing: Pricing,
    pub payment_status: PaymentStatus,
    pub tracking_code: String,
    pub notes: Option<String>,
    pub rating: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobStatusUpdate {
    pub job_id: String,
    pub status: JobStatus,
    pub driver_id: Option<String>,
    pub notes: Option<String>, // Reason for cancellation, etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobAssignment {
    pub job_id: String,
    pub driver_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobRejection {
    pub job_id: String,
    pub driver_id: String,
    pub reason: Option<String>, // Why driver rejected the job
}

// Tracking Models
#[derive(Debug, Serialize, Deserialize)]
pub struct JobTracking {
    pub job_id: String,
    pub status: JobStatus,
    pub current_location: Option<LocationUpdate>,
    pub driver_location: Option<LocationUpdate>,
    pub estimated_arrival: Option<DateTime<Utc>>,
    pub events: Vec<JobEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationUpdate {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
    pub accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobEvent {
    pub event_type: JobEventType,
    pub timestamp: DateTime<Utc>,
    pub location: Option<LocationUpdate>,
    pub actor: String, // "system", "customer", "driver:{id}"
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum JobEventType {
    JobCreated,
    DriverAssigned,
    DriverEnRoute,
    ArrivedAtPickup,
    PackagePickedUp,
    InTransit,
    ArrivedAtDropoff,
    DeliveryCompleted,
    JobCancelled,
    StatusUpdated,
    LocationUpdated,
    PaymentProcessed,
}

// Driver Job Models
#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableJob {
    pub job: JobResponse,
    pub estimated_earnings: f64,
    pub distance_to_pickup_km: f64,
    pub time_to_pickup_min: i32,
    pub customer_rating: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverJobStats {
    pub driver_id: String,
    pub total_jobs: u32,
    pub completed_jobs: u32,
    pub active_jobs: u32,
    pub cancellation_rate: f32,
    pub average_rating: f32,
    pub total_earnings: f64,
    pub today_earnings: f64,
    pub weekly_earnings: f64,
    pub monthly_earnings: f64,
}

// Search and Filter Models
#[derive(Debug, Serialize, Deserialize)]
pub struct JobFilter {
    pub status: Option<Vec<JobStatus>>,
    pub priority: Option<Vec<JobPriority>>,
    pub date_range: Option<DateRange>,
    pub customer_id: Option<String>,
    pub driver_id: Option<String>,
    pub has_rating: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobSearchResult {
    pub jobs: Vec<JobResponse>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
}

// Analytics Models
#[derive(Debug, Serialize, Deserialize)]
pub struct JobAnalytics {
    pub total_jobs: u32,
    pub completed_jobs: u32,
    pub cancelled_jobs: u32,
    pub average_completion_time_min: f64,
    pub average_rating: f32,
    pub total_revenue: f64,
    pub popular_package_types: Vec<PackageTypeStats>,
    pub busiest_regions: Vec<RegionStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageTypeStats {
    pub package_type: PackageType,
    pub count: u32,
    pub percentage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegionStats {
    pub region: String,
    pub job_count: u32,
    pub total_revenue: f64,
}

// Helper implementations
impl Job {
    pub fn new(job_request: JobRequest, pricing: Pricing) -> Self {
        let tracking_code = format!("GH{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        
        Self {
            id: Uuid::new_v4().to_string(),
            customer_id: job_request.customer_id,
            driver_id: None,
            status: JobStatus::Pending,
            priority: job_request.priority,
            pickup_location: job_request.pickup_location,
            dropoff_location: job_request.dropoff_location,
            estimated_distance_km: 0.0, // Will be calculated
            estimated_duration_min: 0,   // Will be calculated
            package: job_request.package,
            created_at: Utc::now(),
            accepted_at: None,
            pickup_time: None,
            dropoff_time: None,
            cancelled_at: None,
            expires_at: Utc::now() + chrono::Duration::hours(2), // 2 hours to accept
            pricing,
            payment_method_id: job_request.payment_method_id,
            payment_status: PaymentStatus::Pending,
            tracking_code,
            notes: job_request.notes,
            rating: None,
            feedback: None,
            offered_to_drivers: Vec::new(),
            rejected_by_drivers: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

impl Dimensions {
    pub fn volume(&self) -> f32 {
        self.length_cm * self.width_cm * self.height_cm
    }
}

impl PackageType {
    pub fn base_weight_limit(&self) -> f32 {
        match self {
            PackageType::Document => 0.5,
            PackageType::SmallPackage => 5.0,
            PackageType::MediumPackage => 15.0,
            PackageType::LargePackage => 30.0,
            PackageType::ExtraLarge => 100.0,
            PackageType::Food => 10.0,
            PackageType::Grocery => 20.0,
            PackageType::Pharmacy => 5.0,
            PackageType::Electronics => 15.0,
            PackageType::Fragile => 10.0,
        }
    }
}