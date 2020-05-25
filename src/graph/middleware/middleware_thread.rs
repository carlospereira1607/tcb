use super::dot::Dot;
use super::graph::GRAPH;
use super::message_types::ClientMessage;
use crate::configuration::middleware_configuration::Configuration;
use crate::graph::structs::message::Message;
use crate::graph::structs::message_type::ClientPeerMiddleware;
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
    client: Sender<ClientMessage>,
    peer_channels: Vec<Sender<(Arc<Barrier>, Arc<Vec<u8>>)>>,
    configuration: Arc<Configuration>,
) {
    let mut tcb = GRAPH::new(
        local_id,
        peer_addresses.len() + 1,
        client.clone(),
        Arc::clone(&configuration),
    );

    loop {
        match receive_channel.recv() {
            Ok(ClientPeerMiddleware::Client { dot, msg, context }) => {
                handle_message_from_client(&mut tcb, msg, &peer_channels, context, dot);
            }
            Ok(ClientPeerMiddleware::Peer { msg }) => {
                tcb.receive(msg);
            }
            Ok(ClientPeerMiddleware::Setup) => {}
            Ok(ClientPeerMiddleware::Stable { dot }) => {
                tcb.deletestable(dot);
            }
            Ok(ClientPeerMiddleware::End) => {
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
    tcb: &mut GRAPH,
    payload: Vec<u8>,
    channels: &Vec<Sender<(Arc<Barrier>, Arc<Vec<u8>>)>>,
    context: Vec<Dot>,
    dot: Dot,
) {
    //Creating a new struct Message
    let message = Message::new(payload, dot, context);

    //Calling the dequeue function
    tcb.dequeue(message.clone());

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
fn handle_finished_setup(client: &Sender<ClientMessage>) {
    client
        .send(ClientMessage::Empty)
        .expect("ERROR: Failed to send the finishing SETUP message to client");
}
