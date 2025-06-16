use vstd::prelude::*;
use vstd::multiset::Multiset;

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


    pub closed spec fn well_formed(&self) -> bool {
        forall |i: int| {
            0 <= i && i < self.data.len() as int ==> #[trigger] self.data[i as int] >= self.data[(i / 2) as int]
        }
    }

    pub fn push(&mut self, value: u32) 
    requires
        old(self).well_formed(),
    ensures
        self.well_formed(),
        self@ == old(self)@.insert(value)
    {
        self.data.push(value);
        let len = self.data.len();
        let mut index = self.data.len() - 1;
        while index > 0 
        invariant
            forall |i: int|
                0 <= i && i < (len as int) && (i != index) ==> 
                    #[trigger] self.data[i as int] >= self.data[(i / 2) as int],
        decreases
            index,
        {
            let parent_index = index / 2;
            if self.data[index] < self.data[parent_index] {
                // Swap the current node with its parent
                let tmp = self.data[index];
                self.data[index] = self.data[parent_index];
                self.data[parent_index] = tmp;
                index = parent_index;
            } else {
                break; // The heap property is satisfied
            }
        }
    }

}

}

fn main() {}