// src/models/user.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserType {
    Customer,    // Someone ordering deliveries
    Driver,      // Someone delivering packages
    Admin,       // Platform administrator
    Dispatcher,  // Manages deliveries and drivers
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Address {
    pub id: String,
    pub label: String,           // e.g., "Home", "Work", "Mom's House"
    pub street: String,
    pub city: String,
    pub region: String,          // Region/State/Province
    pub country: String,         // e.g., "Ghana"
    pub postal_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentMethod {
    pub id: String,
    pub method_type: PaymentMethodType,
    pub provider: String,        // e.g., "MTN Mobile Money", "Vodafone Cash", "Visa"
    pub account_number: String,  // Phone number for Mobile Money, card number for cards
    pub account_name: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PaymentMethodType {
    MobileMoney,  // MTN, Vodafone, AirtelTigo, etc.
    BankCard,     // Visa, Mastercard
    BankAccount,  // Bank transfer
    Cash,         // Cash on delivery
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPreferences {
    pub language: String,           // e.g., "en", "fr", "ak", "tw"
    pub currency: String,           // e.g., "GHS" for Ghana Cedis
    pub notifications: NotificationPreferences,
    pub theme: String,              // e.g., "light", "dark", "system"
    pub search_history: Vec<String>, // Recent search queries
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationPreferences {
    pub push_notifications: bool,
    pub email_notifications: bool,
    pub sms_notifications: bool,
    pub ride_updates: bool,
    pub promotional_offers: bool,
    pub security_alerts: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub date_of_birth: Option<DateTime<Utc>>,
    pub gender: Option<String>,      // e.g., "male", "female", "other"
    pub profile_picture: Option<String>, // URL to profile image
    pub bio: Option<String>,
    pub preferences: UserPreferences,
    pub addresses: Vec<Address>,
    pub payment_methods: Vec<PaymentMethod>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub user_type: UserType,
    pub status: UserStatus,
    pub email: String,
    pub phone_number: String,
    pub country_code: String,    // e.g., "+233" for Ghana
    pub first_name: String,
    pub last_name: String,
    pub display_name: Option<String>,
    pub is_email_verified: bool,
    pub is_phone_verified: bool,
    pub device_tokens: Vec<String>, // For push notifications
    pub last_login: Option<DateTime<Utc>>,
    pub current_session: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response Models
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistration {
    pub user_type: UserType,
    pub email: String,
    pub phone_number: String,
    pub country_code: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,        // Will be hashed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub password: String,
    pub device_token: Option<String>, // For push notifications
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressCreate {
    pub label: String,
    pub street: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub postal_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentMethodCreate {
    pub method_type: PaymentMethodType,
    pub provider: String,
    pub account_number: String,
    pub account_name: String,
    pub is_primary: bool,
}

// Response Models (for API responses)
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub user_type: UserType,
    pub status: UserStatus,
    pub email: String,
    pub phone_number: String,
    pub country_code: String,
    pub first_name: String,
    pub last_name: String,
    pub display_name: Option<String>,
    pub is_email_verified: bool,
    pub is_phone_verified: bool,
    pub profile_picture: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
    pub device_id: Option<String>,
    pub is_revoked: bool,
}

// Statistics and analytics
#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: String,
    pub total_rides: u32,
    pub completed_rides: u32,
    pub cancelled_rides: u32,
    pub total_spent: f64,        // In GHS
    pub average_rating: f32,
    pub joined_at: DateTime<Utc>,
    pub last_ride: Option<DateTime<Utc>>,
}

// Support and verification
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub user_id: String,
    pub document_type: String,   // e.g., "national_id", "driver_license", "passport"
    pub document_front: String,  // URL or base64 encoded image
    pub document_back: Option<String>,
    pub selfie: Option<String>,  // URL or base64 encoded selfie with document
    pub status: VerificationStatus,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Rejected,
    UnderReview,
}

// Support tickets
#[derive(Debug, Serialize, Deserialize)]
pub struct SupportTicket {
    pub id: String,
    pub user_id: String,
    pub category: String,        // e.g., "payment", "delivery", "technical"
    pub subject: String,
    pub description: String,
    pub status: TicketStatus,
    pub priority: TicketPriority,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

// Loyalty and rewards
#[derive(Debug, Serialize, Deserialize)]
pub struct LoyaltyProgram {
    pub user_id: String,
    pub points: u32,
    pub tier: LoyaltyTier,
    pub rides_this_month: u32,
    pub total_rides: u32,
    pub rewards: Vec<Reward>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum LoyaltyTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reward {
    pub id: String,
    pub name: String,
    pub description: String,
    pub points_required: u32,
    pub is_claimed: bool,
    pub claimed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}