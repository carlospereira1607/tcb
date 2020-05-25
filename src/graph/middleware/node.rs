use super::dot::Dot;
use bit_vec::BitVec;
use smallvec::SmallVec;

type BV = BitVec<u64>;

/**
 * Stages that a node in the causal dependency graph can have.
 */
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Stage {
    ///Slot
    SLT,
    ///Received
    RCV,
    ///Delivered
    DLV,
    ///Stable
    STB,
}

/**
 * Struct of a node from the causal dependency graph.
 */
#[derive(Debug, Clone)]
pub struct Node {
    ///Message dot
    pub dot: Dot,
    ///Current stage
    pub stage: Stage,
    ///Bit string
    pub bits: BV,
    ///Serialized message payload
    pub payload: Option<Vec<u8>>,
    ///Message context
    pub context: Option<Vec<Dot>>,
    ///Indexes to the predecessors that are still in the graph
    pub predecessors: SmallVec<[usize; 4]>,
    ///Indexes to the successors that are still in the graph
    pub successors: SmallVec<[usize; 4]>,
}

impl Node {
    /**
     * Creates a new node with the passed dot, no payload, no context and stage SLT.
     *
     * # Arguments
     *
     * `dot` - Message dot
     */
    pub fn new(dot: Dot) -> Node {
        let predecessors = SmallVec::new();
        let successors = SmallVec::new();
        let bits = BV::default();

        Node {
            payload: None,
            dot,
            context: None,
            predecessors,
            successors,
            stage: Stage::SLT,
            bits,
        }
    }
}
