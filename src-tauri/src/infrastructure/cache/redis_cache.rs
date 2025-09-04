use crate::shared::errors::{AppError, AppResult};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct RedisCache {
    client: Arc<Client>,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> AppResult<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| AppError::CacheError(format!("Redis connection failed: {}", e)))?;

        // Be explicit about the expected return type from Redis:
        let data: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| AppError::CacheError(format!("Failed to get from cache: {}", e)))?;

        match data {
            Some(json) => {
                let value = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    // seconds must be u64 for `SETEX`
    pub async fn set<T>(&self, key: &str, value: &T, expiry_secs: u64) -> AppResult<()>
    where
        T: Serialize,
    {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| AppError::CacheError(format!("Redis connection failed: {}", e)))?;

        let json = serde_json::to_string(value)?;

        // Make the return type explicit to avoid never-type fallback:
        // set_ex<K, V, RV>(...)
        let _: () = conn
            .set_ex::<_, _, ()>(key, json, expiry_secs)
            .await
            .map_err(|e| AppError::CacheError(format!("Failed to set cache: {}", e)))?;

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| AppError::CacheError(format!("Redis connection failed: {}", e)))?;

        // `DEL` can return the deleted count; if you don't need it, request `()`.
        // If you *do* want it, use `u64` instead of `()`.
        let _: () = conn
            .del::<_, ()>(key)
            .await
            .map_err(|e| AppError::CacheError(format!("Failed to delete from cache: {}", e)))?;

        Ok(())
    }
}
