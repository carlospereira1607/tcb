use crate::vv::structs::messages::StreamMsg;
use bincode::{deserialize_from, serialize_into};
use std::net::TcpStream;

/**
 * Sends a handshake message to a peer.
 *
 * # Arguments
 *
 * `stream` - TCP stream to write the handshake message into.
 *
 * `local_id` - Local peer's globally unique id.
 */
pub fn send_handshake(mut stream: &TcpStream, local_index: usize) {
    serialize_into::<_, StreamMsg>(&mut stream, &StreamMsg::HND { index: local_index })
        .expect("ERROR: Couldn't write handshake message to peer socket");
}

/**
 * Finishes the handshake process.
 *
 * # Arguments
 *
 * `stream` - TCP stream to read the handshake message from.
 */
pub fn finish_protocol(stream: &TcpStream) -> usize {
    match deserialize_from::<_, StreamMsg>(stream) {
        Ok(decoded_handshake) => match decoded_handshake {
            StreamMsg::HND { index } => index,
            _ => {
                panic!("ERROR: Unexpected message type");
            }
        },
        Err(_) => {
            panic!("ERROR: Occurred when handling the receiver handshake message");
        }
    }
}
