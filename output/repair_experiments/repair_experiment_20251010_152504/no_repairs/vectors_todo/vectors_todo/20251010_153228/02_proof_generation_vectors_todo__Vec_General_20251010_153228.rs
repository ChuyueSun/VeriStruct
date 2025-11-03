/// This module provides basic vector algorithms with specifications suitable for formal verification.
///
/// - `binary_search`: Performs a binary search on a sorted vector to find the index of a given key. The vector must be sorted in ascending order and the key must be present in the vector.
/// - `reverse`: Reverses the elements of a vector in place, with postconditions about the resulting order.
/// - `binary_search_no_spinoff`: Variant of binary search with loop isolation disabled for verification purposes.

use vstd::prelude::*;

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
        v.len() > 0,
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|n: int| 0 <= n < v.len() && v[n] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            // Basic range constraints
            0 <= i1 <= i2 < v.len(),
            // Array is sorted (inherit from preconditions so the loop can use it)
            forall|x: int, y: int| 0 <= x <= y < v.len() ==> v[x] <= v[y],
            // The key k still lies in [i1, i2] somewhere
            exists|m: int| i1 <= m <= i2 && v[m] == k
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
        old(v).len() <= u64::MAX - 1 as usize,
    ensures
        v.len() == old(v).len(),
        forall|i: int| 0 <= i < v.len() ==> v[i] == old(v)[v.len() - i - 1],
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            0 <= n <= length / 2,
            // Region 1: [0..n) reversed
            forall|i: int|
                0 <= i < n ==> v[i] == v1[length - 1 - i],
            // Region 2: [n..(length-n)] untouched
            forall|i: int|
                n <= i < length - n ==> v[i] == v1[i],
            // Region 3: [length-n..length) reversed
            forall|i: int|
                length - n <= i < length ==> v[i] == v1[length - 1 - i]
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
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|n: int| 0 <= n < v.len() && v[n] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            // Basic range constraints
            0 <= i1 <= i2 < v.len(),
            // Array is sorted
            forall|x: int, y: int| 0 <= x <= y < v.len() ==> v[x] <= v[y],
            // The key k still lies in [i1, i2]
            exists|m: int| i1 <= m <= i2 && v[m] == k
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

} // verus!

// Step 2 (proof_generation) VEval Score: Compilation Error: False, Verified: 10, Errors: 0, Verus Errors: 0
// Verified: 10, Errors: 0, Verus Errors: 0
