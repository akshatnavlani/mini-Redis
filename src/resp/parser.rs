// Incremental RESP2 parser.
//
// Add imports for the byte-buffer types you choose, plus the protocol error and
// frame types defined by the RESP module.

// Function reference: RESP-PARSER-001
// Function name: parse_frame
// Input: mutable received-byte buffer.
// Output: one frame, incomplete input, or a protocol error.
// Description: Inspect the leading RESP prefix, parse one complete frame, and
// consume only that frame's bytes.
// Why it exists: A single read can contain part of a command, one command, or
// many pipelined commands.

// Function reference: RESP-PARSER-002
// Function name: parse_line
// Input: byte slice and starting offset.
// Output: the position after CRLF, or an incomplete-input result.
// Description: Locate a valid RESP line terminator without reading beyond the
// available bytes.
// Why it exists: RESP requires CRLF, and accepting incomplete delimiters causes
// stream desynchronization.

// Function reference: RESP-PARSER-003
// Function name: parse_bulk_string
// Input: buffer positioned at a bulk-string frame.
// Output: byte content, a null value for $-1, incomplete input, or an error.
// Description: Read the declared byte length, payload, and trailing CRLF.
// Why it exists: Bulk strings are binary-safe and their declared byte length is
// more reliable than searching for a delimiter in the payload.

// Function reference: RESP-PARSER-004
// Function name: parse_array
// Input: buffer positioned at an array frame.
// Output: ordered nested frames, including supported null elements.
// Description: Parse each child frame recursively and preserve its order.
// Why it exists: Redis clients send commands as arrays of bulk strings.