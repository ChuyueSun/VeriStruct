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
        amount <= orig.balance,
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


fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
    requires
        true, // Permission to modify the pointer
    ensures
        *counter === old(*counter) + 1,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        true, // Permission to modify the pointer
    ensures
        *counter === 6,
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: type `vstd::simple_pptr::PPtr<u64>` cannot be dereferencedVerusErrorType.Other: type `vstd::simple_pptr::PPtr<u64>` cannot be dereferencedVerusErrorType.TypeAnnotation: type annotations neededVerusErrorType.Other: type `vstd::simple_pptr::PPtr<u64>` cannot be dereferenced
// {"$message_type":"diagnostic","message":"type `vstd::simple_pptr::PPtr<u64>` cannot be dereferenced","code":{"code":"E0614","explanation":"Attempted to dereference a variable which cannot be dereferenced.\n\nErroneous code example:\n\n```compile_fail,E0614\nlet y = 0u32;\n*y; // error: type `u32` cannot be dereferenced\n```\n\nOnly types implementing `std::ops::Deref` can be dereferenced (such as `&T`).\nExample:\n\n```\nlet y = 0u32;\nlet x = &y;\n// So here, `x` is a `&u32`, so we can dereference it:\n*x; // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4","byte_start":937,"byte_end":945,"line_start":32,"line_end":32,"column_start":9,"column_end":17,"is_primary":true,"text":[{"text":"        *counter === old(*counter) + 1,","highlight_start":9,"highlight_end":17}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0614]: type `vstd::simple_pptr::PPtr<u64>` cannot be dereferenced\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4:32:9\n   |\n32 |         *counter === old(*counter) + 1,\n   |         ^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"type `vstd::simple_pptr::PPtr<u64>` cannot be dereferenced","code":{"code":"E0614","explanation":"Attempted to dereference a variable which cannot be dereferenced.\n\nErroneous code example:\n\n```compile_fail,E0614\nlet y = 0u32;\n*y; // error: type `u32` cannot be dereferenced\n```\n\nOnly types implementing `std::ops::Deref` can be dereferenced (such as `&T`).\nExample:\n\n```\nlet y = 0u32;\nlet x = &y;\n// So here, `x` is a `&u32`, so we can dereference it:\n*x; // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4","byte_start":954,"byte_end":962,"line_start":32,"line_end":32,"column_start":26,"column_end":34,"is_primary":true,"text":[{"text":"        *counter === old(*counter) + 1,","highlight_start":26,"highlight_end":34}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0614]: type `vstd::simple_pptr::PPtr<u64>` cannot be dereferenced\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4:32:26\n   |\n32 |         *counter === old(*counter) + 1,\n   |                          ^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4","byte_start":950,"byte_end":953,"line_start":32,"line_end":32,"column_start":22,"column_end":25,"is_primary":true,"text":[{"text":"        *counter === old(*counter) + 1,","highlight_start":22,"highlight_end":25}],"label":"cannot infer type of the type parameter `A` declared on the function `old`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4","byte_start":953,"byte_end":953,"line_start":32,"line_end":32,"column_start":25,"column_end":25,"is_primary":true,"text":[{"text":"        *counter === old(*counter) + 1,","highlight_start":25,"highlight_end":25}],"label":null,"suggested_replacement":"::<A>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4:32:22\n   |\n32 |         *counter === old(*counter) + 1,\n   |                      ^^^ cannot infer type of the type parameter `A` declared on the function `old`\n   |\nhelp: consider specifying the generic argument\n   |\n32 |         *counter === old::<A>(*counter) + 1,\n   |                         +++++\n\n"}
// {"$message_type":"diagnostic","message":"type `vstd::simple_pptr::PPtr<u64>` cannot be dereferenced","code":{"code":"E0614","explanation":"Attempted to dereference a variable which cannot be dereferenced.\n\nErroneous code example:\n\n```compile_fail,E0614\nlet y = 0u32;\n*y; // error: type `u32` cannot be dereferenced\n```\n\nOnly types implementing `std::ops::Deref` can be dereferenced (such as `&T`).\nExample:\n\n```\nlet y = 0u32;\nlet x = &y;\n// So here, `x` is a `&u32`, so we can dereference it:\n*x; // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4","byte_start":1234,"byte_end":1242,"line_start":42,"line_end":42,"column_start":9,"column_end":17,"is_primary":true,"text":[{"text":"        *counter === 6,","highlight_start":9,"highlight_end":17}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0614]: type `vstd::simple_pptr::PPtr<u64>` cannot be dereferenced\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpbo15wus4:42:9\n   |\n42 |         *counter === 6,\n   |         ^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0282, E0614.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0282, E0614.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0282`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0282`.\n"}
// 
// 