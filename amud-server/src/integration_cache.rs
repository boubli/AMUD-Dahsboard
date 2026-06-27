//! Server-side TTL cache with singleflight for integration API responses.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex as AsyncMutex, Notify};

#[derive(Clone)]
struct CacheEntry {
    value: Value,
    fetched_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn is_valid(&self) -> bool {
        self.fetched_at.elapsed() < self.ttl
    }
}

#[derive(Default)]
struct CacheInner {
    entries: HashMap<i64, CacheEntry>,
    access_order: Vec<i64>,
}

pub struct IntegrationCache {
    inner: RwLock<CacheInner>,
    max_entries: usize,
    default_ttl: Duration,
    flights: AsyncMutex<HashMap<i64, Arc<Notify>>>,
}

impl IntegrationCache {
    pub fn new(max_entries: usize, default_ttl_secs: u64) -> Self {
        Self {
            inner: RwLock::new(CacheInner::default()),
            max_entries: max_entries.max(1),
            default_ttl: Duration::from_secs(default_ttl_secs.max(5)),
            flights: AsyncMutex::new(HashMap::new()),
        }
    }

    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    pub fn get(&self, app_id: i64) -> Option<Value> {
        let inner = self.inner.read().unwrap();
        inner
            .entries
            .get(&app_id)
            .filter(|e| e.is_valid())
            .map(|e| e.value.clone())
    }

    pub fn insert(&self, app_id: i64, value: Value, ttl: Duration) {
        let mut inner = self.inner.write().unwrap();
        inner.access_order.retain(|id| *id != app_id);
        inner.access_order.push(app_id);
        inner.entries.insert(
            app_id,
            CacheEntry {
                value,
                fetched_at: Instant::now(),
                ttl,
            },
        );
        while inner.entries.len() > self.max_entries {
            if let Some(oldest) = inner.access_order.first().copied() {
                inner.access_order.remove(0);
                inner.entries.remove(&oldest);
            } else {
                break;
            }
        }
    }

    pub fn invalidate(&self, app_id: i64) {
        let mut inner = self.inner.write().unwrap();
        inner.entries.remove(&app_id);
        inner.access_order.retain(|id| *id != app_id);
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Fetch with singleflight: concurrent requests for the same app_id share one upstream call.
    pub async fn get_or_fetch<F, Fut>(&self, app_id: i64, ttl: Duration, fetch: F) -> Option<Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Option<Value>>,
    {
        if let Some(cached) = self.get(app_id) {
            return Some(cached);
        }

        let notify = {
            let mut flights = self.flights.lock().await;
            if let Some(existing) = flights.get(&app_id) {
                let wait = existing.clone();
                drop(flights);
                tokio::time::timeout(Duration::from_secs(30), wait.notified())
                    .await
                    .ok();
                return self.get(app_id);
            }
            let n = Arc::new(Notify::new());
            flights.insert(app_id, n.clone());
            n
        };

        let result = fetch().await;
        if let Some(ref value) = result {
            self.insert(app_id, value.clone(), ttl);
        }

        {
            let mut flights = self.flights.lock().await;
            flights.remove(&app_id);
        }
        notify.notify_waiters();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn cache_hit_avoids_second_fetch() {
        let cache = IntegrationCache::new(64, 60);
        let calls = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c1 = calls.clone();
        let v1 = cache
            .get_or_fetch(1, Duration::from_secs(60), || {
                let c = c1.clone();
                async move {
                    c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Some(json!({"ok": true}))
                }
            })
            .await;
        assert!(v1.is_some());
        let c2 = calls.clone();
        let _v2 = cache
            .get_or_fetch(1, Duration::from_secs(60), || {
                let c = c2.clone();
                async move {
                    c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Some(json!({"ok": false}))
                }
            })
            .await;
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn lru_evicts_oldest() {
        let cache = IntegrationCache::new(2, 60);
        cache.insert(1, json!(1), Duration::from_secs(60));
        cache.insert(2, json!(2), Duration::from_secs(60));
        cache.insert(3, json!(3), Duration::from_secs(60));
        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_some());
        assert!(cache.get(3).is_some());
    }
}
