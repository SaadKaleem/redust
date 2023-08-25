use crate::cmd::ParseError;
use chrono::{DateTime, Duration, Utc};
use mockall::automock;
use std::{
    collections::{HashMap, LinkedList},
    sync::{Arc, Mutex},
};

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

    fn exists(&self, keys: Vec<String>) -> u64;

    fn del(&self, keys: Vec<String>) -> u64;

    fn incr(&self, key: String) -> Result<i64, ParseError>;

    fn decr(&self, key: String) -> Result<i64, ParseError>;
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
        SharedStore { shared }
    }

    /// Adjust the `key` by the `amount`, this can be used to increment or decrement
    /// the `value` at `key`
    pub fn _adjust_by(
        &self,
        mutex: &mut std::sync::MutexGuard<'_, DataStore>,
        key: &String,
        amount: i64,
    ) -> Result<i64, ParseError> {
        match mutex.data.get(key) {
            Some(value) => match value {
                DataType::String(val) => {
                    let mut parsed_number = val.parse::<i64>();

                    match parsed_number {
                        Ok(ref mut num) => {
                            *num += amount;

                            mutex
                                .data
                                .insert(key.clone(), DataType::String(num.to_string()));

                            return Ok(*num);
                        }
                        Err(_) => {
                            return Err(ParseError::ConditionNotMet(
                                "ERR value type is not integer or out of range".to_string(),
                            ))
                        }
                    }
                }
                _ => {
                    return Err(ParseError::ConditionNotMet(
                        "ERR value type is not string".to_string(),
                    ));
                }
            },
            // Key:Val didn't exist
            None => {
                let value: i64 = amount;
                mutex
                    .data
                    .insert(key.clone(), DataType::String(value.to_string()));
                return Ok(value);
            }
        }
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
        let mutex: std::sync::MutexGuard<'_, DataStore> = self.shared.store.lock().unwrap();

        // TODO: If the value is expired, we return `None`

        // If the value exists, and is not expired we return `DataType`
        match mutex.data.get(&key) {
            Some(value) => {
                return Some(value.clone());
            }
            None => {
                return None;
            }
        }
    }

    fn exists(&self, keys: Vec<String>) -> u64 {
        // Acquire the Mutex
        let mutex: std::sync::MutexGuard<'_, DataStore> = self.shared.store.lock().unwrap();

        let mut count: u64 = 0;

        for k in keys {
            if mutex.data.contains_key(&k) == true {
                count += 1;
            }
        }

        count
    }

    fn del(&self, keys: Vec<String>) -> u64 {
        // Acquire the Mutex
        let mut mutex: std::sync::MutexGuard<'_, DataStore> = self.shared.store.lock().unwrap();

        let mut count: u64 = 0;

        for k in keys {
            match mutex.data.remove(&k) {
                Some(_) => {
                    // Also try to remove from `date_time` map
                    mutex.date_time.remove(&k);
                    count += 1;
                }
                None => {}
            }
        }

        count
    }

    fn incr(&self, key: String) -> Result<i64, ParseError> {
        // Acquire the Mutex
        let mut mutex: std::sync::MutexGuard<'_, DataStore> = self.shared.store.lock().unwrap();

        return self._adjust_by(&mut mutex, &key, 1);
    }

    fn decr(&self, key: String) -> Result<i64, ParseError> {
        // Acquire the Mutex
        let mut mutex: std::sync::MutexGuard<'_, DataStore> = self.shared.store.lock().unwrap();

        return self._adjust_by(&mut mutex, &key, -1);
    }
}
