//! Battleships protocol communication messages
//! and their serialization and serialization logic.

mod message;
mod codec;
mod deserialize;
mod serialize;

pub use message::ClientMessage;
pub use message::ServerMessage;

pub use deserialize::DeserializationError;
pub use deserialize::DeserializationErrorKind;
pub use deserialize::StructDeserializationError;
pub use deserialize::StructDeserializeErrorKind;

pub use deserialize::Deserializer;
pub use serialize::Serializer;