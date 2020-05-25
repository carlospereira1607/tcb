use crate::configuration::middleware_configuration::Configuration;
use crate::graph::communication::sender::*;
use crate::vv::communication::handshake;
use crate::vv::structs::messages::StreamMsg;
use bincode::{serialize_into, serialized_size};
use crossbeam::crossbeam_channel::RecvTimeoutError;
use crossbeam::Receiver;
use std::io::BufWriter;
use std::net::TcpStream;
use std::sync::{Arc, Barrier};
use std::time::Duration;

/**
 * Starts a Sender thread that sends messages to a peer.
 *
 * # Arguments
 *
 * `stream` - TCP stream between the peers.
 *
 * `middleware_channel` - Channel from the the Middleware to the Sender.
 *
 * `local_id` - Local peer's globally unique id.
 *
 * `configuration` - Middleware's configuration file.
 */
pub fn start(
    stream: TcpStream,
    middleware_channel: Receiver<(Arc<Barrier>, Arc<Vec<u8>>)>,
    local_id: usize,
    configuration: Arc<Configuration>,
) {
    //Starting handshake protocol
    handshake::send_handshake(&stream, local_id);

    //Receiving the id from the peer
    let peer_id = handshake::finish_protocol(&stream);

    let mut buffered_messages: usize = 0;
    let mut buffered_bytes: u64 = 0;

    //Flag that determines if the thread is in the new messages period
    //True  - NEW MESSAGES timeout
    //False - NO MESSAGES timeout
    let mut sender_timeout_flag: bool = true;
    let mut timeout: Duration = configuration.get_stream_sender_timeout();

    let mut stream = BufWriter::new(stream);

    loop {
        match middleware_channel.recv_timeout(timeout) {
            Ok((message_barrier, msg)) => {
                if !sender_timeout_flag {
                    sender_timeout_flag = true;
                    timeout = configuration.get_stream_sender_timeout();
                }

                message_barrier.wait();

                let stream_msg = StreamMsg::MSG {
                    msg: (*msg).clone(),
                    peer_id: local_id,
                };

                //Sending the message type and message payload as a single array of bytes
                match serialize_into::<_, StreamMsg>(&mut stream, &stream_msg) {
                    Ok(_) => {
                        buffered_messages += 1;
                        buffered_bytes += serialized_size::<StreamMsg>(&stream_msg).unwrap();
                    }
                    Err(_) => {
                        println!(
                            "WARN: Stream was closed between {} and {}",
                            local_id, peer_id
                        );
                        break;
                    }
                }
            }
            Err(e) => {
                match e {
                    RecvTimeoutError::Disconnected => {
                        //Creating and serializing CLOSE message
                        let stream_msg = StreamMsg::CLOSE;

                        match serialize_into::<_, StreamMsg>(&mut stream, &stream_msg) {
                            Ok(_) => {}
                            Err(_) => {}
                        }

                        break;
                    }
                    _ => {}
                }

                check_buffer_flush(
                    &mut sender_timeout_flag,
                    &mut stream,
                    &mut buffered_messages,
                    &mut buffered_bytes,
                    &mut timeout,
                    &configuration,
                    true,
                );
            }
        }
        check_buffer_flush(
            &mut sender_timeout_flag,
            &mut stream,
            &mut buffered_messages,
            &mut buffered_bytes,
            &mut timeout,
            &configuration,
            false,
        );
    }
}
