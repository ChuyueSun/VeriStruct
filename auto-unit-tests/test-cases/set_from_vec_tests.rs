struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub fn new() -> Self {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64) {
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> bool {
        for i in 0..self.vt.len() {
            if self.vt[i] == v {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vecset_is_empty() {
        let vs = VecSet::new();
        // Ensure that an empty set does not contain any element.
        assert!(!vs.contains(0));
        assert!(!vs.contains(42));
    }

    #[test]
    fn test_insert_single_value() {
        let mut vs = VecSet::new();
        vs.insert(42);
        // Check that the inserted value is contained.
        assert!(vs.contains(42));
        // Check that a value which was not inserted returns false.
        assert!(!vs.contains(100));
    }

    #[test]
    fn test_insert_multiple_values() {
        let mut vs = VecSet::new();
        let values = [10, 20, 30, 40];
        for &v in values.iter() {
            vs.insert(v);
        }
        // Assert all inserted values are found.
        for &v in values.iter() {
            assert!(vs.contains(v));
        }
        // Check that a non-inserted value returns false.
        assert!(!vs.contains(50));
    }

    #[test]
    fn test_duplicate_inserts() {
        let mut vs = VecSet::new();
        // Insert the same value multiple times.
        vs.insert(99);
        vs.insert(99);
        vs.insert(99);
        // The set should still report the value as contained.
        assert!(vs.contains(99));
    }

    #[test]
    fn test_edge_values() {
        let mut vs = VecSet::new();
        // Insert the minimum value for u64.
        vs.insert(0);
        assert!(vs.contains(0));
        // Insert the maximum value for u64.
        vs.insert(u64::MAX);
        assert!(vs.contains(u64::MAX));
    }
}