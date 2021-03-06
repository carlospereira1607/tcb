use super::version_vector::VV;
use crate::configuration::middleware_configuration::Configuration;
use crate::vv::structs::messages::{ClientPeerMiddleware, Message, MiddlewareClient};
use crate::vv::structs::version_vector::VersionVector;
use bincode::serialize;
use crossbeam::{Receiver, Sender};
use std::sync::{Arc, Barrier};

/**
 * Starts the Middleware thread that receives messages from the Client to
 * be broadcast, receives messages from other peers and handles the delivery
 * of messages to the Client.
 *
 * # Arguments
 *
 * `local_id` - Local peer's globally unique id.
 *
 * `peer_addresses` - Addresses the middleware will connect to.
 *
 * `receive_channel` - Channel where the middleware will receive messages from the Client and Peers.
 *
 * `client` - Channel where the middleware will send delivered/stable messages to the Client.
 *
 * `peer_channels` - Channels to the Sender threads to send broadcast messages.
 *
 * `configuration` - Middleware's configuration file.
 */
pub fn start(
    local_id: usize,
    peer_addresses: Vec<String>,
    receive_channel: Receiver<ClientPeerMiddleware>,
    client: Sender<MiddlewareClient>,
    peer_channels: Vec<Sender<(Arc<Barrier>, Arc<Vec<u8>>)>>,
    configuration: Arc<Configuration>,
) {
    let mut vv = VV::new(
        peer_addresses.len() + 1,
        local_id,
        client.clone(),
        Arc::clone(&configuration),
    );

    loop {
        match receive_channel.recv() {
            Ok(ClientPeerMiddleware::CLIENT {
                msg_id,
                payload,
                version_vector,
            }) => {
                handle_message_from_client(
                    &mut vv,
                    msg_id,
                    payload,
                    version_vector,
                    &peer_channels,
                );
            }
            Ok(ClientPeerMiddleware::PEER { message, peer_id }) => {
                vv.receive(peer_id, message);
            }
            Ok(ClientPeerMiddleware::SETUP) => {}
            Ok(ClientPeerMiddleware::END) => {
                handle_finished_setup(&client);
                break;
            }
            Err(_) => {
                break;
            }
        }
    }
}

/**
 * Handles a message from the client by writing it in the channels
 * connected to the sender threads.
 */
fn handle_message_from_client(
    vv: &mut VV,
    msg_id: usize,
    payload: Vec<u8>,
    version_vector: VersionVector,
    channels: &Vec<Sender<(Arc<Barrier>, Arc<Vec<u8>>)>>,
) {
    let message = Message::new(msg_id, payload, version_vector);
    vv.dequeue(message.clone());

    //Creating a new struct Message
    //let message = Message::new(msg_id, payload, version_vector);
    //Serializing the struct with the new message
    let encoded_message: Vec<u8> =
        serialize(&message).expect("ERROR: Couldn't serialize the CLIENT message");

    //Creating a new arc with the serialized message
    let arc_msg = Arc::new(encoded_message);
    let stream_sender_barrier = Arc::new(Barrier::new(channels.len()));

    //Writing the message arc into the channels connected to each peer stream sender thread
    for channel in channels {
        match &channel.send((Arc::clone(&stream_sender_barrier), Arc::clone(&arc_msg))) {
            Ok(_) => {}
            Err(e) => {
                println!("ERROR: Could not send message to sender threads\n\t- {}", e);
            }
        }
    }
}

/**
 * Handles the setup end from the transport layer. The Middleware informs
 * the Client about this by sending a message.
 */
fn handle_finished_setup(client: &Sender<MiddlewareClient>) {
    match client.send(MiddlewareClient::SETUP) {
        Ok(_) => {}
        Err(e) => {
            println!(
                "ERROR: Failed to send the finishing SETUP message to client\n\t- {}",
                e
            );
        }
    }
}
