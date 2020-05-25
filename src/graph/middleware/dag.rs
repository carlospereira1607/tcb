use std::ops::{Deref, DerefMut};

/**
 * Struct of a directed acyclic graph mapped as an array.
 */
#[derive(Debug, Clone)]
pub struct ArrayMap<T> {
    ///Array with the nodes
    nodes: Vec<T>,
    ///Array with the available indexes
    available_indexes: Vec<usize>,
}

impl<T> ArrayMap<T> {
    /**
     * Builds a new directed acyclic graph represented as an array.
     *
     * # Arguments
     *
     * `initial_capacity` - Initial capacity of the array.
     *  
     * */
    pub fn new(initial_capacity: usize) -> ArrayMap<T> {
        let nodes: Vec<T> = Vec::with_capacity(initial_capacity);
        let mut available_indexes: Vec<usize> = Vec::with_capacity(initial_capacity);

        for i in 0..initial_capacity {
            available_indexes.push(initial_capacity - 1 - i);
        }

        ArrayMap {
            nodes,
            available_indexes,
        }
    }

    /**
     * Adds a new node to the graph vector and returns its index.
     *
     * # Arguments
     *
     * `node` - New node to add to the graph.
     * */
    pub fn push(&mut self, node: T) -> usize {
        match self.available_indexes.pop() {
            Some(index) => match self.nodes.get(index) {
                Some(_) => {
                    //Its a node softly deleted
                    self.nodes[index] = node;
                    index
                }
                None => {
                    //The vec has len() < capacity()
                    //This means that there aren't nodes softly deleted
                    //The new node has to be added to the back of the vec
                    self.nodes.push(node);
                    index
                }
            },
            None => {
                //There aren't available nodes in the current vec
                //Push to the back so more memory is allocated
                self.nodes.push(node);
                self.nodes.len() - 1
            }
        }
    }

    /**
     * Softly deletes a node of the graph vector.
     *
     * # Arguments
     *
     * `index` - Softly deletes a node from the graph by adding its position to the available indexes array.
     * */
    pub fn remove(&mut self, index: usize) {
        let _ = self
            .nodes
            .get(index)
            .expect("ERROR: Was expecting a node in this position");
        self.available_indexes.push(index);
    }

    /**
     * Returns the number of nodes in the graph.
     * */
    pub fn node_number(&self) -> usize {
        self.nodes.len()
    }
}

impl<T> Deref for ArrayMap<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Vec<T> {
        &self.nodes
    }
}

impl<T> DerefMut for ArrayMap<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.nodes
    }
}
