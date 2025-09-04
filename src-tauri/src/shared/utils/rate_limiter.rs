use crate::shared::errors::AppError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct RateLimiter {
    last_request: Arc<Mutex<Instant>>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64) -> Self {
        let min_interval = Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            last_request: Arc::new(Mutex::new(Instant::now() - min_interval)),
            min_interval,
        }
    }

    pub async fn wait(&self) -> Result<(), AppError> {
        let mut last = self.last_request.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            sleep(wait_time).await;
        }

        *last = Instant::now();
        Ok(())
    }
}
