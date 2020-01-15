use mio::{Token, Poll, Events, Ready, PollOpt};
use std::net::SocketAddr;
use std::io;
use mio::net::TcpListener;
use std::collections::HashMap;
use std::time::Duration;
use rand::Rng;
use log::{trace, warn, error, info, debug};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::net::ServerManager;
use crate::net::peer::{Peer, PeerError};
use crate::proto::ClientMessage;
use std::sync::Arc;

/// A handle containing an atomic boolean flag.
/// It's used by in the Server to tell it to stop
/// even from another thread.
pub struct StopHandle {
    should_stop: Arc<AtomicBool>
}

impl Clone for StopHandle {
    fn clone(&self) -> Self {
        StopHandle{
            should_stop: self.should_stop.clone()
        }
    }
}

impl StopHandle {
    /// Create a new stop handle, that returns should_stop = false
    /// by default.
    fn new() -> Self {
        StopHandle {
            should_stop: Arc::new(AtomicBool::new(false))
        }
    }

    /// Ask the associated worker to stop
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst)
    }

    /// Check the flag if the worker should stop.
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Ordering::SeqCst)
    }
}

/// A TCP server.
///
/// In order to start the server logic, the run function must be called.
///
/// Continuously accepts new peers, read messages from them,
/// passes the messages to the servers manager and than eventually writes
/// back messages or disconnects the peers. It automatically disconnects
/// inactive peers.
///
/// Can be stopped and closed from another thread by calling the stop function.
pub struct Server {
    manager: ServerManager,
    peer_timeout: Duration,
    listener_token: Token,
    listener: TcpListener,
    peers: HashMap<Token, Peer>,
    poll: Poll,
    events: Events,
    stop_handle: StopHandle,
}

impl Server {

    /// Create a new server.
    ///
    /// This function creates all structures needed for the server execution including
    /// binding the listener address.
    pub fn new(address: &SocketAddr, peer_timeout: Duration, manager: ServerManager) -> io::Result<Self> {
        let mut server = Server {
            manager,
            peer_timeout,
            listener_token: Token(0),
            listener: TcpListener::bind(&address)?,
            peers: HashMap::new(),
            poll: Poll::new()?,
            events: Events::with_capacity(128),
            stop_handle: StopHandle::new(),
        };

        // register server listener for polling
        server.poll.register(
            &mut server.listener,
            server.listener_token,
            Ready::readable(),
            PollOpt::edge())?;

        Ok(server)
    }

    /// Get the servers stop handle.
    pub fn stop_handle(&self) -> &StopHandle {
        &self.stop_handle
    }

    /// Run the server.
    pub fn run(&mut self) -> Result<(), io::Error> {
        while !self.stop_handle.should_stop() {
            self.poll.poll(&mut self.events, Some(Duration::from_secs(1)))?;

            for event in self.events.iter() {
                if event.token() == self.listener_token {
                    let (mut stream, address) = self.listener.accept()?;
                    println!("Accepted connection from: {}", address);

//                    let token = next(&mut unique_token);
//                    poll.registry().register(
//                        &mut stream,
//                        token,
//                        Interest::READABLE.add(Interest::WRITABLE),
//                    )?;

                    let token = self.unique_token();

                    self.poll.register(
                        &mut stream,
                        token,
                        Ready::readable(), PollOpt::edge()

                    )?;

                    self.peers.insert(token, Peer::new(stream, address));
                } else {
                    // (maybe) received an event for a TCP connection.
                    if let Some(peer) = self.peers.get_mut(&event.token()) {
                        if event.readiness().is_readable() {
                            match peer.read() {
                                Ok(message) => {
                                    if let Some(m) = message {
                                        println!("received: {}", m);
                                    }
                                },
                                Err(error) => {
                                    println!("error: {}", error);
                                },
                            }
                        }
                    } else {
                        // Sporadic events happen.
                    };
//                    if done {
//                        connections.remove(&token);
//                    }
                }
            }
        }

        // TODO: disconnects peers

        info!("closing the server");

        Ok(())
    }

    /// Get unique token for a new peer.
    fn unique_token(&self) -> Token {
        loop {
            let token = Token(rand::thread_rng().gen());
            if token != self.listener_token && !self.peers.contains_key(&token) {
                break token
            }
        }
    }
}


//fn main() -> io::Result<()> {
//    env_logger::init();
//
//
//}
//
//fn next(current: &mut Token) -> Token {
//    let next = current.0;
//    current.0 += 1;
//    Token(next)
//}
//
///// Returns `true` if the connection is done.
//fn handle_connection_event(
//    registry: &Registry,
//    connection: &mut TcpStream,
//    event: &Event,
//) -> io::Result<bool> {
//    if event.is_writable() {
//        // We can (maybe) write to the connection.
//        match connection.write(DATA) {
//            // We want to write the entire `DATA` buffer in a single go. If we
//            // write less we'll return a short write error (same as
//            // `io::Write::write_all` does).
//            Ok(n) if n < DATA.len() => return Err(io::ErrorKind::WriteZero.into()),
//            Ok(_) => {
//                // After we've written something we'll reregister the connection
//                // to only respond to readable events.
//                registry.reregister(connection, event.token(), Interest::READABLE)?
//            }
//            // Would block "errors" are the OS's way of saying that the
//            // connection is not actually ready to perform this I/O operation.
//            Err(ref err) if would_block(err) => {}
//            // Got interrupted (how rude!), we'll try again.
//            Err(ref err) if interrupted(err) => {
//                return handle_connection_event(registry, connection, event)
//            }
//            // Other errors we'll consider fatal.
//            Err(err) => return Err(err),
//        }
//    }
//
//    if event.is_readable() {
//        let mut connection_closed = false;
//        let mut received_data = Vec::with_capacity(4096);
//        // We can (maybe) read from the connection.
//        loop {
//            let mut buf = [0; 256];
//            match connection.read(&mut buf) {
//                Ok(0) => {
//                    // Reading 0 bytes means the other side has closed the
//                    // connection or is done writing, then so are we.
//                    connection_closed = true;
//                    break;
//                }
//                Ok(n) => received_data.extend_from_slice(&buf[..n]),
//                // Would block "errors" are the OS's way of saying that the
//                // connection is not actually ready to perform this I/O operation.
//                Err(ref err) if would_block(err) => break,
//                Err(ref err) if interrupted(err) => continue,
//                // Other errors we'll consider fatal.
//                Err(err) => return Err(err),
//            }
//        }
//
//        if let Ok(str_buf) = from_utf8(&received_data) {
//            println!("Received data: {}", str_buf.trim_end());
//        } else {
//            println!("Received (none UTF-8) data: {:?}", &received_data);
//        }
//
//        if connection_closed {
//            println!("Connection closed");
//            return Ok(true);
//        }
//    }
//
//    Ok(false)
//}
//
//fn would_block(err: &io::Error) -> bool {
//    err.kind() == io::ErrorKind::WouldBlock
//}
//
//fn interrupted(err: &io::Error) -> bool {
//    err.kind() == io::ErrorKind::Interrupted