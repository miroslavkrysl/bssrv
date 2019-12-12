use std::collections::VecDeque;
use std::io;

mod server;
mod listener;
mod peer;

pub trait Codec {
    type In;
    type Out;

    fn decode(&mut self, buffer: &mut VecDeque<u8>) -> Result<Option<In>, io::Error>;
    fn encode(&mut self, message: Out, into: &mut VecDeque<u8>) -> io::Result<()>;
}
