// Top-level docstring can go here if desired

use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {
/*
 This lemma is often useful before a vector-remove call, and it can be useful to prove what element is contained in a vector.
 The parameters to this lemma function should match the executable code after it.
 Do NOT pass `old(..)' to this lemma as parameter.
 Example usage:
    proof{
	lemma_vec_remove(vec@, index);
    }
    vec.remove(index);
 */
proof fn lemma_vec_remove<T>(vec: Seq<T>, i: int)
    requires
        0 <= i < vec.len(),
    ensures
        forall |k: int| 0 <= k < i ==> #[trigger] vec[k] == vec.remove(i)[k],
        forall |k: int| i < k  < vec.len() ==> #[trigger] vec[k] ==  vec.remove(i)[k-1],
{

}

/*
 This lemma is often useful before a vector-push call, and it can be useful to prove what element is contained in a vector.
 Example usage:
    proof{
	lemma_vec_push(vec@, value, vec.len());
    }
    vec.push(value);
 */
proof fn lemma_vec_push<T>(vec: Seq<T>, i: T, l: usize)
    requires
        l == vec.len(),
    ensures
        forall |k: int| 0 <= k < vec.len() ==> #[trigger] vec[k] == vec.push(i)[k],
        vec.push(i).index(l as int) == i,
{
}

/*
 This lemma is often useful before a vector-remove call, and it can be useful to prove what element is contained in a vector.
 The parameters to this lemma function should match the executable code after it.
 Do NOT pass `old(..)' to this lemma as parameter.
 Example usage:
    proof{
	lemma_vec_remove(vec@, index);
    }
    vec.remove(index);
 */
proof fn lemma_vec_remove_2 < T>(vec: Seq<T>, i: int) // Changed function name to fix syntax error
    requires
        0 <= i < vec.len(),
    ensures
        forall |k: int| 0 <= k < i ==> #[trigger] vec[k] == vec.remove(i)[k],
        forall |k: int| i < k  < vec.len() ==> #[trigger] vec[k] ==  vec.remove(i)[k-1],
{

}

/*
 This lemma is often useful before a vector-push call, and it can be useful to prove what element is contained in a vector.
 Example usage:
    proof{
	lemma_vec_push(vec@, value, vec.len());
    }
    vec.push(value);
 */
// Renamed function from lemma_vec_push to lemma_vec_push_2 to fix the syntax error.
proof fn lemma_vec_push_2<T>(vec: Seq<T>, i: T, l: usize)
    requires
        l == vec.len(),
    ensures
        forall |k: int| 0 <= k < vec.len() ==> #[trigger] vec[k] == vec.push(i)[k],
        vec.push(i).index(l as int) == i,
{
}


struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// The logical View of this structure is a set of all elements stored in `vt`.
    /// Because duplicates in the Vec do not affect membership in the set,
    /// we represent the entire collection as:
    ///
    ///   Set::new(|x: u64| exists i: int {
    ///       0 <= i && i < self.vt@.len() && self.vt@[i] == x
    ///   })
    ///
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64| exists|i: int| 0 <= i && i < self.vt@.len() && self.vt@[i] == x )
    }

    /// Creates a new, empty VecSet.
    /// ensures the resulting set is empty
    pub fn new() -> (s: Self)
        ensures
            s@ =~= set![]
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts a value into the set.
    /// ensures the resulting set is the old set plus `v`
    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.insert(v)
    {
        self.vt.push(v);
        proof {
            // Prove the resulting set is old(self)@ plus v
            broadcast use group_seq_properties;

            // Forward direction:
            assert forall|x: u64| #[trigger] self@.contains(x)
                implies old(self).view().insert(v).contains(x) by {
                // If x is in self@, then there exists an index i < self.vt@.len() s.t. self.vt@[i] == x.
                // If i < old(self).vt@.len(), then x was already in old(self).view().
                // Otherwise, i == old(self).vt@.len() and hence x == v.
            };

            // Backward direction:
            assert forall|x: u64| #[trigger] old(self).view().insert(v).contains(x)
                implies self@.contains(x) by {
                // If x is in old(self).view().insert(v), then either x is in old(self).view() or x == v.
                // If x == v, then it's stored at the new index i = old(self).vt@.len() in self.vt.
                // Otherwise x was in old(self).view(), so there's an i < old(self).vt@.len() with vt@[i] == x.
            };
        }
    }

    /// Returns true if `v` is in the set, false otherwise.
    /// ensures contained == self@.contains(v)
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in 0..self.vt.len()
            invariant
                i <= self.vt.len(),
                // Keep track that we haven't found `v` before index i
                forall|j: int| 0 <= j && j < i ==> self.vt@[j] != v,
            decreases (self.vt.len() - i)
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

} // verus!

// VEval Score: Compilation Error: False, Verified: 9, Errors: 1, Verus Errors: 1
