// RESP2 response encoder.
//
// Add imports for the output byte buffer and the RESP response types.
use bytes::{Bytes,BytesMut};
use super::Response;
// Function reference: RESP-ENCODER-001
// Function name: encode_frame
// Input: a typed RESP response.
// Output: one encoded RESP2 response as bytes.
// Description: Serialize every supported response type with its correct prefix,
// length, payload, and CRLF terminator.
// Why it exists: Redis clients use the prefix to determine how to interpret the
// response; one malformed prefix can corrupt every later response.

pub fn encode_frame(frame: &Response)-> Bytes{
    match frame{
        Response::SimpleString(value)=>{
            let mut buffer = BytesMut::new();
            buffer.extend_from_slice(b"+");
            buffer.extend_from_slice(value);
            buffer.extend_from_slice(b"\r\n");
            buffer.freeze()
        }
        Response::Error(value)=>{
            let mut buffer = BytesMut::new();
            buffer.extend_from_slice(b"-");
            buffer.extend_from_slice(value);
            buffer.extend_from_slice(b"\r\n");
            buffer.freeze()
        }
        Response::Integer(value)=>{
            let mut buffer = BytesMut::new();
            buffer.extend_from_slice(b":");
            buffer.extend_from_slice(value.to_string().as_bytes());
            buffer.extend_from_slice(b"\r\n");
            buffer.freeze()
        }
        Response::BulkString(value)=>{
            encode_bulk_string(value.as_deref())
        }
        Response::Array(elements)=>{
            let mut buffer = BytesMut::new();
            buffer.extend_from_slice(b"*");
            buffer.extend_from_slice(elements.len().to_string().as_bytes());
            buffer.extend_from_slice(b"\r\n");

            for element in elements{
                let encoded_element = encode_frame(element);
                buffer.extend_from_slice(&encoded_element);
            }

            buffer.freeze()
        }
    }
}

// Function reference: RESP-ENCODER-002
// Function name: encode_bulk_string
// Input: optional byte content.
// Output: a bulk-string frame or the null bulk-string frame.
// Description: Encode byte length rather than character count and represent a
// missing value with RESP's null form.
// Why it exists: GET must distinguish a missing key from a present empty value.
fn encode_bulk_string(value: Option<&[u8]>) -> Bytes{
    let mut buffer = BytesMut::new();

    match value {
        None => buffer.extend_from_slice(b"$-1\r\n"),
        Some(payload) => {
            let payload_length_text = payload.len().to_string();

            buffer.extend_from_slice(b"$");
            buffer.extend_from_slice(payload_length_text.as_bytes());
            buffer.extend_from_slice(b"\r\n");
            buffer.extend_from_slice(payload);
            buffer.extend_from_slice(b"\r\n");
        }
    }

    buffer.freeze()
}