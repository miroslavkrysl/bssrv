mod server;
mod listener;
mod peer;

pub use server::Server;
pub use listener::Listener;
pub use peer::Peer;

pub type PeerId = usize;

pub trait HandleIO {
    fn incoming(peer_id: PeerId, data: Vec<u8>);
    fn new_peer(peer_id: PeerId);
    fn closed_peer(peer_id: PeerId);
}