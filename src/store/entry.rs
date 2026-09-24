// Store entry data model.
//
// Add imports for binary-safe values and a monotonic time instant.

// Function reference: ENTRY-001
// Function name: Entry::new
// Input: value bytes and optional expiration instant.
// Output: a new Entry.
// Description: Package the value together with its absolute expiration time.
// Why it exists: Keeping metadata beside the value makes lazy expiry local and
// avoids a second map for deadlines.

// Function reference: ENTRY-002
// Function name: Entry::is_expired
// Input: an Entry and current instant.
// Output: boolean.
// Description: Return whether the optional deadline has passed.
// Why it exists: Monotonic time avoids wall-clock adjustments affecting TTLs.