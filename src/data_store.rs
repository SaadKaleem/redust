use crate::{cmd::ParseError, KEY_EXPIRY_DELAY_MS, KEY_EXPIRY_NUM_KEYS_TO_CHECK};
use chrono::{DateTime, Duration, Utc};
use mockall::automock;
use rand::seq::index::sample;
use std::{
    collections::{HashMap, LinkedList},
    sync::{Arc, Mutex},
};
use tokio::time::sleep;

#[automock]
pub trait SharedStoreBase: Send + Sync {
    fn set(
        &self,
        key: String,
        value: DataType,
        duration: Option<Duration>,
        nx: bool,
        xx: bool,
    ) -> Result<Option<DataType>, ParseError>;

    fn get(&self, key: String) -> Option<DataType>;
}

/// Shared Data Store across all the connections
///
/// Cloning `SharedStore` only increments an atomic reference count,
/// It does not copy it deeply, but rather shallowly.
///
#[derive(Debug, Clone)]
pub struct SharedStore {
    /// An Arc to provide shared ownership across the Tokio threads
    ///
    /// Invoking `Clone` on Arc produces a new pointer to the `GuardedDataStore`
    /// value in the heap.
    ///
    shared: Arc<GuardedDataStore>,
}

#[derive(Debug)]
pub struct GuardedDataStore {
    /// The `DataStore` which contains the `data` and `date_time`
    /// is guarded by a `Mutex` to prevent concurrent access as
    /// this may lead to data inconsistency
    ///
    /// Since there are no asynchronous operations as part of the
    /// critical section, we opt to use `std::sync::Mutex`. Moreover,
    /// the critical section is pretty small.
    ///
    store: Mutex<DataStore>,
}

#[derive(Debug)]
pub struct DataStore {
    /// The main key-value data store. The `DataType`
    /// depends on which cmd was used to insert the data
    data: HashMap<String, DataType>,

    /// Not all keys are part of this HashMap, depending on whether
    /// they have a Key Expiry or not.
    /// TimeSpan holds the value when this key will expire.
    date_time: HashMap<String, TimeSpan>,
}

/// The supported data types which can be stored in the `DataStore`
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    String(String),
    LinkedList(LinkedList<String>),
}

#[derive(Debug)]
pub struct TimeSpan {
    expires_at: DateTime<Utc>,
}

impl SharedStore {
    pub fn new() -> SharedStore {
        let shared = Arc::new(GuardedDataStore {
            store: Mutex::new(DataStore {
                data: HashMap::new(),
                date_time: HashMap::new(),
            }),
        });

        tokio::spawn(run_key_expiry(shared.clone()));

        SharedStore { shared }
    }
}

impl SharedStoreBase for SharedStore {
    /// Set the `value` associated with the `key`, and an expiration
    /// duration, if provided
    ///
    /// Values are overrided, if the key already exists
    ///
    fn set(
        &self,
        key: String,
        value: DataType,
        duration: Option<Duration>,
        nx: bool,
        xx: bool,
    ) -> Result<Option<DataType>, ParseError> {
        // Acquire the Mutex
        let mut mutex: std::sync::MutexGuard<'_, DataStore> = self.shared.store.lock().unwrap();

        // To check for xx and nx flags
        //
        // The corresponding `ParseError` is returned
        if nx == true && xx == true {
            return Err(ParseError::SyntaxError("syntax error".to_string()));
        } else if nx == true && xx == false {
            // Ensure that the key does not exist first
            if mutex.data.contains_key(&key) {
                return Err(ParseError::ConditionNotMet(
                    "NX condition not met".to_string(),
                ));
            }
        } else if nx == false && xx == true {
            // Ensure that the key already exists first
            if !mutex.data.contains_key(&key) {
                return Err(ParseError::ConditionNotMet(
                    "XX condition not met".to_string(),
                ));
            }
        }

        // Assign the `value` to the `key`
        // If an old `value` existed for this `key`, it is returned.
        // We clone the key, as to not "move" its ownership, since we need its reference
        // for the expiry tasks later.
        let old_value: Option<DataType> = mutex.data.insert(key.clone(), value.clone());

        // Replace or delete the date_time entry.
        // If the `old_value` existed, then check if the corresponding key had an `expiration` time
        // If it did, remove this key
        if old_value.is_some() {
            // Attempt to remove the key from the `date_time`, if there was any.
            let _ = mutex.date_time.remove(&key);
        }

        // If an expiry duration is provided, we add it to the `date_time` map
        if duration.is_some() {
            let expires_at: DateTime<Utc> = Utc::now() + duration.unwrap();

            mutex.date_time.insert(key.clone(), TimeSpan { expires_at });
        }

        // Release the mutex
        drop(mutex);

        return Ok(old_value);
    }

    /// Get the value associated with a Key
    ///
    /// Will return `None` if no value is found for the corresponding key.
    fn get(&self, key: String) -> Option<DataType> {
        // Acquire the Mutex
        let mut mutex: std::sync::MutexGuard<'_, DataStore> = self.shared.store.lock().unwrap();

        // If the value exists, and is not expired we return `DataType`
        match mutex.data.get(&key) {
            // If we have a `value` at `key`
            Some(value) => {
                // Check if the key exists in the `date_time` HashMap,
                // and remove from both HashMaps, if `current_time` > `expires_at`
                match mutex.date_time.get(&key) {
                    Some(val) => {
                        if Utc::now() >= val.expires_at {
                            mutex.date_time.remove(&key);
                            mutex.data.remove(&key);
                            return None;
                        } else {
                            return Some(value.clone());
                        }
                    }
                    None => return Some(value.clone()),
                }
            }
            None => {
                return None;
            }
        }
    }
}

impl GuardedDataStore {
    /// Expiry Algorithm:
    /// 1. Every 100 ms, test 20 keys at random from the set of keys, which have an expiry time set.
    /// 2. Delete all the expired keys from both HashMaps.
    /// 3. If more than 25% of the set of 20 keys was expired (5 keys were expired), restart the process from step 1.
    ///
    fn purge_expired_keys(&self) {
        let mut restart: bool = true;
        // Acquire the Mutex
        let mut mutex: std::sync::MutexGuard<'_, DataStore> = self.store.lock().unwrap();

        while restart == true {
            let random_keys: Vec<String>;
            {
                let mut rng = rand::thread_rng();
                // Get the iterator of the keys, of which have an expiry.
                // Borrow the hashmap
                let mut keys = mutex.date_time.keys();

                // Get the random indices to collect, for checking purposes.
                let num_keys = mutex.date_time.len();

                if num_keys == 0 {
                    return;
                }

                let num_to_get = std::cmp::min(num_keys, KEY_EXPIRY_NUM_KEYS_TO_CHECK);

                let indices = sample(&mut rng, num_keys, num_to_get);

                // Cloning the individual key is necessary, otherwise we'll still have borrowed the
                // reference, and wouldn't be able to perform a mutable operation on the HashMap
                // via mutex.data.remove(...)
                random_keys = indices
                    .iter()
                    .map(|i| keys.nth(i).unwrap().clone())
                    .collect();
            }

            // For each of the random keys, check if its expiry has met, if so remove it from both HashMaps.
            let mut keys_removed: i32 = 0;
            let keys_picked_length: usize = random_keys.len();

            for key in random_keys {
                let value = mutex.date_time.get(&key).unwrap();
                if Utc::now() >= value.expires_at {
                    mutex.data.remove(&key);
                    mutex.date_time.remove(&key);
                    keys_removed += 1;
                }
            }

            // If less than 25% of keys, have been removed we do not restart
            if (keys_removed as f32 / keys_picked_length as f32) < 0.25 {
                restart = false;
            }
        }
        drop(mutex);
    }
}

/// Async function, which calls the `purge_expired_keys` function every X duration
///
/// The reason to split this is that async context and synchronized mutexes cannot be shared.
async fn run_key_expiry(shared: Arc<GuardedDataStore>) {
    loop {
        // Purge the expired keys
        shared.purge_expired_keys();

        // Sleep for 100 ms
        let _ = sleep(std::time::Duration::from_millis(KEY_EXPIRY_DELAY_MS)).await;
    }
}
