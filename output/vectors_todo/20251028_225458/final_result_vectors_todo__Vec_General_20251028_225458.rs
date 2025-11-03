use vstd::prelude::*;

fn main() {}

verus! {

/// Performs a binary search on a sorted vector to find the index of a given key. The key must be present in the vector
fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v@.len() > 0,
        forall|i: int, j: int| 0 <= i && i <= j < v@.len() ==> v@[i] <= v@[j],
        exists|idx: int| 0 <= idx && idx < v@.len() && v@[idx] == k
    ensures
        r < v@.len(),
        v@[r as int] == k
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 && i1 <= i2 < v.len(),
            exists|idx: int| i1 as int <= idx <= i2 as int && v@[idx] == k,
            forall|x: int, y: int| 0 <= x && x <= y < v@.len() ==> v@[x] <= v@[y],
        decreases i2 - i1
    {
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
    }
    i1
}

/// Reverses the elements of a vector in place.
fn reverse(v: &mut Vec<u64>)
    requires
        old(v)@.len() < u64::MAX - 1
    ensures
        v@.len() == old(v)@.len(),
        forall|i: int|
            0 <= i && i < old(v)@.len() ==>
            v@[i] == old(v)@[old(v)@.len() - i - 1]
{
    let length = v.len();
    let ghost v_old = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            forall|i: int|
                0 <= i && i < n ==>
                #[trigger] v@[i] == v_old[length as int - i - 1]
                && v@[length as int - i - 1] == v_old[i],
            forall|i: int| n <= i && i < length - n ==> v@[i] == v_old[i],
    {
        let x = v[n];
        let y = v[length - 1 - n];
        v.set(n, y);
        v.set(length - 1 - n, x);
    }
    proof {
        assert(forall|i: int|
            0 <= i && i < length ==>
            v@[i] == v_old[length - i - 1]
        ) by {
            // case distinction on i < length/2 or i >= length/2
        }
    }
}

#[verifier::loop_isolation(false)]
fn binary_search_no_spinoff(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v@.len() > 0,
        forall|i: int, j: int| 0 <= i && i <= j < v@.len() ==> v@[i] <= v@[j],
        exists|idx: int| 0 <= idx && idx < v@.len() && v@[idx] == k
    ensures
        r < v@.len(),
        v@[r as int] == k
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 && i1 <= i2 < v.len(),
            exists|idx: int| i1 as int <= idx <= i2 as int && v@[idx] == k,
            forall|x: int, y: int| 0 <= x && x <= y < v@.len() ==> v@[x] <= v@[y],
        decreases i2 - i1
    {
        let ghost d = i2 - i1;
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
        assert(i2 - i1 < d);
    }
    i1
}

fn binary_search_test(t: Vec<u64>)
requires
    t.len() > 0,
    t.len() < u64::MAX - 1 as usize,
    forall|i: int, j: int| 0 <= i && i <= j < t.len() ==> t[i] <= t[j]
{
    for i in 0 .. t.len()
    invariant
        forall|x: int, y: int| 0 <= x && x <= y < t.len() ==> t[x] <= t[y],
    {
        let k = t[i];
        let r = binary_search(&t, k);
        assert(r < t.len());
        assert(t[r as int] == k);
        let r = binary_search_no_spinoff(&t, k);
        assert(r < t.len());
        assert(t[r as int] == k);
    }
}

fn reverse_test(t: &mut Vec<u64>)
requires
    old(t).len() > 0,
    old(t).len() < u64::MAX - 1 as usize
{
    let ghost t_old = t@;
    reverse(t);
    assert(t.len() == t_old.len());
    assert(forall|i: int| 0 <= i && i < t_old.len() ==> t@[i] == t_old[t_old.len() - i - 1]);
}

pub fn test() {
}

pub fn main() {
}

}

// Final VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
