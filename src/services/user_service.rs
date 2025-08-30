// src/services/user_service.rs
use async_trait::async_trait;
use chrono::{Utc};
use std::sync::Arc;
use tracing;

use crate::{
    errors::SparrowError as AppError,
    models::user::{
        Address, PaymentMethod, User, UserLogin, UserPreferences, UserRegistration, UserResponse, UserStatus, UserUpdate
    },
    services::{cache_service::{CacheKey, CacheService}, messaging_service::{self, NotificationService}},
    utils::id_generator::{IdGenerator, IdType, WithGeneratedId}, ValidationError,
};

#[async_trait]
pub trait UserOperations: Send + Sync {
    async fn register_user(&self, registration: UserRegistration) -> Result<UserResponse, AppError>;
    async fn login_user(&self, login: UserLogin) -> Result<(UserResponse, String), AppError>; // Returns user + auth token
    async fn get_user(&self, user_id: &str) -> Result<Option<UserResponse>, AppError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserResponse>, AppError>;
    async fn get_user_by_phone(&self, phone: &str) -> Result<Option<UserResponse>, AppError>;
    async fn update_user(&self, user_id: &str, update: UserUpdate) -> Result<UserResponse, AppError>;
    async fn update_user_device_token(&self, user_id: &str, device_token: String) -> Result<UserResponse, AppError>;
    async fn add_user_address(&self, user_id: &str, address: Address) -> Result<UserResponse, AppError>;
    async fn set_primary_address(&self, user_id: &str, address_id: &str) -> Result<UserResponse, AppError>;
    async fn add_payment_method(&self, user_id: &str, payment_method: PaymentMethod) -> Result<UserResponse, AppError>;
    async fn set_primary_payment_method(&self, user_id: &str, payment_id: &str) -> Result<UserResponse, AppError>;
    async fn update_user_preferences(&self, user_id: &str, preferences: UserPreferences) -> Result<UserResponse, AppError>;
    async fn verify_user_email(&self, user_id: &str) -> Result<UserResponse, AppError>;
    async fn verify_user_phone(&self, user_id: &str) -> Result<UserResponse, AppError>;
    async fn deactivate_user(&self, user_id: &str) -> Result<(), AppError>;
}

pub struct UserService {
    cache_service: Arc<CacheService>,
    notification_service: Arc<dyn NotificationService>,
}

impl UserService {
    pub fn new(
        cache_service: Arc<CacheService>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            cache_service,
            notification_service,
        }
    }
    
    fn to_response(&self, user: User) -> UserResponse {
        UserResponse {
            id: user.id,
            user_type: user.user_type,
            status: user.status,
            email: user.email,
            phone_number: user.phone_number,
            country_code: user.country_code,
            first_name: user.first_name,
            last_name: user.last_name,
            display_name: user.display_name,
            is_email_verified: user.is_email_verified,
            is_phone_verified: user.is_phone_verified,
            profile_picture: None, // Would come from user profile
            created_at: user.created_at,
        }
    }
    
    async fn hash_password(&self, password: &str) -> Result<String, AppError> {
        // In production, use argon2 or bcrypt
        // For now, simple placeholder
        Ok(format!("hashed_{}", password))
    }
    
    async fn verify_password(&self, password: &str, hashed_password: &str) -> Result<bool, AppError> {
        Ok(hashed_password == format!("hashed_{}", password))
    }
    
    async fn generate_auth_token(&self, user_id: &str) -> Result<String, AppError> {
        // In production, use JWT or similar
        // For now, simple token generation
        Ok(format!("token_{}_{}", user_id, Utc::now().timestamp()))
    }
}

#[async_trait]
impl UserOperations for UserService {
    async fn register_user(&self, registration: UserRegistration) -> Result<UserResponse, AppError> {
        tracing::info!("Registering user: {}", registration.email);
        
        // Check if user already exists
        if let phone_number = &registration.phone_number {
            if self.get_user_by_phone(phone_number).await?.is_some() {
                return Err(AppError::validation_error("phone_number", "User already exists with this phone number"));
            }
        }
        
        if let Some(_) = self.get_user_by_email(&registration.email).await? {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                message: "User already exists with this email".to_string(),
                field: "email".to_string(),
            }]));
        }
        
        // Hash password
        let hashed_password = self.hash_password(&registration.password).await?;
        
        // Create user with our ID generator
        let mut user = User {
            id: String::new(), // Will be set by with_generated_id
            user_type: registration.user_type,
            status: UserStatus::PendingVerification,
            email: registration.email,
            phone_number: registration.phone_number,
            country_code: registration.country_code,
            first_name: registration.first_name,
            last_name: registration.last_name,
            display_name: None,
            is_email_verified: false,
            is_phone_verified: false,
            device_tokens: Vec::new(),
            last_login: None,
            current_session: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Use our WithGeneratedId trait to set the ID
        user.set_generated_id(IdType::User);
        
        // Cache the user
        self.cache_service.cache_user(&user).await?;
        
        // Cache credentials (in production, use proper auth service)
        self.cache_service.cache_user_credentials(&user.id, &hashed_password).await?;
        
        // Cache lookup indices
        self.cache_service.cache_user_index(&user).await?;
        
        // Send welcome notification
        self.notification_service.send_to_user(
            &user.id,
            messaging_service::NotificationMessage {
                title: "👋 Welcome to Ghana Delivery!".to_string(),
                body: "Thank you for joining our delivery platform. Start shipping today!".to_string(),
                data: Some(serde_json::json!({
                    "type": "welcome",
                    "user_id": user.id,
                    "timestamp": Utc::now().to_rfc3339(),
                })),
                priority: messaging_service::NotificationPriority::Normal,
            }
        ).await?;
        
        tracing::info!("User registered successfully: {}", user.id);
        
        Ok(self.to_response(user))
    }
    
    async fn login_user(&self, login: UserLogin) -> Result<(UserResponse, String), AppError> {
        tracing::info!("User login attempt");
        
        // Find user by email or phone
        let user = if let Some(email) = login.email {
            self.get_user_by_email(&email).await?
        } else if let Some(phone) = login.phone_number {
            self.get_user_by_phone(&phone).await?
        } else {
            None
        };
        
        let user = user.ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;
        
        // Verify password (in production, get from auth service)
        let hashed_password = self.cache_service.get_user_credentials(&user.id).await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;
        
        if !self.verify_password(&login.password, &hashed_password).await? {
            return Err(AppError::Unauthorized("Invalid password".to_string()));
        }
        
        // Update device token if provided
        if let Some(device_token) = login.device_token {
            self.update_user_device_token(&user.id, device_token).await?;
        }
        
        // Generate auth token
        let auth_token = self.generate_auth_token(&user.id).await?;
        
        // Update last login
        let mut user_full: User = self.cache_service.get_user(&CacheKey::Simple(user.id)).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        user_full.last_login = Some(Utc::now());
        user_full.current_session = Some(auth_token.clone());
        user_full.updated_at = Utc::now();
        
        self.cache_service.cache_user(&user_full).await?;
        
        tracing::info!("User logged in successfully: {}", user_full.id);
        
        Ok((self.to_response(user_full), auth_token))
    }
    
    async fn get_user(&self, user_id: &str) -> Result<Option<UserResponse>, AppError> {
        if !IdGenerator::validate_id(user_id, Some(IdType::User)) {
            tracing::warn!("Invalid user ID format: {}", user_id);
            return Ok(None);
        }
        
        tracing::debug!("Getting user: {}", user_id);
        
        if let Some(user) = self.cache_service.get_user(&CacheKey::Simple(user_id.to_string())).await? {
            return Ok(Some(self.to_response(user)));
        }
        
        Ok(None)
    }
    
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserResponse>, AppError> {
        tracing::debug!("Getting user by email: {}", email);
        
        if let Some(user_id) = self.cache_service.get_user_id_by_email(email).await? {
            return self.get_user(&user_id).await;
        }
        
        Ok(None)
    }
    
    async fn get_user_by_phone(&self, phone: &str) -> Result<Option<UserResponse>, AppError> {
        tracing::debug!("Getting user by phone: {}", phone);
        
        if let Some(user_id) = self.cache_service.get_user_id_by_phone(phone).await? {
            return self.get_user(&user_id).await;
        }
        
        Ok(None)
    }
    
    async fn update_user(&self, user_id: &str, update: UserUpdate) -> Result<UserResponse, AppError> {
        if !IdGenerator::validate_id(user_id, Some(IdType::User)) {
            return Err(AppError::ValidationFailed(vec![ ValidationError{
            message: "Invalid user ID format".to_string(),
            field: "user_id".to_string(),
        }]));
        }
        
        tracing::info!("Updating user: {}", user_id);
        
        let mut user: User = self.cache_service.get_user(&CacheKey::Simple(user_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        // Apply updates
        if let Some(first_name) = update.first_name {
            user.first_name = first_name;
        }
        if let Some(last_name) = update.last_name {
            user.last_name = last_name;
        }
        if let Some(display_name) = update.display_name {
            user.display_name = Some(display_name);
        }
        if let Some(email) = update.email {
            user.email = email;
            user.is_email_verified = false; // Require re-verification
        }
        if let Some(phone_number) = update.phone_number {
            user.phone_number = phone_number;
            user.is_phone_verified = false; // Require re-verification
        }
        if let Some(country_code) = update.country_code {
            user.country_code = country_code;
        }
        
        user.updated_at = Utc::now();
        
        // Update cache
        self.cache_service.cache_user(&user).await?;
        
        tracing::debug!("User updated successfully: {}", user_id);
        
        Ok(self.to_response(user))
    }
    
    async fn update_user_device_token(&self, user_id: &str, device_token: String) -> Result<UserResponse, AppError> {
        if !IdGenerator::validate_id(user_id, Some(IdType::User)) {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                message: "Invalid user ID format".to_string(),
                field: user_id.to_string(),
            }]));
        }
        
        tracing::debug!("Updating device token for user: {}", user_id);
        
        let mut user: User = self.cache_service.get_user(&CacheKey::Simple(user_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        // Add or update device token
        if !user.device_tokens.contains(&device_token) {
            user.device_tokens.push(device_token);
            user.updated_at = Utc::now();
            self.cache_service.cache_user(&user).await?;
        }
        
        Ok(self.to_response(user))
    }
    
    async fn add_user_address(&self, user_id: &str, address: Address) -> Result<UserResponse, AppError> {
        // Implementation would handle address management
        // For now, placeholder
        self.get_user(user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }
    
    async fn set_primary_address(&self, user_id: &str, address_id: &str) -> Result<UserResponse, AppError> {
        // Implementation would handle address management
        self.get_user(user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }
    
    async fn add_payment_method(&self, user_id: &str, payment_method: PaymentMethod) -> Result<UserResponse, AppError> {
        // Implementation would handle payment methods
        self.get_user(user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }
    
    async fn set_primary_payment_method(&self, user_id: &str, payment_id: &str) -> Result<UserResponse, AppError> {
        // Implementation would handle payment methods
        self.get_user(user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }
    
    async fn update_user_preferences(&self, user_id: &str, preferences: UserPreferences) -> Result<UserResponse, AppError> {
        // Implementation would handle preferences
        self.get_user(user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }
    
    async fn verify_user_email(&self, user_id: &str) -> Result<UserResponse, AppError> {
        if !IdGenerator::validate_id(user_id, Some(IdType::User)) {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                message: "Invalid user ID format".to_string(),
                field: user_id.to_string(),
            }]));
        }
        
        tracing::info!("Verifying email for user: {}", user_id);
        
        let mut user: User = self.cache_service.get_user(&CacheKey::Simple(user_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        user.is_email_verified = true;
        if user.is_phone_verified && user.status == UserStatus::PendingVerification {
            user.status = UserStatus::Active;
        }
        user.updated_at = Utc::now();
        
        self.cache_service.cache_user(&user).await?;
        
        tracing::debug!("Email verified for user: {}", user_id);
        
        Ok(self.to_response(user))
    }
    
    async fn verify_user_phone(&self, user_id: &str) -> Result<UserResponse, AppError> {
        if !IdGenerator::validate_id(user_id, Some(IdType::User)) {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                message: "Invalid user ID format".to_string(),
                field: user_id.to_string(),
            }]));
        }
        
        tracing::info!("Verifying phone for user: {}", user_id);
        
        let mut user: User = self.cache_service.get_user(&CacheKey::Simple(user_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        user.is_phone_verified = true;
        if user.is_email_verified && user.status == UserStatus::PendingVerification {
            user.status = UserStatus::Active;
        }
        user.updated_at = Utc::now();
        
        self.cache_service.cache_user(&user).await?;
        
        tracing::debug!("Phone verified for user: {}", user_id);
        
        Ok(self.to_response(user))
    }
    
    async fn deactivate_user(&self, user_id: &str) -> Result<(), AppError> {
        if !IdGenerator::validate_id(user_id, Some(IdType::User)) {
            return Err(AppError::ValidationFailed(vec![ValidationError {
                message: "Invalid user ID format".to_string(),
                field: user_id.to_string(),
            }]));
        }
        
        tracing::info!("Deactivating user: {}", user_id);
        
        let mut user: User = self.cache_service.get_user(&CacheKey::Simple(user_id.to_string())).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        user.status = UserStatus::Inactive;
        user.updated_at = Utc::now();
        
        self.cache_service.cache_user(&user).await?;
        
        tracing::info!("User deactivated: {}", user_id);
        
        Ok(())
    }
}
