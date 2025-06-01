use vstd::prelude::*;

verus! {

fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        v.len() > 0,
        // The array must be sorted (non-decreasing).
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        // k must appear at least once in the array.
        exists|i: int| 0 <= i < v.len() && v[i] == k,
    ensures
        r < v.len(),
        v[( r ) as int] == k,
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
    requires
        // No special precondition needed.
        // This function correctly handles empty or non-empty vectors.
        true,
    ensures
        // The length remains the same.
        v@.len() == old(v)@.len(),
        // The result is indeed the reverse of the old version.
        forall|i: int| 0 <= i < v@.len() ==> v@[i] == old(v)@[v@.len() - i - 1],
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
        v.len() > 0,
        // The array must be sorted (non-decreasing).
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        // k must appear at least once in the array.
        exists|i: int| 0 <= i < v.len() && v[i] == k,
    ensures
        r < v.len(),
        v[( r ) as int] == k,
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

#[verifier::loop_isolation(false)]
fn reverse_no_spinoff(v: &mut Vec<u64>)
    requires
        // No special precondition needed.
        true,
    ensures
        v@.len() == old(v)@.len(),
        forall|i: int| 0 <= i < v@.len() ==> v@[i] == old(v)@[v@.len() - i - 1],
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

} // verus!

fn main() {
    let mut v = vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90];
    println!("{}", binary_search(&v, 70));
    println!();
    reverse(&mut v);
    for x in v {
        println!("{}", x);
    }

    println!("Pushed 5 values:");
    for x in pusher() {
        println!("{}", x);
    }
}



// Step 1 (spec_inference) VEval Score: Compilation Error: False, Verified: 9, Errors: 0, Verus Errors: 0
// Verified: 9, Errors: 0, Verus Errors: 0