use super::msg_types::*;
use super::{handshake, reader};
use crate::configuration::middleware_configuration::Configuration;
use crate::graph::structs::message_type::ClientPeerMiddleware;
use bincode::deserialize_from;
use crossbeam::Sender;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Barrier};
use std::thread;

/**
 * Starts the Acceptor thread that waits for connections from other peers and
 * spawns a Reader for each. This function is called on a thread as to not
 * block the caller.
 *
 * # Arguments
 *
 * `local_id` - Local peer's globally unique id.
 *
 * `local_port` - Port where the middleware will be listening for connections.
 *
 * `peer_addresses` - Addresses the middleware will connect to.
 *
 * `middleware_channel` - Channel from the middleware to the peer.
 *
 * `configuration` - Middleware's configuration file.
 *
 * `setup_end_barrier` - Barrier signalling the middleware connected to every peer.
 */
pub fn start(
    local_id: usize,
    local_port: usize,
    peer_addresses: Vec<String>,
    middleware_channel: Sender<ClientPeerMiddleware>,
    configuration: Arc<Configuration>,
    setup_end_barrier: Arc<Barrier>,
) {
    //Binding middleware TCP listener and setting blocking behaviour
    let server = TcpListener::bind(format!("0.0.0.0:{}", local_port))
        .expect("ERROR: Stream failed to connect");

    server
        .set_nonblocking(false)
        .expect("ERROR: Failed to set stream non-blocking mode");

    let mut connected_peers = 0;

    loop {
        match server.accept() {
            Ok((stream, _)) => match deserialize_from::<_, StreamMessages>(&stream) {
                Ok(decoded_msg_type) => match decoded_msg_type {
                    StreamMessages::Handshake { index } => {
                        let setup_end_barrier_clone = Arc::clone(&setup_end_barrier);

                        handle_new_connection(
                            local_id,
                            &peer_addresses,
                            stream,
                            &middleware_channel,
                            index,
                            &mut connected_peers,
                            &configuration,
                            setup_end_barrier_clone,
                        );
                    }
                    m => {
                        println!("ERROR: Acceptor received unexpected type - {:?}", m);
                        break;
                    }
                },
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            },
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
}

/**
 * Handles a new peer connection.
 */
fn handle_new_connection(
    local_id: usize,
    peer_addresses: &Vec<String>,
    stream: TcpStream,
    middleware_channel: &Sender<ClientPeerMiddleware>,
    peer_id: usize,
    connected_peers: &mut usize,
    configuration: &Arc<Configuration>,
    setup_end_barrier: Arc<Barrier>,
) {
    handshake::send_handshake(&stream, local_id);

    let middleware_channel_temp = middleware_channel.clone();

    let thread_name = format!("stream_reader_{}_{}", local_id, peer_id);
    let builder = thread::Builder::new()
        .name(thread_name)
        .stack_size(configuration.thread_stack_size);

    builder
        .spawn(move || {
            reader::start(
                stream,
                middleware_channel_temp,
                local_id,
                peer_id,
                setup_end_barrier,
            );
        })
        .unwrap();

    *connected_peers += 1;

    if *connected_peers == peer_addresses.len() {
        let setup = ClientPeerMiddleware::Setup;
        match middleware_channel.send(setup) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "ERROR: Failed to send the SETUP message to client\n\t- {}",
                    e
                );
            }
        }
    }
}
