use vstd::prelude::*;

verus! {

fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v.len() > 0,
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|idx: int| 0 <= idx < v.len() && v[idx] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 <= i2 < v.len(),
            exists|idx: int| i1 <= idx <= i2 && v[idx] == k,
        decreases i2 - i1
    {
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
    }
    proof {
        assert(exists|idx: int| i1 <= idx <= i2 && v[idx] == k);
    } // Added by AI
    i1
}

fn reverse(v: &mut Vec<u64>)
    ensures
        v.len() == old(v).len(),
        forall|i: int|
            0 <= i && i < v.len() ==> v[i] == old(v)[v.len() - 1 - i],
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            forall|i: int| 0 <= i < n ==> v[i] == v1[length - 1 - i],
            forall|i: int| length - n <= i < length ==> v[i] == v1[length - 1 - i],
            forall|i: int| n <= i < length - n ==> v[i] == v1[i],
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
        exists|idx: int| 0 <= idx < v.len() && v[idx] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 <= i2 < v.len(),
            exists|idx: int| i1 <= idx <= i2 && v[idx] == k,
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
