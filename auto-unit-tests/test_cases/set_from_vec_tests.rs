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
    fn test_new_returns_empty_set() {
        let set = VecSet::new();
        // Since no elements are inserted, any number should not be contained
        assert!(!set.contains(1));
        assert!(!set.contains(0));
        assert!(!set.contains(u64::MAX));
    }

    #[test]
    fn test_insert_and_contains_single_element() {
        let mut set = VecSet::new();
        set.insert(5);
        assert!(set.contains(5));
        assert!(!set.contains(10));
    }

    #[test]
    fn test_insert_duplicates() {
        let mut set = VecSet::new();
        set.insert(10);
        set.insert(10);
        // Even though the number was inserted twice, contains should return true
        assert!(set.contains(10));
    }

    #[test]
    fn test_multiple_inserts() {
        let mut set = VecSet::new();
        let numbers = [1, 2, 3, 4, 5];
        for &num in &numbers {
            set.insert(num);
        }
        for &num in &numbers {
            assert!(set.contains(num));
        }
        // Check for an element that was not inserted
        assert!(!set.contains(6));
    }

    #[test]
    fn test_edge_values() {
        let mut set = VecSet::new();
        // Insert boundary values
        set.insert(0);
        set.insert(u64::MAX);
        assert!(set.contains(0));
        assert!(set.contains(u64::MAX));
        // Check elements around the edge values to ensure they are not erroneously contained
        assert!(!set.contains(1));
        if u64::MAX > 0 {
            assert!(!set.contains(u64::MAX - 1));
        }
    }
}