// RESP2 response encoder.
//
// Add imports for the output byte buffer and the RESP response types.

// Function reference: RESP-ENCODER-001
// Function name: encode_frame
// Input: a typed RESP response.
// Output: one encoded RESP2 response as bytes.
// Description: Serialize every supported response type with its correct prefix,
// length, payload, and CRLF terminator.
// Why it exists: Redis clients use the prefix to determine how to interpret the
// response; one malformed prefix can corrupt every later response.

// Function reference: RESP-ENCODER-002
// Function name: encode_bulk_string
// Input: optional byte content.
// Output: a bulk-string frame or the null bulk-string frame.
// Description: Encode byte length rather than character count and represent a
// missing value with RESP's null form.
// Why it exists: GET must distinguish a missing key from a present empty value.