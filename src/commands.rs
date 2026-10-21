// Command validation and dispatch module.
//
// Add imports for duration handling, RESP frame types, and the shared store.
use bytes::Bytes;
use std::time::Duration;
use std::time::Instant;

use crate::resp::{Frame, Response};
use crate::store::{self, StoreHandle};

pub enum Command{
    PING,
    GET{key:Vec<u8>},
    SET{key:Vec<u8>,value:Vec<u8>,expiration:Option<Duration>},
    DEL{keys:Vec<Vec<u8>>},
    EXISTS{key:Vec<u8>},
    EXPIRE{key:Vec<u8>,expiration:Duration},
    TTL{key:Vec<u8>},
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandError{
    InvalidCommandType,
    EmptyCommand,
    InvalidArgumentType,
    UnknownCommand,
    WrongArgumentCount,
    InvalidOption,
    InvalidExpiration,
}
// Function reference: COMMANDS-001
// Function name: parse_command
// Input: a parsed RESP array of bulk-string arguments.
// Output: a typed command or command-specific error.
// Description: Validate command name, arity, options, and numeric arguments.
// Why it exists: Keep wire parsing separate from command grammar and semantics.
pub fn parse_command(frame: Frame<'_>) -> Result<Command, CommandError>{
    let elements = match frame{
        Frame::Array(elements)=> elements,
        _ => return Err(CommandError::InvalidCommandType),
    };
    if elements.is_empty(){
        return Err(CommandError::EmptyCommand);
    }
    let arguments: Vec<Vec<u8>> = elements.into_iter().map(|element| match element{
        Frame::BulkString(Some(value))=>Ok(value.to_vec()),
        _=> Err(CommandError::InvalidArgumentType),
    }).collect::<Result<_,_>>()?;
    let command_name= &arguments[0];
    if command_name.eq_ignore_ascii_case(b"PING"){
        if arguments.len()!=1{
            return Err(CommandError::WrongArgumentCount);
        }
        return Ok(Command::PING);
    };
    if command_name.eq_ignore_ascii_case(b"GET"){
        if arguments.len()!=2{
        return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::GET{
            key:arguments[1].clone()
        });
    }
    if command_name.eq_ignore_ascii_case(b"SET"){
        if arguments.len()!=3 && arguments.len()!=5{
        return Err(CommandError::WrongArgumentCount)   
        }
        let expiration = if arguments.len()==5{
            if !arguments[3].eq_ignore_ascii_case(b"EX"){
                return Err(CommandError::InvalidOption)
            }
            Some(parse_expiration(&arguments[4])?)
        }
        else{
            None
        };
        return Ok(Command::SET{
            key:arguments[1].clone(),
            value:arguments[2].clone(),
            expiration,
        });
    }
    if command_name.eq_ignore_ascii_case(b"DEL"){
        if arguments.len()<2{
            return Err(CommandError::WrongArgumentCount);
        }
        return Ok(Command::DEL{
            keys: arguments[1..].to_vec(),
        });
    }
    if command_name.eq_ignore_ascii_case(b"EXISTS"){
        if arguments.len()!=2{
            return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::EXISTS{
            key:arguments[1].clone(),
        });
    }
    if command_name.eq_ignore_ascii_case(b"EXPIRE"){
        if arguments.len()!=3{
            return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::EXPIRE{
            key: arguments[1].clone(),
            expiration: parse_expiration(&arguments[2])?
        });
    }
    if command_name.eq_ignore_ascii_case(b"TTL"){
        if arguments.len()!=2{
            return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::TTL{
            key: arguments[1].clone()
        });
    }
    Err(CommandError::UnknownCommand)
}

// Function reference: COMMANDS-002
// Function name: dispatch
// Input: typed command and shared store handle.
// Output: typed RESP response.
// Description: Execute PING, GET, SET, DEL, EXISTS, EXPIRE, or TTL and map the
// result to the proper response type.
// Why it exists: This is the narrow boundary between protocol and storage code.
pub(crate) async fn dispatch(
    command: Command,
    store_handle: &StoreHandle,
) -> Response {
    match command {
        Command::PING => Response::SimpleString(Bytes::from_static(b"PONG")),
        Command::GET { key } => {
            match store::get(store_handle, &key).await {
                Some(value) => Response::BulkString(Some(value)),
                None => Response::BulkString(None),
            }
        }
        Command::SET {
            key,
            value,
            expiration,
        } => {
            let expires_at = match expiration {
                Some(duration) => match Instant::now().checked_add(duration) {
                    Some(deadline) => Some(deadline),
                    None => {
                        return Response::Error(Bytes::from_static(
                            b"ERR expiration is too large",
                        ));
                    }
                },
                None => None,
            };

            store::set(
                store_handle,
                key,
                Bytes::from(value),
                expires_at,
            )
            .await;

            Response::SimpleString(Bytes::from_static(b"OK"))
        }
        Command::DEL { keys } => {
            let deleted_count = store::delete(store_handle, &keys).await;
            Response::Integer(deleted_count as i64)
        }
        Command::EXISTS { key } => {
            let exists = store::exists(store_handle, &key).await;
            Response::Integer(if exists { 1 } else { 0 })
        }
        Command::EXPIRE { key, expiration } => {
            let updated = store::expire(store_handle, &key, expiration).await;
            Response::Integer(if updated { 1 } else { 0 })
        }
        Command::TTL { key } => {
            let ttl = store::ttl(store_handle, &key).await;
            Response::Integer(ttl)
        }
    }
}

// Function reference: COMMANDS-003
// Function name: parse_expiration
// Input: SET option tokens such as EX and its seconds argument.
// Output: optional duration or validation error.
// Description: Validate and convert expiration syntax before it reaches storage.
// Why it exists: Invalid, negative, incomplete, or overflowing TTL input should
// fail as a command error rather than corrupting store state.
fn parse_expiration(value: &[u8])-> Result<Duration,CommandError>{
    if value.is_empty()||value.iter().any(|byte| !byte.is_ascii_digit()){
        return Err(CommandError::InvalidExpiration);
    }
    let value_text=std::str::from_utf8(value).map_err(|_|CommandError::InvalidExpiration)?;
    let seconds = value_text.parse::<u64>().map_err(|_|CommandError::InvalidExpiration)?;
    Ok(Duration::from_secs(seconds))
}