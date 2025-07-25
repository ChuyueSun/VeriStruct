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
    fn test_new_empty() {
        let set = VecSet::new();
        // Since nothing was inserted, contains should return false for any number.
        assert!(!set.contains(0));
        assert!(!set.contains(1));
        assert!(!set.contains(u64::MAX));
    }

    #[test]
    fn test_insert_and_contains() {
        let mut set = VecSet::new();
        set.insert(42);
        assert!(set.contains(42));
        // Test that an element not inserted returns false.
        assert!(!set.contains(41));
    }

    #[test]
    fn test_multiple_inserts() {
        let mut set = VecSet::new();
        let values = [3, 5, 7, 11];
        for &v in values.iter() {
            set.insert(v);
        }
        for &v in values.iter() {
            assert!(set.contains(v));
        }
        // Verify that a non-inserted value is not present.
        assert!(!set.contains(13));
    }

    #[test]
    fn test_duplicate_inserts() {
        let mut set = VecSet::new();
        // Inserting duplicate values. The current implementation doesn't prevent duplicates.
        set.insert(10);
        set.insert(10);
        // The contains function should return true.
        assert!(set.contains(10));
    }

    #[test]
    fn test_edge_cases() {
        let mut set = VecSet::new();
        // Test inserting and checking for edge values.
        set.insert(0);
        set.insert(u64::MAX);
        assert!(set.contains(0));
        assert!(set.contains(u64::MAX));
        // Test that a value near the upper boundary but not inserted returns false.
        assert!(!set.contains(u64::MAX - 1));
    }
}