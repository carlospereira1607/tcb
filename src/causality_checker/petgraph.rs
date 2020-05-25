use super::causality_checker_structs::CheckNode;
use crate::graph::middleware::dag::ArrayMap;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::fs::File;
use std::io::Write;

/**
 * Writes to a file the graph built by the causality checker using petgraph format.
 * This file can be visualized by oppening it in a program that can read this format.
 * The graph from the checker is returned from the check_causal_delivery function call.
 *
 * # Arguments
 *
 * `dag` - Graph built by the causality checker.
 *
 * `filename` - Filename to write the output into.
 */
pub fn plot_graph(dag: ArrayMap<CheckNode>, filename: &String) {
    let mut graph = Graph::<_, ()>::new();
    let nmbr_nodes = dag.node_number();

    for i in 0..nmbr_nodes {
        let node = format!("({}, {})", dag[i].dot.id, dag[i].dot.counter);
        graph.add_node(node);
    }

    for i in 0..nmbr_nodes {
        for succ in &dag[i].successors {
            graph.add_edge(NodeIndex::new(i), NodeIndex::new(*succ), ());
        }
    }

    let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    let output = format!("{:?}", dot);
    let mut file = File::create(filename.clone()).unwrap();

    write!(file, "{}", output).unwrap();
}
