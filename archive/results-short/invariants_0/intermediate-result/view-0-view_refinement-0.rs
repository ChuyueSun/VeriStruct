#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::invariant::*;

verus! {

struct ModPredicate {}

impl InvariantPredicate<int, u32> for ModPredicate {
    closed spec fn inv(k: int, v: u32) -> bool {
        v % 2 == 1
    }
}

// Refined "View" function using a flattened tuple:
closed spec fn View(ai: AtomicInvariant<int, u32, ModPredicate>) -> (int, bool) {
    let (k, v, _) = ai.view();
    (k, v % 2 == 1)
}

pub fn main() {
    let tracked u: u32 = 5u32;
    let tracked i: AtomicInvariant<int, u32, ModPredicate> = AtomicInvariant::new(1, u, 0);
    open_atomic_invariant!(&i => inner => {
      proof {
          if inner == 1u32 {
              inner = 3u32;
          }
      }
    });
    let tracked j: AtomicInvariant<int, u32, ModPredicate> = AtomicInvariant::new(1, 7u32, 1);
    open_atomic_invariant!(&i => inner_i => {
      open_atomic_invariant!(&j => inner_j => {
          proof {
              let tracked tmp = inner_i;
              inner_i = inner_j;
              inner_j = tmp;
          }
      });
    });
    let tracked j = i.into_inner();
    assert(j % 2 == 1);
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: no method named `view` found for struct `vstd::invariant::AtomicInvariant` in the current scope
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::invariant::AtomicInvariant` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpivn5ran1","byte_start":395,"byte_end":399,"line_start":18,"line_end":18,"column_start":24,"column_end":28,"is_primary":true,"text":[{"text":"    let (k, v, _) = ai.view();","highlight_start":24,"highlight_end":28}],"label":"method not found in `AtomicInvariant<int, u32, ModPredicate>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::invariant::AtomicInvariant` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpivn5ran1:18:24\n   |\n18 |     let (k, v, _) = ai.view();\n   |                        ^^^^ method not found in `AtomicInvariant<int, u32, ModPredicate>`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
// 
// 