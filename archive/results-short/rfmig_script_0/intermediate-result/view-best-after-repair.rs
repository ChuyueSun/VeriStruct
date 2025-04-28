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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
// VerusErrorType.Other: in expressions, `_` can only be used on the left-hand side of an assignmentVerusErrorType.Other: the trait bound `&mut _: builtin::Integer` is not satisfiedVerusErrorType.Other: the trait bound `vstd::raw_ptr::MemContents<u64>: std::marker::Copy` is not satisfied
// {"$message_type":"diagnostic","message":"in expressions, `_` can only be used on the left-hand side of an assignment","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpemqot3_y","byte_start":879,"byte_end":880,"line_start":29,"line_end":29,"column_start":55,"column_end":56,"is_primary":true,"text":[{"text":"        perm.opt_value() === MemContents::<u64>::Init(_),","highlight_start":55,"highlight_end":56}],"label":"`_` not allowed here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: in expressions, `_` can only be used on the left-hand side of an assignment\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpemqot3_y:29:55\n   |\n29 |         perm.opt_value() === MemContents::<u64>::Init(_),\n   |                                                       ^ `_` not allowed here\n\n"}
// {"$message_type":"diagnostic","message":"the trait bound `&mut _: builtin::Integer` is not satisfied","code":{"code":"E0277","explanation":"You tried to use a type which doesn't implement some trait in a place which\nexpected that trait.\n\nErroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function. Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function. It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpemqot3_y","byte_start":937,"byte_end":943,"line_start":31,"line_end":31,"column_start":43,"column_end":49,"is_primary":true,"text":[{"text":"        match old(( perm.opt_value() ) as &mut _) {","highlight_start":43,"highlight_end":49}],"label":"the trait `builtin::Integer` is not implemented for `&mut _`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"required by a bound in `builtin::spec_cast_integer`","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":24453,"byte_end":24527,"line_start":934,"line_end":934,"column_start":1,"column_end":75,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider removing the leading `&`-reference","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpemqot3_y","byte_start":937,"byte_end":942,"line_start":31,"line_end":31,"column_start":43,"column_end":48,"is_primary":true,"text":[{"text":"        match old(( perm.opt_value() ) as &mut _) {","highlight_start":43,"highlight_end":48}],"label":null,"suggested_replacement":"","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0277]: the trait bound `&mut _: builtin::Integer` is not satisfied\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpemqot3_y:31:43\n   |\n31 |         match old(( perm.opt_value() ) as &mut _) {\n   |                                           ^^^^^^ the trait `builtin::Integer` is not implemented for `&mut _`\n   |\nnote: required by a bound in `builtin::spec_cast_integer`\n  --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:934:1\nhelp: consider removing the leading `&`-reference\n   |\n31 -         match old(( perm.opt_value() ) as &mut _) {\n31 +         match old(( perm.opt_value() ) as _) {\n   |\n\n"}
// {"$message_type":"diagnostic","message":"the trait bound `vstd::raw_ptr::MemContents<u64>: std::marker::Copy` is not satisfied","code":{"code":"E0277","explanation":"You tried to use a type which doesn't implement some trait in a place which\nexpected that trait.\n\nErroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function. Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function. It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpemqot3_y","byte_start":913,"byte_end":943,"line_start":31,"line_end":31,"column_start":19,"column_end":49,"is_primary":true,"text":[{"text":"        match old(( perm.opt_value() ) as &mut _) {","highlight_start":19,"highlight_end":49}],"label":"the trait `std::marker::Copy` is not implemented for `vstd::raw_ptr::MemContents<u64>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"required by a bound in `builtin::spec_cast_integer`","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":24453,"byte_end":24527,"line_start":934,"line_end":934,"column_start":1,"column_end":75,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0277]: the trait bound `vstd::raw_ptr::MemContents<u64>: std::marker::Copy` is not satisfied\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpemqot3_y:31:19\n   |\n31 |         match old(( perm.opt_value() ) as &mut _) {\n   |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `std::marker::Copy` is not implemented for `vstd::raw_ptr::MemContents<u64>`\n   |\nnote: required by a bound in `builtin::spec_cast_integer`\n  --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:934:1\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0277`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0277`.\n"}
//
//
