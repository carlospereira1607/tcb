use crate::configuration::middleware_configuration::Configuration;
use crossbeam::{RecvError, RecvTimeoutError, TryRecvError};
use std::time::Duration;

/**
 * Required API for the Tagged Causal Broadcast middleware.
 * This trait is implemented by the GRAPH and VV middleware implementations.
 * The delivery functions return messages wrapped in a generic enum so that this
 * trait can be implemented by both approaches.
 */
pub trait TCB {
    /**
     * Type of the return from a send call.
     */
    type SendCallReturn;

    /**
     * Creates a new middleware instance. This function only returns after the middleware
     * has a connection to every other peer in both directions.
     *
     * # Arguments
     *
     * `local_id` - Peer's globally unique id in the group.
     *
     * `local_port` - Port where the middleware will be listening for connections.
     *
     * `peer_addresses` - Addresses the middleware will connect to.
     *
     * `configuration` - Middleware's configuration file.
     */
    fn new(
        local_id: usize,
        local_port: usize,
        peer_addresses: Vec<String>,
        configuration: Configuration,
    ) -> Self;

    /**
     * Broadcasts a message to every peer in the group.
     * Returns the sent message context if successfull.
     *
     * # Arguments
     *
     * `msg` - Serialized message to be broadcast
     */
    fn send(&mut self, msg: Vec<u8>) -> Self::SendCallReturn;

    /**
     * Signals and waits for the middleware to terminate.
     */
    fn end(&self);

    /**
     * Delivers a message from the middleware. Blocks the calling thread
     * until a message is delivered or the channel to the middleware is
     * empty or disconnected.
     */
    fn recv(&mut self) -> Result<GenericReturn, RecvError>;

    /**
     * Attempts to deliver a message from the middleware without blocking
     * the caller thread. Either a message is immeadiately delivered
     * from the channel or an error is returned if the channel is empty.
     */
    fn try_recv(&mut self) -> Result<GenericReturn, TryRecvError>;

    /**
     * Waits for a message to be delivered from the middleware for a
     * limited time. If the channel is empty and not disconnected, the
     * caller thread is blocked until a message is received in the channel
     * or the timeout ends. If there are no messages until the timeout ends or
     * the channel becomes disconnected, an error is returned.
     *
     * # Arguments
     *
     * `duration` - Timeout duration
     */
    fn recv_timeout(&mut self, duration: Duration) -> Result<GenericReturn, RecvTimeoutError>;

    /**
     * ACKS a stable message. This is needed for the GRAPH approach so the node with
     * the message's information can be deleted from the graph and its position in the
     * array be available and reused for another message. Otherwise the array that maps
     * the causal dependency graph will grow exponentially. However, if stability was
     * disabled from the configuration file, then the message's are directly removed
     * from the graph upon delivery, rendering the call to this method unnecessary.
     *
     * The VV implementation doesn't require the call of this method.
     *
     * # Arguments
     *
     * `id` - Stable dot id field
     *
     * `counter` - Stable dot counter field
     */
    fn tcbstable(&mut self, id: usize, counter: usize);
}

/**
 * Enum for a generic message delivery call return from the BroadcastAPI trait.
 * If its a delivery, the return will the serialized message, the sender's id
 * and the message's id.
 * If its a stable message, the return will be the sender's id and the message's id.
*/
pub enum GenericReturn {
    ///Tuple with the serialized message, sender id and message id
    Delivery(Vec<u8>, usize, usize),
    ///Tuple with the sender id and message id
    Stable(usize, usize),
}
