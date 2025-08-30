// src/utils/id_generator.rs
use chrono::{DateTime, Utc, TimeZone};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdType {
    User,
    Driver,
    Job,
    Vehicle,
    Payment,
    Address,
    Notification,
    SupportTicket,
    Verification,
    Reward,
}

impl IdType {
    pub fn to_prefix(&self) -> &'static str {
        match self {
            IdType::User => "usr",
            IdType::Driver => "drv",
            IdType::Job => "job",
            IdType::Vehicle => "veh",
            IdType::Payment => "pay",
            IdType::Address => "add",
            IdType::Notification => "not",
            IdType::SupportTicket => "tic",
            IdType::Verification => "ver",
            IdType::Reward => "rew",
        }
    }
}



impl fmt::Display for IdType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_prefix())
    }
}

pub struct IdGenerator;

impl IdGenerator {
    /// Generate a unique ID with format: {prefix}-{date}-{random_suffix}
    /// Where random_suffix is 5 characters: 3 hexchars + 2 alphanumeric or 3 alphanumeric + 2 hexchars
    pub fn generate(id_type: IdType) -> String {
        Self::generate_with_timestamp(id_type, Utc::now())
    }

    /// Generate ID with a specific timestamp (useful for testing)
    pub fn generate_with_timestamp(id_type: IdType, timestamp: DateTime<Utc>) -> String {
        let date_part = timestamp.format("%y%m%d").to_string(); // YYMMDD format
        let random_suffix = Self::generate_random_suffix();
        
        format!("{}-{}-{}", id_type.to_prefix(), date_part, random_suffix)
    }

    /// Generate the random suffix (5 characters mixing hex and alphanumeric)
    fn generate_random_suffix() -> String {
        // 50% chance: 3 hexchars + 2 alphanumeric
        // 50% chance: 3 alphanumeric + 2 hexchars
        if rand::random::<bool>() {
            format!(
                "{}{}",
                Self::generate_hex_chars(3),
                Self::generate_alphanumeric_chars(2)
            )
        } else {
            format!(
                "{}{}",
                Self::generate_alphanumeric_chars(3),
                Self::generate_hex_chars(2)
            )
        }
    }

    /// Generate n hexadecimal characters (0-9, a-f)
    fn generate_hex_chars(n: usize) -> String {
        const HEX_CHARS: &[u8] = b"0123456789abcdef";
        Self::generate_from_chars(HEX_CHARS, n)
    }

    /// Generate n alphanumeric characters (a-z, A-Z, 0-9)
    fn generate_alphanumeric_chars(n: usize) -> String {
        const ALPHANUMERIC_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        Self::generate_from_chars(ALPHANUMERIC_CHARS, n)
    }

    /// Generate n random characters from a given character set
    fn generate_from_chars(charset: &[u8], n: usize) -> String {
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        (0..n)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    }

    /// Parse an ID to extract its components
    pub fn parse_id(id: &str) -> Option<ParsedId> {
        let parts: Vec<&str> = id.split('-').collect();
        if parts.len() != 3 {
            return None;
        }

        let prefix = parts[0];
        let date_part = parts[1];
        let random_suffix = parts[2];

        if date_part.len() != 6 || random_suffix.len() != 5 {
            return None;
        }

        // Determine ID type from prefix
        let id_type = match prefix {
            "usr" => IdType::User,
            "drv" => IdType::Driver,
            "job" => IdType::Job,
            "veh" => IdType::Vehicle,
            "pay" => IdType::Payment,
            "add" => IdType::Address,
            "not" => IdType::Notification,
            "tic" => IdType::SupportTicket,
            "ver" => IdType::Verification,
            "rew" => IdType::Reward,
            _ => return None,
        };

        // Parse date (YYMMDD format)
        let year = format!("20{}", &date_part[0..2]).parse::<i32>().ok()?;
        let month = date_part[2..4].parse::<u32>().ok()?;
        let day = date_part[4..6].parse::<u32>().ok()?;

        // Validate date components
        if month < 1 || month > 12 || day < 1 || day > 31 {
            return None;
        }

        Some(ParsedId {
            id_type,
            year,
            month,
            day,
            random_suffix: random_suffix.to_string(),
        })
    }

    /// Validate if an ID matches the expected format and type
    pub fn validate_id(id: &str, expected_type: Option<IdType>) -> bool {
        match Self::parse_id(id) {
            Some(parsed) => {
                if let Some(expected) = expected_type {
                    parsed.id_type == expected
                } else {
                    true
                }
            }
            None => false,
        }
    }

    /// Generate a batch of unique IDs
    pub fn generate_batch(id_type: IdType, count: usize) -> Vec<String> {
        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            ids.push(Self::generate(id_type));
        }
        ids
    }

    /// Generate a readable ID for display purposes (shorter format)
    pub fn generate_readable(id_type: IdType) -> String {
        let timestamp = Utc::now();
        let date_part = timestamp.format("%y%m").to_string(); // YYMM format
        let random_suffix = Self::generate_alphanumeric_chars(4); // Shorter suffix
        
        format!("{}-{}-{}", id_type.to_prefix(), date_part, random_suffix)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedId {
    pub id_type: IdType,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub random_suffix: String,
}

// Custom error type for ID generation
#[derive(Debug, thiserror::Error)]
pub enum IdError {
    #[error("Invalid ID format")]
    InvalidFormat,
    
    #[error("Unknown ID type: {0}")]
    UnknownType(String),
    
    #[error("Invalid date component in ID")]
    InvalidDate,
}

// Integration with your models
pub trait WithGeneratedId {
    fn set_generated_id(&mut self, id_type: IdType);
    
    fn with_generated_id(mut self, id_type: IdType) -> Self
    where
        Self: Sized,
    {
        self.set_generated_id(id_type);
        self
    }
}

// Example implementation for your models
impl WithGeneratedId for crate::models::user::User {
    fn set_generated_id(&mut self, id_type: IdType) {
        self.id = IdGenerator::generate(id_type);
    }
}

impl WithGeneratedId for crate::models::driver::Driver {
    fn set_generated_id(&mut self, id_type: IdType) {
        self.id = IdGenerator::generate(id_type);
    }
}

impl WithGeneratedId for crate::models::job::Job {
    fn set_generated_id(&mut self, id_type: IdType) {
        self.id = IdGenerator::generate(id_type);
    }
}

// Utility functions for common ID types
pub fn generate_user_id() -> String {
    IdGenerator::generate(IdType::User)
}

pub fn generate_driver_id() -> String {
    IdGenerator::generate(IdType::Driver)
}

pub fn generate_job_id() -> String {
    IdGenerator::generate(IdType::Job)
}

pub fn generate_vehicle_id() -> String {
    IdGenerator::generate(IdType::Vehicle)
}

pub fn generate_payment_id() -> String {
    IdGenerator::generate(IdType::Payment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_id_generation() {
        let user_id = IdGenerator::generate(IdType::User);
        assert!(user_id.starts_with("usr-"));
        assert_eq!(user_id.split('-').count(), 3);
        
        let job_id = IdGenerator::generate(IdType::Job);
        assert!(job_id.starts_with("job-"));
    }

    #[test]
    fn test_id_parsing() {
        let test_date = Utc.with_ymd_and_hms(2023, 12, 7, 0, 0, 0).unwrap();
        let id = IdGenerator::generate_with_timestamp(IdType::Driver, test_date);
        
        let parsed = IdGenerator::parse_id(&id).unwrap();
        assert_eq!(parsed.id_type, IdType::Driver);
        assert_eq!(parsed.year, 2023);
        assert_eq!(parsed.month, 12);
        assert_eq!(parsed.day, 7);
        assert_eq!(parsed.random_suffix.len(), 5);
    }

    #[test]
    fn test_validation() {
        let valid_id = "usr-231207-a1b2c";
        assert!(IdGenerator::validate_id(valid_id, Some(IdType::User)));
        assert!(!IdGenerator::validate_id(valid_id, Some(IdType::Driver)));
        
        let invalid_id = "invalid-format";
        assert!(!IdGenerator::validate_id(invalid_id, None));
    }

    #[test]
    fn test_random_suffix_pattern() {
        for _ in 0..100 {
            let suffix = IdGenerator::generate_random_suffix();
            assert_eq!(suffix.len(), 5);
            
            // Check that it contains both hex and alphanumeric characters
            let has_hex = suffix.chars().any(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase());
            let has_alnum = suffix.chars().any(|c| c.is_ascii_alphanumeric());
            
            assert!(has_hex, "Suffix should contain hex characters: {}", suffix);
            assert!(has_alnum, "Suffix should contain alphanumeric characters: {}", suffix);
        }
    }
}

impl IdGenerator {
    // Add date parsing capability to our ID generator
    pub fn parse_creation_date(id: &str) -> Option<DateTime<Utc>> {
        if let Some(parsed) = Self::parse_id(id) {
            // Convert YYMMDD to DateTime
            // Example: "231207" -> December 7, 2023
            let year = 2000 + parsed.year; // Assuming YY is years since 2000
            let date = Utc.with_ymd_and_hms(year, parsed.month, parsed.day, 0, 0, 0);
            date.single()
        } else {
            None
        }
    }
    
    pub fn is_id_recent(id: &str, max_age_days: i64) -> Option<bool> {
        Self::parse_creation_date(id).map(|created_at| {
            let age = Utc::now().signed_duration_since(created_at);
            age.num_days() <= max_age_days
        })
    }
}

impl ParsedId {
    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        Utc.with_ymd_and_hms(self.year, self.month, self.day, 0, 0, 0).single()
    }
}