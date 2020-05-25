use crate::graph::middleware::dot::Dot;

/**
 * Struct for the message sent over the network.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    ///Message dot
    pub dot: Dot,
    ///Message payload
    pub payload: Vec<u8>,
    ///Message context
    pub context: Vec<Dot>,
}

impl Message {
    /**
     * Creates an empty message.
     */
    pub fn empty() -> Self {
        Self {
            dot: Dot::new(0, 0),
            payload: Vec::new(),
            context: Vec::new(),
        }
    }

    /**
     * Creates a message with payload, dot and context.
     */
    pub fn new(payload: Vec<u8>, dot: Dot, context: Vec<Dot>) -> Self {
        Self {
            payload,
            dot,
            context,
        }
    }
}
