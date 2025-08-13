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
    fn test_new_set_is_empty() {
        let set = VecSet::new();
        // The set should not contain any value.
        assert!(!set.contains(1));
        assert!(!set.contains(0));
    }

    #[test]
    fn test_insert_single_value() {
        let mut set = VecSet::new();
        set.insert(10);
        // After insertion, the set should contain the inserted value.
        assert!(set.contains(10));
        // It should not contain other values.
        assert!(!set.contains(20));
    }

    #[test]
    fn test_insert_duplicate_values() {
        let mut set = VecSet::new();
        // Insert a duplicate value.
        set.insert(5);
        set.insert(5);
        // The set should report the value as present.
        assert!(set.contains(5));
    }

    #[test]
    fn test_multiple_values() {
        let mut set = VecSet::new();
        let values = [3, 7, 9, 15, 23];
        for &v in values.iter() {
            set.insert(v);
        }
        // All inserted values should be present.
        for &v in values.iter() {
            assert!(set.contains(v));
        }
        // Value not inserted should not be found.
        assert!(!set.contains(100));
    }

    #[test]
    fn test_edge_values() {
        let mut set = VecSet::new();
        // Insert u64 minimum and maximum values.
        set.insert(0);
        set.insert(u64::MAX);
        assert!(set.contains(0));
        assert!(set.contains(u64::MAX));
        // A value that wasn't inserted should not be found.
        assert!(!set.contains(1));
    }

    #[test]
    fn test_contains_before_and_after_insertion() {
        let mut set = VecSet::new();
        // Calling contains on an empty set.
        assert!(!set.contains(50));
        // After insertion, contains should return true.
        set.insert(50);
        assert!(set.contains(50));
    }

    #[test]
    fn test_large_number_of_inserts() {
        let mut set = VecSet::new();
        // Insert a large number of values.
        for i in 0..1000 {
            set.insert(i);
        }
        // Verify all inserted values are present.
        for i in 0..1000 {
            assert!(set.contains(i));
        }
        // Verify a value not inserted is absent.
        assert!(!set.contains(1001));
    }
}