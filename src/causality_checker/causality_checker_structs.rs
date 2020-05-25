use crate::graph::middleware::dag::ArrayMap;
use crate::graph::middleware::dot::Dot;
use crate::vv::structs::version_vector::VersionVector;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::Write;
use std::io::BufWriter;

/**
 * Enum for the type of dots in the peer sequences.
 */
#[derive(Debug)]
pub enum CausalCheck {
    ///Sent message
    Send { sent_dot: Dot, context: Vec<Dot> },
    ///Delivered message
    Delivery { dev_dot: Dot },
    ///Stable message
    Stable { stb_dot: Dot },
}

impl CausalCheck {
    /**
     * Returns the inner dot from the CausalCheck enum's variations.
     *
     * # Arguments
     *
     * * `entry` - CausalCheck enum to retrieve the dot from.
     */
    pub fn get_dot(entry: &CausalCheck) -> Dot {
        match entry {
            CausalCheck::Send { sent_dot, .. } => sent_dot.clone(),
            CausalCheck::Delivery { dev_dot } => dev_dot.clone(),
            CausalCheck::Stable { stb_dot } => stb_dot.clone(),
        }
    }
}

/**
 * Enum with the results for the causality checker.
 */
#[derive(Debug)]
pub enum CausalityChecker {
    ///Causal delivery and stability of all messages was correct.
    Ok(ArrayMap<CheckNode>),
    ///An error was thrown while traversing the dot sequences.
    Error(CausalityCheckerError),
}

/**
 * Enum with type of causality checker errors thrown while traversing the dot sequences.
 */
#[derive(Debug)]
pub enum CausalityCheckerErrorEnum {
    ///Send error.
    Send,
    ///Causal delivery error.
    Delivery,
    ///Causal stability error.
    Stability,
}

impl fmt::Display for CausalityCheckerErrorEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CausalityCheckerErrorEnum::Send => write!(f, "Send"),
            CausalityCheckerErrorEnum::Delivery => write!(f, "Delivery"),
            CausalityCheckerErrorEnum::Stability => write!(f, "Stability"),
        }
    }
}

/**
 * State of the causality checker when the error was detected.
 */
#[derive(Debug)]
pub struct CausalityCheckerError {
    ///Type of the causality checker error
    error_type: CausalityCheckerErrorEnum,
    ///Message detailing the cause of the error.
    message: String,
    ///Graph built by the causality checker while traversing the dot sequences.
    global_causal_dag: ArrayMap<CheckNode>,
    ///Dot sequences of the group's peers.
    peer_dot_sequences: Vec<Vec<CausalCheck>>,
    ///Struct that maps a dot to its index in the causal dependency graph mapped as an array.
    dot_to_index_map: HashMap<Dot, usize>,
    ///Version vector of each peer.
    peer_version_vectors: Vec<VersionVector>,
    ///Structs with the version vectors of each dot.
    dot_version_vector_map: HashMap<Dot, VersionVector>,
    ///Vector with each peer's current dot sequence index when the error was thrown.
    peer_dot_sequence_indexes: Vec<usize>,
    ///Vector with each peer's previous sent message dot sequence index when the error was thrown.
    peer_dot_sequence_prev_indexes: Vec<usize>,
    ///Dot where the error was thrown.
    current_dot: Dot,
    ///Peer where the error was thrown.
    current_peer: usize,
    ///Index in the peer's dot sequence where the error was thrown.
    current_peer_dot_sequence_index: usize,
}

impl CausalityCheckerError {
    /**
     * Builds a new CausalityCheckerError wrapper.
     *
     * # Arguments
     *
     * `error_type` - Type of the causality checker error.
     *
     * `message` - Message detailing the cause of the error.
     *
     * `global_causal_dag` - Graph built by the causality checker while traversing the dot sequences.
     *
     * `peer_dot_sequences` - Dot sequences of the group's peers.
     *
     * `dot_to_index_map` - Struct that maps a dot to its index in the causal dependency graph mapped as an array.
     *
     * `peer_version_vectors` - Version vector of each peer.
     *
     * `dot_version_vector_map` - Structs with the version vectors of each dot.
     *
     * `peer_dot_sequence_indexes` - Vector with each peer's current dot sequence index when the error was thrown.
     *
     * `peer_dot_sequence_prev_indexes` - Vector with each peer's previous sent message dot sequence index when the error was thrown.
     *
     * `current_dot` - Dot where the error was thrown.
     *
     * `current_peer` - Peer where the error was thrown.
     *
     * `current_peer_dot_sequence_index` - Index in the peer's dot sequence where the error was thrown.
     */
    pub fn new(
        error_type: CausalityCheckerErrorEnum,
        message: String,
        global_causal_dag: ArrayMap<CheckNode>,
        peer_dot_sequences: Vec<Vec<CausalCheck>>,
        dot_to_index_map: HashMap<Dot, usize>,
        peer_version_vectors: Vec<VersionVector>,
        dot_version_vector_map: HashMap<Dot, VersionVector>,
        peer_dot_sequence_indexes: Vec<usize>,
        peer_dot_sequence_prev_indexes: Vec<usize>,
        current_dot: Dot,
        current_peer: usize,
        current_peer_dot_sequence_index: usize,
    ) -> Self {
        Self {
            error_type,
            message,
            global_causal_dag,
            peer_dot_sequences,
            dot_to_index_map,
            peer_version_vectors,
            dot_version_vector_map,
            peer_dot_sequence_indexes,
            peer_dot_sequence_prev_indexes,
            current_dot,
            current_peer,
            current_peer_dot_sequence_index,
        }
    }

    /**
     * Logs the causality checker error in a readable format and into multiple files.
     *
     * # Arguments
     *
     * `output_base_file_path` - Path to the base file in the output directory.
     */
    pub fn log_causal_check_error(self, output_base_file_path: String) {
        println!("Message {}\n\n", self.message);
        println!("Error type {}\n\n", self.error_type);

        let file = File::create(output_base_file_path).unwrap();
        let mut file_buffer = BufWriter::new(file);

        file_buffer
            .write(
                format!(
                    "{:?} | Peer {} | Seq Index {} | Error Type {}\n",
                    self.current_dot,
                    self.current_peer,
                    self.current_peer_dot_sequence_index,
                    self.error_type
                )
                .as_bytes(),
            )
            .unwrap();
        file_buffer
            .write(format!("--------------------------\n").as_bytes())
            .unwrap();
        file_buffer
            .write(format!("Sequence Indexes\n\t{:?}\n", self.peer_dot_sequence_indexes).as_bytes())
            .unwrap();
        file_buffer
            .write(format!("--------------------------\n").as_bytes())
            .unwrap();

        file_buffer
            .write(
                format!(
                    "Sequence Prev Indexes\n\t{:?}\n",
                    self.peer_dot_sequence_prev_indexes
                )
                .as_bytes(),
            )
            .unwrap();

        file_buffer
            .write(format!("--------------------------\n").as_bytes())
            .unwrap();

        for i in 0..self.peer_version_vectors.len() {
            file_buffer
                .write(format!("Peer {} VV:\n\t{:?}\n", i, self.peer_version_vectors[i]).as_bytes())
                .unwrap();
        }

        file_buffer
            .write(format!("--------------------------\n").as_bytes())
            .unwrap();

        for i in 0..self.peer_dot_sequences.len() {
            let sequence_file =
                File::create(format!("tcb_output/causal_error_peer_sequence{}.txt", i)).unwrap();
            let mut sequence_file_buffer = BufWriter::new(sequence_file);
            sequence_file_buffer
                .write(format!("Peer Dot Seq {}\n", i).as_bytes())
                .unwrap();
            for j in 0..self.peer_dot_sequences[i].len() {
                sequence_file_buffer
                    .write(format!("\n\t{} - {:?}\n", j, self.peer_dot_sequences[i][j]).as_bytes())
                    .unwrap();
            }

            sequence_file_buffer.flush().unwrap();
        }

        file_buffer
            .write(format!("--------------------------\n").as_bytes())
            .unwrap();

        file_buffer
            .write(format!("\t Dot Version Vector\n").as_bytes())
            .unwrap();
        for (dot, version_vector) in self.dot_version_vector_map.iter() {
            let line = format!("\t{:?} - {:?}\n", dot, version_vector);
            file_buffer.write_all(line.as_bytes()).unwrap();
        }

        file_buffer
            .write(format!("--------------------------\n").as_bytes())
            .unwrap();

        file_buffer
            .write(format!("\t Dot to Index Map\n").as_bytes())
            .unwrap();
        for (dot, index) in self.dot_to_index_map.iter() {
            let line = format!("\t{:?} - {}\n", dot, index);
            file_buffer.write_all(line.as_bytes()).unwrap();
        }

        file_buffer
            .write(format!("--------------------------\n").as_bytes())
            .unwrap();

        file_buffer
            .write(format!("\t Causal Graph\n").as_bytes())
            .unwrap();

        for i in 0..self.global_causal_dag.len() {
            let temp_node = &self.global_causal_dag[i];
            let line = format!(
                "{} - Dot({}, {})\n\tPred - {:?}\n\tSucc - {:?}\n",
                i,
                temp_node.dot.id,
                temp_node.dot.counter,
                temp_node.predecessors,
                temp_node.successors
            );

            file_buffer.write_all(line.as_bytes()).unwrap();
        }

        file_buffer.flush().unwrap();
    }
}

/**
 * Auxiliary eum for errors that occur during the recursive call.
 */
pub enum HandlePeerDotCausalError {
    ///No error was thrown during the recursive call.
    Ok,
    ///A delivery error was thrown during the recursive call.
    CausalDeliveryError {
        message: String,
        current_dot: Dot,
        current_peer: usize,
        current_peer_dot_sequence_index: usize,
    },
    ///A stability error was thrown during the recursive call.
    CausalStabilityError {
        message: String,
        current_dot: Dot,
        current_peer: usize,
        current_peer_dot_sequence_index: usize,
    },
}

/**
 * Matrix where each row is a peer's version vector. This is used to determine causal stability.
 */
#[derive(Debug)]
pub struct VersionMatrix {
    pub matrix: Vec<VersionVector>,
}

impl VersionMatrix {
    /**
     * Creates a new version matrix that is NxN where N is the group size.
     * Each row will be a peer's version vector, where the peer's id is the
     * row number.
     *
     * # Arguments
     *
     * `peer_number` - Number of peers in the group.
     */
    pub fn new(peer_number: usize) -> VersionMatrix {
        let mut matrix: Vec<VersionVector> = Vec::with_capacity(peer_number);

        for _ in 0..peer_number {
            matrix.push(VersionVector::new(peer_number));
        }

        VersionMatrix { matrix: matrix }
    }

    /**
     * Compares the causal stability in each row to a peer's version vector.
     *
     * # Arguments
     *
     * `dot_version_vector` - Peer's version vector to check stability.
     */
    pub fn check_stability(&self, dot_version_vector: &VersionVector) -> bool {
        for i in 0..self.matrix.len() {
            if !VersionVector::cmp(&self.matrix[i], dot_version_vector) {
                return false;
            }
        }
        true
    }

    /**
     * Updates a peer's row in the version vector matrix.
     *
     * # Arguments
     *
     * `peer_id` - Peer's id that will be its row in the matrix.
     * `dot_version_vector` - Peer's new version vector.
     */
    pub fn update_peer_entry(&mut self, peer_id: usize, dot_version_vector: VersionVector) {
        self.matrix[peer_id] = dot_version_vector;
    }
}

/**
 * Node of the causal graph built while looping through the dot sequences.
 */
#[derive(Debug)]
pub struct CheckNode {
    ///Message's dot
    pub dot: Dot,
    ///Predecessor indexes
    pub predecessors: SmallVec<[usize; 4]>,
    ///Successors indexes
    pub successors: SmallVec<[usize; 4]>,
}

impl CheckNode {
    /**
     * Creates a new node of the causal graph.
     *
     * # Arguments
     *
     * `dot` - The message's dot.
     */
    pub fn new(dot: Dot) -> CheckNode {
        let predecessors = SmallVec::<[usize; 4]>::new();
        let successors = SmallVec::<[usize; 4]>::new();

        CheckNode {
            dot: dot,
            predecessors: predecessors,
            successors: successors,
        }
    }
}
