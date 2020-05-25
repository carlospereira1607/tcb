use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

/**
 * Version vector struct.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionVector(pub Vec<usize>);

impl VersionVector {
    /**
     * Creates an empty version vector.
     */
    pub fn empty() -> VersionVector {
        let vec: Vec<usize> = Vec::new();

        VersionVector(vec)
    }
    /**
     * Creates an empty version vector.
     *
     * # Arguments
     *
     * `length` - Length of the version vector to create.
     */
    pub fn new(length: usize) -> VersionVector {
        VersionVector(vec![0; length])
    }

    /**
     * Appends a value to the version vector.
     *
     * # Arguments
     *
     * `value` - Value to append to the version vector.
     */
    pub fn push(&mut self, value: usize) {
        self.0.push(value);
    }

    /**
     * Checks if every entry in the first version vector is equal
     * or greater to the same entry in the second version vector.
     *
     * # Arguments
     *
     * `a` - Greater version vector.
     *
     * `b` - Smaller version vector.
     */
    pub fn cmp(a: &VersionVector, b: &VersionVector) -> bool {
        let mut ret = true;

        for i in 0..a.len() {
            ret = ret && (a[i] >= b[i]);

            if !ret {
                break;
            }
        }

        ret
    }

    /**
     * Compares if a version vector is greater or equal to another.
     *
     * # Arguments
     *
     * `index` - Position in the greater version vector that should be -1 to the value in the other.
     *
     * `a` - Greater version vector
     *
     * `b` - Smaller version vector
     */
    pub fn compare_version_vectors(index: usize, a: &VersionVector, b: &VersionVector) -> bool {
        let mut ret = true;

        for i in 0..a.0.len() {
            if i != index {
                ret = ret && (b[i] <= a[i]);
            } else {
                ret = ret && (b[i] == a[i] + 1);
            }

            if !ret {
                return false;
            }
        }

        ret
    }

    /**
     * Checks if two version vectors are equal.
     *
     * # Arguments
     *
     * `b` - Version vector to compare to.
     */
    pub fn equal(&self, b: &VersionVector) -> bool {
        let compare = self.0.cmp(&b.0);
        compare == Ordering::Equal
    }

    /**
     * Returns the different values between two version vectors.
     *
     * # Arguments
     *
     * `greater` - Greater version vector
     *
     * `lesser` - Smaller version vector
     *
     * `length` - Length of the version vectors
     */
    pub fn dif(
        greater: &VersionVector,
        lesser: &VersionVector,
        length: usize,
    ) -> Vec<(usize, usize)> {
        let mut dots = Vec::new();

        for i in 0..length {
            if greater[i] - lesser[i] > 0 {
                for cntr in lesser[i] + 1..greater[i] + 1 {
                    dots.push((i, cntr));
                }
            }
        }

        dots
    }
}

impl Deref for VersionVector {
    type Target = Vec<usize>;

    fn deref(&self) -> &Vec<usize> {
        &self.0
    }
}

impl DerefMut for VersionVector {
    fn deref_mut(&mut self) -> &mut Vec<usize> {
        &mut self.0
    }
}
