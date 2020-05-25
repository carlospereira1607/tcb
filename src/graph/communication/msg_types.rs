/**
 * Enum of the messages sent/received in the streams between peers.
 * */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StreamMessages {
    ///Handshake message
    Handshake { index: usize },
    ///Message payload
    Message { msg: Vec<u8> },
    ///Terminating the connection
    Close,
}
