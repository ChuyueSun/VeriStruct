// #![allow(unused_imports, unused_macros, non_camel_case_types)] #![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
    requires
        orig.balance >= amount,
    ensures
        orig.balance == old(orig.balance) - amount,
        dest.balance == old(dest.balance) + amount
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
}

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
    requires
        matches!(perm@.opt_value, MemContents::Init(_)),
    ensures
        matches!((old(perm@.opt_value), perm@.opt_value),
            (MemContents::Init(v_old), MemContents::Init(v_new)) if v_new == v_old + 1
        )
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        true,
    ensures
        matches!(perm@.opt_value, MemContents::Init(6))
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: no rules expected the token `@`VerusErrorType.Other: expected one of `!`, `)`, `,`, `.`, `::`, `?`, `{`, or an operator, found `@`VerusErrorType.Other: expected one of `!`, `)`, `,`, `.`, `::`, `?`, `{`, or an operator, found `@`VerusErrorType.Other: no rules expected the token `@`
// {"$message_type":"diagnostic","message":"no rules expected the token `@`","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0","byte_start":808,"byte_end":809,"line_start":28,"line_end":28,"column_start":22,"column_end":23,"is_primary":true,"text":[{"text":"        matches!(perm@.opt_value, MemContents::Init(_)),","highlight_start":22,"highlight_end":23}],"label":"no rules expected this token in macro call","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"while trying to match `,`","code":null,"level":"note","spans":[{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs","byte_start":17480,"byte_end":17481,"line_start":474,"line_end":474,"column_start":22,"column_end":23,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error: no rules expected the token `@`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0:28:22\n   |\n28 |         matches!(perm@.opt_value, MemContents::Init(_)),\n   |                      ^ no rules expected this token in macro call\n   |\nnote: while trying to match `,`\n  --> /rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs:474:22\n\n"}
// {"$message_type":"diagnostic","message":"expected one of `!`, `)`, `,`, `.`, `::`, `?`, `{`, or an operator, found `@`","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0","byte_start":882,"byte_end":883,"line_start":30,"line_end":30,"column_start":27,"column_end":28,"is_primary":true,"text":[{"text":"        matches!((old(perm@.opt_value), perm@.opt_value),","highlight_start":27,"highlight_end":28}],"label":"expected one of 8 possible tokens","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected one of `!`, `)`, `,`, `.`, `::`, `?`, `{`, or an operator, found `@`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0:30:27\n   |\n30 |         matches!((old(perm@.opt_value), perm@.opt_value),\n   |                           ^ expected one of 8 possible tokens\n\n"}
// {"$message_type":"diagnostic","message":"expected one of `!`, `)`, `,`, `.`, `::`, `?`, `{`, or an operator, found `@`","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0","byte_start":900,"byte_end":901,"line_start":30,"line_end":30,"column_start":45,"column_end":46,"is_primary":true,"text":[{"text":"        matches!((old(perm@.opt_value), perm@.opt_value),","highlight_start":45,"highlight_end":46}],"label":"expected one of 8 possible tokens","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected one of `!`, `)`, `,`, `.`, `::`, `?`, `{`, or an operator, found `@`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0:30:45\n   |\n30 |         matches!((old(perm@.opt_value), perm@.opt_value),\n   |                                             ^ expected one of 8 possible tokens\n\n"}
// {"$message_type":"diagnostic","message":"no rules expected the token `@`","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0","byte_start":1253,"byte_end":1254,"line_start":42,"line_end":42,"column_start":22,"column_end":23,"is_primary":true,"text":[{"text":"        matches!(perm@.opt_value, MemContents::Init(6))","highlight_start":22,"highlight_end":23}],"label":"no rules expected this token in macro call","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"while trying to match `,`","code":null,"level":"note","spans":[{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs","byte_start":17480,"byte_end":17481,"line_start":474,"line_end":474,"column_start":22,"column_end":23,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error: no rules expected the token `@`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpid21m1s0:42:22\n   |\n42 |         matches!(perm@.opt_value, MemContents::Init(6))\n   |                      ^ no rules expected this token in macro call\n   |\nnote: while trying to match `,`\n  --> /rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs:474:22\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// 
// 