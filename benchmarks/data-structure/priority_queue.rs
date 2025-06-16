use vstd::prelude::*;
use vstd::multiset::*;
use vstd::seq_lib::*;

verus! {

struct PriorityQueue {
    data: Vec<u32>,
}

impl View for PriorityQueue {
    type V = Multiset<u32>;

    closed spec fn view(&self) -> Self::V {
        self.data@.to_multiset()
    }
}

impl PriorityQueue {
    pub fn new() -> Self {
        PriorityQueue { data: Vec::new() }
    }

    pub closed spec fn ordered(&self, i: int) -> bool {
        self.data[i as int] >= self.data[((i - 1) / 2) as int]
    }

    pub closed spec fn well_formed(&self) -> bool {
        forall |i: int| {
            1 <= i && i < self.data.len() as int ==> #[trigger] self.ordered(i)
        }
    }

    pub closed spec fn almost_well_formed(&self, index: int) -> bool {
        forall |i: int| {
            1 <= i && i < (self.data.len() as int) && (i != index) ==> #[trigger] self.ordered(i)
        }
    }

    fn direct_push(&mut self, value: u32)
    requires
        old(self).well_formed(),
    ensures
        self@ == old(self)@.insert(value)
    {
        let mut new = vec![value];
        self.data.append(&mut new);
        proof {
            assert(self.data@ == old(self).data@ + seq![value]);
            lemma_multiset_commutative(old(self).data@, seq![value]);
            assert(seq![value].to_multiset() == Multiset::<u32>::singleton(value));
            assert(self@ == old(self)@.add(seq![value].to_multiset()));
        }
    }


    pub fn push(&mut self, value: u32) 
    requires
        old(self).well_formed(),
    ensures
        self.well_formed(),
        self@ == old(self)@.insert(value)
    {
        let ghost gstate = old(self)@.insert(value);
        self.direct_push(value);
        let mut index = self.data.len() - 1;
        while index > 0 
        invariant
            self@ =~= gstate,
            self.almost_well_formed(index as int),
            index < self.data.len() as int
        decreases
            index,
        {
            let parent_index = (index - 1) / 2;
            assert(parent_index < index);
            if self.data[index] < self.data[parent_index] {
                // Swap the current node with its parent
                let tmp = self.data[index];
                self.data[index] = self.data[parent_index];
                self.data[parent_index] = tmp;
                assert(self.ordered(index as int));
                index = parent_index;
            } else {
                break; // The heap property is satisfied
            }
        }
    }

}

}

fn main() {}