use crossbeam::crossbeam_channel::RecvTimeoutError;
use std::error::Error;
use std::time::{Duration, SystemTime};
use tcb::broadcast::broadcast_trait::{GenericReturn, TCB};
use tcb::configuration::middleware_configuration::read_configuration_file;
use tcb::graph::graph::GRAPH;
use tcb::vv::version_vector::VV;

pub fn main() -> Result<(), Box<dyn Error>> {
    //Vec with the addresses and ports where the other peers are waiting for connections
    let group_addresses = vec![format!("localhost:61888")];

    //Simulating a client that uses the graph based middleware approach
    generic_client::<GRAPH>(0, 12345, group_addresses)?;

    //Vec with the addresses and ports where the other peers are waiting for connections
    let group_addresses = vec![format!("localhost:61889")];

    //Simulating a client that uses the version vector middleware approach
    generic_client::<VV>(0, 54321, group_addresses)?;

    Ok(())
}

/**
 * Spawns a generic client. The middleware instance to be used must implement
 * the BroadcastAPI trait. This client will send 100 messages and before each
 * send will read messages for 10s.
 *
 * # Arguments
 *
 * `local_id` - Client globally unique id
 *
 * `local_port` - Port where the middleware instance will listen for connections  
 *
 * `peer_addresses` - Addresses of the other peers in the group
*/
pub fn generic_client<T: TCB>(
    local_id: usize,
    local_port: usize,
    peer_addresses: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let mut sent_messages = 0;

    //String with the path to the configuration file
    let configuration_file = format!("path-to-config-file.toml");

    //Reading the configuration file
    let configuration = read_configuration_file(configuration_file)?;

    let mut middleware = T::new(local_id, local_port, peer_addresses, configuration);

    while sent_messages < 100 {
        deliver_messages(&mut middleware)?;

        //Creating and serializing the message to be sent
        let message = format!("{} {}", local_id, sent_messages);
        let serialized_message = message.into_bytes();

        //Sending the message
        let _ = middleware.send(serialized_message);

        //Updating sent messages counter
        sent_messages += 1;
    }

    //Client sent 100 messages while delivering messages for 10s before each send

    Ok(())
}

/**
 * Delivers all available messages for 10 seconds.
 *
 * # Arguments
 *
 * `tcb` - Middleware instance
*/
fn deliver_messages<T: TCB>(tcb: &mut T) -> Result<(), Box<dyn Error>> {
    //Getting the current time
    let mut now = SystemTime::now();
    let end_time = now + Duration::from_secs(10);
    let mut delivery_timeout = end_time.duration_since(now).unwrap();

    loop {
        match tcb.recv_timeout(delivery_timeout) {
            Ok(GenericReturn::Delivery(serialized_message, id, cntr)) => {
                let delivered_message = String::from_utf8(serialized_message)?;
                println!(
                    "Delivered message -> ({}, {}) {}",
                    id, cntr, delivered_message
                );
            }
            Ok(GenericReturn::Stable(id, cntr)) => {
                println!("Stable message -> ({}, {})", id, cntr);
            }
            Err(e) => match e {
                RecvTimeoutError::Timeout => {
                    //Timeout finished and no more message delivery
                    //Exit function call
                    break;
                }
                RecvTimeoutError::Disconnected => {
                    //An error occurred before the fimeout finished
                    //Handle it
                    panic!("Error was thrown before reading timeout ended");
                }
            },
        }

        now = SystemTime::now();

        //A message was delivered/stable before the timeout ended
        //Therefore its necessary to calculate the new timeout
        match end_time.duration_since(now) {
            Ok(value) => {
                //The new timeout value is the difference between the
                //earlier time (now) and the limit time for reading from
                //the channel (end_time)
                delivery_timeout = value;
            }
            Err(_) => {
                //If new duration surpasses end time - exit loop
                //Exit function call
                break;
            }
        }
    }

    Ok(())
}
