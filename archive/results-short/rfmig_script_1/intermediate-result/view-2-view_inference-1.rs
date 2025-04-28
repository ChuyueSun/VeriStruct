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
        old(orig).balance >= amount,
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

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
    requires
        exists |n: u64| old(perm).opt_value() == MemContents::Init(n),
    ensures
        exists |n: u64| old(perm).opt_value() == MemContents::Init(n)
            && perm.opt_value() == MemContents::Init(n + 1),
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        perm.opt_value() == MemContents::Uninit,
    ensures
        perm.opt_value() == MemContents::Init(6),
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.TypeAnnotation: type annotations needed
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuyw0_ehx","byte_start":1308,"byte_end":1327,"line_start":40,"line_end":40,"column_start":29,"column_end":48,"is_primary":true,"text":[{"text":"        perm.opt_value() == MemContents::Uninit,","highlight_start":29,"highlight_end":48}],"label":"cannot infer type of the type parameter `T` declared on the enum `MemContents`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuyw0_ehx","byte_start":1319,"byte_end":1319,"line_start":40,"line_end":40,"column_start":40,"column_end":40,"is_primary":true,"text":[{"text":"        perm.opt_value() == MemContents::Uninit,","highlight_start":40,"highlight_end":40}],"label":null,"suggested_replacement":"::<T>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuyw0_ehx:40:29\n   |\n40 |         perm.opt_value() == MemContents::Uninit,\n   |                             ^^^^^^^^^^^^^^^^^^^ cannot infer type of the type parameter `T` declared on the enum `MemContents`\n   |\nhelp: consider specifying the generic argument\n   |\n40 |         perm.opt_value() == MemContents::<T>::Uninit,\n   |                                        +++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0282`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0282`.\n"}
//
//
