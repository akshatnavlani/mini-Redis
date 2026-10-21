// Shared in-memory store module.
//
// Add child-module declarations for entry.rs and the store implementation.
// Add imports for Arc, the chosen lock, the LRU cache, and Entry.
mod entry;
use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use tokio::sync::RwLock;

use self::entry::Entry;

pub(crate) struct Store{
    cache: LruCache<Vec<u8>, Entry>,
}

pub (crate) type StoreHandle = Arc<RwLock<Store>>;

#[derive(Debug,PartialEq,Eq,Clone)]
pub(crate) enum StoreError{
    InvalidCapacity,
}

use bytes::Bytes;
use std::time::{Duration,Instant};

// Function reference: STORE-001
// Function name: new_store
// Input: maximum key count and store configuration.
// Output: shared store handle.
// Description: Create the V1 lock-protected store and configure its capacity.
// Why it exists: Arc shares ownership across client tasks and a lock protects
// shared mutable state without unsafe code.
pub (crate) fn new_store(max_keys:usize)-> Result<StoreHandle,StoreError>{
    let capacity = NonZeroUsize::new(max_keys).ok_or(StoreError::InvalidCapacity)?;
    let cache = LruCache::new(capacity);
    let store = Store{
        cache,
    };
    let locked_store = RwLock::new(store);
    Ok(Arc::new(locked_store))
}

// Function reference: STORE-002
// Function name: get
// Input: key.
// Output: present, unexpired value or no value.
// Description: Read a key, apply lazy expiry, and update LRU recency.
// Why it exists: Expired keys must behave as missing before the sweeper runs.
pub(crate) async fn get(store_handle:&StoreHandle,key:&[u8])->Option<Bytes>{
    let mut store = store_handle.write().await;
    let current_time= Instant::now();
    let value = match store.cache.get(key){
        None => return None,
        Some(entry) if entry.is_expired(current_time)=> None,
        Some(entry)=> Some(entry.value.clone())
    };
    if value.is_none(){
        store.cache.pop(key);
    }
    value
}

// Function reference: STORE-003
// Function name: set
// Input: key, value, and optional expiration time.
// Output: no value; store state changes.
// Description: Insert or replace a value and evict the least-recently-used key
// when capacity requires it.
// Why it exists: This is the core mutation and the source of LRU behavior.
pub (crate) async fn set(store_handle: &StoreHandle,key: Vec<u8>,value: Bytes,expires_at:Option<Instant>){
    let mut store = store_handle.write().await;
    let entry = Entry::new(value,expires_at);
    store.cache.put(key,entry);
}
// Function reference: STORE-004
// Function name: delete
// Input: one or more keys.
// Output: count of keys actually removed.
// Description: Remove requested keys and report the deletion count.
// Why it exists: DEL's integer response depends on actual prior existence.
pub (crate) async fn delete(store_handle: &StoreHandle, keys: &[Vec<u8>]) -> usize{
    let mut store = store_handle.write().await;
    let current_time = Instant::now();
    let mut deleted_count =0;
    for key in keys{
        let should_count = match store.cache.peek(key){
            Some(entry) => !entry.is_expired(current_time),
            None => false,
        };
        if store.cache.pop(key).is_some()&& should_count{
            deleted_count +=1;
        }
    }
    deleted_count
}
// Function reference: STORE-005
// Function name: exists
// Input: key.
// Output: boolean logical existence.
// Description: Check existence after applying lazy expiry.
// Why it exists: A stale expired entry must not count as existing.
pub (crate) async fn exists(store_handle:&StoreHandle, key: &[u8])-> bool{
    let mut store = store_handle.write().await;
    let current_time = Instant::now();
    let is_expired=match store.cache.get(key){
        None => return false,
        Some(entry) => entry.is_expired(current_time),
    };
    if is_expired{
        store.cache.pop(key);
        false
    }
    else{
        true
    }
}
// Function reference: STORE-006
// Function name: expire
// Input: key and TTL in seconds.
// Output: 1 when a key was updated, otherwise 0.
// Description: Add or replace expiry metadata without creating missing keys.
// Why it exists: This matches Redis EXPIRE semantics.
pub(crate) async fn expire(store_handle:&StoreHandle,key:&[u8],expiration:Duration)-> bool{
    let mut store = store_handle.write().await;
    let current_time = Instant::now();
    let expires_at = match current_time.checked_add(expiration){
        Some(deadline)=> deadline,
        None => return false,
    };
    let should_remove={
        let entry = match store.cache.get_mut(key){
            None => return false,
            Some(entry) => entry,
        };
        if entry.is_expired(current_time){
            true
        }
        else{
            entry.expires_at=Some(expires_at);
            false
        }
    };
    if should_remove{
        store.cache.pop(key);
        return false
    }
    true
}
// Function reference: STORE-007
// Function name: ttl
// Input: key.
// Output: seconds remaining, -1 for no TTL, or -2 for missing.
// Description: Report Redis-compatible TTL state after lazy expiry.
// Why it exists: TTL has intentionally distinct missing and persistent states.

pub (crate) async fn ttl(store_handle:&StoreHandle,key:&[u8])-> i64{
    let mut store = store_handle.write().await;
    let current_time = Instant::now();
    let remaining = match store.cache.get(key){
        None => return -2,
        Some(entry) => match entry.expires_at{
            None => return -1,
            Some(expires_at) => {
                expires_at.checked_duration_since(current_time)
            }
        }
    };
    match remaining{
        Some(duration)=>{i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)}
        None => {
            store.cache.pop(key);
            -2
        }
    }
}
// Function reference: STORE-008
// Function name: remove_expired
// Input: current monotonic instant.
// Output: number of expired keys removed.
// Description: Scan or sample the store and remove entries past their deadline.
// Why it exists: Active expiry reclaims memory without waiting for reads.

pub (crate) async fn remove_expired(store_handle:&StoreHandle, current_time: Instant)-> usize{
    let mut store = store_handle.write().await;
    let expired_keys: Vec<Vec<u8>> = store.cache.iter().filter_map(|(key,entry)|{
        if entry.is_expired(current_time){
            Some(key.clone())
        }
        else{
            None
        }
    }).collect();
    let mut removed_count = 0;
    for key in expired_keys{
        if store.cache.pop(&key).is_some(){
            removed_count +=1;
        }
    }
    removed_count
}