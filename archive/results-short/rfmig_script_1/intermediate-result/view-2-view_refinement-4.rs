// #![allow(unused_imports, unused_macros, non_camel_case_types)]
#![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

impl Account {
    /// Refined View function using a flattened tuple
    pub open spec fn View(&self) -> (nat) {
        (self.balance as nat)
    }
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
    requires
        amount <= orig.balance,
    ensures
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
        orig.balance + dest.balance == old(orig).balance + old(dest).balance
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
}


fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
    requires
        perm.opt_value() === MemContents::Init(_),
    ensures
        let old_val = old(perm).opt_value().get_Init_0();
        perm.opt_value().get_Init_0() == old_val + 1
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        perm.opt_value() === MemContents::Uninit,
    ensures
        perm.opt_value().get_Init_0() == 6
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: expected `!`
// {"$message_type":"diagnostic","message":"expected `!`","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5v3zx7k_","byte_start":1151,"byte_end":1152,"line_start":41,"line_end":41,"column_start":13,"column_end":14,"is_primary":true,"text":[{"text":"        perm.opt_value().get_Init_0() == old_val + 1","highlight_start":13,"highlight_end":14}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected `!`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5v3zx7k_:41:13\n   |\n41 |         perm.opt_value().get_Init_0() == old_val + 1\n   |             ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
//
//
