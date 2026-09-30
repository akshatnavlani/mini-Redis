// Store entry data model.
//
// Add imports for binary-safe values and a monotonic time instant.
use bytes::Bytes;
use std::time::Instant;

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) value: Bytes,
    pub(crate) expires_at: Option<Instant>,
}
// Function reference: ENTRY-001
// Function name: Entry::new
// Input: value bytes and optional expiration instant.
// Output: a new Entry.
// Description: Package the value together with its absolute expiration time.
// Why it exists: Keeping metadata beside the value makes lazy expiry local and
// avoids a second map for deadlines.
impl Entry {
    pub(crate) fn new(
        value: Bytes,
        expires_at: Option<Instant>,
    ) -> Self {
        Self{
            value,
            expires_at,
        }
    }

    pub(crate) fn is_expired(&self, current_instant: Instant) -> bool {
        match self.expires_at{
            Some(expires_at) => expires_at <= current_instant,
            None => false,
        }
    }
}
// Function reference: ENTRY-002
// Function name: Entry::is_expired
// Input: an Entry and current instant.
// Output: boolean.
// Description: Return whether the optional deadline has passed.
// Why it exists: Monotonic time avoids wall-clock adjustments affecting TTLs.