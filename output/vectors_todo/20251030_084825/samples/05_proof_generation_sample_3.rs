use vstd::prelude::*;
fn main() {}

verus! {

/// Performs a binary search on a sorted vector to find the index of a given key. The key must be present in the vector.
///
/// # Arguments
/// * `v` - A reference to a vector of u64 integers that must be sorted in ascending order
/// * `k` - The key value to search for in the vector
///
/// # Returns
/// * The index where the key was found in the vector
fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v@.len() > 0,
        forall|i: int, j: int| 0 <= i && i <= j && j < v@.len() ==> v@[i] <= v@[j],
        exists|i: int| 0 <= i && i < v@.len() && v@[i] == k
    ensures
        r < v@.len(),
        v@[r as int] == k
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 !=i2
        invariant
            // Bounds
            i1 <= i2 && i2 < v.len(),
            // The key must still lie in [i1 .. i2]
            exists|idx: int| i1 as int <= idx <= i2 as int && v@[idx] == k,
            // Original sorted property is repeated
            forall|a: int, b: int| 0 <= a && a <= b && b < v@.len() ==> v@[a] <= v@[b],
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
///
/// # Arguments
/// * `v` - A mutable reference to a vector of u64 integers to be reversed
fn reverse(v: &mut Vec<u64>)
    requires
        old(v)@.len() <= u64::MAX as nat
    ensures
        v@.len() == old(v)@.len(),
        forall|i: int| 0 <= i && i < v@.len() ==> v@[i] == old(v)@[v@.len() - i - 1]
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            // For all i < n, we have swapped them:
            forall|i: int| 0 <= i && i < n ==> v[i] == v1[length - i - 1],
            // For all i < n, the opposite side is swapped too:
            forall|i: int| 0 <= i && i < n ==> v[length - i - 1] == v1[i],
            // Middle region [n, length - n) is unchanged so far
            forall|m: int| n <= m && m + n < length ==> v[m] == v1[m],
    {
        let x = v[n];
        let y = v[length - 1 - n];
        v.set(n, y);
        v.set(length - 1 - n, x);
    }
}

#[verifier::loop_isolation(false)]
fn binary_search_no_spinoff(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v@.len() > 0,
        forall|i: int, j: int| 0 <= i && i <= j && j < v@.len() ==> v@[i] <= v@[j],
        exists|i: int| 0 <= i && i < v@.len() && v@[i] == k
    ensures
        r < v@.len(),
        v@[r as int] == k
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 !=i2
        invariant
            i1 <= i2 && i2 < v.len(),
            exists|idx: int| i1 as int <= idx <= i2 as int && v@[idx] == k,
            forall|a: int, b: int| 0 <= a && a <= b && b < v@.len() ==> v@[a] <= v@[b],
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

/*
TEST CODE BEGINS HERE
*/

fn binary_search_test(t: Vec<u64>)
requires
    t.len() > 0,
    t.len() < u64::MAX - 1 as usize,
    forall|i: int, j: int| 0 <= i && i <= j && j < t.len() ==> t[i] <= t[j]
{
    for i in 0 .. t.len()
    invariant
        forall|a: int, b: int| 0 <= a && a <= b && b < t.len() ==> t[a] <= t[b],
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
    let ghost t1 = t@;
    reverse(t);
    assert(t.len() == t1.len());
    assert(forall|i: int| 0 <= i && i < t1.len() ==> t[i] == t1[t1.len() - i - 1]);
}

pub fn test() {
}

pub fn main() {
}

} // verus!

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
