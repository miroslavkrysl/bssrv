use mio::net::TcpStream;
use mio::{Token, Evented, Poll, Event, Ready, PollOpt};
use std::io;

pub struct Peer {
    stream: TcpStream,
    closed: bool
}

impl Peer {
    pub fn new(stream: TcpStream) -> Self {
        Peer{
            stream,
            closed: false
        }
    }

    pub fn is_closed(&self) -> bool{
        self.closed
    }

    pub fn handle_io(&mut self, event: &Event) {
        if event.readiness().is_readable() {
            // TODO: do read
        }

        if event.readiness().is_writable() {
            // TODO: do write
        }

        // TODO: implement closing conditions
        self.closed = true;
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