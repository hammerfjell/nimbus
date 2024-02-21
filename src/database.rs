use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};


pub struct Database {
    pub storage: RwLock<HashMap<String, (String, Instant)>>
}

impl Database {
    pub fn new() -> Arc<Database> {
        let db = Arc::new(Database {
            storage: RwLock::new(HashMap::new()),
        });
        db
    }

    pub async fn set(&self, key: String, value: String, expire_duration: Option<Duration>) {

        let mut storage = self.storage.write().await;
        let expiration = expire_duration.map(|d| Instant::now() + d).unwrap_or(Instant::now() + Duration::from_secs(60 * 60 * 24 * 7)); // Default: 7 days

        storage.insert(key, (value, expiration));
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let mut storage = self.storage.write().await; // Acquire a write lock
        match storage.get(key) {
            Some((value, expiration)) if Instant::now() <= *expiration => {
                // The key exists and is not expired, return its value
                Some(value.clone())
            },
            Some(_) => {
                // The key exists but is expired, remove it and return None
                storage.remove(key); // Remove the expired key
                None
            },
            None => {
                // The key does not exist
                None
            },
        }
    }

    pub async fn del(&self, keys: &[String]) {
        let mut storage = self.storage.write().await;
        for key in keys {
            storage.remove(key);
        }
    }

    pub async fn garbage_collect(&self) {
        if self.storage.read().await.is_empty() {
            return;
        }
        let mut storage = self.storage.write().await;
        let count = storage.len();
        let start = Instant::now();
        storage.retain(|_, (_, expiration)| Instant::now() <= *expiration);
        let count = count - storage.len();
        if count == 0 {
            return;
        }
        let elapsed = start.elapsed().as_micros() as f64 / 1000.0;
        println!("Garbage collection completed. {} {} removed in {:.3} ms.", count, if count == 1 { "key" } else { "keys" }, elapsed);
    }
}