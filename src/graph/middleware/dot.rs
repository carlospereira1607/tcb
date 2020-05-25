use std::fmt;

/**
 * A dot is a pair id and counter, which are, respectivally, the peer's
 * globally unique identifier and a monotonically increasing counter that
 * grows with each sent message.
 */
#[derive(Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Dot {
    ///Peer's globally unique id
    pub id: usize,
    ///Message's counter
    pub counter: usize,
}

impl Dot {
    /**
     * Creates a new Dot.
     *
     * # Arguments
     *
     * `id` - Peer's globally unique id
     *
     * `counter` - Message's counter
     */
    pub fn new(id: usize, counter: usize) -> Dot {
        Dot {
            id: id,
            counter: counter,
        }
    }
}

impl fmt::Display for Dot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.id, self.counter)
    }
}
