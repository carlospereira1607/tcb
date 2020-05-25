use super::dag::ArrayMap;
use super::dot::Dot;
use super::message_types::ClientMessage;
use super::node::{Node, Stage};
use crate::configuration::middleware_configuration::Configuration;
use crate::graph::structs::message::Message;
use bit_vec::BitVec;
use crossbeam::Sender;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;

type BV = BitVec<u64>;

/**
 * Implementation of the causal delivery algorithm.
 */
#[allow(non_snake_case)]
pub struct GRAPH {
    G: ArrayMap<Node>,
    V: Vec<usize>,
    dot_to_index_map: HashMap<Dot, usize>,
    peer_number: usize,
    peer_index: usize,
    client: Sender<ClientMessage>,
    configuration: Arc<Configuration>,
}

#[allow(non_snake_case)]
impl GRAPH {
    /**
     * Builds a new GRAPH struct that implements the causal delivery algorithm.
     *
     * # Arguments
     *
     * `peer_index` - Local peer's globally unique id.
     *
     * `peer_number` - Number of peers in the group.
     *
     * `client` - Channel between the Middleware and the Peer that will be used to send delivered/stable messages to Peer.
     *
     * `configuration` - Middleware's configuration file.
     */
    pub fn new(
        peer_index: usize,
        peer_number: usize,
        client: Sender<ClientMessage>,
        configuration: Arc<Configuration>,
    ) -> GRAPH {
        let G: ArrayMap<Node> = ArrayMap::new(3 * peer_number);
        let dot_to_index_map: HashMap<Dot, usize> = HashMap::new();
        let V: Vec<usize> = vec![0; peer_number];

        GRAPH {
            G,
            V,
            dot_to_index_map,
            peer_number,
            peer_index,
            client,
            configuration,
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
        //Updating the this sender's version vector entry
        self.V[message.dot.id] = message.dot.counter;

        if self.configuration.track_causal_stability {
            //Calculating the message's predecessors indexes in the VecMap struct
            //that aren't causally stable
            let p_line: Vec<&Dot> = message
                .context
                .iter()
                .filter(|p| !Self::stable(self, &p))
                .collect();

            //Creating a new Node struct
            let mut new_node = Node::new(message.dot);
            //Calculating the bstr of the new message
            new_node.bits.grow(self.peer_number, true);
            //Setting the bit representing the local peer to true
            new_node.bits.set(message.dot.id, false);
            //Setting the node as delivered
            new_node.stage = Stage::DLV;

            //Adding the new node to the causal graph
            let new_graph_index = self.G.push(new_node);

            //Adding the node (dot, index) to the hashmap
            self.dot_to_index_map.insert(message.dot, new_graph_index);

            let mut predecessors_graph_indexes: Vec<usize> = Vec::new();

            //Iterating over the message's causal predecessors that aren't stable
            for p in p_line {
                let pred_graph_index: usize = *self.dot_to_index_map.get(p).unwrap();

                //Adding the predecessor's graph index to a Vec
                predecessors_graph_indexes.push(pred_graph_index);

                let temp_pred_node = &mut self.G[pred_graph_index];
                (*temp_pred_node).successors.push(new_graph_index);
            }

            //Setting the new node's with the predecessors graph indexes
            let temp_new_node = &mut self.G[new_graph_index];
            (*temp_new_node).predecessors = SmallVec::from(predecessors_graph_indexes);
            temp_new_node.payload = Some(message.payload);
            temp_new_node.context = Some(message.context);

            self.updatestability(self.peer_index, new_graph_index);
        }
    }

    /**
     * Handles a message received from a peer via broadcast.
     *
     * # Arguments
     *
     * `message` - Message received from a peer in the group.
     */
    pub fn receive(&mut self, message: Message) {
        //Comparing the peer's entry in the version vector to the message's dot counter
        if self.V[message.dot.id] < message.dot.counter {
            let received_message_index: usize;

            //Getting the message's index in the VecMap graph
            match self.dot_to_index_map.get(&message.dot) {
                Some(index) => received_message_index = *index,
                None => {
                    //If it doesn't exist create a node for it with stage SLT
                    let received_message_node = Node::new(message.dot);
                    //Adding the empty node to the causal graph
                    received_message_index = self.G.push(received_message_node);

                    //Adding the node's (dot, index) to the hashmap
                    self.dot_to_index_map
                        .insert(message.dot, received_message_index);
                }
            }

            //Checking the message's node stage
            if !(self.G[received_message_index].stage == Stage::RCV) {
                //Calculating the message's predecessors indexes in the VecMap struct
                //that aren't causally stable
                let p_line: Vec<&Dot> = message
                    .context
                    .iter()
                    .filter(|p| !Self::stable(self, &p))
                    .collect();

                //let mut predecessors_indexes = SmallVec::<[usize; 4]>::new();
                let smallvec_size = message.context.len();
                let mut predecessors_indexes = SmallVec::with_capacity(smallvec_size);

                //Creating and setting every position of the bstr to 0
                let mut b = BV::default();
                b.grow(self.peer_number, false);

                //Iterating over the message's causal predecessors that aren't stable
                for p in &p_line {
                    let pred_index: usize;

                    //Getting the predecessor's index in the VecMap graph
                    match self.dot_to_index_map.get(p) {
                        Some(graph_index) => {
                            pred_index = *graph_index;
                        }
                        None => {
                            //If it doesn't exist add an empty node with stage SLT
                            let p_dot = *p;
                            let pred_empty_node = Node::new(*p_dot);

                            //Adding the empty node to the causal graph
                            pred_index = self.G.push(pred_empty_node);

                            //Adding the predecessor node (dot, index) to the hashmap
                            self.dot_to_index_map.insert(*p_dot, pred_index);
                        }
                    }

                    //Push the received message's graph index to the predessor's sucessors vec
                    let pred_temp_node = &mut self.G[pred_index];
                    (*pred_temp_node).successors.push(received_message_index);

                    //Setting to 1 in the bstr if the predecessor hasn't stage DLV
                    if pred_temp_node.stage != Stage::DLV {
                        b.set(pred_temp_node.dot.id, true);
                    }

                    drop(pred_temp_node);

                    predecessors_indexes.push(pred_index);
                }

                let received_temp_node = &mut self.G[received_message_index];
                received_temp_node.bits = b;
                received_temp_node.stage = Stage::RCV;
                received_temp_node.payload = Some(message.payload);
                received_temp_node.context = Some(message.context);
                //Setting the predecessors graph indexes to the
                //received message's predecessors vec
                received_temp_node.predecessors = predecessors_indexes;

                //Checking if the message's bstr is 0
                if received_temp_node.bits.none() {
                    //Calling the deliver function
                    self.deliver(received_message_index);
                }
            }
        }
    }

    /**
     * Function that checks if a message is causally stable.
     *
     * A message is stable if the dot's counter is lower than the number in the version vector
     * and if the dot doesn't exist in the domain of the causal graph.
     */
    fn stable(&mut self, dot: &Dot) -> bool {
        match self.dot_to_index_map.get(dot) {
            Some(index) => dot.counter <= self.V[dot.id] && self.G[*index].stage == Stage::STB,
            None => dot.counter <= self.V[dot.id],
        }
    }

    /**
     * Function that delivers a message to the client.
     *
     * A message will be delivered when its predecessors have been delivered.
     */
    fn deliver(&mut self, msg_graph_index: usize) {
        let delivered_node = &mut self.G[msg_graph_index];

        // Building a Message struct to be sent
        let delivered_message = ClientMessage::Delivery {
            payload: delivered_node.payload.as_ref().unwrap().to_vec(),
            dot: delivered_node.dot,
            context: delivered_node.context.as_ref().unwrap().to_vec(),
        };

        // Writing the message to the Client channel
        self.client
            .send(delivered_message)
            .expect("ERROR: Failed to deliver a message to the Client");

        //let temp_node = &mut self.G[msg_graph_index];
        let delivered_dot = delivered_node.dot;

        let (j, n) = (delivered_dot.id, delivered_dot.counter);

        self.V[j] = n;

        if self.configuration.track_causal_stability {
            delivered_node.stage = Stage::DLV;

            let mut b = BV::default();
            b.grow(self.peer_number, true);
            delivered_node.bits = b;
            delivered_node.bits.set(self.peer_index, false);
            delivered_node.bits.set(j, false);
        }

        //Dropping the borrowing temp_node has on G before calling updatestability()
        drop(delivered_node);

        if self.configuration.track_causal_stability {
            //Updating the message's stability
            self.updatestability(j, msg_graph_index);
        }

        let successors_graph_indexes =
            unsafe { &*(&self.G[msg_graph_index].successors as *const _) };

        //Iterating over the message's sucessors
        for &s in successors_graph_indexes {
            let temp_successor_node: &mut Node = &mut self.G[s];

            //Setting the delivered message's entry in the bstr to 0
            temp_successor_node.bits.set(j, false);

            //Check if the sucessor can be delivered
            if temp_successor_node.bits.none() {
                self.deliver(s);
            }
        }

        if !self.configuration.track_causal_stability {
            let temp_node = &self.G[msg_graph_index];
            let temp_node_dot = temp_node.dot;

            self.deletestable(temp_node_dot);
        }
    }

    /**
     * Function that updates the causal stability of a message in the graph.
     */
    fn updatestability(&mut self, j: usize, msg_idx: usize) {
        let pred_idxs = unsafe { &*(&self.G[msg_idx].predecessors as *const _) };

        for &p in pred_idxs {
            let temp_pred_node: &mut Node = &mut self.G[p];

            if temp_pred_node.stage != Stage::STB && temp_pred_node.bits[j] {
                temp_pred_node.bits.set(j, false);

                if temp_pred_node.bits.none() {
                    self.stabilize(p);
                } else {
                    self.updatestability(j, p);
                }
            }
        }
    }

    fn stabilize(&mut self, msg_idx: usize) {
        let pred_idxs = unsafe { &*(&self.G[msg_idx].predecessors as *const _) };

        for &p in pred_idxs {
            let temp_predecessor_node: &mut Node = &mut self.G[p];

            if temp_predecessor_node.stage != Stage::STB {
                self.stabilize(p);
            }
        }

        let stable_node = &mut self.G[msg_idx];
        stable_node.stage = Stage::STB;

        let stable_msg = ClientMessage::Stable {
            dot: stable_node.dot,
        };

        //Sending STABLE message to client
        self.client
            .send(stable_msg)
            .expect("ERROR: Couldn't send a stable message to Client");

        drop(stable_node);
    }

    /**
     * Softly deletes an acked stable message by marking its position in the array available.
     *
     * # Arguments
     *
     * `dot` - Dot acked as stable by the Client.
     */
    pub fn deletestable(&mut self, dot: Dot) {
        let dot_graph_index = self.dot_to_index_map.get(&dot).unwrap();

        let successors_indexes = unsafe { &*(&self.G[*dot_graph_index].successors as *const _) };

        for &s in successors_indexes {
            let predecessor: &mut Node = &mut self.G[s];
            let predecessors_indexes = &mut predecessor.predecessors;
            predecessors_indexes.retain(|idx| idx != dot_graph_index);
        }

        self.G.remove(*dot_graph_index);
        self.dot_to_index_map.remove(&dot);
    }
}
