use mio::net::TcpStream;
use mio::{Token, Evented, Poll, Ready, PollOpt};
use std::io;
use std::io::{Read, Write};

pub struct Peer {
    stream: TcpStream,
    closed: bool,
    in_buffer: Vec<u8>,
    out_buffer: Vec<u8>,
}

impl Peer {
    pub fn new(stream: TcpStream) -> Self {
        Peer {
            stream,
            closed: false,
            in_buffer: Vec::new(),
            out_buffer: Vec::new(),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn interests(&self) -> Ready {
        let mut interests = Ready::readable();

        if !self.out_buffer.is_empty() {
            interests |= Ready::writable();
        }

        interests
    }

    pub fn handle_io(&mut self, readiness: Ready) {
        if readiness.is_readable() {
            self.do_read();
        }

        if readiness.is_writable() {
            self.do_write();
        }
    }

    fn do_read(&mut self) {
        let mut buff = [0u8; 1024];

        let read = self.stream.read(&mut buff);

        match read {
            Ok(0) => {
                self.closed = true;
            }
            Ok(n) => {
                self.in_buffer.extend(buff.iter().take(n));
            }
            Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return;
            }
            Err(_) => {
                self.closed = true
            }
        }
    }

    fn do_write(&mut self) {
        let wrote = self.stream.write(&self.out_buffer);

        match wrote {
            Ok(n) => {
                self.out_buffer.clear();
            }
            Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return;
            }
            Err(_) => {
                self.closed = true
            }
        }
    }

    pub fn add_outgoing(&mut self, mut data: Vec<u8>) {
        self.out_buffer.append(&mut data);
    }
}

impl Evented for Peer {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.stream.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.stream.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.stream.deregister(poll)
    }
}