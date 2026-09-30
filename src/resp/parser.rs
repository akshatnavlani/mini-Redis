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

#[derive(Debug, PartialEq, Eq)]
pub enum Frame<'a> {
    SimpleString(&'a [u8]),
    Error(&'a [u8]),
    Integer(i64),
    BulkString(Option<&'a [u8]>),
    Array(Vec<Frame<'a>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FrameParseResult<'a> {
    Complete {
        frame: Frame<'a>,
        next_position: usize,
    },
    Incomplete,
}

pub fn parse_frame(input: &[u8],start_position: usize) -> Result<FrameParseResult<'_>, ParseError> {
    if start_position > input.len() {
        return Err(ParseError::InvalidStartPosition);
    }

    if start_position == input.len() {
        return Ok(FrameParseResult::Incomplete);
    }

    match input[start_position] {
        b'+' | b'-' => {
            let line_result = parse_line(input, start_position + 1)?;

            match line_result {
                LineParseResult::Complete {
                    line,
                    next_position,
                } => {
                    let frame = if input[start_position] == b'+' {
                        Frame::SimpleString(line)
                    } else {
                        Frame::Error(line)
                    };

                    Ok(FrameParseResult::Complete {
                        frame,
                        next_position,
                    })
                }
                LineParseResult::Incomplete => Ok(FrameParseResult::Incomplete),
            }
        }

        b':' => {
            let line_result = parse_line(input, start_position + 1)?;

            match line_result {
                LineParseResult::Complete {
                    line,
                    next_position,
                } => {
                    let number_text = std::str::from_utf8(line)
                        .map_err(|_| ParseError::InvalidLineEnding)?;

                    let number = number_text
                        .parse::<i64>()
                        .map_err(|_| ParseError::InvalidLineEnding)?;

                    Ok(FrameParseResult::Complete {
                        frame: Frame::Integer(number),
                        next_position,
                    })
                }
                LineParseResult::Incomplete => Ok(FrameParseResult::Incomplete),
            }
        }

        b'$' => match parse_bulk_string(input, start_position)? {
            BulkStringResult::Complete {
                value,
                next_position,
            } => Ok(FrameParseResult::Complete {
                frame: Frame::BulkString(value),
                next_position,
            }),
            BulkStringResult::Incomplete => Ok(FrameParseResult::Incomplete),
        },

        b'*' => match parse_array(input, start_position)? {
            ArrayParseResult::Complete {
                value,
                next_position,
            } => Ok(FrameParseResult::Complete {
                frame: Frame::Array(value),
                next_position,
            }),
            ArrayParseResult::Incomplete => Ok(FrameParseResult::Incomplete),
        },

        _ => Err(ParseError::InvalidLineEnding),
    }
}

// Function reference: RESP-PARSER-002
// Function name: parse_line
// Input: byte slice and starting offset.
// Output: the position after CRLF, or an incomplete-input result.
// Description: Locate a valid RESP line terminator without reading beyond the
// available bytes.
// Why it exists: RESP requires CRLF, and accepting incomplete delimiters causes
// stream desynchronization.
#[derive(Debug, PartialEq, Eq)]
pub enum LineParseResult<'a>{
    Complete{
        line: &'a [u8],
        next_position: usize,
    },
    Incomplete,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError{
    InvalidStartPosition,
    InvalidLineEnding,
}

pub fn parse_line(input:&[u8],start_position:usize)->Result<LineParseResult<'_>,ParseError>{
    if start_position> input.len(){
        return Err(ParseError::InvalidStartPosition);
    }
    let mut position=start_position;
    while position<input.len(){
        match input[position]{
            b'\n'=>{
                if position==start_position|| input[position-1]!= b'\r'{
                    return Err(ParseError::InvalidLineEnding);
            }
            let line_end=position-1;
            return Ok(LineParseResult::Complete{
                line:&input[start_position..line_end],
                next_position: position+1,
            });
        }
        b'\r' if position+1==input.len()=>{
            return Ok(LineParseResult::Incomplete);
        }
        b'\r' if input[position+1]!=b'\n'=>{
            return Err(ParseError::InvalidLineEnding);
        }
        _=>{
            position+=1;}
        }
    }
    Ok(LineParseResult::Incomplete)
}

// Function reference: RESP-PARSER-003
// Function name: parse_bulk_string
// Input: buffer positioned at a bulk-string frame.
// Output: byte content, a null value for $-1, incomplete input, or an error.
// Description: Read the declared byte length, payload, and trailing CRLF.
// Why it exists: Bulk strings are binary-safe and their declared byte length is
// more reliable than searching for a delimiter in the payload.
pub enum BulkStringResult<'a>{
    Complete{
        value: Option<&'a [u8]>,
        next_position:usize,
    },
    Incomplete,
}

pub fn parse_bulk_string(input: &[u8], start_position:usize)-> Result<BulkStringResult<'_>,ParseError>{
    if start_position>=input.len(){
        return Ok(BulkStringResult::Incomplete)
    }
    if input[start_position] != b'$'{
        return Err(ParseError:: InvalidLineEnding);
    }
    let length_start=start_position+1;
    let length_result=parse_line(input,length_start)?;
    let (length_bytes,payload_start)=match length_result{
        LineParseResult::Complete{
            line,
            next_position,
        }=> (line,next_position),
        LineParseResult::Incomplete=>{
            return Ok(BulkStringResult::Incomplete);
        }
    };
    if length_bytes==b"-1"{
        return Ok(BulkStringResult::Complete{
            value:None,
            next_position:payload_start,
        });
    }
    if length_bytes.is_empty()||length_bytes.iter().any(|byte| !byte.is_ascii_digit())
    {
        return Err(ParseError::InvalidLineEnding);
    }
    let length_text=std::str::from_utf8(length_bytes).map_err(|_| ParseError::InvalidLineEnding)?;
    let payload_length = length_text.parse::<usize>().map_err(|_| ParseError::InvalidLineEnding)?;
    let payload_end=payload_start.checked_add(payload_length).and_then(|position| position.checked_add(2)).ok_or(ParseError::InvalidLineEnding)?;
    if payload_end > input.len(){
        return Ok(BulkStringResult::Incomplete);
    }
    let terminator_start=payload_start+payload_length;

    if input[terminator_start..payload_end] != *b"\r\n"{
        return Err(ParseError::InvalidLineEnding)
    }
    Ok(BulkStringResult::Complete{
        value:Some(&input[payload_start..terminator_start]),
        next_position:payload_end,
    })
}
// Function reference: RESP-PARSER-004
// Function name: parse_array
// Input: buffer positioned at an array frame.
// Output: ordered nested frames, including supported null elements.
// Description: Parse each child frame recursively and preserve its order.
// Why it exists: Redis clients send commands as arrays of bulk strings.
#[derive(Debug, PartialEq, Eq)]
pub enum ArrayParseResult<'a>{
    Complete{
        value: Vec<Frame<'a>>,
        next_position: usize,
    },
    Incomplete,
}

pub fn parse_array(input: &[u8],start_position:usize)-> Result<ArrayParseResult<'_>,ParseError>{
    if start_position>=input.len(){
        return Ok(ArrayParseResult::Incomplete);
    }
    if input[start_position]!=b'*'{
        return Err(ParseError::InvalidLineEnding);
    }
    let array_length_result=parse_line(input,start_position+1)?;
    let (length_bytes,mut current_position)=match array_length_result{
        LineParseResult::Complete{
            line,
            next_position,
        }=>(line,next_position),
        LineParseResult::Incomplete=>{
            return Ok(ArrayParseResult::Incomplete);
        }
    };
    if length_bytes.is_empty()||length_bytes.iter().any(|byte| !byte.is_ascii_digit()){
        return Err(ParseError::InvalidLineEnding);
    }
    let length_text=std::str::from_utf8(length_bytes).map_err(|_| ParseError::InvalidLineEnding)?;
    let element_count=length_text.parse::<usize>().map_err(|_| ParseError::InvalidLineEnding)?;
    let mut values=Vec::with_capacity(element_count);
    for _ in 0..element_count{
        let bulk_string_result=parse_bulk_string(input,current_position)?;
        match bulk_string_result{
            BulkStringResult::Complete{
                value,
                next_position,
            }=>{
                values.push(Frame::BulkString(value));
                current_position=next_position;
            }
            BulkStringResult::Incomplete=>{
                return Ok(ArrayParseResult::Incomplete);
            }
        }
    }
    Ok(ArrayParseResult::Complete{
        value:values,
        next_position:current_position,
    })
}