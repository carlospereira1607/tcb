use super::version_vector::VersionVector;

/**
 * Struct for the message sent over the network.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    ///Sender id
    pub id: usize,
    ///Message payload
    pub payload: Vec<u8>,
    ///Message version vector
    pub version_vector: VersionVector,
}

impl Message {
    /**
     * Builds a new Message.
     *
     * # Arguments
     *
     * `id` - Sender id
     *
     * `payload` - Serialized message payload
     *
     * `version_vector` - Message version vector
     */
    pub fn new(id: usize, payload: Vec<u8>, version_vector: VersionVector) -> Self {
        Self {
            id,
            payload,
            version_vector,
        }
    }
}

/**
 * Enum of the messages sent/received in the streams between peers.
 * */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StreamMsg {
    ///Handshake
    HND { index: usize },
    ///Peer message
    MSG { msg: Vec<u8>, peer_id: usize },
    ///Terminate connection
    CLOSE,
}

/**
 * Enum for the messages that will be sent/received in the channels between
 * the main middleware, stream reader and client
 */
pub enum ClientPeerMiddleware {
    ///Message sent by the Client to broadcast
    CLIENT {
        msg_id: usize,
        payload: Vec<u8>,
        version_vector: VersionVector,
    },
    ///Message received from a peer
    PEER { peer_id: usize, message: Message },
    ///Indicates that the Middleware has finished the starting up
    SETUP,
    ///Connection end
    END,
}

/**
 * Enum that will be sent by the Middleware to the Client.
 * */
pub enum MiddlewareClient {
    ///Delivered message with its sender id, payload and version vector
    DELIVER {
        sender_id: usize,
        message: Message,
        version_vector: VersionVector,
    },
    ///Stable message with its sender id, message id and version vector
    STABLE {
        sender_id: usize,
        message_id: usize,
        version_vector: VersionVector,
    },
    ///Setup variation
    SETUP,
}
