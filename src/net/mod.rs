use crate::proto::{ClientMessage, ServerMessage};

mod server;
mod peer;

pub use server::Server;

pub enum ServerCommand {
    Send(usize, ServerMessage),
    Disconnect(usize)
}

pub struct ServerManager {}

impl ServerManager {
    pub fn new() -> Self {
        ServerManager{}
    }
//    fn handle_disconnect(&self, id: usize) -> Vec<ServerCommand<MsgOut>>;
//    fn handle_message(&self, id: usize, message: MsgIn) -> Vec<ServerCommand<MsgOut>>;
}