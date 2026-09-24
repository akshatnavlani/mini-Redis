// Shared in-memory store module.
//
// Add child-module declarations for entry.rs and the store implementation.
// Add imports for Arc, the chosen lock, the LRU cache, and Entry.

// Function reference: STORE-001
// Function name: new_store
// Input: maximum key count and store configuration.
// Output: shared store handle.
// Description: Create the V1 lock-protected store and configure its capacity.
// Why it exists: Arc shares ownership across client tasks and a lock protects
// shared mutable state without unsafe code.

// Function reference: STORE-002
// Function name: get
// Input: key.
// Output: present, unexpired value or no value.
// Description: Read a key, apply lazy expiry, and update LRU recency.
// Why it exists: Expired keys must behave as missing before the sweeper runs.

// Function reference: STORE-003
// Function name: set
// Input: key, value, and optional expiration time.
// Output: no value; store state changes.
// Description: Insert or replace a value and evict the least-recently-used key
// when capacity requires it.
// Why it exists: This is the core mutation and the source of LRU behavior.

// Function reference: STORE-004
// Function name: delete
// Input: one or more keys.
// Output: count of keys actually removed.
// Description: Remove requested keys and report the deletion count.
// Why it exists: DEL's integer response depends on actual prior existence.

// Function reference: STORE-005
// Function name: exists
// Input: key.
// Output: boolean logical existence.
// Description: Check existence after applying lazy expiry.
// Why it exists: A stale expired entry must not count as existing.

// Function reference: STORE-006
// Function name: expire
// Input: key and TTL in seconds.
// Output: 1 when a key was updated, otherwise 0.
// Description: Add or replace expiry metadata without creating missing keys.
// Why it exists: This matches Redis EXPIRE semantics.

// Function reference: STORE-007
// Function name: ttl
// Input: key.
// Output: seconds remaining, -1 for no TTL, or -2 for missing.
// Description: Report Redis-compatible TTL state after lazy expiry.
// Why it exists: TTL has intentionally distinct missing and persistent states.

// Function reference: STORE-008
// Function name: remove_expired
// Input: current monotonic instant.
// Output: number of expired keys removed.
// Description: Scan or sample the store and remove entries past their deadline.
// Why it exists: Active expiry reclaims memory without waiting for reads.