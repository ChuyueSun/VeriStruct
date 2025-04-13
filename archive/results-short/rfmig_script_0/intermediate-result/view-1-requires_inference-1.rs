// #![allow(unused_imports, unused_macros, non_camel_case_types)] #![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> (ret: ())
    requires
        amount <= old(orig).balance,
    ensures
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
        orig.balance + dest.balance == old(orig).balance + old(dest).balance,
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
}

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>) -> (ret: ())
    requires
        true,
    ensures
        true,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>) -> (ret: ())
    requires
        true,
    ensures
        true,
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 7
// VerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.AssertFail: assertion failedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339","byte_start":674,"byte_end":695,"line_start":23,"line_end":23,"column_start":20,"column_end":41,"is_primary":true,"text":[{"text":"    dest.balance = dest.balance + amount;","highlight_start":20,"highlight_end":41}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339:23:20\n   |\n23 |     dest.balance = dest.balance + amount;\n   |                    ^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339","byte_start":708,"byte_end":788,"line_start":24,"line_end":24,"column_start":12,"column_end":92,"is_primary":true,"text":[{"text":"    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);","highlight_start":12,"highlight_end":92}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339:24:12\n   |\n24 |     assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);\n   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/simple_pptr.rs","byte_start":18005,"byte_end":18024,"line_start":511,"line_end":511,"column_start":13,"column_end":32,"is_primary":false,"text":[{"text":"            perm.pptr() == self,","highlight_start":13,"highlight_end":32}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339","byte_start":961,"byte_end":992,"line_start":33,"line_end":33,"column_start":23,"column_end":54,"is_primary":true,"text":[{"text":"    let cur_i: u64 = *counter.borrow(Tracked(&*perm));","highlight_start":23,"highlight_end":54}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339:33:23\n    |\n33  |     let cur_i: u64 = *counter.borrow(Tracked(&*perm));\n    |                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/simple_pptr.rs:511:13\n    |\n511 |             perm.pptr() == self,\n    |             ------------------- failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/simple_pptr.rs","byte_start":17181,"byte_end":17205,"line_start":488,"line_end":488,"column_start":13,"column_end":37,"is_primary":false,"text":[{"text":"            old(perm).pptr() == self,","highlight_start":13,"highlight_end":37}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339","byte_start":998,"byte_end":1039,"line_start":34,"line_end":34,"column_start":5,"column_end":46,"is_primary":true,"text":[{"text":"    counter.replace(Tracked(perm), cur_i + 1);","highlight_start":5,"highlight_end":46}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339:34:5\n    |\n34  |     counter.replace(Tracked(perm), cur_i + 1);\n    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/simple_pptr.rs:488:13\n    |\n488 |             old(perm).pptr() == self,\n    |             ------------------------ failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339","byte_start":1029,"byte_end":1038,"line_start":34,"line_end":34,"column_start":36,"column_end":45,"is_primary":true,"text":[{"text":"    counter.replace(Tracked(perm), cur_i + 1);","highlight_start":36,"highlight_end":45}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339:34:36\n   |\n34 |     counter.replace(Tracked(perm), cur_i + 1);\n   |                                    ^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/simple_pptr.rs","byte_start":15456,"byte_end":15480,"line_start":442,"line_end":442,"column_start":13,"column_end":37,"is_primary":false,"text":[{"text":"            old(perm).pptr() == self,","highlight_start":13,"highlight_end":37}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339","byte_start":1239,"byte_end":1273,"line_start":44,"line_end":44,"column_start":5,"column_end":39,"is_primary":true,"text":[{"text":"    counter.put(Tracked(&mut perm), 5);","highlight_start":5,"highlight_end":39}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339:44:5\n    |\n44  |     counter.put(Tracked(&mut perm), 5);\n    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/simple_pptr.rs:442:13\n    |\n442 |             old(perm).pptr() == self,\n    |             ------------------------ failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339","byte_start":1385,"byte_end":1426,"line_start":47,"line_end":47,"column_start":12,"column_end":53,"is_primary":true,"text":[{"text":"    assert(perm.opt_value() === MemContents::Init(6));","highlight_start":12,"highlight_end":53}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfzsxf339:47:12\n   |\n47 |     assert(perm.opt_value() === MemContents::Init(6));\n   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 7 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 7 previous errors\n\n"}
// 
// 