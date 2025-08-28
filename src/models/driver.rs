// src/models/driver.rs
// Created on 28-08-2025 by Alfred Lotsu
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DriverStatus {
    Offline,       // Driver is not available for work
    Online,        // Driver is available but not on a ride
    OnRide,        // Driver is currently on a delivery
    OnBreak,       // Driver is taking a break
    Maintenance,   // Vehicle is in maintenance
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum VehicleType {
    Motorcycle,
    Car,
    Van,
    Truck,
    Bicycle,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vehicle {
    pub id: String,
    pub license_plate: String,
    pub vehicle_type: VehicleType,
    pub make: String,
    pub model: String,
    pub year: u16,
    pub color: String,
    pub capacity_kg: f32,  // Maximum load capacity in kilograms
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: Option<f64>,  // Accuracy in meters
    pub heading: Option<f64>,   // Direction in degrees (0-360)
    pub speed: Option<f64>,     // Speed in km/h
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Driver {
    pub id: String,
    pub user_id: String,        // Reference to user account
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub email: String,
    pub status: DriverStatus,
    pub current_location: Option<Location>,
    pub vehicle: Vehicle,
    pub rating: f32,            // Average rating (0-5)
    pub total_rides: u32,       // Total completed deliveries
    pub is_verified: bool,
    pub is_active: bool,
    pub current_ride_id: Option<String>, // Currently assigned ride
    pub device_token: Option<String>,    // For push notifications
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverRegistration {
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub email: String,
    pub license_plate: String,
    pub vehicle_type: VehicleType,
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub vehicle_year: u16,
    pub vehicle_color: String,
    pub capacity_kg: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverStatusUpdate {
    pub driver_id: String,
    pub status: DriverStatus,
    pub location: Option<Location>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverLocationUpdate {
    pub driver_id: String,
    pub location: Location,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverResponse {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub status: DriverStatus,
    pub current_location: Option<Location>,
    pub vehicle: Vehicle,
    pub rating: f32,
    pub total_rides: u32,
    pub is_verified: bool,
    pub current_ride_id: Option<String>,
}