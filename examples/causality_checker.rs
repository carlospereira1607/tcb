use tcb::causality_checker::petgraph::plot_graph;
use tcb::causality_checker::{
    causality_checker::check_causal_delivery,
    causality_checker_structs::{CausalCheck, CausalityChecker},
};
use tcb::graph::middleware::dot::Dot;

fn main() {
    //A group with 2 peers
    //The CausalCheck enum has variations for sent, delivered and stable messages.
    //A peer dot sequence is a vec of this type
    //The group's dot sequences is a vec with each peer's sequence
    //The entry in this vec corresponds the peer's id
    //Peer 0 is the first entry and peer 1 is the second

    //Building peer 0 dot sequence
    //The dot sequence:
    //Sent a message
    //Sent a message
    //Sent a message
    //Delivered a message
    //Delivered a message
    let one_peer_sequence = vec![
        CausalCheck::Send {
            sent_dot: Dot::new(0, 1),
            //If it's the graph approach, the message's context must be added
            context: Vec::new(),
        },
        CausalCheck::Send {
            sent_dot: Dot::new(0, 2),
            context: vec![Dot::new(0, 1)],
        },
        CausalCheck::Send {
            sent_dot: Dot::new(0, 3),
            context: vec![Dot::new(0, 2)],
        },
        CausalCheck::Delivery {
            dev_dot: Dot::new(1, 1),
        },
        CausalCheck::Delivery {
            dev_dot: Dot::new(1, 2),
        },
    ];

    //Building peer 1 dot sequence
    //The dot sequence:
    //Sent a message
    //Sent a message
    //Delivered a message
    //Delivered a message
    //Delivered a message
    let another_peer_sequence = vec![
        CausalCheck::Send {
            sent_dot: Dot::new(1, 1),
            context: Vec::new(),
        },
        CausalCheck::Send {
            sent_dot: Dot::new(1, 2),
            context: vec![Dot::new(1, 1)],
        },
        CausalCheck::Delivery {
            dev_dot: Dot::new(0, 1),
        },
        CausalCheck::Delivery {
            dev_dot: Dot::new(0, 2),
        },
        CausalCheck::Delivery {
            dev_dot: Dot::new(0, 3),
        },
    ];

    //Building the group's dot sequences
    let peer_dot_sequences = vec![one_peer_sequence, another_peer_sequence];

    //Calling the causality checker function
    match check_causal_delivery(2, peer_dot_sequences, true) {
        CausalityChecker::Ok(graph) => {
            //It's possible to write the graph used by the causality checker to a file,
            //so it can be visualized. Note that this graph will have all the sent messages
            //during broadcast and therefore there can easily be too many nodes in the graph
            //for the rendering to happen. Only use if the number of nodes in the graph
            //is relatively small. The filename for writing the graph into is passed from the
            //configuration file.
            plot_graph(graph, &format!("graph_filename"));
        }
        CausalityChecker::Error(error) => {
            //An error happened while traversing the dot sequences.
            //The state of the causality checker should be logged as to debug the problem.
            error.log_causal_check_error(format!("tcb_output/causal_error_output.txt"));
        }
    }
}
