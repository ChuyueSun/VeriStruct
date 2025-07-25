use vstd::prelude::*;

verus! {

spec fn sorted(v: seq<u64>) -> bool {
    forall(|i: nat, j: nat| (i < j && j < v.len()) ==> v.index(i) <= v.index(j))
}

pub fn binary_search(v: &Vec<u64>, k: u64) -> (i: usize)
    requires
        v.len() > 0,
        sorted(v@),
    ensures
        i < v.len(),
        (forall |j: usize| j < i ==> v[j] < k),
        (i < v.len() ==> v[i] >= k),
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while (i1 != i2)
        invariant
            0 <= i1 && i1 <= i2,
            i2 < v.len(),
            (forall |j: usize| j < i1 ==> v[j] < k),
            (forall |j: usize| i2 <= j && j < v.len() ==> v[j] >= k),
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

pub fn reverse(v: &mut Vec<u64>)
    ensures
        forall(|i: nat| i < v.len() ==> v.per_index(i) == old(v).per_index(old(v).len() - 1 - i)),
{
    let length = v.len();
    let L = length; // capture old length
    let mut n: usize = 0;
    while (n < (length / 2))
        invariant
            n <= length / 2,
            (forall |i: usize| i < n ==> v[i] == old(v).per_index(L - 1 - i)),
            (forall |i: usize| i < n ==> v.per_index(L - 1 - i) == old(v)[i]),
            (forall |i: usize| n <= i && i < L - n ==> v[i] == old(v)[i]),
        decreases (length / 2) - n
    {
        let x = v[n];
        let y = v[L - 1 - n];
        v[n] = y;
        v[L - 1 - n] = x;
        n = n + 1;
    }
}

pub fn binary_search_no_spinoff(v: &Vec<u64>, k: u64) -> (i: usize)
    requires
        v.len() > 0,
        sorted(v@),
    ensures
        i < v.len(),
        (forall |j: usize| j < i ==> v[j] < k),
        (i < v.len() ==> v[i] >= k),
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while (i1 != i2)
        invariant
            0 <= i1 && i1 <= i2,
            i2 < v.len(),
            (forall |j: usize| j < i1 ==> v[j] < k),
            (forall |j: usize| i2 <= j && j < v.len() ==> v[j] >= k),
        decreases i2 - i1
    {
        let d = i2 - i1;
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
    // test_binary_search_found
    {
        let v = vec![1, 3, 5, 7, 9];
        // Test when the element is exactly present.
        assert(binary_search(&v, 1) == 0);
        assert(binary_search(&v, 3) == 1);
        assert(binary_search(&v, 5) == 2);
        assert(binary_search(&v, 7) == 3);
        assert(binary_search(&v, 9) == 4);
    }

    // test_binary_search_not_found
    {
        let v = vec![1, 3, 5, 7, 9];
        // When the target is not present, binary_search returns the index of the first element not less than the target.
        // For k = 4, the first element >= 4 is 5 at index 2.
        assert(binary_search(&v, 4) == 2);
        // For k = 0, it should return index 0.
        assert(binary_search(&v, 0) == 0);
        // For k = 8, it should return index 4 because 9 is the first element >= 8.
        assert(binary_search(&v, 8) == 4);
    }

    // test_binary_search_empty is omitted because the precondition v.len() > 0 is required.

    // test_reverse_empty
    {
        let mut v: Vec<u64> = vec![];
        reverse(&mut v);
        assert(v.len() == 0);
    }

    // test_reverse_single
    {
        let mut v = vec![42];
        reverse(&mut v);
        assert(v[0] == 42);
    }

    // test_reverse_even
    {
        let mut v = vec![1, 2, 3, 4];
        reverse(&mut v);
        assert(v[0] == 4);
        assert(v[1] == 3);
        assert(v[2] == 2);
        assert(v[3] == 1);
    }

    // test_reverse_odd
    {
        let mut v = vec![1, 2, 3, 4, 5];
        reverse(&mut v);
        assert(v[0] == 5);
        assert(v[1] == 4);
        assert(v[2] == 3);
        assert(v[3] == 2);
        assert(v[4] == 1);
    }

    // test_binary_search_no_spinoff_found
    {
        let v = vec![1, 3, 5, 7, 9];
        assert(binary_search_no_spinoff(&v, 1) == 0);
        assert(binary_search_no_spinoff(&v, 3) == 1);
        assert(binary_search_no_spinoff(&v, 5) == 2);
        assert(binary_search_no_spinoff(&v, 7) == 3);
        assert(binary_search_no_spinoff(&v, 9) == 4);
    }

    // test_binary_search_no_spinoff_not_found
    {
        let v = vec![1, 3, 5, 7, 9];
        assert(binary_search_no_spinoff(&v, 4) == 2);
        assert(binary_search_no_spinoff(&v, 0) == 0);
        assert(binary_search_no_spinoff(&v, 8) == 4);
    }

    // test_binary_search_with_duplicates
    {
        let v = vec![1, 2, 2, 3, 3, 5];
        // For duplicate elements, binary_search should return the first index at which the element appears.
        assert(binary_search(&v, 2) == 1);
        assert(binary_search_no_spinoff(&v, 2) == 1);
        // Similarly, for 3.
        assert(binary_search(&v, 3) == 3);
        assert(binary_search_no_spinoff(&v, 3) == 3);
    }
}

}