use crate::broadcast::broadcast_trait::{GenericReturn, TCB};
use crate::configuration::middleware_configuration::Configuration;
use crate::vv::communication::{acceptor, connector};
use crate::vv::middleware::middleware_thread;
use crate::vv::structs::messages::{ClientPeerMiddleware, MiddlewareClient};
use crate::vv::structs::version_vector::VersionVector;
use crossbeam::crossbeam_channel::unbounded;
use crossbeam::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};
use std::sync::{Arc, Barrier};
use std::time::Duration;
use std::{thread, usize};

/**
 * Client side of the version vector based middleware service.
 * Maintains the API and necessary state to send and deliver messages.
 */
#[allow(non_snake_case)]
pub struct VV {
    //Receiver end of the channel between the client and the middleware thread
    receive_channel: Receiver<MiddlewareClient>,
    //Sender end of the channel between the client and the middleware thread
    middleware_channel: Sender<ClientPeerMiddleware>,
    message_id: usize,
    //Dot of the next sent message
    V: VersionVector,
    //Peer's id
    local_id: usize,
}

impl VV {
    /**
     * Updates the next sent message's version vector upon a delivery.
     *
     * # Arguments
     *
     * `message` - Delivered or stable message.
     */
    fn handle_delivery(&mut self, message: MiddlewareClient) -> GenericReturn {
        match message {
            MiddlewareClient::DELIVER {
                sender_id,
                version_vector,
                message,
            } => {
                self.V[sender_id] = version_vector[sender_id];

                GenericReturn::Delivery(message.payload, sender_id, version_vector[sender_id])
            }
            MiddlewareClient::STABLE {
                sender_id,
                message_id,
                ..
            } => GenericReturn::Stable(sender_id, message_id),
            _ => {
                panic!("ERROR: Received a SETUP when it shouldn't!");
            }
        }
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
    ) -> (Sender<ClientPeerMiddleware>, Receiver<MiddlewareClient>) {
        //Creating the clone of the middleware configuration arc
        let configuration_clone = Arc::clone(&configuration);

        //Creating the channel where the middleware writes to
        //and the client reads from
        let (middleware_send_channel, peer_receive_channel) = unbounded::<MiddlewareClient>();

        //Creating the channel where the main middleware thread reads from
        //and the peer threads and client write to
        let (peer_reader_send_channel, middleware_receive_channel) =
            unbounded::<ClientPeerMiddleware>();

        let peer_reader_send_channel_clone = peer_reader_send_channel.clone();

        //Cloning the peer addresses for the acceptor thread
        let acceptor_thread_peer_addresses = peer_addresses.clone();

        //Formatting the peer's acceptor thread name
        let thread_name = format!("acceptor_thread_{}", local_id);
        let builder = thread::Builder::new()
            .name(thread_name)
            .stack_size(configuration.thread_stack_size);

        //Cloning the channel to the logging service
        let setup_end_barrier = Arc::new(Barrier::new(peer_addresses.len() + 1));
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

#[allow(non_snake_case)]
impl TCB for VV {
    /**
     * Type of the return from a send call, which is an empty value or an error.
     */
    type SendCallReturn = Result<(), SendError<ClientPeerMiddleware>>;

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
        let client_number = peer_addresses.len() + 1;

        let (middleware_channel, receive_channel) =
            Self::start_service(local_id, local_port, peer_addresses, configuration);

        //Initializing the version vector
        let V = VersionVector::new(client_number);

        VV {
            receive_channel,
            middleware_channel,
            message_id: 0,
            V,
            local_id,
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
    fn send(&mut self, message: Vec<u8>) -> Self::SendCallReturn {
        self.message_id += 1;
        self.V[self.local_id] = self.message_id;

        let msg = ClientPeerMiddleware::CLIENT {
            msg_id: self.message_id,
            payload: message,
            version_vector: self.V.clone(),
        };

        self.middleware_channel.send(msg)?;

        Ok(())
    }

    /**
     * Signals and waits for the middleware to terminate.
     */
    fn end(&self) {
        let end_message = ClientPeerMiddleware::END;
        self.middleware_channel.send(end_message).unwrap();

        loop {
            match self.receive_channel.recv() {
                Ok(msg) => match msg {
                    MiddlewareClient::SETUP => {
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
            Ok(msg) => Ok(self.handle_delivery(msg)),
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
            Ok(msg) => Ok(self.handle_delivery(msg)),
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
            Ok(msg) => Ok(self.handle_delivery(msg)),
            Err(e) => Err(e),
        }
    }

    /**
     * ACKS a stable message, but is not necessary to call in the VV approach.
     *
     * * # Arguments
     *
     * `id` - Stable dot id field
     *
     * `counter` - Stable dot counter field
     */
    fn tcbstable(&mut self, _: usize, _: usize) {
        //Not implemented for VV
    }
}
