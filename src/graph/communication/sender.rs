use super::handshake;
use super::msg_types::StreamMessages;
use crate::configuration::middleware_configuration::Configuration;
use bincode::{serialize_into, serialized_size};
use crossbeam::crossbeam_channel::RecvTimeoutError;
use crossbeam::Receiver;
use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::ops::Mul;
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

                let stream_msg = StreamMessages::Message {
                    msg: (*msg).clone(),
                };

                //Sending the message type and message payload as a single array of bytes
                match serialize_into::<_, StreamMessages>(&mut stream, &stream_msg) {
                    Ok(_) => {
                        buffered_messages += 1;
                        buffered_bytes += serialized_size::<StreamMessages>(&stream_msg).unwrap();
                    }
                    Err(_) => {
                        //When the stream is closed, a warning is printed
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
                        //Creating and serializing close message
                        let stream_msg = StreamMessages::Close;

                        serialize_into::<_, StreamMessages>(&mut stream, &stream_msg).unwrap();

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

pub fn calculate_timeout(
    timeout_flag: bool,
    timeout: Duration,
    config: &Arc<Configuration>,
) -> Duration {
    let ret_timeout: Duration;
    //True  - NEW MESSAGES timeout
    //False - NO MESSAGES timeout

    if timeout_flag {
        ret_timeout = config.batching.get_lower_timeout();
    } else {
        if timeout.as_micros() * 2 <= config.batching.get_upper_timeout().as_micros() {
            ret_timeout = timeout.mul(2);
        } else {
            ret_timeout = config.batching.get_upper_timeout();
        }
    }

    ret_timeout
}

/**
 * Checks if its necessary to write the bytes from the buffer to the TCP stream.
 *
 * # Arguments
 *
 * `sender_timeout_flag` - Flag for determining if the reading timeout has expired.
 *
 * `stream` - TCP stream between the peers.
 *
 * `buffered_messages` - Number of buffered messages.
 *
 * `buffered_bytes` - Number of buffered bytes.
 *
 * `timeout` - Timeout duration.
 *
 * `configuration` - Middleware configuration.
 *
 * `error` - Flag for determining if the reading from the channel threw an error.
 */
pub fn check_buffer_flush(
    sender_timeout_flag: &mut bool,
    stream: &mut BufWriter<TcpStream>,
    buffered_messages: &mut usize,
    buffered_bytes: &mut u64,
    timeout: &mut Duration,
    configuration: &Arc<Configuration>,
    error: bool,
) {
    if *buffered_messages >= configuration.batching.message_number
        || *buffered_bytes > configuration.batching.size
        || (error && *buffered_messages > 0)
    {
        //Check if the error happened because of the SEND or the NO MESSAGES timeout
        if error && *sender_timeout_flag {
            //Change to false if it was the SEND timeout
            *sender_timeout_flag = false;
        }

        stream.flush().expect("ERROR: Could not flush stream!");
        *buffered_messages = 0;
        *buffered_bytes = 0;
    } else {
        //Check if the error happened because of the SEND or the NO MESSAGES timeout
        if error && *sender_timeout_flag {
            //Change to false if it was the SEND timeout
            *sender_timeout_flag = false;
        }
        if error {
            *timeout = calculate_timeout(*sender_timeout_flag, *timeout, configuration);
        }
    }
}
