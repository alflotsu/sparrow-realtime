use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Main error type for the sparrow-realtime service
#[derive(Debug)]
pub enum SparrowError {
    // HTTP and API errors
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    TooManyRequests(String),
    InternalServer(String),

    // Database and Redis errors
    RedisConnection(String),
    RedisQuery(String),
    RedisTimeout,
    RedisSerialization(String),

    // External service errors
    FirebaseAuth(String),
    FirebaseDatabase(String),
    FcmDelivery(String),
    FcmInvalidToken(String),
    FcmQuotaExceeded,

    // Network and HTTP client errors
    NetworkTimeout,
    NetworkConnection(String),
    HttpClient(String),
    InvalidUrl(String),

    // Serialization and parsing errors
    JsonParsing(String),
    JsonSerialization(String),
    InvalidFormat(String),

    // Business logic errors
    InvalidUserId(String),
    InvalidDriverId(String),
    InvalidJobId(String),
    UserNotFound(String),
    DriverNotFound(String),
    JobNotFound(String),
    JobAlreadyAssigned,
    JobAlreadyCompleted,
    DriverNotAvailable,
    InvalidJobStatus(String),

    // Realtime communication errors
    WebSocketConnection(String),
    WebSocketMessage(String),
    ChannelClosed,
    MessageDeliveryFailed(String),
    BroadcastFailed(String),

    // Validation errors
    ValidationFailed(Vec<ValidationError>),
    MissingRequiredField(String),
    InvalidFieldValue { field: String, value: String, reason: String },

    // Configuration and setup errors
    ConfigurationError(String),
    MissingEnvironmentVariable(String),
    InvalidConfiguration(String),

    // Security and authentication errors
    TokenExpired,
    TokenInvalid,
    InsufficientPermissions,
    RateLimitExceeded,

    // Resource management errors
    ResourceNotAvailable(String),
    ResourceExhausted(String),
    ServiceUnavailable(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl fmt::Display for SparrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SparrowError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            SparrowError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            SparrowError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            SparrowError::NotFound(msg) => write!(f, "Not found: {}", msg),
            SparrowError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            SparrowError::TooManyRequests(msg) => write!(f, "Too many requests: {}", msg),
            SparrowError::InternalServer(msg) => write!(f, "Internal server error: {}", msg),

            SparrowError::RedisConnection(msg) => write!(f, "Redis connection error: {}", msg),
            SparrowError::RedisQuery(msg) => write!(f, "Redis query error: {}", msg),
            SparrowError::RedisTimeout => write!(f, "Redis operation timed out"),
            SparrowError::RedisSerialization(msg) => write!(f, "Redis serialization error: {}", msg),

            SparrowError::FirebaseAuth(msg) => write!(f, "Firebase authentication error: {}", msg),
            SparrowError::FirebaseDatabase(msg) => write!(f, "Firebase database error: {}", msg),
            SparrowError::FcmDelivery(msg) => write!(f, "FCM delivery error: {}", msg),
            SparrowError::FcmInvalidToken(msg) => write!(f, "Invalid FCM token: {}", msg),
            SparrowError::FcmQuotaExceeded => write!(f, "FCM quota exceeded"),

            SparrowError::NetworkTimeout => write!(f, "Network request timed out"),
            SparrowError::NetworkConnection(msg) => write!(f, "Network connection error: {}", msg),
            SparrowError::HttpClient(msg) => write!(f, "HTTP client error: {}", msg),
            SparrowError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),

            SparrowError::JsonParsing(msg) => write!(f, "JSON parsing error: {}", msg),
            SparrowError::JsonSerialization(msg) => write!(f, "JSON serialization error: {}", msg),
            SparrowError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),

            SparrowError::InvalidUserId(id) => write!(f, "Invalid user ID: {}", id),
            SparrowError::InvalidDriverId(id) => write!(f, "Invalid driver ID: {}", id),
            SparrowError::InvalidJobId(id) => write!(f, "Invalid job ID: {}", id),
            SparrowError::UserNotFound(id) => write!(f, "User not found: {}", id),
            SparrowError::DriverNotFound(id) => write!(f, "Driver not found: {}", id),
            SparrowError::JobNotFound(id) => write!(f, "Job not found: {}", id),
            SparrowError::JobAlreadyAssigned => write!(f, "Job is already assigned to another driver"),
            SparrowError::JobAlreadyCompleted => write!(f, "Job is already completed"),
            SparrowError::DriverNotAvailable => write!(f, "Driver is not available"),
            SparrowError::InvalidJobStatus(status) => write!(f, "Invalid job status: {}", status),

            SparrowError::WebSocketConnection(msg) => write!(f, "WebSocket connection error: {}", msg),
            SparrowError::WebSocketMessage(msg) => write!(f, "WebSocket message error: {}", msg),
            SparrowError::ChannelClosed => write!(f, "Communication channel closed"),
            SparrowError::MessageDeliveryFailed(msg) => write!(f, "Message delivery failed: {}", msg),
            SparrowError::BroadcastFailed(msg) => write!(f, "Broadcast failed: {}", msg),

            SparrowError::ValidationFailed(errors) => {
                write!(f, "Validation failed: {} errors", errors.len())
            }
            SparrowError::MissingRequiredField(field) => write!(f, "Missing required field: {}", field),
            SparrowError::InvalidFieldValue { field, value, reason } => {
                write!(f, "Invalid value '{}' for field '{}': {}", value, field, reason)
            }

            SparrowError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            SparrowError::MissingEnvironmentVariable(var) => {
                write!(f, "Missing environment variable: {}", var)
            }
            SparrowError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),

            SparrowError::TokenExpired => write!(f, "Authentication token has expired"),
            SparrowError::TokenInvalid => write!(f, "Authentication token is invalid"),
            SparrowError::InsufficientPermissions => write!(f, "Insufficient permissions for this operation"),
            SparrowError::RateLimitExceeded => write!(f, "Rate limit exceeded"),

            SparrowError::ResourceNotAvailable(resource) => write!(f, "Resource not available: {}", resource),
            SparrowError::ResourceExhausted(resource) => write!(f, "Resource exhausted: {}", resource),
            SparrowError::ServiceUnavailable(service) => write!(f, "Service unavailable: {}", service),
        }
    }
}

impl std::error::Error for SparrowError {}

impl IntoResponse for SparrowError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details) = match self {
            SparrowError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg, None),
            SparrowError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg, None),
            SparrowError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg, None),
            SparrowError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg, None),
            SparrowError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg, None),
            SparrowError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, "too_many_requests", msg, None),

            SparrowError::ValidationFailed(errors) => {
                let details = serde_json::to_value(&errors).ok();
                (StatusCode::BAD_REQUEST, "validation_failed", "Validation errors occurred".to_string(), details)
            }
            SparrowError::MissingRequiredField(field) => {
                (StatusCode::BAD_REQUEST, "missing_field", format!("Missing required field: {}", field), None)
            }
            SparrowError::InvalidFieldValue { field, value, reason } => {
                (StatusCode::BAD_REQUEST, "invalid_field", format!("Invalid value for {}: {}", field, reason), None)
            }

            SparrowError::UserNotFound(id) => (StatusCode::NOT_FOUND, "user_not_found", format!("User not found: {}", id), None),
            SparrowError::DriverNotFound(id) => (StatusCode::NOT_FOUND, "driver_not_found", format!("Driver not found: {}", id), None),
            SparrowError::JobNotFound(id) => (StatusCode::NOT_FOUND, "job_not_found", format!("Job not found: {}", id), None),

            SparrowError::JobAlreadyAssigned => (StatusCode::CONFLICT, "job_already_assigned", "Job is already assigned".to_string(), None),
            SparrowError::JobAlreadyCompleted => (StatusCode::CONFLICT, "job_already_completed", "Job is already completed".to_string(), None),
            SparrowError::DriverNotAvailable => (StatusCode::CONFLICT, "driver_not_available", "Driver is not available".to_string(), None),

            SparrowError::TokenExpired => (StatusCode::UNAUTHORIZED, "token_expired", "Authentication token has expired".to_string(), None),
            SparrowError::TokenInvalid => (StatusCode::UNAUTHORIZED, "token_invalid", "Authentication token is invalid".to_string(), None),
            SparrowError::InsufficientPermissions => (StatusCode::FORBIDDEN, "insufficient_permissions", "Insufficient permissions".to_string(), None),
            SparrowError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded", "Rate limit exceeded".to_string(), None),

            SparrowError::ServiceUnavailable(service) => {
                (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", format!("Service unavailable: {}", service), None)
            }

            // All other errors are treated as internal server errors
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", self.to_string(), None),
        };

        let error_response = ErrorResponse {
            error: error_type.to_string(),
            message,
            details,
        };

        (status, axum::Json(error_response)).into_response()
    }
}

// Convenience type alias for Results
pub type SparrowResult<T> = Result<T, SparrowError>;

// Conversion implementations for common error types
impl From<redis::RedisError> for SparrowError {
    fn from(err: redis::RedisError) -> Self {
        match err.kind() {
            redis::ErrorKind::IoError => SparrowError::RedisConnection(err.to_string()),
            redis::ErrorKind::ResponseError => SparrowError::RedisQuery(err.to_string()),
            redis::ErrorKind::AuthenticationFailed => SparrowError::RedisConnection("Authentication failed".to_string()),
            _ => SparrowError::RedisQuery(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for SparrowError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            SparrowError::NetworkTimeout
        } else if err.is_connect() {
            SparrowError::NetworkConnection(err.to_string())
        } else {
            SparrowError::HttpClient(err.to_string())
        }
    }
}

impl From<serde_json::Error> for SparrowError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_syntax() {
            SparrowError::JsonParsing(err.to_string())
        } else {
            SparrowError::JsonSerialization(err.to_string())
        }
    }
}

impl From<uuid::Error> for SparrowError {
    fn from(err: uuid::Error) -> Self {
        SparrowError::InvalidFormat(format!("Invalid UUID: {}", err))
    }
}

impl From<chrono::ParseError> for SparrowError {
    fn from(err: chrono::ParseError) -> Self {
        SparrowError::InvalidFormat(format!("Invalid date/time format: {}", err))
    }
}

// Helper functions for creating common errors
impl SparrowError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        SparrowError::BadRequest(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        SparrowError::Unauthorized(msg.into())
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        SparrowError::NotFound(resource.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        SparrowError::InternalServer(msg.into())
    }

    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        SparrowError::ValidationFailed(vec![ValidationError {
            field: field.into(),
            message: message.into(),
        }])
    }

    pub fn user_not_found(user_id: impl Into<String>) -> Self {
        SparrowError::UserNotFound(user_id.into())
    }

    pub fn driver_not_found(driver_id: impl Into<String>) -> Self {
        SparrowError::DriverNotFound(driver_id.into())
    }

    pub fn job_not_found(job_id: impl Into<String>) -> Self {
        SparrowError::JobNotFound(job_id.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = SparrowError::UserNotFound("123".to_string());
        assert_eq!(error.to_string(), "User not found: 123");
    }

    #[test]
    fn test_validation_error() {
        let error = SparrowError::validation_error("email", "Invalid email format");
        match error {
            SparrowError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "email");
                assert_eq!(errors[0].message, "Invalid email format");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_helper_functions() {
        assert!(matches!(SparrowError::bad_request("test"), SparrowError::BadRequest(_)));
        assert!(matches!(SparrowError::unauthorized("test"), SparrowError::Unauthorized(_)));
        assert!(matches!(SparrowError::not_found("test"), SparrowError::NotFound(_)));
        assert!(matches!(SparrowError::internal_error("test"), SparrowError::InternalServer(_)));
    }
}
