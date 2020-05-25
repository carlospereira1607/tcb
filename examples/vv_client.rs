use std::error::Error;
use std::time::Duration;
use tcb::broadcast::broadcast_trait::TCB;
use tcb::configuration::middleware_configuration::read_configuration_file;
use tcb::vv::version_vector::VV;

fn main() -> Result<(), Box<dyn Error>> {
    //String with the path to the configuration file
    let configuration_file = format!("path-to-config-file.toml");

    //Reading the configuration file
    let configuration = read_configuration_file(configuration_file)?;

    //The peer's unique id in the group that must be a natural number
    //This id starts at 0 and sequentially grows with each peer
    let id: usize = 0;

    //The local port where the middleware will wait for connections
    let port: usize = 61887;

    //Vec with the addresses and ports where the other peers are waiting for connections
    let group_addresses = vec![format!("localhost:61888")];

    //Creates a new version vector based middleware instance
    let mut vv = VV::new(id, port, group_addresses, configuration);

    //Creating and serializing the message to be sent
    let message = format!("Hello world");
    let serialized_message = message.into_bytes();

    //Sending the message and ignoring the returned ()
    let _ = vv.send(serialized_message);

    //Blocks the caller thread until a message is delivered or stable
    let blocking_delivery = vv.recv()?;

    //Returns a delivered or stable message with blocking the caller
    let non_blocking_delivery = vv.try_recv()?;

    //Blocks the caller during a timeout or until a message is returned
    let timeout_delivery = vv.recv_timeout(Duration::from_secs(1))?;

    Ok(())
}
