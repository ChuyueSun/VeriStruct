use vstd::prelude::*;

verus! {

proof fn fix_invariant(
    i1_old: usize, i2_old: usize, i1_new: usize, i2_new: usize, ix: usize, seq: Seq<u64>, k: u64
)
    requires
        0 <= i1_old <= i2_old < seq.len(),
        exists|idx: int| i1_old <= idx <= i2_old && seq[idx] == k,
        ix == i1_old + (i2_old - i1_old) / 2,
        (if seq[ix] < k {
            i1_new == ix + 1 && i2_new == i2_old
        } else {
            i1_new == i1_old && i2_new == ix
        }),
        0 <= i1_new <= i2_new < seq.len(),
    ensures
        exists|idx: int| i1_new <= idx <= i2_new && seq[idx] == k
{
    let witness_idx = choose|idx: int| i1_old <= idx <= i2_old && seq[idx] == k;
    if seq[ix] < k {
        if witness_idx <= ix {
            assert(false); // contradiction: seq[witness_idx] <= seq[ix] < k
        }
        assert(witness_idx >= ix + 1);
    } else {
        if witness_idx > ix {
            assert(false); // contradiction: seq[witness_idx] >= seq[ix] >= k
        }
        assert(witness_idx <= ix);
    }
}

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
        let ghost old_i1 = i1;
        let ghost old_i2 = i2;
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
        fix_invariant(old_i1, old_i2, i1, i2, ix, v@, k);
    }
    proof {
        assert(exists|idx: int| i1 <= idx <= i2 && v[idx] == k);
    }
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

fn binary_search_test1(t: Vec<u64>)
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
        // assert(r < t.len());
        // assert(t[r as int] == k);
        let r = binary_search_no_spinoff(&t, k);
        // assert(r < t.len());
        // assert(t[r as int] == k);
    }
}

fn binary_search_test2(t: Vec<u64>)
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
        // assert(r < t.len());
        // assert(t[r as int] == k);
    }
}

fn binary_search_test3(t: Vec<u64>)
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

fn reverse_test1(t: &mut Vec<u64>)
requires
    old(t).len() > 0,
    old(t).len() < u64::MAX - 1 as usize,
{
    let ghost t1 = t@;
    reverse(t);
    // assert(t.len() == t1.len());
    // assert(forall|i: int| 0 <= i < t1.len() ==> t[i] == t1[t1.len() - i - 1]);
}

fn reverse_test2(t: &mut Vec<u64>)
requires
    old(t).len() > 0,
    old(t).len() < u64::MAX - 1 as usize,
{
    let ghost t1 = t@;
    reverse(t);
    assert(t.len() == t1.len());
    // assert(forall|i: int| 0 <= i < t1.len() ==> t[i] == t1[t1.len() - i - 1]);
}

fn reverse_test3(t: &mut Vec<u64>)
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
