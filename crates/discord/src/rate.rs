use std::time::Duration;
use governor::{Quota, RateLimiter, state::InMemoryState, clock::DefaultClock};

pub struct Bucket {
    pub inner: RateLimiter<InMemoryState, DefaultClock>,
}

impl Bucket {
    pub fn new(permits_per_second: u32) -> Self {
        let q = Quota::per_second(std::num::NonZeroU32::new(permits_per_second).unwrap());
        Self { inner: RateLimiter::direct(q) }
    }
    pub async fn acquire(&self) {
        let _ = self.inner.until_ready().await;
    }
}
