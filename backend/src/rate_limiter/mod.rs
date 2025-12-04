use std::time::Duration;
use tokio::time::sleep;

/// Controls the rate of requests to prevent API throttling
pub struct RateLimiter {
    delay: Duration,
    request_count: usize,
}

impl RateLimiter {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: Duration::from_millis(delay_ms),
            request_count: 0,
        }
    }

    pub async fn wait(&mut self) {
        if self.should_wait() {
            self.apply_delay().await;
        }
        self.increment();
    }

    pub fn reset(&mut self) {
        self.request_count = 0;
    }

    fn should_wait(&self) -> bool {
        self.request_count > 0
    }

    async fn apply_delay(&self) {
        sleep(self.delay).await;
    }

    fn increment(&mut self) {
        self.request_count += 1;
    }
}
