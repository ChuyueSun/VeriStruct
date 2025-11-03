verus! {
    use vstd::prelude::*;
    use vstd::seq_lib::group_seq_properties;

    struct VecSet {
        vt: Vec<u64>,
    }

    impl VecSet {
        pub closed spec fn view(&self) -> Set<u64>
            ensures
                |res| res == Set::<u64>::new(|x: u64| exists|i: int| 0 <= i < self.vt@.len() && self.vt@[i] == x)
        {
            Set::new(|x: u64| exists|i: int| 0 <= i < self.vt@.len() && self.vt@[i] == x)
        }

        pub fn new() -> (s: Self)
            ensures
                s@ == set![]
        {
            VecSet { vt: Vec::new() }
        }

        pub fn insert(&mut self, v: u64)
            ensures
                self@ == old(self)@.insert(v)
        {
            proof {
                // The definition of self@ is determined by self.vt@,
                // so pushing 'v' onto self.vt corresponds to inserting 'v' into self@.
            }
            self.vt.push(v);
        }

        pub fn contains(&self, v: u64) -> (contained: bool)
            ensures
                contained == (v in self@)
        {
            for i in 0..self.vt.len()
                invariant
                    0 <= i <= self.vt@.len(),
                    self@ == old(self)@,
                    forall|j: int| 0 <= j < i ==> self.vt@[j] != v
            {
                if self.vt[i] == v {
                    return true;
                }
            }
            false
        }
    }

    /* TSET CODE BELOW */

    fn test(t: Vec<u64>)
    {
        let mut vs: VecSet = VecSet::new();
        assert(vs@ =~= set![]);
        vs.insert(3);
        vs.insert(5);
        let contains2 = vs.contains(2);
        assert(!contains2);
        let contains3 = vs.contains(3);
        assert(contains3);
        assert(vs@ =~= set![3, 5]);
    }

    pub fn main() {}
}
