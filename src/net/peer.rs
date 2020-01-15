use mio::net::TcpStream;
use std::net::SocketAddr;
use crate::proto::{Deserializer, Serializer, ClientMessage, DeserializationError, ServerMessage};
use mio::{Poll, Token, Ready, PollOpt};
use std::{io, fmt};
use std::fmt::{Display, Formatter};
use std::error::Error;
use std::io::{Read, Write};

/// A network remote point with associated stream, address, serializer and deserializer.
pub struct Peer {
    stream: TcpStream,
    address: SocketAddr,
    deserializer: Deserializer,
    serializer: Serializer
}

impl Peer {
    /// Create new peer.
    pub fn new(stream: TcpStream, address: SocketAddr) -> Self {
        Peer {
            stream,
            address,
            deserializer: Deserializer::new(),
            serializer: Serializer::new()
        }
    }

    /// Get the peers remote address.
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    /// Register the peer for polling.
    pub fn register(&self, poll: &Poll, token: Token) -> io::Result<()> {
        poll.register(
            &self.stream,
            token,
            Ready::readable(),
            PollOpt::edge())
    }

    /// Reregister the peer for polling.
    pub fn reregister(&self, poll: &Poll, token: Token) -> io::Result<()> {
        let mut ready = Ready::readable();

        if self.serializer.has_bytes() {
            ready = ready | Ready::writable();
        }

        poll.register(
            &self.stream,
            token,
            Ready::readable(),
            PollOpt::edge())
    }

    /// Deregister the the peer from polling.
    pub fn deregister(&self, poll: &Poll) -> Result<(), io::Error> {
        poll.deregister(&self.stream)
    }

    /// Deserialize message into bytes and prepare them to stream write operation.
    pub fn add_message(&mut self, message: &ServerMessage) {
        self.serializer.serialize(&message);
    }

    /// Read as much data as possible at the moment from peer and build messages from it.
    pub fn do_read(&mut self) -> Result<Vec<ClientMessage>, PeerError> {

        // buffer for incoming bytes
        let mut buffer = [0; 1024];

        loop {
            // read available bytes into the buffer
            let n = self.stream.read(&mut buffer);

            match n {
                Ok(0) => {
                    // stream was properly closed
                    return Err(PeerErrorKind::Closed.into());
                }
                Ok(n) => {
                    // some are bytes available
                    self.deserializer.deserialize(&buffer[0..n])?;
                }
                Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {
                    // no more available data
                    break;
                }
                Err(ref error) if error.kind() == io::ErrorKind::Interrupted => {
                    // interrupted, try again
                    continue;
                }
                Err(error) => {
                    // fatal error
                    return Err(PeerErrorKind::Closed.into());
                }
            }
        }

        Ok(self.deserializer.take_messages())
    }

    /// Write as many buffered bytes as possible at the moment.
    pub fn do_write(&mut self) -> Result<(), PeerError> {
        let to_write = self.serializer.bytes();

        let bytes_written;

        loop {
            match self.stream.write(to_write) {
                Ok(n) if n < to_write.len() => {
                    // not all bytes were written because
                    // it's not possible at the moment
                    bytes_written = n;
                    break;
                }
                Ok(n) => {
                    // all bytes were written
                    bytes_written = n;
                    break;
                }
                Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {
                    // no bytes can be written at the moment
                    bytes_written = 0;
                    break;
                }
                Err(ref error) if error.kind() == io::ErrorKind::Interrupted => {
                    // interrupted, try again
                }
                Err(error) => {
                    // fatal error
                    return Err(PeerErrorKind::Closed.into())
                },
            }
        }

        self.serializer.clear(bytes_written);

        Ok(())
    }
}


// ---ERRORS---

/// A peer error kind.
#[derive(Debug, Eq, PartialEq)]
pub enum PeerErrorKind {
    Closed,
    Deserialization(DeserializationError),
}

impl Display for PeerErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            PeerErrorKind::Closed => write!(f, "Stream is closed."),
            PeerErrorKind::Deserialization(error) => write!(f, "Deserialization failed: {}", error),
        }
    }
}

/// An error indicating that an error happened on the peer.
#[derive(Debug, Eq, PartialEq)]
pub struct PeerError {
    /// Kind of peer error.
    kind: PeerErrorKind
}

impl PeerError {
    /// Create a new peer error.
    pub fn new(kind: PeerErrorKind) -> Self {
        PeerError {
            kind
        }
    }

    /// Get the error kind.
    pub fn kind(&self) -> &PeerErrorKind {
        &self.kind
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

impl From<DeserializationError> for PeerError {
    fn from(error: DeserializationError) -> Self {
        PeerErrorKind::Deserialization(error).into()
    }
}

impl Error for PeerError {}