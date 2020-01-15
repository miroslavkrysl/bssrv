use mio::net::TcpStream;
use std::io::{Read, Write};
use crate::proto::{Deserializer, Serializer, ClientMessage, DeserializeError};
use log::{trace, info, error, debug, warn};
use std::net::SocketAddr;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error;


/// A peer error kind.
#[derive(Debug, Eq, PartialEq)]
pub enum PeerErrorKind {
    Closed,
    WouldBlock,
    Deserialization(DeserializeError),
}

impl Display for PeerErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            PeerErrorKind::Closed => write!(f, "Stream is closed."),
            PeerErrorKind::WouldBlock => write!(f, "Read or write operation would bock."),
            PeerErrorKind::Deserialization(error) => write!(f, "Deserialization failed: {}", error),
        }
    }
}

/// An error indicating that some error happened on the peer.
#[derive(Debug, Eq, PartialEq)]
pub struct PeerError {
    /// Kind of peer error.
    kind: PeerErrorKind
}

impl PeerError {
    /// Create a new peer error.
    fn new(kind: PeerErrorKind) -> Self {
        PeerError {
            kind
        }
    }
}

impl Display for PeerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Peer error: {}", self.kind)
    }
}

impl From<PeerErrorKind> for PeerError {
    fn from(kind: PeerErrorKind) -> Self {
        PeerError::new(kind)
    }
}

impl From<DeserializeError> for PeerError {
    fn from(error: DeserializeError) -> Self {
        PeerErrorKind::Deserialization(error).into()
    }
}

impl Error for PeerError {}


/// A network remote point with associated stream and address.
pub struct Peer {
    stream: TcpStream,
    address: SocketAddr,
    deserializer: Deserializer,
    serializer: Serializer,
    buffer: [u8; 1024],
}

impl Peer {
    pub fn new(stream: TcpStream, address: SocketAddr) -> Self {
        Peer {
            stream,
            address,
            deserializer: Deserializer::new(),
            serializer: Serializer::new(),
            buffer: [0; 1024],
        }
    }

    pub fn read(&mut self) -> Result<Option<ClientMessage>, PeerError> {
        debug!("reading from peer: {}", self.address);

        loop {
            // read available bytes
            let n = self.stream.read(&mut self.buffer);

            match n {
                Ok(0) => {
                    // proper stream close

                    debug!("peer {} has been properly closed", self.address);
                    return Err(PeerErrorKind::Closed.into());
                }
                Ok(n) => {
                    // some bytes available

                    let message = self.deserializer.deserialize(&self.buffer[0..n])?;
                    return Ok(message);
                }
                Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    // no more available bytes

                    trace!("all available bytes from peer {} has been read", self.address);
                    return Err(PeerErrorKind::WouldBlock.into())
                }
                Err(ref error) if error.kind() == std::io::ErrorKind::Interrupted =>
                    // interrupted, try again
                    continue,
                Err(error) => {
                    warn!("peer {} is closed: {}", self.address, error);
                    return Err(PeerErrorKind::Closed.into())
                }
            }
        }
    }

//    fn handle_connection_event(registry: &Registry, connection: &mut TcpStream,
//                               event: &Event,
//    ) -> io::Result<bool> {
//        if event.is_writable() {
//            // We can (maybe) write to the connection.
//            match connection.write(DATA) {
//                // We want to write the entire `DATA` buffer in a single go. If we
//                // write less we'll return a short write error (same as
//                // `io::Write::write_all` does).
//                Ok(n) if n < DATA.len() => return Err(io::ErrorKind::WriteZero.into()),
//                Ok(_) => {
//                    // After we've written something we'll reregister the connection
//                    // to only respond to readable events.
//                    registry.reregister(connection, event.token(), Interest::READABLE)?
//                }
//                // Would block "errors" are the OS's way of saying that the
//                // connection is not actually ready to perform this I/O operation.
//                Err(ref err) if would_block(err) => {}
//                // Got interrupted (how rude!), we'll try again.
//                Err(ref err) if interrupted(err) => {
//                    return handle_connection_event(registry, connection, event);
//                }
//                // Other errors we'll consider fatal.
//                Err(err) => return Err(err),
//            }
//        }
//
//        if event.is_readable() {
//            let mut connection_closed = false;
//            let mut received_data = Vec::with_capacity(4096);
//            // We can (maybe) read from the connection.
//            loop {
//                let mut buf = [0; 256];
//                match connection.read(&mut buf) {
//                    Ok(0) => {
//                        // Reading 0 bytes means the other side has closed the
//                        // connection or is done writing, then so are we.
//                        connection_closed = true;
//                        break;
//                    }
//                    Ok(n) => received_data.extend_from_slice(&buf[..n]),
//                    // Would block "errors" are the OS's way of saying that the
//                    // connection is not actually ready to perform this I/O operation.
//                    Err(ref err) if would_block(err) => break,
//                    Err(ref err) if interrupted(err) => continue,
//                    // Other errors we'll consider fatal.
//                    Err(err) => return Err(err),
//                }
//            }
//
//            if let Ok(str_buf) = from_utf8(&received_data) {
//                println!("Received data: {}", str_buf.trim_end());
//            } else {
//                println!("Received (none UTF-8) data: {:?}", &received_data);
//            }
//
//            if connection_closed {
//                println!("Connection closed");
//                return Ok(true);
//            }
//        }
//
//        Ok(false)
//    }
//
//    fn would_block(err: &io::Error) -> bool {
//        err.kind() == io::ErrorKind::WouldBlock
//    }
//
//    fn interrupted(err: &io::Error) -> bool {
//        err.kind() == io::ErrorKind::Interrupted
//    }
}