#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::invariant::*;

verus! {

struct ModPredicate {}

impl InvariantPredicate<int, u32> for ModPredicate {
    closed spec fn inv(k: int, v: u32) -> bool {
        v % 2 == 1
    }

    closed spec fn view(k: int, v: u32) -> (int, bool) {
        (k, v % 2 == 1)
    }
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
// VerusErrorType.Other: method `view` is not a member of trait `InvariantPredicate`
// {"$message_type":"diagnostic","message":"method `view` is not a member of trait `InvariantPredicate`","code":{"code":"E0407","explanation":"A definition of a method not in the implemented trait was given in a trait\nimplementation.\n\nErroneous code example:\n\n```compile_fail,E0407\ntrait Foo {\n    fn a();\n}\n\nstruct Bar;\n\nimpl Foo for Bar {\n    fn a() {}\n    fn b() {} // error: method `b` is not a member of trait `Foo`\n}\n```\n\nPlease verify you didn't misspell the method name and you used the correct\ntrait. First example:\n\n```\ntrait Foo {\n    fn a();\n    fn b();\n}\n\nstruct Bar;\n\nimpl Foo for Bar {\n    fn a() {}\n    fn b() {} // ok!\n}\n```\n\nSecond example:\n\n```\ntrait Foo {\n    fn a();\n}\n\nstruct Bar;\n\nimpl Foo for Bar {\n    fn a() {}\n}\n\nimpl Bar {\n    fn b() {}\n}\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo1h9q3py","byte_start":252,"byte_end":322,"line_start":15,"line_end":17,"column_start":17,"column_end":6,"is_primary":true,"text":[{"text":"    closed spec fn view(k: int, v: u32) -> (int, bool) {","highlight_start":17,"highlight_end":57},{"text":"        (k, v % 2 == 1)","highlight_start":1,"highlight_end":24},{"text":"    }","highlight_start":1,"highlight_end":6}],"label":"not a member of trait `InvariantPredicate`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0407]: method `view` is not a member of trait `InvariantPredicate`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo1h9q3py:15:17\n   |\n15 |       closed spec fn view(k: int, v: u32) -> (int, bool) {\n   |  _________________^\n16 | |         (k, v % 2 == 1)\n17 | |     }\n   | |_____^ not a member of trait `InvariantPredicate`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0407`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0407`.\n"}
// 
// 