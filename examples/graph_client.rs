use std::error::Error;
use std::time::Duration;
use tcb::broadcast::broadcast_trait::{GenericReturn, TCB};
use tcb::configuration::middleware_configuration::read_configuration_file;
use tcb::graph::graph::GRAPH;

/**
 * The GRAPH approach uses a graph to determine causal dependencies between messages.
 * This graph was mapped as an array, as to keep locality between nodes.
 * Moreover, the elements in this graph are softly deleted as to reuse their positions
 * for other messages. If stability is being calculated, then it's necessary to call
 * tcbstable so its position can be marked as available to be reused. Otherwise, the graph
 * exponentially grows and so does the array. However, if stability is disabled in the
 * configuration file, callinng tcbstable is not necessary as messages are removed from the
 * graph as they are delivered.
 */
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

    //Creates a new graph based middleware instance
    let mut graph = GRAPH::new(id, port, group_addresses, configuration);

    //Creating and serializing the message to be sent
    let message = format!("Hello world");
    let serialized_message = message.into_bytes();

    //Sending the message
    let sent_message_context = graph.send(serialized_message);

    //Blocks the caller thread until a message is delivered or stable
    let blocking_delivery = graph.recv()?;

    //Returns a delivered or stable message with blocking the caller
    let non_blocking_delivery = graph.try_recv()?;

    //Blocks the caller during a timeout or until a message is returned
    let timeout_delivery = graph.recv_timeout(Duration::from_secs(1))?;

    match blocking_delivery {
        GenericReturn::Delivery(serialized_delivery, _id, _counter) => {
            //Deserializing the delivered message
            let delivered_message = String::from_utf8(serialized_delivery)?;
            println!("Delivered message -> {}", delivered_message);
        }
        GenericReturn::Stable(id, counter) => {
            //Acking the stable message so it can be softly deleted from the causal graph
            //and its position reused by another message.
            //Otherwise the causal graph will exponentially grow and
            //constantly allocate more positions.
            graph.tcbstable(id, counter);
        }
    }

    Ok(())
}
