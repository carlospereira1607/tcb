use super::dot::Dot;

/**
 * Enum that will be sent by the Middleware to the Client.
 * */
#[derive(Debug, Clone)]
pub enum ClientMessage {
    ///Empty variation
    Empty,
    ///Delivered message with its payload, dot and context
    Delivery {
        payload: Vec<u8>,
        dot: Dot,
        context: Vec<Dot>,
    },
    ///Stable message with its dot
    Stable { dot: Dot },
}
