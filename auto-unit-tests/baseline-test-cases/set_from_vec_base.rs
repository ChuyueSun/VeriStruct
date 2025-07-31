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

/* TEST CODE BELOW */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vecset() {
        let mut vs: VecSet = VecSet::new();
        assert!(vs.vt == vec![]);
        vs.insert(3);
        vs.insert(5);
        let contains2 = vs.contains(2);
        assert!(!contains2);
        let contains3 = vs.contains(3);
        assert!(contains3);
        assert!(vs.vt == vec![3, 5]);
    }
}

pub fn main() {}