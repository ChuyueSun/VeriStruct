use vstd::prelude::*;

verus! {

/// This module provides basic vector algorithms with specifications suitable for formal verification.
/// 
/// - `binary_search`: Performs a binary search on a sorted vector to find the index of a given key. The vector must be sorted in ascending order and the key must be present in the vector.
/// - `reverse`: Reverses the elements of a vector in place, with postconditions about the resulting order.
/// - `binary_search_no_spinoff`: Variant of binary search with loop isolation disabled for verification purposes.

fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|i: int| 0 <= i < v.len() && k == v[i],
    ensures
        r < v.len(),
        k == v[r as int],
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            i2 < v.len(),
            exists|i: int| i1 <= i <= i2 && k == v[i],
            forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        decreases i2 - i1,
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

fn reverse(v: &mut Vec<u64>)
    ensures
        v.len() == old(v).len(),
        forall|i: int| 0 <= i < old(v).len() ==> v[i] == old(v)[old(v).len() - i - 1],
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            forall|i: int| 0 <= i < n ==> v[i] == v1[length - i - 1],
            forall|i: int| 0 <= i < n ==> v1[i] == v[length - i - 1],
            forall|i: int| n <= i && i + n < length ==> #[trigger] v[i] == v1[i],
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
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|i: int| 0 <= i < v.len() && k == v[i],
    ensures
        r < v.len(),
        k == v[r as int],
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            i2 < v.len(),
            exists|i: int| i1 <= i <= i2 && k == v[i],
        decreases i2 - i1,
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

/* TEST CODE BELOW */
pub fn main() {
    // Test binary_search on a sorted vector.
    let v: Vec<u64> = vec![1, 2, 3, 4, 5];
    let len = v.len();
    for n in 0..len {
        let key = v[n];
        let idx = binary_search(&v, key);
        assert(idx < len);
        assert(v[idx] == key);
    }

    // Test reverse by checking that reversing yields the reversed order.
    let mut v1: Vec<u64> = vec![1, 2, 3, 4, 5];
    let old_v1 = v1@;  // ghost copy of the original vector
    reverse(&mut v1);
    let len1 = v1.len();
    let new_v1 = v1@;
    assert(new_v1.len() == old_v1.len());
    assert(forall|i: int| 0 <= i < (len1 as int) ==> new_v1[i] == old_v1[len1 as int - i - 1]);

    // Test binary_search_no_spinoff on another sorted vector.
    let v2: Vec<u64> = vec![10, 20, 30, 40, 50, 60];
    let len2 = v2.len();
    for n in 0..len2 {
        let key = v2[n];
        let idx = binary_search_no_spinoff(&v2, key);
        assert(idx < len2);
        assert(v2[idx] == key);
    }

    // Additional test: binary_search on a single-element vector.
    let v3: Vec<u64> = vec![42];
    let idx3 = binary_search(&v3, 42);
    assert(idx3 < v3.len());
    assert(v3[idx3] == 42);

    // Additional reverse test on a different vector.
    let mut v4: Vec<u64> = vec![7, 8, 9, 10];
    let old_v4 = v4@;
    reverse(&mut v4);
    let len4 = v4.len();
    assert(v4@.len() == old_v4.len());
    assert(forall|i: int| 0 <= i < (len4 as int) ==> v4@[i] == old_v4[len4 as int - i - 1]);
}
} // verus!