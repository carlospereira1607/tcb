use crate::configuration::middleware_configuration::Configuration;
use crate::graph::middleware::dot::Dot;
use crate::vv::structs::messages::{Message, MiddlewareClient};
use crate::vv::structs::version_vector::VersionVector;
use crossbeam::Sender;
use std::collections::HashMap;
use std::sync::Arc;

/**
 * Struct for wrapping received messages waiting to be delivered.
 */
#[derive(Debug, Clone)]
pub struct QueueNode {
    ///Sender id
    pub j: usize,
    ///Payload
    pub message: Message,
}

/**
 * Struct for wrapping delivered messages waiting to de stable.
 */
#[derive(Debug, Clone)]
pub struct StableDot {
    ///Message counter
    pub ctr: usize,
    ///Sender id
    pub j: usize,
    ///Payload    
    pub message: Message,
}

impl StableDot {
    /**
     * Creates a new StableDot.
     *
     * # Arguments
     *
     * `ctr` - Counter field
     *
     * `j` - Sender id
     *
     * `message` - Payload
     */
    pub fn new(ctr: usize, j: usize, message: Message) -> Self {
        Self { ctr, j, message }
    }
}

/**
 * Implementation of the causal delivery algorithm.
 */
#[allow(non_snake_case)]
pub struct VV {
    pub V: VersionVector,
    pub R: VersionVector,
    pub DQ: Vec<QueueNode>,
    pub M: Vec<VersionVector>,
    pub M_entry_row_num: VersionVector,
    pub SV: VersionVector,
    pub SMap: HashMap<Dot, StableDot>,
    pub ctr: usize,
    pub peer_index: usize,
    pub client: Sender<MiddlewareClient>,
    pub configuration: Arc<Configuration>,
    pub peer_number: usize,
}

#[allow(non_snake_case)]
impl VV {
    /**
     * Builds a new VV struct that implements the causal delivery algorithm.
     *
     * # Arguments
     *
     * `peer_number` - Number of peers in the group.
     *
     * `peer_index` - Local peer's globally unique id.
     *
     * `client` - Channel between the Middleware and the Peer that will be used to send delivered/stable messages to Peer.
     *
     * `configuration` - Middleware's configuration file.
     */
    pub fn new(
        peer_number: usize,
        peer_index: usize,
        client: Sender<MiddlewareClient>,
        configuration: Arc<Configuration>,
    ) -> Self {
        let DQ: Vec<QueueNode> = Vec::with_capacity(peer_number * 2);
        let mut M: Vec<VersionVector> = Vec::new();

        for _ in 0..peer_number {
            M.push(VersionVector::new(peer_number));
        }

        Self {
            V: VersionVector::new(peer_number),
            R: VersionVector::new(peer_number),
            DQ,
            M,
            M_entry_row_num: VersionVector::new(peer_number),
            SV: VersionVector::new(peer_number),
            SMap: HashMap::new(),
            ctr: 0,
            peer_index,
            client,
            configuration,
            peer_number,
        }
    }

    /**
     * Handler for a message sent by the Client to be broadcast. This function adds a
     * message to the causal graph.
     *
     * # Arguments
     *
     * `message` - Message received from the Client.
     */
    pub fn dequeue(&mut self, message: Message) {
        self.V[self.peer_index] += 1;

        if self.configuration.track_causal_stability {
            self.updatestability(self.peer_index, message);
        }
    }

    /**
     * Handles a message received from a peer via broadcast.
     *
     * # Arguments
     *
     * `j` - Sender id
     *
     * `message` - Message received from a peer in the group.
     */
    pub fn receive(&mut self, j: usize, message: Message) {
        if self.R[j] < message.version_vector[j] {
            self.R[j] += 1;

            if VersionVector::compare_version_vectors(j, &self.V, &message.version_vector) {
                self.deliver_and_log_message(None, Some(message), Some(j));

                if self.DQ.len() > 0 {
                    self.deliver();
                }
            } else {
                let queue_node = QueueNode { j, message };
                self.DQ.push(queue_node);
            }
        }
    }

    fn deliver(&mut self) {
        let mut delivered_index = 0;
        let mut received_index = 0;

        loop {
            if delivered_index >= self.DQ.len() {
                //Reached the end of the queue
                if received_index < delivered_index {
                    //If messages were delivered
                    //Truncate the vec to the remaining received messages' positions
                    self.DQ.truncate(received_index);
                    if self.DQ.len() > 0 {
                        //If the received queue still has messages after truncating
                        //Loop again
                        delivered_index = 0;
                        received_index = 0;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                let queue_node = self.DQ[delivered_index].clone();

                if VersionVector::compare_version_vectors(
                    queue_node.j,
                    &self.V,
                    &queue_node.message.version_vector,
                ) {
                    //Message can be delivered
                    self.deliver_and_log_message(Some(delivered_index), None, None);

                    delivered_index += 1;
                } else {
                    //Current message can't be delivered
                    //Copy value to "new" position and advance indexes
                    self.DQ[received_index] = queue_node.clone();
                    received_index += 1;
                    delivered_index += 1;
                }
            }
        }
    }

    fn deliver_and_log_message(
        &mut self,
        message_index: Option<usize>,
        received_message: Option<Message>,
        j: Option<usize>,
    ) {
        let message: Message;
        let sender_id: usize;

        if let Some(index) = message_index {
            message = self.DQ[index].message.clone();
            sender_id = self.DQ[index].j;
        } else {
            message = received_message.unwrap();
            sender_id = j.unwrap();
        }

        self.V[sender_id] += 1;

        let delivered_message = MiddlewareClient::DELIVER {
            sender_id,
            message: message.clone(),
            version_vector: message.version_vector.clone(),
        };

        self.client.send(delivered_message).unwrap();

        if self.configuration.track_causal_stability {
            self.updatestability(sender_id, message);
        }
    }

    fn updatestability(&mut self, j: usize, message: Message) {
        self.M[self.peer_index] = self.V.clone();

        if j != self.peer_index {
            self.M[j] = message.version_vector.clone();
        }

        let temp_dot = Dot::new(j, message.version_vector[j]);
        self.ctr += 1;

        if self.SMap.contains_key(&temp_dot) {
            panic!("Repeated dot on SMap!");
        }

        let stable_dot = StableDot::new(self.ctr, j, message);

        self.SMap.insert(temp_dot, stable_dot);

        //Making it a smarter Stable Vector
        //Only calculate the new SV if the new stable message from j
        //Was a previous row for the minimum of the matrix M
        //Therefore if a new message from j arrives
        //The minimum at each column needs to be recalculated
        if self.M_entry_row_num.contains(&j) {
            let newSV = self.calculateSV(j);

            if !self.SV.equal(&newSV) {
                let stable_dot_counters = VersionVector::dif(&newSV, &self.SV, newSV.len());
                let mut SD: Vec<Dot> = Vec::new();

                for (id, counter) in stable_dot_counters {
                    SD.push(Dot::new(id, counter));
                }

                //My code
                self.SV = newSV;

                self.stabilize(SD);
            }
        }
    }

    fn stabilize(&mut self, mut SD: Vec<Dot>) {
        SD.sort_by(|dot_a, dot_b| {
            let stable_dot_a = self.SMap.get(&dot_a).unwrap();
            let stable_dot_b = self.SMap.get(&dot_b).unwrap();
            stable_dot_a.ctr.cmp(&stable_dot_b.ctr)
        });

        for s in &SD {
            if !self.SMap.contains_key(&s) {
                let error_message =
                    format!("ERROR {} {:?} Dot key isn't in SMap", self.peer_index, s);
                panic!(error_message);
            }

            let stable_dot = self.SMap.remove(&s).unwrap();

            let stable_message = MiddlewareClient::STABLE {
                sender_id: stable_dot.j,
                message_id: stable_dot.message.id,
                version_vector: stable_dot.message.version_vector,
            };

            self.client.send(stable_message).unwrap();
        }
    }

    fn calculateSV(&mut self, sender_id: usize) -> VersionVector {
        let mut new_sv = self.SV.clone();
        let mut min: usize;
        let mut min_row_num;

        for column in 0..self.peer_number {
            if self.M_entry_row_num[column] == sender_id {
                min = self.M[0][column];
                min_row_num = 0;

                for row in 1..self.peer_number {
                    if self.M[row][column] < min {
                        min = self.M[row][column];
                        min_row_num = row;
                    }
                }

                new_sv[column] = min;
                self.M_entry_row_num[column] = min_row_num;
            }
        }

        new_sv
    }
}
