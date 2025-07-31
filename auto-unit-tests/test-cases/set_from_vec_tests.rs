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
    fn test_new_contains_none() {
        let set = VecSet::new();
        // Test that the new set does not contain arbitrary elements
        assert_eq!(set.contains(0), false);
        assert_eq!(set.contains(42), false);
        assert_eq!(set.contains(u64::MAX), false);
    }

    #[test]
    fn test_insert_single() {
        let mut set = VecSet::new();
        set.insert(10);
        assert!(set.contains(10));
        // Ensure no false positives
        assert!(!set.contains(0));
        assert!(!set.contains(20));
    }

    #[test]
    fn test_insert_multiple() {
        let mut set = VecSet::new();
        let values = [1, 2, 3, 100, 999];
        for &v in &values {
            set.insert(v);
        }
        for &v in &values {
            assert!(set.contains(v));
        }
        // Test an element not inserted
        assert!(!set.contains(0));
        assert!(!set.contains(500));
    }

    #[test]
    fn test_insert_duplicates() {
        let mut set = VecSet::new();
        set.insert(50);
        set.insert(50);
        // Regardless of duplicate, contains should be true
        assert!(set.contains(50));
        // Still false for non-inserted value
        assert!(!set.contains(51));
    }

    #[test]
    fn test_edge_values() {
        let mut set = VecSet::new();
        // Test with edge values
        set.insert(0);
        set.insert(u64::MAX);
        assert!(set.contains(0));
        assert!(set.contains(u64::MAX));
        // Test a value that was not inserted
        assert!(!set.contains(1));
    }
}