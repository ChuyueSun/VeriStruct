// #![allow(unused_imports, unused_macros, non_camel_case_types)] #![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
// TODO: add requires and ensures
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

/* TEST CODE BELOW */

fn start_thread(
    counter: PPtr<u64>, 
    Tracked(perm): Tracked<PointsTo<u64>>,
    init_value: u64,
)
requires
    counter == perm.pptr(),
    perm.opt_value() === MemContents::Uninit,
    init_value < 99
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), init_value);
    assert(perm.opt_value() === MemContents::Init(init_value));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init((init_value + 1) as u64));

}

} // verus!
