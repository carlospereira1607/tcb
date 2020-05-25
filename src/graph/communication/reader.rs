use super::msg_types::StreamMessages;
use crate::graph::structs::message::Message;
use crate::graph::structs::message_type::ClientPeerMiddleware;
use bincode::{deserialize, deserialize_from};
use crossbeam::Sender;
use std::net::TcpStream;
use std::sync::{Arc, Barrier};
use std::usize;

/**
 * Starts a Reader thread that receives messages from a stream
 * and sends them to the middleware.
 *
 * # Arguments
 *
 * `stream` - TCP stream between the peers.
 *
 * `middleware_channel` - Channel from the the Reader to the Middleware.
 *
 * `local_id` - Local peer's globally unique id.
 *
 * `peer_id` - Other peer's globally unique id.
 *
 * `setup_end_barrier` - Barrier signalling the middleware connected to every peer.
 */
pub fn start(
    stream: TcpStream,
    middleware_channel: Sender<ClientPeerMiddleware>,
    local_id: usize,
    peer_id: usize,
    setup_end_barrier: Arc<Barrier>,
) {
    setup_end_barrier.wait();

    loop {
        match deserialize_from::<_, StreamMessages>(&stream) {
            Ok(decoded_msg_type) => match decoded_msg_type {
                StreamMessages::Message { msg } => {
                    handle_received_peer_msg(msg, &middleware_channel);
                }

                StreamMessages::Close => {
                    break;
                }
                m => {
                    println!("ERROR: Reader received unexpected type - {:?}", m);
                    break;
                }
            },
            Err(e) => {
                println!(
                    "ERROR: {} is closing a connection with: {}\n\t{}",
                    local_id, peer_id, e
                );
                break;
            }
        }
    }
}

fn handle_received_peer_msg(msg: Vec<u8>, send_main_mid: &Sender<ClientPeerMiddleware>) {
    //Deserializing the vec of bytes to Message struct
    let decoded_msg: Message = deserialize(&msg)
        .expect("ERROR: Couldn't deserialize the Message type after reading from the stream");

    let peer_msg = ClientPeerMiddleware::Peer { msg: decoded_msg };

    //Sending the payload to the main middleware thread
    send_main_mid
        .send(peer_msg)
        .expect("ERROR: Failed to send message to main middleware thread");
}
