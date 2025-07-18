use vstd::prelude::*;

verus! {

fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
// TODO: add specification
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
    // TODO: add invariants
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
// TODO: add specification
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
    // TODO: add invariants
    {
        let x = v[n];
        let y = v[length - 1 - n];
        v.set(n, y);
        v.set(length - 1 - n, x);
    }
}

#[verifier::loop_isolation(false)]
fn binary_search_no_spinoff(v: &Vec<u64>, k: u64) -> (r: usize)
// TODO: add specification
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
    // TODO: add invariants
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

fn pusher() -> Vec<u64> {
    let mut v = Vec::new();
    v.push(0);
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);
    let ghost goal = Seq::new(5, |i: int| i as u64);
    assert(v@ =~= goal);
    assert(v[2] == 2);
    v.pop();
    v.push(4);
    assert(v@ =~= goal);
    v
}

spec fn uninterp_fn(x: u64) -> bool;

fn pop_test(t: Vec<u64>)
    requires
        t.len() > 0,
        forall|i: int| #![auto] 0 <= i < t.len() ==> uninterp_fn(t[i]),
{
    let mut t = t;
    let x = t.pop().unwrap();
    assert(uninterp_fn(x));
    assert(forall|i: int| #![auto] 0 <= i < t.len() ==> uninterp_fn(t[i]));
}

fn push_test(t: Vec<u64>, y: u64)
    requires
        forall|i: int| #![auto] 0 <= i < t.len() ==> uninterp_fn(t[i]),
        uninterp_fn(y),
{
    let mut t = t;
    t.push(y);
    assert(forall|i: int| #![auto] 0 <= i < t.len() ==> uninterp_fn(t[i]));
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

fn test() {
}

pub fn main() {
}

} // verus!
