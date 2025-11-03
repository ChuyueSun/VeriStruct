use vstd::prelude::*;

verus! {

/// This module provides basic vector algorithms with specifications suitable for formal verification.
///
/// - `binary_search`: Performs a binary search on a sorted vector to find the index of a given key. The vector must be sorted in ascending order and the key must be present in the vector.
/// - `reverse`: Reverses the elements of a vector in place, with postconditions about the resulting order.
/// - `binary_search_no_spinoff`: Variant of binary search with loop isolation disabled for verification purposes.

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
        v.len() > 0,
        v.len() < u64::MAX - 1 as usize,
        forall|i: int, j: int|
            0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|ix: int|
            0 <= ix < v.len() && v[ix] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            // Precondition properties repeated:
            forall|p: int, q: int| 0 <= p <= q < v.len() ==> v[p] <= v[q],
            // Bounds on the loop variables:
            i1 <= i2,
            i2 < v.len(),
            // Key is in [i1, i2]:
            exists|m: int| i1 <= m <= i2 && v[m] == k,
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
        old(v).len() > 0,
        old(v).len() < u64::MAX - 1 as usize,
    ensures
        v@.len() == old(v)@.len(),
        forall|i: int|
            0 <= i < v@.len() ==> v@[i] == old(v)@[old(v)@.len() - 1 - i],
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            // Preserve the length of v:
            length == v.len(),
            // Swapped region [0 .. n):
            forall|i: int| 0 <= i < n ==> v[i] == v1[length - 1 - i],
            forall|i: int| 0 <= i < n ==> v[length - 1 - i] == v1[i],
            // Unswapped region remains equal to old(v):
            forall|k: int| n <= k < length - n ==> v[k] == v1[k],
        decreases (length / 2) - n
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
        v.len() > 0,
        v.len() < u64::MAX - 1 as usize,
        forall|i: int, j: int|
            0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|ix: int|
            0 <= ix < v.len() && v[ix] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            // Precondition properties repeated:
            forall|p: int, q: int| 0 <= p <= q < v.len() ==> v[p] <= v[q],
            // Bounds on loop variables:
            i1 <= i2,
            i2 < v.len(),
            // Key is within [i1, i2]:
            exists|m: int| i1 <= m <= i2 && v[m] == k,
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

pub fn test() {
}

pub fn main() {
}
