use super::message::Message;
use crate::graph::middleware::dot::Dot;

/**
 * Enum for the messages that will be sent/received in the channels between
 * the main middleware, stream reader and client
 */
pub enum ClientPeerMiddleware {
    ///Message sent by the Client to broadcast
    Client {
        dot: Dot,
        msg: Vec<u8>,
        context: Vec<Dot>,
    },
    ///Message received from a peer
    Peer { msg: Message },
    ///Indicates that the Middleware has finished the starting up
    Setup,
    ///ACK by the Client that a message is causally stable
    Stable { dot: Dot },
    ///Connection end
    End,
}
