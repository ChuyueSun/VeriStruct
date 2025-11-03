use vstd::prelude::*;

verus! {

fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v.len() > 0,
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|m: int| 0 <= m < v.len() && v[m] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 <= i2 < v.len(),
            i1 + (i2 - i1) / 2 < v.len(),
        decreases (i2 - i1)
    {
        let ghost d = i2 - i1; // Added by AI
        let ix = i1 + (i2 - i1) / 2;
        proof {
            assert(ix < v.len());
        }
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
        assert(i2 - i1 < d); // Added by AI
    }
    i1
}

fn reverse(v: &mut Vec<u64>)
    requires
        old(v).len() >= 0,
    ensures
        v.len() == old(v).len(),
        forall|i: int|
            0 <= i && i < v.len() ==> v[i] == old(v)[v.len() as int - 1 - i],
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
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
        exists|m: int| 0 <= m < v.len() && v[m] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            0 <= i1 <= i2 < v.len(),
            i1 + (i2 - i1) / 2 < v.len(),
        decreases (i2 - i1)
    {
        let ghost d = i2 - i1; // Added by AI
        let ix = i1 + (i2 - i1) / 2;
        proof {
            assert(ix < v.len());
        }
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
    forall|i: int, j: int| 0 <= i <= j < t.len() ==> t[i] <= t[j],
{
    for i in 0 .. t.len()
    invariant
        forall|i: int, j: int| 0 <= i <= j < t.len() ==> t[i] <= t[j],
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
    old(t).len() < u64::MAX - 1 as usize,
{
    let ghost t1 = t@;
    reverse(t);
    assert(t.len() == t1.len());
    assert(forall|i: int| 0 <= i < t1.len() ==> t[i] == t1[t1.len() - i - 1]);
}

pub fn test() {
}

pub fn main() {
}

}

// VEval Score: Compilation Error: False, Verified: 6, Errors: 4, Verus Errors: 8
