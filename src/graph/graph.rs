use crate::broadcast::broadcast_trait::{GenericReturn, TCB};
use crate::configuration::middleware_configuration::Configuration;
use crate::graph::communication::{acceptor, connector};
use crate::graph::middleware::dot::Dot;
use crate::graph::middleware::message_types::ClientMessage;
use crate::graph::middleware::middleware_thread;
use crate::graph::structs::message_type::ClientPeerMiddleware;
use crossbeam::crossbeam_channel::unbounded;
use crossbeam::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};
use std::sync::{Arc, Barrier};
use std::time::Duration;
use std::{thread, usize};

/**
 * Client side of the graph based middleware service.
 * Maintains the API and necessary state to send and deliver messages.
 */
pub struct GRAPH {
    ///Receiver end of the channel between the client and the middleware thread
    receive_channel: Receiver<ClientMessage>,
    ///Sender end of the channel between the client and the middleware thread
    middleware_channel: Sender<ClientPeerMiddleware>,
    ///Dot of the next sent message
    dot: Dot,
    ///Context of the next sent message
    context: Vec<Dot>,
}

impl GRAPH {
    /**
     * Updates the next sent message's context upon a delivery.
     *
     * # Arguments
     *
     * `message` - Delivered or stable message.
     */
    fn handle_delivery(&mut self, message: ClientMessage) -> GenericReturn {
        match message {
            ClientMessage::Delivery {
                ref payload,
                dot,
                ref context,
            } => {
                Self::update_context(&dot, context, &mut self.context);

                GenericReturn::Delivery(payload.to_vec(), dot.id, dot.counter)
            }
            ClientMessage::Stable { dot } => GenericReturn::Stable(dot.id, dot.counter),
            _ => {
                panic!("ERROR: Received an EMPTY when it shouldn't!");
            }
        }
    }

    /**
     * Updates the Peer's context upon delivery. A dot's context dots are removed
     * from the Peer's context when adding it.
     *
     * # Arguments
     *
     * `dot` - Delivered message dot.
     *
     * `message_context` - Delivered message context.
     *
     * `local_context` - Next sent message message context.
     */
    fn update_context(dot: &Dot, message_context: &Vec<Dot>, local_context: &mut Vec<Dot>) {
        local_context.retain(|&client_dot| !message_context.contains(&client_dot));
        local_context.push(dot.clone());
    }

    /**
     * Starting method of the Middleware service. It creates and initializes
     * the necessary variables, communication channels and threads.
     *
     * # Arguments
     *
     * `local_id` - Local peer's globally unique id.
     *
     * `local_port` - Port where the middleware will be listening for connections.
     *
     * `peer_addresses` - Addresses the middleware will connect to.
     *
     * `configuration` - Middleware's configuration file.
     */
    fn start_service(
        local_id: usize,
        local_port: usize,
        peer_addresses: Vec<String>,
        configuration: Arc<Configuration>,
    ) -> (Sender<ClientPeerMiddleware>, Receiver<ClientMessage>) {
        let setup_end_barrier = Arc::new(Barrier::new(peer_addresses.len() + 1));

        //Creating the clone of the middleware configuration arc
        let configuration_clone = Arc::clone(&configuration);

        //Creating the channel where the middleware writes to
        //and the client reads from
        let (middleware_send_channel, peer_receive_channel) = unbounded::<ClientMessage>();

        //Creating the channel where the main middleware thread reads from
        //and the peer threads and client write to
        let (peer_reader_send_channel, middleware_receive_channel) =
            unbounded::<ClientPeerMiddleware>();

        let peer_reader_send_channel_clone = peer_reader_send_channel.clone();

        //Cloning the port array for the acceptor thread
        let acceptor_thread_peer_addresses = peer_addresses.clone();

        //Formatting the peer's acceptor thread name
        let thread_name = format!("acceptor_thread_{}", local_id);
        let builder = thread::Builder::new()
            .name(thread_name)
            .stack_size(configuration.thread_stack_size);

        let setup_end_barrier_clone = Arc::clone(&setup_end_barrier);

        //Spawning the acceptor thread
        builder
            .spawn(move || {
                acceptor::start(
                    local_id,
                    local_port,
                    acceptor_thread_peer_addresses,
                    peer_reader_send_channel_clone,
                    configuration,
                    setup_end_barrier_clone,
                );
            })
            .unwrap();

        //Connecting to the peers' ports and getting the channels sender ends
        //between the middleware and the sender thread
        let channels_to_socket_threads: Vec<Sender<(Arc<Barrier>, Arc<Vec<u8>>)>> =
            connector::start(local_id, &peer_addresses, &configuration_clone);

        //Formatting the peer's middlware thread name
        let thread_name = format!("middleware_thread_{}", local_id);
        let builder = thread::Builder::new()
            .name(thread_name)
            .stack_size(configuration_clone.middleware_thread_stack_size);

        //Spawning the main middleware thread
        builder
            .spawn(move || {
                middleware_thread::start(
                    local_id,
                    peer_addresses,
                    middleware_receive_channel,
                    middleware_send_channel,
                    channels_to_socket_threads,
                    configuration_clone,
                )
            })
            .unwrap();

        setup_end_barrier.wait();
        //Return the channels the peer writes and reads from to the middleware
        (peer_reader_send_channel, peer_receive_channel)
    }
}

impl TCB for GRAPH {
    /**
     * Type of the return from a send call, which is the sent message context or an error.
     */
    type SendCallReturn = Result<Vec<Dot>, SendError<ClientPeerMiddleware>>;

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
    ) -> Self {
        let configuration = Arc::new(configuration);

        let (middleware_channel, receive_channel) =
            Self::start_service(local_id, local_port, peer_addresses, configuration);

        //Initializing the context and dot variables
        let context: Vec<Dot> = Vec::new();
        let dot = Dot::new(local_id, 0);

        GRAPH {
            receive_channel,
            middleware_channel,
            dot,
            context,
        }
    }

    /**
     * Broadcasts a message to every peer in the group.
     * Returns the sent message context if successfull.
     *
     * # Arguments
     *
     * `msg` - Serialized message to be broadcast
     */
    fn send(&mut self, msg: Vec<u8>) -> Self::SendCallReturn {
        //Incrementing the dot's counter entry
        self.dot.counter += 1;

        //Building the enum of the new message
        let client_message = ClientPeerMiddleware::Client {
            dot: self.dot.clone(),
            msg,
            context: self.context.clone(),
        };

        //Sending the enum to the middleware thread
        self.middleware_channel.send(client_message)?;
        //.expect("ERROR: Client could not send message to main middleware");

        //Clearing the context for the next sent message
        let context: Vec<Dot> = self.context.drain(..).collect();

        //Adding the last sent message's dot to the new context
        self.context.push(self.dot.clone());

        //Returning the previous message's context
        Ok(context)
    }

    /**
     * Signals and waits for the middleware to terminate.
     */
    fn end(&self) {
        let end_message = ClientPeerMiddleware::End;
        self.middleware_channel.send(end_message).unwrap();

        loop {
            match self.receive_channel.recv() {
                Ok(msg) => match msg {
                    ClientMessage::Empty => {
                        break;
                    }
                    _ => {}
                },
                Err(_) => {}
            }
        }
    }

    /**
     * Delivers a message from the middleware. Blocks the calling thread
     * until a message is delivered or the channel to the middleware is
     * empty or disconnected.
     */
    fn recv(&mut self) -> Result<GenericReturn, RecvError> {
        match self.receive_channel.recv() {
            Ok(message) => Ok(self.handle_delivery(message)),
            Err(e) => Err(e),
        }
    }

    /**
     * Attempts to deliver a message from the middleware without blocking
     * the caller thread. Either a message is immeadiately delivered
     * from the channel or an error is returned if the channel is empty.
     */
    fn try_recv(&mut self) -> Result<GenericReturn, TryRecvError> {
        match self.receive_channel.try_recv() {
            Ok(message) => Ok(self.handle_delivery(message)),
            Err(e) => Err(e),
        }
    }

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
    fn recv_timeout(&mut self, duration: Duration) -> Result<GenericReturn, RecvTimeoutError> {
        match self.receive_channel.recv_timeout(duration) {
            Ok(message) => Ok(self.handle_delivery(message)),
            Err(e) => Err(e),
        }
    }

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
    fn tcbstable(&mut self, id: usize, counter: usize) {
        let dot = Dot::new(id, counter);
        let stable_dot = ClientPeerMiddleware::Stable { dot };

        self.middleware_channel
            .send(stable_dot)
            .expect("ERROR: When the Client sends a STABLE message");
    }
}
