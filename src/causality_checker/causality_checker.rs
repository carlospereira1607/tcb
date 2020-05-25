use super::causality_checker_structs::*;
use crate::graph::middleware::dag::ArrayMap;
use crate::graph::middleware::dot::Dot;
use crate::vv::structs::version_vector::VersionVector;
use bit_vec::BitVec;
use std::collections::HashMap;
use std::usize;

/**
 * Starts the causality checker by using the group's dot sequences.
 *
 * # Arguments
 *
 * `peer_number` - group size
 *
 * `peer_dot_sequences` - sequences with the messages' dots
 *
 * `graph_implementation` - flag that if True the middleware used a graph implementation
 */
pub fn check_causal_delivery(
    peer_number: usize,
    peer_dot_sequences: Vec<Vec<CausalCheck>>,
    graph_implementation: bool,
) -> CausalityChecker {
    let mut global_causal_dag: ArrayMap<CheckNode> = ArrayMap::new(2 * peer_number);
    let mut dot_to_index_map: HashMap<Dot, usize> = HashMap::new();
    let mut peer_version_vectors: Vec<VersionVector> = Vec::with_capacity(peer_number);
    let mut dot_version_vector_map: HashMap<Dot, VersionVector> = HashMap::new();
    let mut peer_dot_sequence_indexes: Vec<usize> = Vec::with_capacity(peer_number);
    let mut peer_dot_sequence_prev_indexes: Vec<usize> = Vec::with_capacity(peer_number);
    let mut peer_version_matrices: Vec<VersionMatrix> = Vec::with_capacity(peer_number);

    for _ in 0..peer_number {
        peer_version_vectors.push(VersionVector::new(peer_number));
        peer_dot_sequence_indexes.push(0);
        peer_dot_sequence_prev_indexes.push(0);
        peer_version_matrices.push(VersionMatrix::new(peer_number));
    }

    for i in 0..peer_number {
        let initial_vec_dot_index = peer_dot_sequence_indexes[i];
        let current_peer_dot_sequence = peer_dot_sequences
            .get(i)
            .expect("ERROR: When getting the current peer dot sequence");

        for j in initial_vec_dot_index..current_peer_dot_sequence.len() {
            match current_peer_dot_sequence
                .get(j)
                .expect("ERROR: When getting the dot of current peer dot sequence")
            {
                CausalCheck::Send { sent_dot, context } => {
                    let current_peer_dot = sent_dot.clone();

                    if current_peer_dot.id != i {
                        return CausalityChecker::Error(CausalityCheckerError::new(
                            CausalityCheckerErrorEnum::Send,
                            "A Dot's id and a peer's id don't match!".to_string(),
                            global_causal_dag,
                            peer_dot_sequences,
                            dot_to_index_map,
                            peer_version_vectors,
                            dot_version_vector_map,
                            peer_dot_sequence_indexes,
                            peer_dot_sequence_prev_indexes,
                            current_peer_dot.clone(),
                            i,
                            j,
                        ));
                    }

                    if !handle_sender_delivered_message(
                        current_peer_dot,
                        &mut global_causal_dag,
                        &mut dot_to_index_map,
                        &mut peer_version_vectors,
                        &mut dot_version_vector_map,
                        &mut peer_dot_sequence_indexes,
                        &mut peer_dot_sequence_prev_indexes,
                        current_peer_dot_sequence,
                        &mut peer_version_matrices,
                        &context,
                        &graph_implementation,
                    ) {
                        return CausalityChecker::Error(CausalityCheckerError::new(
                            CausalityCheckerErrorEnum::Delivery,
                            "The Sender's Dot was already in the graph!".to_string(),
                            global_causal_dag,
                            peer_dot_sequences,
                            dot_to_index_map,
                            peer_version_vectors,
                            dot_version_vector_map,
                            peer_dot_sequence_indexes,
                            peer_dot_sequence_prev_indexes,
                            current_peer_dot.clone(),
                            i,
                            j,
                        ));
                    }
                }

                CausalCheck::Delivery { dev_dot } => {
                    let current_peer_dot = dev_dot.clone();

                    if !dot_to_index_map.contains_key(&current_peer_dot) {
                        let mut sender_bits = BitVec::from_elem(peer_number, false);
                        sender_bits.set(i, true);

                        match handle_peer_dot(
                            &current_peer_dot,
                            &peer_dot_sequences,
                            &mut global_causal_dag,
                            &mut dot_to_index_map,
                            &mut peer_version_vectors,
                            &mut dot_version_vector_map,
                            &mut peer_dot_sequence_indexes,
                            &mut peer_dot_sequence_prev_indexes,
                            &mut peer_version_matrices,
                            &mut sender_bits,
                            &graph_implementation,
                        ) {
                            HandlePeerDotCausalError::Ok => {}
                            HandlePeerDotCausalError::CausalDeliveryError {
                                message,
                                current_dot,
                                current_peer,
                                current_peer_dot_sequence_index,
                            } => {
                                return CausalityChecker::Error(CausalityCheckerError::new(
                                    CausalityCheckerErrorEnum::Delivery,
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
                                ));
                            }
                            HandlePeerDotCausalError::CausalStabilityError {
                                message,
                                current_dot,
                                current_peer,
                                current_peer_dot_sequence_index,
                            } => {
                                return CausalityChecker::Error(CausalityCheckerError::new(
                                    CausalityCheckerErrorEnum::Stability,
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
                                ));
                            }
                        }
                    }

                    match handle_peer_delivered_message(
                        i,
                        current_peer_dot,
                        &mut dot_version_vector_map,
                        &mut peer_version_vectors,
                        &mut peer_version_matrices,
                    ) {
                        true => {}
                        false => {
                            return CausalityChecker::Error(CausalityCheckerError::new(
                                CausalityCheckerErrorEnum::Stability,
                                format!(
                                    "When comparing VVs of peer {} and dot {:?}",
                                    i, current_peer_dot
                                ),
                                global_causal_dag,
                                peer_dot_sequences,
                                dot_to_index_map,
                                peer_version_vectors,
                                dot_version_vector_map,
                                peer_dot_sequence_indexes,
                                peer_dot_sequence_prev_indexes,
                                current_peer_dot.clone(),
                                i,
                                j,
                            ));
                        }
                    }
                }
                CausalCheck::Stable { stb_dot } => {
                    let current_peer_version_matrix = &peer_version_matrices[i];
                    match handle_stable_message(
                        &stb_dot,
                        current_peer_version_matrix,
                        &dot_version_vector_map,
                    ) {
                        true => {}
                        false => {
                            let current_dot = stb_dot.clone();
                            return CausalityChecker::Error(CausalityCheckerError::new(
                                CausalityCheckerErrorEnum::Stability,
                                "".to_string(),
                                global_causal_dag,
                                peer_dot_sequences,
                                dot_to_index_map,
                                peer_version_vectors,
                                dot_version_vector_map,
                                peer_dot_sequence_indexes,
                                peer_dot_sequence_prev_indexes,
                                current_dot,
                                i,
                                j,
                            ));
                        }
                    }
                }
            }

            peer_dot_sequence_indexes[i] += 1;
        }
    }

    CausalityChecker::Ok(global_causal_dag)
}

fn handle_peer_dot(
    dot: &Dot,
    peer_dot_sequences: &Vec<Vec<CausalCheck>>,
    global_causal_dag: &mut ArrayMap<CheckNode>,
    dot_to_index_map: &mut HashMap<Dot, usize>,
    peer_version_vectors: &mut Vec<VersionVector>,
    dot_version_vector_map: &mut HashMap<Dot, VersionVector>,
    peer_dot_sequence_indexes: &mut Vec<usize>,
    peer_dot_sequence_prev_indexes: &mut Vec<usize>,
    peer_version_matrices: &mut Vec<VersionMatrix>,
    sender_bits: &mut BitVec,
    graph_implementation: &bool,
) -> HandlePeerDotCausalError {
    let initial_vec_dot_index = peer_dot_sequence_indexes[dot.id];
    let current_peer_dot_sequence = &peer_dot_sequences[dot.id];

    for j in initial_vec_dot_index..current_peer_dot_sequence.len() {
        match current_peer_dot_sequence
            .get(j)
            .expect("ERROR: When getting the dot current peer dot sequence")
        {
            CausalCheck::Send { sent_dot, context } => {
                let current_peer_dot = sent_dot.clone();

                if current_peer_dot.id != dot.id {
                    return HandlePeerDotCausalError::CausalDeliveryError {
                        message: "handle_peer_dot() - A Dot's id and a peer's id don't match!"
                            .to_string(),
                        current_dot: current_peer_dot,
                        current_peer: dot.id,
                        current_peer_dot_sequence_index: j,
                    };
                }

                if !handle_sender_delivered_message(
                    current_peer_dot,
                    global_causal_dag,
                    dot_to_index_map,
                    peer_version_vectors,
                    dot_version_vector_map,
                    peer_dot_sequence_indexes,
                    peer_dot_sequence_prev_indexes,
                    current_peer_dot_sequence,
                    peer_version_matrices,
                    &context,
                    graph_implementation,
                ) {
                    return HandlePeerDotCausalError::CausalDeliveryError {
                        message: "handle_peer_dot() - The Sender's Dot was already in the graph!"
                            .to_string(),
                        current_dot: current_peer_dot,
                        current_peer: dot.id,
                        current_peer_dot_sequence_index: j,
                    };
                }

                if current_peer_dot == *dot {
                    peer_dot_sequence_indexes[dot.id] += 1;
                    return HandlePeerDotCausalError::Ok;
                }
            }

            CausalCheck::Delivery { dev_dot } => {
                let current_peer_dot = dev_dot.clone();

                if !dot_to_index_map.contains_key(&current_peer_dot) {
                    if sender_bits.get(current_peer_dot.id).unwrap() {
                        return HandlePeerDotCausalError::CausalDeliveryError {
                            message: format!("Repeated calling of sender {}", current_peer_dot.id)
                                .to_string(),
                            current_dot: current_peer_dot.clone(),
                            current_peer: dot.id,
                            current_peer_dot_sequence_index: j,
                        };
                    } else {
                        sender_bits.set(current_peer_dot.id, true);
                    }

                    match handle_peer_dot(
                        &current_peer_dot,
                        peer_dot_sequences,
                        global_causal_dag,
                        dot_to_index_map,
                        peer_version_vectors,
                        dot_version_vector_map,
                        peer_dot_sequence_indexes,
                        peer_dot_sequence_prev_indexes,
                        peer_version_matrices,
                        sender_bits,
                        graph_implementation,
                    ) {
                        HandlePeerDotCausalError::Ok => {}
                        HandlePeerDotCausalError::CausalDeliveryError {
                            message,
                            current_dot,
                            current_peer,
                            current_peer_dot_sequence_index,
                        } => {
                            return HandlePeerDotCausalError::CausalDeliveryError {
                                message: message,
                                current_dot: current_dot,
                                current_peer: current_peer,
                                current_peer_dot_sequence_index: current_peer_dot_sequence_index,
                            };
                        }
                        HandlePeerDotCausalError::CausalStabilityError {
                            message,
                            current_dot,
                            current_peer,
                            current_peer_dot_sequence_index,
                        } => {
                            return HandlePeerDotCausalError::CausalStabilityError {
                                message: message,
                                current_dot: current_dot,
                                current_peer: current_peer,
                                current_peer_dot_sequence_index: current_peer_dot_sequence_index,
                            };
                        }
                    }
                }

                match handle_peer_delivered_message(
                    dot.id,
                    current_peer_dot,
                    dot_version_vector_map,
                    peer_version_vectors,
                    peer_version_matrices,
                ) {
                    true => {}
                    false => {
                        return HandlePeerDotCausalError::CausalDeliveryError {
                            message: format!(
                                "handle_peer_dot - When comparing VVs of peer {} and dot {:?}",
                                dot.id, current_peer_dot
                            ),
                            current_dot: current_peer_dot,
                            current_peer: dot.id,
                            current_peer_dot_sequence_index: j,
                        };
                    }
                }
            }

            CausalCheck::Stable { stb_dot } => {
                let current_peer_version_matrix = &peer_version_matrices[dot.id];

                match handle_stable_message(
                    &stb_dot,
                    current_peer_version_matrix,
                    dot_version_vector_map,
                ) {
                    true => {}
                    false => {
                        return HandlePeerDotCausalError::CausalStabilityError {
                            message: "".to_string(),
                            current_dot: *stb_dot,
                            current_peer: dot.id,
                            current_peer_dot_sequence_index: j,
                        };
                    }
                }
            }
        }

        peer_dot_sequence_indexes[dot.id] += 1;
    }

    HandlePeerDotCausalError::Ok
}

fn handle_sender_delivered_message(
    current_peer_dot: Dot,
    global_causal_dag: &mut ArrayMap<CheckNode>,
    dot_to_index_map: &mut HashMap<Dot, usize>,
    peer_version_vectors: &mut Vec<VersionVector>,
    dot_version_vector_map: &mut HashMap<Dot, VersionVector>,
    peer_dot_sequence_indexes: &mut Vec<usize>,
    peer_dot_sequence_prev_indexes: &mut Vec<usize>,
    current_peer_dot_sequence: &Vec<CausalCheck>,
    peer_version_matrices: &mut Vec<VersionMatrix>,
    context: &Vec<Dot>,
    graph_implementation: &bool,
) -> bool {
    if !dot_to_index_map.contains_key(&current_peer_dot) {
        let peer_version_vector = peer_version_vectors
            .get_mut(current_peer_dot.id)
            .expect("ERROR: When getting the mutable peer version vector");
        let node = CheckNode::new(current_peer_dot.clone());
        let new_graph_index = global_causal_dag.push(node);
        dot_to_index_map.insert(current_peer_dot.clone(), new_graph_index);

        (*peer_version_vector)[current_peer_dot.id] += 1;

        let dot_version_vector = peer_version_vector.clone();
        let dot_version_vector_clone = dot_version_vector.clone();

        dot_version_vector_map.insert(current_peer_dot.clone(), dot_version_vector);

        update_graph_dependencies(
            global_causal_dag,
            dot_to_index_map,
            dot_version_vector_map,
            current_peer_dot_sequence,
            &current_peer_dot,
            peer_dot_sequence_indexes[current_peer_dot.id],
            peer_dot_sequence_prev_indexes[current_peer_dot.id],
            context,
            graph_implementation,
        );

        peer_dot_sequence_prev_indexes[current_peer_dot.id] =
            peer_dot_sequence_indexes[current_peer_dot.id];

        let peer_version_matrix = &mut peer_version_matrices[current_peer_dot.id];
        peer_version_matrix.update_peer_entry(current_peer_dot.id, dot_version_vector_clone);

        true
    } else {
        false
    }
}

fn handle_peer_delivered_message(
    i: usize,
    current_peer_dot: Dot,
    dot_version_vector_map: &mut HashMap<Dot, VersionVector>,
    peer_version_vectors: &mut Vec<VersionVector>,
    peer_version_matrices: &mut Vec<VersionMatrix>,
) -> bool {
    match dot_version_vector_map.get(&current_peer_dot) {
        Some(dot_version_vector) => {
            let peer_version_vector = peer_version_vectors
                .get_mut(i)
                .expect("ERROR: When getting the mutable peer version vector");

            match VersionVector::compare_version_vectors(
                current_peer_dot.id,
                &peer_version_vector,
                &dot_version_vector,
            ) {
                true => {
                    (*peer_version_vector)[current_peer_dot.id] += 1;

                    let peer_version_matrix = &mut peer_version_matrices[i];
                    peer_version_matrix
                        .update_peer_entry(current_peer_dot.id, dot_version_vector.clone());

                    peer_version_matrix.update_peer_entry(i, (*peer_version_vector).clone());

                    true
                }
                false => false,
            }
        }
        None => false,
    }
}

fn handle_stable_message(
    dot: &Dot,
    peer_version_matrix: &VersionMatrix,
    dot_version_vector_map: &HashMap<Dot, VersionVector>,
) -> bool {
    let stable_dot_version_vector = dot_version_vector_map
        .get(dot)
        .expect("ERROR: When getting the stable dot version vector");
    peer_version_matrix.check_stability(stable_dot_version_vector)
}

fn update_graph_dependencies(
    global_causal_dag: &mut ArrayMap<CheckNode>,
    dot_to_index_map: &mut HashMap<Dot, usize>,
    dot_version_vector_map: &mut HashMap<Dot, VersionVector>,
    current_peer_dot_sequence: &Vec<CausalCheck>,
    dot: &Dot,
    current_sequence_index: usize,
    previous_sequence_index: usize,
    context: &Vec<Dot>,
    graph_implementation: &bool,
) {
    if previous_sequence_index < current_sequence_index {
        let predecessors_indexes: Vec<usize>;

        if current_sequence_index > 0 && dot.counter == 1 {
            let dot_version_vector = dot_version_vector_map
                .get(&dot)
                .expect("ERROR: When getting dot's version vector");

            let previous_dot_version_vector = VersionVector::new(dot_version_vector.len());
            let previous_dot = Dot::new(dot.id, 0);

            predecessors_indexes = compare_dot_version_vectors(
                &previous_dot,
                dot,
                &previous_dot_version_vector,
                dot_version_vector,
                dot_to_index_map,
                global_causal_dag,
            );
        } else {
            let previous_dot = CausalCheck::get_dot(
                current_peer_dot_sequence
                    .get(previous_sequence_index)
                    .expect("ERROR: When getting the previous dot"),
            );
            let previous_dot_version_vector = dot_version_vector_map
                .get(&previous_dot)
                .expect("ERROR: When getting the previous dot's version vector");

            let dot_version_vector = dot_version_vector_map
                .get(&dot)
                .expect("ERROR: When getting the current dot's version vector");

            predecessors_indexes = compare_dot_version_vectors(
                &previous_dot,
                dot,
                previous_dot_version_vector,
                dot_version_vector,
                dot_to_index_map,
                global_causal_dag,
            );
        }

        let dot_graph_index = dot_to_index_map
            .get(dot)
            .expect("ERROR: When getting the dot's causal graph index");

        let mut counter: usize;

        if *graph_implementation {
            counter = context.len();
        } else {
            counter = 0;
        }

        for predecessor_graph_index in predecessors_indexes {
            let pred_dot = &mut global_causal_dag[predecessor_graph_index];
            pred_dot.successors.push(*dot_graph_index);

            if *graph_implementation {
                if context.contains(&pred_dot.dot) {
                    counter -= 1;
                }
            }

            drop(pred_dot);

            let dot_node = &mut global_causal_dag[*dot_graph_index];
            dot_node.predecessors.push(predecessor_graph_index);
        }

        assert!(counter == 0, "ERROR when calculating dot's context");
    } else {
        if previous_sequence_index != current_sequence_index {
            panic!("ERROR: Previous sequence index is not less that the current sequence index");
        }
    }
}

fn compare_dot_version_vectors(
    lower_dot: &Dot,
    upper_dot: &Dot,
    lower_dot_version_vector: &VersionVector,
    upper_dot_version_vector: &VersionVector,
    dot_to_index_map: &mut HashMap<Dot, usize>,
    global_causal_dag: &mut ArrayMap<CheckNode>,
) -> Vec<usize> {
    let mut predecessor_dot_graph_indexes: Vec<usize> = Vec::new();
    let mut predecessors_dots: Vec<Dot> = Vec::new();

    for i in 0..lower_dot_version_vector.len() {
        if lower_dot_version_vector[i] < upper_dot_version_vector[i] {
            if i == lower_dot.id && i == upper_dot.id && lower_dot.counter != 0 {
                let dot = Dot::new(i, lower_dot_version_vector[i]);
                predecessors_dots.push(dot);
            } else {
                if i != upper_dot.id {
                    let dot = Dot::new(i, upper_dot_version_vector[i]);
                    predecessors_dots.push(dot);
                }
            }
        }
    }

    let mut dependency_flag: bool;

    for predecessor_dot in &predecessors_dots {
        dependency_flag = false;
        let predecessor_graph_index = dot_to_index_map
            .get(predecessor_dot)
            .expect("ERROR: When getting the predecessor dot's causal graph index");

        for i in 0..predecessors_dots.len() {
            let temp_predecessor_dot = predecessors_dots[i];

            if *predecessor_dot != temp_predecessor_dot {
                let temp_predecessor_graph_index = dot_to_index_map
                    .get(&temp_predecessor_dot)
                    .expect("ERROR: When getting the temp predecessor dot's causal graph index");

                let temp_predecessor_node = &global_causal_dag[*temp_predecessor_graph_index];

                if temp_predecessor_node
                    .predecessors
                    .contains(&predecessor_graph_index)
                {
                    dependency_flag = true;
                    break;
                }
            }
        }

        if !dependency_flag {
            predecessor_dot_graph_indexes.push(*predecessor_graph_index);
        }
    }

    predecessor_dot_graph_indexes
}
