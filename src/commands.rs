// Command validation and dispatch module.
//
// Add imports for duration handling, RESP frame types, and the shared store.
use std::time::Duration;
enum Command{
    PING,
    GET{key:Vec<u8>},
    SET{key:Vec<u8>,value:Vec<u8>,expiration:Option<Duration>},
    DEL{key:Vec<Vec<u8>>},
    EXISTS{key:Vec<u8>},
    EXPIRE{key:Vec<u8>,expiration:Duration},
    TTL{key:Vec<u8>},
}

#[derive(Debug, PartialEq, Eq)]
enum CommandError{
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
pub fn parse_command(frame:Frame)->Result<Command,CommandError>{
    let elements = match frame{
        Frame::Array(elements)=> elements,
        _ => return Err(CommandError::InvalidCommandType),
    };
    if elements.is_empty(){
        return Err(CommandError::EmptyCommand);
    }
    let arguments: Vec<Vec<u8>> = elements.into_iter().map(|element| match element{
        Frame::BulkString(Some(vlaue))=>Ok(value.to_vec()),
        _=> Err(CommandError::InvalidArgumentType),
    }).collect::<Result<_,_>>()?;
    let command_name= &arguments[0];
    if command_name.eq_ignore_ascii_case(b"PING"){
        if arguments.len()!=1{
            return Err(CommandError::WrongArgumentCount);
        }
        return Ok(Command::PING)
    };
    if command_name.eq_ignore_ascii_case(b"GET"){
        if arguments.len()!=2{
        return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::GET{
            key:arguments[1].clone()
        })
    }
    if command_name.eq_ignore_ascii_case(b"SET"){
        if arguments.len!=3 && arguments.len()!=5{
        return Err(CommandError::WrongArgumentCount)   
        }
        let expiration = if argument.len()==5{
            if !arguments[3].eq_ignore_ascii(b"EX"){
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
        })
    }
    if command_name.eq_ignore_ascii_case(b"DEL"){
        if arguments.len()<2{
            return Err(CommandError::WrongArgumentCount);
        }
        return Ok(Command::DEL{
            key: arguments[1..].to_vec(),
        });
    }
    if command_name.eq_ignore_ascii_case(b"EXISTS"){
        if arguments.len()!=2{
            return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::EXISTS{
            key:argument[1].clone(),
        });
    }
    if command_name.eq_ignore_ascii_case(b"EXPIRE"){
        if arguments.len()!=2{
            return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::EXPIRE{
            key: arguments[1].clone(),
            expiration: parse_expiration(&arguments[2])?
        })
    }
    if command_name.eq_ignore_ascii_case(b"TTL"){
        if arguments.len()!=1{
            return Err(CommandError::WrongArgumentCount)
        }
        return Ok(Command::TTL{
            key: arguments[1].clone()
        })
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
fn dispatch(command:Command,store:StoreHandle)->Frame{
    match command {
        Command::PING => Frame::SimpleString("PONG".into()),
        Command::GET{key}=>{
            let value=store.get(&key);
            match value{
                Some(v)=>Frame::BulkString(Some(v)),
                None=>Frame::BulkString(None),
            }
        }
        Command::SET{key, value, expiration}=>{
            let result=store.set(key,value,expiration);
            match result{
                Ok(())=>Frame::SimpleString("OK".into()),
                Err(_)=>Frame::Error("SET failed".into()),
            }
        }
        Command::DEL{keys}=>{
            let result=store.delete(&keys);
            match result{
                Ok(count)=>Frame::Integer(count as i64),
                Err(_)=>Frame::Error("DEL failed".into()),
            }
        }
        Command::EXISTS{key}=>{
            let exists=store.exists(&key);
            Frame::Integer(if exists{1}else{0})
        }
        Command::EXPIRE{key,duration}=>{
            let result=store.expire(&key,duration);
            match result{
                Ok(updated)=>Frame::Integer(if updated{1}else{0}),
                Err(_)=>Frame::Error("EXPIRE failed".into()),
            }
        }
        Command::TTL{key}=>{
            let ttl=store.ttl(&key);
            Frame::Integer(ttl as i64)
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