use super::sender;
use crate::configuration::middleware_configuration::Configuration;
use crossbeam::crossbeam_channel::unbounded;
use crossbeam::Sender;
use std::net::TcpStream;
use std::sync::{Arc, Barrier};
use std::thread;

/**
 * Starts the Connector thread that connects to every peer in the group and ends when
 * successfully connected to all of them.
 *
 * # Arguments
 *
 * `local_id` - Local peer's globally unique id.
 *
 * `peer_addresses` - Addresses the middleware will connect to.
 *
 * `configuration` - Middleware's configuration file.
 */
pub fn start(
    local_id: usize,
    peer_addresses: &Vec<String>,
    configuration: &Arc<Configuration>,
) -> Vec<Sender<(Arc<Barrier>, Arc<Vec<u8>>)>> {
    let mut peers_channels_to_sockets_threads = Vec::new();
    let mut channels_thread_spawn = Vec::new();

    //The connections to the peers will be concurrent
    for i in 0..peer_addresses.len() {
        let peer_id: usize;

        if i < local_id {
            peer_id = i;
        } else {
            peer_id = i + 1;
        }

        let temp_peer_port = peer_addresses[i].clone();
        let temp_configuration = Arc::clone(configuration);

        channels_thread_spawn.push(thread::spawn(move || {
            connect_to_single_peer(local_id, peer_id, temp_peer_port, temp_configuration)
        }));
    }

    for channel_spawn_result in channels_thread_spawn {
        match channel_spawn_result.join() {
            Ok(channel) => {
                peers_channels_to_sockets_threads.push(channel);
            }
            Err(_) => {
                println!("ERROR: There were problems when joining the peer channels");
            }
        }
    }

    peers_channels_to_sockets_threads
}

/**
 * Connects to a single peer. The call to this will only end when the
 * connection to the peer is successfull.
 */
fn connect_to_single_peer(
    local_index: usize,
    peer_index: usize,
    peer_address: String,
    configuration: Arc<Configuration>,
) -> Sender<(Arc<Barrier>, Arc<Vec<u8>>)> {
    let out: Sender<(Arc<Barrier>, Arc<Vec<u8>>)>;

    loop {
        let connect = TcpStream::connect(&peer_address);
        match connect {
            Ok(stream) => {
                stream
                    .set_nonblocking(false)
                    .expect("ERROR: Failed to set stream non-blocking mode");

                let (socket_thread_send, socket_thread_recv) =
                    unbounded::<(Arc<Barrier>, Arc<Vec<u8>>)>();

                out = socket_thread_send;

                let temp_config_arc = Arc::clone(&configuration);

                let thread_name = format!("sender_thread_{}_{}", local_index, peer_index);
                let builder = thread::Builder::new()
                    .name(thread_name)
                    .stack_size(configuration.thread_stack_size);

                builder
                    .spawn(move || {
                        sender::start(stream, socket_thread_recv, local_index, temp_config_arc);
                    })
                    .unwrap();

                return out;
            }
            Err(_) => {}
        }
    }
}
