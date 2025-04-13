use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

#[verifier::loop_isolation(false)]
pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()
    requires
        amount <= old(orig).balance,
    ensures
        orig.balance + dest.balance == old(orig).balance + old(dest).balance,
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
}

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>) -> ()
    requires
        perm.opt_value() === MemContents::<u64>::Init(_),
    ensures
        match old(( perm.opt_value() ) as &mut _) {
            MemContents::<u64>::Init(x) => perm.opt_value() === MemContents::<u64>::Init(( x + 1 ) as u64),
            _ => false
        },
{
    let old_val: Ghost<u64> = Ghost(*counter.borrow(Tracked(&*perm)));
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>) -> ()
    requires
        true,
    ensures
        perm.opt_value() === MemContents::Init(6),
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!


//         match old(( perm.opt_value() ) as &mut _) {
//             MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),
//   type must be known at this point: x + 1
//   cannot infer type of the type parameter `A` declared on the function `old`: old

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3