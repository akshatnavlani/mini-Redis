// TCP server module.
//
// Add imports for TCP listener/stream types, async read/write traits, shared
// store state, RESP parsing/encoding, and interval timers.
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use std::time::{Duration,Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::commands::{dispatch, parse_command};
use crate::resp::{
    encode_frame,
    parse_frame,
    FrameParseResult,
    Response,
};
use crate::store::{self, StoreHandle};
// Function reference: SERVER-001
// Function name: run
// Input: bind address and server configuration.
// Output: asynchronous result that ends on shutdown or fatal listener error.
// Description: Bind the listener, accept clients, spawn connection tasks, and
// start the expiry sweeper.
// Why it exists: The listener should remain responsive and delegate client work
// to lightweight Tokio tasks.
pub(crate) async fn run(bind_address:SocketAddr,store_handle:StoreHandle, sweep_interval:Duration)-> io::Result<()>{
    let listener = TcpListener::bind(bind_address).await?;
    tokio::spawn(run_expiry_sweeper(Arc::clone(&store_handle), sweep_interval));
    loop{
        let (stream, _) = listener.accept().await?;
        let client_store=Arc::clone(&store_handle);
        tokio::spawn(async move{
            if let Err(error) = handle_connection(stream,client_store).await{
                tracing::error!(%error,"Client Connection Failed");
            }
        });
    }
}

// Function reference: SERVER-002
// Function name: handle_connection
// Input: accepted TCP stream and shared store.
// Output: asynchronous connection result.
// Description: Maintain a read buffer, drain all complete pipelined frames,
// dispatch commands, and write ordered responses.
// Why it exists: TCP reads are not command boundaries, and pipelining is required
// for realistic Redis benchmark behavior.
pub(crate) async fn handle_connection(mut stream: TcpStream, store_handle:StoreHandle) -> io::Result<()>{
    let mut read_buffer = BytesMut::with_capacity(4096);
    loop{
        let bytes_read = stream.read_buf(&mut read_buffer).await?;
        if bytes_read == 0{
            return Ok(());
        }
        loop{
            let parsed_frame = parse_frame(&read_buffer, 0)
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid RESP frame",
                    )
                })?;
            let (frame,next_position) = match parsed_frame{
                FrameParseResult::Incomplete => break,
                FrameParseResult::Complete{
                    frame,
                    next_position
                }=> (frame,next_position),
            };

            let response = match parse_command(frame){
                Ok(command) => dispatch(command,&store_handle).await,
                Err(_)=> Response::Error(Bytes::from_static(b"ERR Invalid Command"))
            };
            let encoded_response = encode_frame(&response);
            stream.write_all(&encoded_response).await?;
            let _ = read_buffer.split_to(next_position);
        }
    }
}

// Function reference: SERVER-003
// Function name: run_expiry_sweeper
// Input: shared store and configured interval.
// Output: asynchronous background task.
// Description: Periodically invoke active expiry cleanup.
// Why it exists: Memory cleanup must continue even when no client reads keys.
pub (crate) async fn run_expiry_sweeper(store_handle:StoreHandle,sweep_interval:Duration){
    let mut interval = tokio::time::interval(sweep_interval);
    loop{
        interval.tick().await;
        let current_time = Instant::now();
        let removed_count = store::remove_expired(&store_handle,current_time).await;
        if removed_count>0{
            tracing::debug!(removed_count,"removed expired keys");
        }
    }
}