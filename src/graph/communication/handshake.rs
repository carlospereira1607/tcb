use super::msg_types::*;
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
pub fn send_handshake(mut stream: &TcpStream, local_id: usize) {
    serialize_into::<_, StreamMessages>(
        &mut stream,
        &StreamMessages::Handshake { index: local_id },
    )
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
    match deserialize_from::<_, StreamMessages>(stream) {
        Ok(decoded_handshake) => match decoded_handshake {
            StreamMessages::Handshake { index } => index,
            m => {
                panic!("ERROR: Handshake received unexpected type - {:?}", m);
            }
        },
        Err(e) => {
            panic!(
                "ERROR: Occurred when handling the receiver handshake message - {}",
                e
            );
        }
    }
}
