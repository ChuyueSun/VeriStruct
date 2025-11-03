use vstd::prelude::*;

verus! {

/// Performs a binary search on a sorted vector to find the index of a given key. The key must be present in the vector.
fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v@.len() > 0,
        forall|i: int, j: int| (0 <= i && i <= j && j < v@.len()) ==> v@[i] <= v@[j],
        exists|i: int| (0 <= i && i < v@.len()) && v@[i] == k,
    ensures
        r < v@.len(),
        v@[(r as int)] == k
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 as int,
            (i2 as int) < v@.len(),
            (i1 as int) <= (i2 as int),
            exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k,
        decreases (i2 as int - i1 as int)
    {
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            proof {
                // Let w be a witness in the current interval.
                let w = choose|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k;
                // If w were <= ix then, by sortedness, v@[w] <= v@[ix] < k, contradicting v@[w] == k.
                if w <= ix as int {
                    assert(v@[w] <= v@[ix]);
                    assert(v@[ix] < k);
                    assert(v@[w] < k);
                    // contradiction
                    assert(false);
                }
            }
            i1 = ix + 1;
            proof {
                // After updating i1, the previous witness remains valid.
                let w = choose|i: int| ( (ix + 1) as int) <= i <= (i2 as int) && v@[i] == k;
                // By the above proof, such a w exists.
                assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
            }
        } else {
            proof {
                // In this branch v[ix] >= k.
                // Let w be a witness in the current interval.
                let w = choose|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k;
                // If w were > ix then, by sortedness, v@[ix] <= v@[w] = k.
                // But then v@[ix] == k, so ix itself is a valid witness.
                if w > ix as int {
                    // Replace witness with ix.
                    // (Since v@[ix] >= k and v@[w] == k, sortedness forces v@[ix] == k.)
                    // Thus, we may assume a witness exists at ix.
                    assert(v@[ix] == k);
                }
            }
            i2 = ix;
            proof {
                // Now ix is a valid witness in the new interval.
                assert(v@[ix] == k);
                assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
            }
        }
        proof {
            // Re-establish the invariant.
            assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
        }
    }
    proof {
        // At loop exit, i1 == i2, so the invariant gives some witness at index i1.
        assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
        assert(v@[(i1 as int)] == k);
    }
    i1
}

/// Reverses the elements of a vector in place.
fn reverse(v: &mut Vec<u64>)
    requires
        old(v)@.len() > 0,
    ensures
        v@.len() == old(v)@.len(),
        forall|i: int| (0 <= i && i < v@.len()) ==> v@[i] == old(v)@[old(v)@.len() - 1 - i]
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            forall|i: int| (0 <= i && i < n) ==> v[i] == v1[length - i - 1],
            forall|i: int| (0 <= i && i < n) ==> v1[i] == v[length - i - 1],
            forall|i: int| (n <= i && i < length - n) ==> v[i] == v1[i],
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
        forall|i: int, j: int| (0 <= i && i <= j && j < v@.len()) ==> v@[i] <= v@[j],
        exists|i: int| (0 <= i && i < v@.len()) && v@[i] == k,
    ensures
        r < v@.len(),
        v@[(r as int)] == k
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 as int,
            (i2 as int) < v@.len(),
            (i1 as int) <= (i2 as int),
            exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k,
        decreases (i2 as int - i1 as int)
    {
        let ghost d = i2 - i1;
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            proof {
                let w = choose|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k;
                if w <= ix as int {
                    assert(v@[w] <= v@[ix]);
                    assert(v@[ix] < k);
                    assert(v@[w] < k);
                    assert(false);
                }
            }
            i1 = ix + 1;
            proof {
                assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
            }
        } else {
            proof {
                let w = choose|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k;
                if w > ix as int {
                    assert(v@[ix] == k);
                }
            }
            i2 = ix;
            proof {
                assert(v@[ix] == k);
                assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
            }
        }
        assert(i2 - i1 < d);
        proof {
            assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
        }
    }
    proof {
        assert(exists|i: int| (i1 as int) <= i <= (i2 as int) && v@[i] == k);
        assert(v@[(i1 as int)] == k);
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
        forall|i: int, j: int| (0 <= i && i <= j && j < t.len()) ==> t@[i] <= t@[j]
{
    for i in 0 .. t.len()
        invariant
            forall|i: int, j: int| (0 <= i && i <= j && j < t.len()) ==> t@[i] <= t@[j],
    {
        let k = t[i];
        let r = binary_search(&t, k);
        assert(r < t.len());
        assert(t@[(r as int)] == k);
        let r = binary_search_no_spinoff(&t, k);
        assert(r < t.len());
        assert(t@[(r as int)] == k);
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
    assert(forall|i: int| (0 <= i && i < t1.len()) ==> t@[i] == t1[t1.len() - i - 1]);
}

pub fn test() {
}

pub fn main() {
}

} // verus!

// Repair Round 4 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 8
// Verified: -1, Errors: 999, Verus Errors: 8
