// RESP protocol module boundary.
//
// Add this module's child-module declarations after creating parser.rs and
// encoder.rs. Re-export only the protocol types the command and server layers
// actually need.
//
mod encoder;
mod parser;
pub use parser::{parse_frame, Frame, FrameParseResult, ParseError};
pub use encoder::encode_frame;
// Function reference: RESP-MOD-001
// Function name: parse_frame
// Input: a mutable byte buffer containing received TCP bytes.
// Output: one complete RESP frame, an incomplete-input result, or a protocol
// error.
// Description: Public entry point for decoding exactly one frame while keeping
// any following pipelined bytes in the buffer.
// Why it exists: TCP does not preserve message boundaries, so parsing needs a
// stable boundary between network reads and command handling.

// Function reference: RESP-MOD-002
// Function name: encode_frame
// Input: a typed RESP response.
// Output: bytes in RESP2 wire format.
// Description: Public entry point for serializing responses for Redis clients.
// Why it exists: The server must produce exact simple-string, error, integer,
// bulk-string, null, and array frames.