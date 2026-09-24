// RESP and command integration-test plan.
//
// Add imports for a TCP test client, async test support, and the test-server
// startup helper once those APIs exist.

// Function reference: TEST-PROTOCOL-001
// Function name: protocol_round_trip
// Input: running test-server address.
// Output: test result.
// Description: Exercise PING, GET, SET, DEL, EXISTS, EXPIRE, and TTL over raw
// TCP and verify exact response behavior.
// Why it exists: Unit tests cannot prove that parser, dispatcher, store, and
// encoder work correctly together.

// Function reference: TEST-PROTOCOL-002
// Function name: partial_frame_is_buffered
// Input: one RESP request delivered across multiple writes.
// Output: test result.
// Description: Confirm split TCP delivery waits for the complete frame.
// Why it exists: TCP preserves byte order, not message boundaries.

// Function reference: TEST-PROTOCOL-003
// Function name: pipelined_frames_are_drained
// Input: several RESP requests written without waiting for responses.
// Output: test result.
// Description: Confirm every complete frame receives one ordered response.
// Why it exists: redis-benchmark pipelines requests and depends on this behavior.