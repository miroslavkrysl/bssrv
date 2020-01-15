use crate::net::ServerManager;
use mio::{Token, Poll, Events, Ready, PollOpt};
use std::net::SocketAddr;
use std::io;
use mio::net::TcpListener;
use std::collections::HashMap;
use std::time::Duration;
use crate::net::peer::{Peer, PeerError};
use std::marker::PhantomData;
use rand::Rng;
use crate::proto::ClientMessage;


pub struct Server {
    manager: ServerManager,
    timeout: Duration,
    listener_token: Token,
    listener: TcpListener,
    peers: HashMap<Token, Peer>,
    poll: Poll,
    events: Events,
    closed: bool
}

impl Server {
    pub fn new(address: &SocketAddr, timeout: Duration, manager: ServerManager) -> io::Result<Self> {
        Ok(Server {
            manager,
            timeout,
            listener_token: Token(0),
            listener: TcpListener::bind(&address)?,
            peers: HashMap::new(),
            poll: Poll::new()?,
            events: Events::with_capacity(128),
            closed: false
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        let mut i = 0;
        self.poll.register(&mut self.listener, self.listener_token, Ready::readable(),
                      PollOpt::edge())?;
        loop {
            println!("loop {}", i);
            i += 1;
            self.poll.poll(&mut self.events, Some(self.timeout))?;

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
    }

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