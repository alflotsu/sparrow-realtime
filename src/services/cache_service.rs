// // src/services/cache_service.rs
// use async_trait::async_trait;
// use redis::{Client};
// use serde::{de::DeserializeOwned, Serialize};
// use std::sync::Arc;
// use tokio::sync::RwLock;
// use chrono::{DateTime, Utc};
// use tracing;

// use crate::models::{user::User, job::Job};
// use crate::errors::SparrowError as AppError;

// // Cache configuration
// #[derive(Debug, Clone)]
// pub struct CacheConfig {
//     pub default_ttl_seconds: u64,
//     pub redis_url: String,
//     pub enabled: bool,
// }

// impl Default for CacheConfig {
//     fn default() -> Self {
//         Self {
//             default_ttl_seconds: 300, // 5 minutes
//             redis_url: "redis://127.0.0.1:6379".to_string(),
//             enabled: true,
//         }
//     }
// }

// // Cache key strategies
// #[derive(Debug, Clone)]
// pub enum CacheKey {
//     Simple(String),
//     Composite(Vec<String>),
//     Pattern(String),
// }

// impl CacheKey {
//     pub fn to_string(&self) -> String {
//         match self {
//             CacheKey::Simple(key) => key.clone(),
//             CacheKey::Composite(parts) => parts.join(":"),
//             CacheKey::Pattern(pattern) => pattern.clone(),
//         }
//     }
// }

// // ------------------------------
// // Traits (split to avoid E0283)
// // ------------------------------

// #[async_trait]
// pub trait CacheOperations<T>: Send + Sync
// where
//     T: Serialize + DeserializeOwned + Send + Sync + 'static,
// {
//     async fn get(&self, key: &CacheKey) -> Result<Option<T>, CacheError>;
//     async fn set(&self, key: &CacheKey, value: &T, ttl: Option<u64>) -> Result<(), CacheError>;
//     async fn get_or_set<F>(&self, key: &CacheKey, ttl: Option<u64>, factory: F) -> Result<T, CacheError>
//     where
//         F: Fn() -> futures::future::BoxFuture<'static, Result<T, CacheError>> + Send + Sync;
// }

// #[async_trait]
// pub trait KeyOperations: Send + Sync {
//     async fn delete(&self, key: &CacheKey) -> Result<(), CacheError>;
//     async fn exists(&self, key: &CacheKey) -> Result<bool, CacheError>;
// }

// #[async_trait]
// pub trait SetOperations: Send + Sync {
//     async fn sadd(&self, key: &CacheKey, value: &str) -> Result<(), CacheError>;
//     async fn smembers(&self, key: &CacheKey) -> Result<Vec<String>, CacheError>;
//     async fn srem(&self, key: &CacheKey, value: &str) -> Result<(), CacheError>;
// }

// // Enum to wrap different cache implementations
// pub enum Cache {
//     Redis(RedisCache),
//     Memory(MemoryCache),
// }

// // Redis-based cache implementation
// pub struct RedisCache {
//     client: Client,
//     config: CacheConfig,
//     connection: RwLock<Option<redis::aio::Connection>>,
// }

// impl RedisCache {
//     pub async fn new(config: CacheConfig) -> Result<Self, CacheError> {
//         let client = Client::open(config.redis_url.clone())
//             .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

//         let instance = Self {
//             client,
//             config,
//             connection: RwLock::new(None),
//         };

//         instance.connect().await?;
//         Ok(instance)
//     }

//     async fn connect(&self) -> Result<(), CacheError> {
//         let mut conn = self.connection.write().await;
//         if conn.is_none() {
//             *conn = Some(
//                 self.client
//                     .get_async_connection()
//                     .await
//                     .map_err(|e| CacheError::ConnectionError(e.to_string()))?,
//             );
//         }
//         Ok(())
//     }

//     async fn get_connection(&self) -> Result<redis::aio::Connection, CacheError> {
//         self.client
//             .get_async_connection()
//             .await
//             .map_err(|e| CacheError::ConnectionError(e.to_string()))
//     }
// }

// // -------- Redis impls --------

// #[async_trait]
// impl<T> CacheOperations<T> for RedisCache
// where
//     T: Serialize + DeserializeOwned + Send + Sync + 'static,
// {
//     async fn get(&self, key: &CacheKey) -> Result<Option<T>, CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let mut conn = self.get_connection().await?;

//         let data: Option<String> = redis::cmd("GET")
//             .arg(&key_str)
//             .query_async(&mut conn)
//             .await
//             .map_err(|e| CacheError::OperationError(e.to_string()))?;

//         match data {
//             Some(json) => {
//                 let value: T = serde_json::from_str(&json)
//                     .map_err(|e| CacheError::SerializationError(e.to_string()))?;
//                 Ok(Some(value))
//             }
//             None => Ok(None),
//         }
//     }

//     async fn set(&self, key: &CacheKey, value: &T, ttl: Option<u64>) -> Result<(), CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let json = serde_json::to_string(value)
//             .map_err(|e| CacheError::SerializationError(e.to_string()))?;

//         let mut conn = self.get_connection().await?;
//         let ttl = ttl.unwrap_or(self.config.default_ttl_seconds);

//         if ttl > 0 {
//             let _: () = redis::cmd("SET")
//                 .arg(&key_str)
//                 .arg(json)
//                 .arg("EX")
//                 .arg(ttl)
//                 .query_async(&mut conn)
//                 .await
//                 .map_err(|e| CacheError::OperationError(e.to_string()))?;
//         } else {
//             let _: () = redis::cmd("SET")
//                 .arg(&key_str)
//                 .arg(json)
//                 .query_async(&mut conn)
//                 .await
//                 .map_err(|e| CacheError::OperationError(e.to_string()))?;
//         }

//         Ok(())
//     }

//     async fn get_or_set<F>(&self, key: &CacheKey, ttl: Option<u64>, factory: F) -> Result<T, CacheError>
//     where
//         F: Fn() -> futures::future::BoxFuture<'static, Result<T, CacheError>> + Send + Sync,
//     {
//         if let Some(cached) = self.get(key).await? {
//             tracing::debug!("Cache hit for key: {}", key.to_string());
//             return Ok(cached);
//         }

//         tracing::debug!("Cache miss for key: {}, executing factory", key.to_string());
//         let value = factory().await?;
//         self.set(key, &value, ttl).await?;
//         Ok(value)
//     }
// }

// #[async_trait]
// impl KeyOperations for RedisCache {
//     async fn delete(&self, key: &CacheKey) -> Result<(), CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let mut conn = self.get_connection().await?;

//         let _: () = redis::cmd("DEL")
//             .arg(&key_str)
//             .query_async(&mut conn)
//             .await
//             .map_err(|e| CacheError::OperationError(e.to_string()))?;

//         Ok(())
//     }

//     async fn exists(&self, key: &CacheKey) -> Result<bool, CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let mut conn = self.get_connection().await?;

//         let exists: bool = redis::cmd("EXISTS")
//             .arg(&key_str)
//             .query_async(&mut conn)
//             .await
//             .map_err(|e| CacheError::OperationError(e.to_string()))?;

//         Ok(exists)
//     }
// }

// #[async_trait]
// impl SetOperations for RedisCache {
//     async fn sadd(&self, key: &CacheKey, value: &str) -> Result<(), CacheError> {
//         let mut conn = self.get_connection().await?;
//         let key_str = key.to_string();
//         let _: () = redis::cmd("SADD")
//             .arg(&key_str)
//             .arg(value)
//             .query_async(&mut conn)
//             .await
//             .map_err(|e| CacheError::OperationError(e.to_string()))?;
//         Ok(())
//     }

//     async fn smembers(&self, key: &CacheKey) -> Result<Vec<String>, CacheError> {
//         let mut conn = self.get_connection().await?;
//         let key_str = key.to_string();
//         let members: Vec<String> = redis::cmd("SMEMBERS")
//             .arg(&key_str)
//             .query_async(&mut conn)
//             .await
//             .map_err(|e| CacheError::OperationError(e.to_string()))?;
//         Ok(members)
//     }

//     async fn srem(&self, key: &CacheKey, value: &str) -> Result<(), CacheError> {
//         let mut conn = self.get_connection().await?;
//         let key_str = key.to_string();
//         let _: () = redis::cmd("SREM")
//             .arg(&key_str)
//             .arg(value)
//             .query_async(&mut conn)
//             .await
//             .map_err(|e| CacheError::OperationError(e.to_string()))?;
//         Ok(())
//     }
// }

// // Memory cache for development/testing
// pub struct MemoryCache {
//     store: RwLock<std::collections::HashMap<String, (String, Option<DateTime<Utc>>)>>,
//     config: CacheConfig,
// }

// impl MemoryCache {
//     pub fn new(config: CacheConfig) -> Self {
//         Self {
//             store: RwLock::new(std::collections::HashMap::new()),
//             config,
//         }
//     }

//     fn is_expired(&self, expires_at: Option<DateTime<Utc>>) -> bool {
//         match expires_at {
//             Some(expiry) => Utc::now() > expiry,
//             None => false,
//         }
//     }
// }

// // -------- Memory impls --------

// #[async_trait]
// impl<T> CacheOperations<T> for MemoryCache
// where
//     T: Serialize + DeserializeOwned + Send + Sync + 'static,
// {
//     async fn get(&self, key: &CacheKey) -> Result<Option<T>, CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let store = self.store.read().await;

//         if let Some((json, expiry)) = store.get(&key_str) {
//             if self.is_expired(*expiry) {
//                 return Ok(None);
//             }

//             let value: T = serde_json::from_str(json)
//                 .map_err(|e| CacheError::SerializationError(e.to_string()))?;
//             Ok(Some(value))
//         } else {
//             Ok(None)
//         }
//     }

//     async fn set(&self, key: &CacheKey, value: &T, ttl: Option<u64>) -> Result<(), CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let json = serde_json::to_string(value)
//             .map_err(|e| CacheError::SerializationError(e.to_string()))?;

//         let expires_at = ttl.map(|seconds| Utc::now() + chrono::Duration::seconds(seconds as i64));

//         let mut store = self.store.write().await;
//         store.insert(key_str, (json, expires_at));

//         Ok(())
//     }

//     async fn get_or_set<F>(&self, key: &CacheKey, ttl: Option<u64>, factory: F) -> Result<T, CacheError>
//     where
//         F: Fn() -> futures::future::BoxFuture<'static, Result<T, CacheError>> + Send + Sync,
//     {
//         if let Some(cached) = self.get(key).await? {
//             return Ok(cached);
//         }

//         let value = factory().await?;
//         self.set(key, &value, ttl).await?;
//         Ok(value)
//     }
// }

// #[async_trait]
// impl KeyOperations for MemoryCache {
//     async fn delete(&self, key: &CacheKey) -> Result<(), CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let mut store = self.store.write().await;
//         store.remove(&key_str);
//         Ok(())
//     }

//     async fn exists(&self, key: &CacheKey) -> Result<bool, CacheError> {
//         if !self.config.enabled {
//             return Err(CacheError::CacheDisabled);
//         }

//         let key_str = key.to_string();
//         let store = self.store.read().await;

//         Ok(store.contains_key(&key_str))
//     }
// }

// #[async_trait]
// impl SetOperations for MemoryCache {
//     async fn sadd(&self, _key: &CacheKey, _value: &str) -> Result<(), CacheError> {
//         // Not implemented for memory cache
//         Ok(())
//     }

//     async fn smembers(&self, _key: &CacheKey) -> Result<Vec<String>, CacheError> {
//         // Not implemented for memory cache
//         Ok(vec![])
//     }

//     async fn srem(&self, _key: &CacheKey, _value: &str) -> Result<(), CacheError> {
//         // Not implemented for memory cache
//         Ok(())
//     }
// }

// // Error types
// #[derive(Debug, thiserror::Error)]
// pub enum CacheError {
//     #[error("Connection error: {0}")]
//     ConnectionError(String),

//     #[error("Operation error: {0}")]
//     OperationError(String),

//     #[error("Serialization error: {0}")]
//     SerializationError(String),

//     #[error("Cache is disabled")]
//     CacheDisabled,

//     #[error("Cache miss")]
//     CacheMiss,
// }

// // Cache key generators for different resources
// pub struct CacheKeys;

// impl CacheKeys {
//     // User cache keys
//     pub fn user_by_id(user_id: &str) -> CacheKey {
//         CacheKey::Composite(vec!["user".to_string(), "id".to_string(), user_id.to_string()])
//     }

//     pub fn user_by_email(email: &str) -> CacheKey {
//         CacheKey::Composite(vec!["user".to_string(), "email".to_string(), email.to_string()])
//     }

//     pub fn user_by_phone(phone: &str) -> CacheKey {
//         CacheKey::Composite(vec!["user".to_string(), "phone".to_string(), phone.to_string()])
//     }

//     pub fn user_credentials(user_id: &str) -> CacheKey {
//         CacheKey::Composite(vec![
//             "user".to_string(),
//             "credentials".to_string(),
//             user_id.to_string(),
//         ])
//     }

//     pub fn all_users() -> CacheKey {
//         CacheKey::Simple("users:all".to_string())
//     }

//     // Driver cache keys
//     pub fn driver_by_id(driver_id: &str) -> CacheKey {
//         CacheKey::Composite(vec!["driver".to_string(), "id".to_string(), driver_id.to_string()])
//     }

//     pub fn driver_by_user_id(user_id: &str) -> CacheKey {
//         CacheKey::Composite(vec![
//             "driver".to_string(),
//             "user_id".to_string(),
//             user_id.to_string(),
//         ])
//     }

//     pub fn online_drivers() -> CacheKey {
//         CacheKey::Simple("drivers:online".to_string())
//     }

//     // Job cache keys
//     pub fn job_by_id(job_id: &str) -> CacheKey {
//         CacheKey::Composite(vec!["job".to_string(), "id".to_string(), job_id.to_string()])
//     }

//     pub fn jobs_by_customer(customer_id: &str) -> CacheKey {
//         CacheKey::Composite(vec![
//             "jobs".to_string(),
//             "customer".to_string(),
//             customer_id.to_string(),
//         ])
//     }

//     pub fn jobs_by_driver(driver_id: &str) -> CacheKey {
//         CacheKey::Composite(vec![
//             "jobs".to_string(),
//             "driver".to_string(),
//             driver_id.to_string(),
//         ])
//     }

//     pub fn active_jobs() -> CacheKey {
//         CacheKey::Simple("jobs:active".to_string())
//     }

//     // Location cache keys
//     pub fn driver_location(driver_id: &str) -> CacheKey {
//         CacheKey::Composite(vec![
//             "location".to_string(),
//             "driver".to_string(),
//             driver_id.to_string(),
//         ])
//     }

//     // Pattern keys for bulk operations
//     pub fn all_users_pattern() -> CacheKey {
//         CacheKey::Pattern("user:*".to_string())
//     }

//     pub fn all_drivers_pattern() -> CacheKey {
//         CacheKey::Pattern("driver:*".to_string())
//     }
// }

// // Cache service wrapper
// pub struct CacheService {
//     user_cache: Arc<Cache>,
//     job_cache: Arc<Cache>,
//     config: CacheConfig,
// }

// impl CacheService {
//     pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
//         let config = CacheConfig {
//             redis_url: redis_url.to_string(),
//             ..Default::default()
//         };

//         Ok(Self {
//             user_cache: Arc::new(Cache::Redis(RedisCache::new(config.clone()).await?)),
//             job_cache: Arc::new(Cache::Redis(RedisCache::new(config.clone()).await?)),
//             config,
//         })
//     }

//     pub fn new_memory(config: CacheConfig) -> Self {
//         Self {
//             user_cache: Arc::new(Cache::Memory(MemoryCache::new(config.clone()))),
//             job_cache: Arc::new(Cache::Memory(MemoryCache::new(config.clone()))),
//             config,
//         }
//     }

//     pub async fn get_user(&self, key: &CacheKey) -> Result<Option<User>, AppError> {
//         self.user_cache.get(key).await.map_err(|e| e.into())
//     }

//     pub async fn set_user(&self, key: &CacheKey, value: &User, ttl: Option<u64>) -> Result<(), AppError> {
//         self.user_cache.set(key, value, ttl).await.map_err(|e| e.into())
//     }

//     pub async fn get_job(&self, key: &CacheKey) -> Result<Option<Job>, AppError> {
//         self.job_cache.get(key).await.map_err(|e| e.into())
//     }

//     pub async fn set_job(&self, key: &CacheKey, value: &Job, ttl: Option<u64>) -> Result<(), AppError> {
//         self.job_cache.set(key, value, ttl).await.map_err(|e| e.into())
//     }

//     // User caching methods
//     pub async fn cache_user(&self, user: &User) -> Result<(), AppError> {
//         let key = CacheKeys::user_by_id(&user.id);
//         self.set_user(&key, user, Some(86400 * 7)).await?; // 7 days TTL

//         // Update indices
//         self.cache_user_by_phone(&user.phone_number, &user.id).await?;
//         self.cache_user_by_email(&user.email, &user.id).await?;

//         Ok(())
//     }

//     pub async fn get_user_credentials(&self, _user_id: &str) -> Result<Option<String>, AppError> {
//         unimplemented!()
//     }

//     pub async fn cache_user_credentials(&self, _user_id: &str, _hashed_password: &str) -> Result<(), AppError> {
//         unimplemented!()
//     }

//     pub async fn cache_user_by_email(&self, email: &str, user_id: &str) -> Result<(), AppError> {
//         let key = CacheKeys::user_by_email(email);
//         self.user_cache
//             .set(&key, &user_id.to_string(), Some(86400 * 7))
//             .await
//             .map_err(|e| e.into())?;
//         Ok(())
//     }

//     pub async fn get_user_id_by_email(&self, email: &str) -> Result<Option<String>, AppError> {
//         let key = CacheKeys::user_by_email(email);
//         self.user_cache.get(&key).await.map_err(|e| e.into())
//     }

//     pub async fn cache_user_by_phone(&self, phone: &str, user_id: &str) -> Result<(), AppError> {
//         let key = CacheKeys::user_by_phone(phone);
//         self.user_cache
//             .set(&key, &user_id.to_string(), Some(86400 * 7))
//             .await
//             .map_err(|e| e.into())?;
//         Ok(())
//     }

//     pub async fn get_user_id_by_phone(&self, phone: &str) -> Result<Option<String>, AppError> {
//         let key = CacheKeys::user_by_phone(phone);
//         self.user_cache.get(&key).await.map_err(|e| e.into())
//     }

//     pub async fn cache_user_index(&self, user: &User) -> Result<(), AppError> {
//         // Add to all users set
//         let all_users_key = CacheKeys::all_users();
//         self.user_cache
//             .sadd(&all_users_key, &user.id)
//             .await
//             .map_err(|e| e.into())?;
//         Ok(())
//     }

//     // Job caching methods
//     pub async fn cache_job(&self, job: &Job) -> Result<(), AppError> {
//         let key = CacheKeys::job_by_id(&job.id);
//         self.set_job(&key, job, Some(3600)).await?; // 1 hour TTL
//         Ok(())
//     }

//     pub async fn get_customer_jobs(&self, customer_id: &str) -> Result<Vec<String>, AppError> {
//         let key = CacheKeys::jobs_by_customer(customer_id);
//         self.job_cache.smembers(&key).await.map_err(|e| e.into())
//     }

//     pub async fn cache_customer_job(&self, customer_id: &str, job_id: &str) -> Result<(), AppError> {
//         let key = CacheKeys::jobs_by_customer(customer_id);
//         self.job_cache.sadd(&key, job_id).await.map_err(|e| e.into())
//     }

//     pub async fn get_driver_jobs(&self, driver_id: &str) -> Result<Vec<String>, AppError> {
//         let key = CacheKeys::jobs_by_driver(driver_id);
//         self.job_cache.smembers(&key).await.map_err(|e| e.into())
//     }

//     pub async fn remove_driver_job(&self, driver_id: &str, job_id: &str) -> Result<(), AppError> {
//         let key = CacheKeys::jobs_by_driver(driver_id);
//         self.job_cache.srem(&key, job_id).await.map_err(|e| e.into())
//     }

//     pub async fn cache_driver_job(&self, driver_id: &str, job_id: &str) -> Result<(), AppError> {
//         let key = CacheKeys::jobs_by_driver(driver_id);
//         self.job_cache.sadd(&key, job_id).await.map_err(|e| e.into())
//     }

//     // Bulk operations / invalidation
//     pub async fn invalidate_user(&self, user_id: &str) -> Result<(), AppError> {
//         let key = CacheKeys::user_by_id(user_id);
//         self.user_cache.delete(&key).await?;
//         Ok(())
//     }
// }

// // Health check
// impl CacheService {
//     pub async fn health_check(&self) -> Result<bool, AppError> {
//         unimplemented!()
//     }
// }

// impl From<CacheError> for AppError {
//     fn from(error: CacheError) -> Self {
//         AppError::ResourceExhausted(error.to_string())
//     }
// }

// // ------------------------------
// // Enum delegations (Cache)
// // ------------------------------

// #[async_trait]
// impl<T> CacheOperations<T> for Cache
// where
//     T: Serialize + DeserializeOwned + Send + Sync + 'static,
// {
//     async fn get(&self, key: &CacheKey) -> Result<Option<T>, CacheError> {
//         match self {
//             Cache::Redis(cache) => cache.get(key).await,
//             Cache::Memory(cache) => cache.get(key).await,
//         }
//     }

//     async fn set(&self, key: &CacheKey, value: &T, ttl: Option<u64>) -> Result<(), CacheError> {
//         match self {
//             Cache::Redis(cache) => cache.set(key, value, ttl).await,
//             Cache::Memory(cache) => cache.set(key, value, ttl).await,
//         }
//     }

//     async fn get_or_set<F>(&self, key: &CacheKey, ttl: Option<u64>, factory: F) -> Result<T, CacheError>
//     where
//         F: Fn() -> futures::future::BoxFuture<'static, Result<T, CacheError>> + Send + Sync,
//     {
//         match self {
//             Cache::Redis(cache) => cache.get_or_set(key, ttl, factory).await,
//             Cache::Memory(cache) => cache.get_or_set(key, ttl, factory).await,
//         }
//     }
// }

// #[async_trait]
// impl KeyOperations for Cache {
//     async fn delete(&self, key: &CacheKey) -> Result<(), CacheError> {
//         match self {
//             Cache::Redis(cache) => cache.delete(key).await,
//             Cache::Memory(cache) => cache.delete(key).await,
//         }
//     }

//     async fn exists(&self, key: &CacheKey) -> Result<bool, CacheError> {
//         match self {
//             Cache::Redis(cache) => cache.exists(key).await,
//             Cache::Memory(cache) => cache.exists(key).await,
//         }
//     }
// }

// #[async_trait]
// impl SetOperations for Cache {
//     async fn sadd(&self, key: &CacheKey, value: &str) -> Result<(), CacheError> {
//         match self {
//             Cache::Redis(cache) => cache.sadd(key, value).await,
//             Cache::Memory(cache) => cache.sadd(key, value).await,
//         }
//     }

//     async fn smembers(&self, key: &CacheKey) -> Result<Vec<String>, CacheError> {
//         match self {
//             Cache::Redis(cache) => cache.smembers(key).await,
//             Cache::Memory(cache) => cache.smembers(key).await,
//         }
//     }

//     async fn srem(&self, key: &CacheKey, value: &str) -> Result<(), CacheError> {
//         match self {
//             Cache::Redis(cache) => cache.srem(key, value).await,
//             Cache::Memory(cache) => cache.srem(key, value).await,
//         }
//     }
// }

// // ------------------------------
// // get_or_set helper in service
// // ------------------------------

// impl CacheService {
//     // Get or set pattern with automatic caching
//     pub async fn get_user_or_fetch<F>(&self, user_id: &str, fetch_fn: F) -> Result<User, AppError>
//     where
//         F: Fn() -> futures::future::BoxFuture<'static, Result<User, AppError>> + Send + Sync,
//     {
//         let key = CacheKeys::user_by_id(user_id);
//         self.user_cache
//             .get_or_set(&key, Some(3600), || {
//                 Box::pin(async move {
//                     fetch_fn()
//                         .await
//                         .map_err(|e| CacheError::OperationError(e.to_string()))
//                 })
//             })
//             .await
//             .map_err(|e| e.into())
//     }
// }
