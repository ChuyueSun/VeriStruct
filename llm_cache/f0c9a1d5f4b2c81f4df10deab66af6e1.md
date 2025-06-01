# Prompt

## System
You are a helpful AI assistant specialized in Verus formal verification.

## Instruction

You are a highly experienced expert in Verus (the verifier for Rust). Your task is to refine the "View" function within the given Verus file. The "View" function is the mathematical abstraction for a data structure, capturing the minimal information needed for its specification in Verus.

Your responsibilities:
  1. Analyze the current "View" function to determine if its tuple (or other structure) adequately represents the module.
  2. Evaluate whether the abstraction can be improved. (Hint: If the tuple is identical to the internal fields, that is likely not an ideal abstraction.)
  3. Modify only the "View" function to improve its abstraction while leaving all other parts of the file unchanged.
  4. Use a flattened tuple.
  5. Return the **entire updated Verus file** with your refined "View" function.

Please provide only the complete Rust code of the refined file with no additional commentary.


# Verus Common Knowledge

## Important Notes
- Don't delete existing non-buggy `#[trigger]`!
- Don't change "unwind" to `(unwind) as bool`!
- Return the complete modified Rust code in your response without explanations.

## Spec Functions
1. No Direct Method Calls:
   In a spec function, you cannot directly call instance methods such as vector.is_full().
2. Use the @ Operator:
   To invoke methods on a variable within a spec, first convert it to its specification-level representation View with @.
3. Always use vector.len() instead of vector@.len().
4. Simplify Boolean Conjunctions:
   When combining multiple conditions, avoid excessive &&&. Fewer (or well-structured) conjunctions make the spec code easier to read and debug.

## Operators
Verus extends Rust logical operators with low-precedence forms that are especially helpful in specification code:

Standard Operators: &&, ||, ==>, <==>
Low-Precedence Variants: &&& and |||

The meaning of &&& is the same as && (logical AND), and ||| is the same as || (logical OR), but with lower precedence. This allows you to write conditions in a "bulleted list" style that remains grouped in a logical manner:

```
&&& a ==> b
&&& c
&&& d <==> e && f
```

is equivalent to:

```
(a ==> b) && c && (d <==> (e && f))
```

Note:
- Implication (==>) and equivalence (<==>) bind more tightly than &&& and |||.
- Using &&&/||| can make long specifications clearer by grouping logical clauses neatly.


# Verus View Function Guidelines

## View Refinement Guidelines
1. A good View abstraction should:
   - Represent the essential state of the data structure, not just copy its fields
   - Hide implementation details while preserving behavior
   - Be as simple as possible while being complete

2. Common refinements:
   - For collections (arrays, lists): Use Seq<T> instead of raw arrays
   - For indices: Use meaningful representations (e.g., range of valid elements)
   - For flag fields: Consider if they can be derived from other state

3. Avoid redundancy:
   - Only include fields necessary for specification
   - Derive computable properties in method ensures clauses, not in the view

4. Prefer mathematical types over concrete types when possible


## Query
#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use vstd::atomic_ghost::*;
use vstd::prelude::*;
use vstd::{pervasive::*, *};

verus! {

struct_with_invariants!{
    struct Lock<T> {
        field: AtomicBool<_, Option<T>, _>,
    }

    spec fn well_formed(&self) -> bool {
        // TODO: add specification
    }
}

// ------------------- ADDED View IMPLEMENTATION -------------------
impl<T> View for Lock<T> {
    type V = (bool, Option<T>);

    closed spec fn view(&self) -> Self::V {
        let (b, g) = self.field.view();
        (b, g)
    }
}
// -----------------------------------------------------------------

fn take<T>(lock: &Lock<T>) -> (t: Tracked<T>)
    // TODO: add requires and ensures
{
    loop
        invariant
            lock.well_formed(),
    {
        let tracked ghost_value: Option<T>;
        let result =
            atomic_with_ghost!(
            &lock.field => compare_exchange(true, false);
            update prev -> next;
            ghost g => {
                if prev == true {
                    ghost_value = g;
                    g = Option::None;
                } else {
                    ghost_value = Option::None;
                }
            }
        );
        if let Result::Ok(_) = result {
            return Tracked(
                match ghost_value {
                    Option::Some(s) => s,
                    _ => { proof_from_false() },
                },
            );
        }
    }
}

struct VEqualG {}

impl AtomicInvariantPredicate<(), u64, u64> for VEqualG {
    closed spec fn atomic_inv(k: (), v: u64, g: u64) -> bool {
        // TODO: add specification
    }
}

proof fn proof_int(x: u64) -> (tracked y: u64)
    ensures
        x == y,
{
    assume(false);
    proof_from_false()
}

pub fn main() {
    // TODO Tracked of int-literal is currently unsupported.
    // Should support it, or rewrite this example
    /*
    let ato = AtomicU64::<(), u64, VEqualG>::new(Ghost(()), 10, Tracked(10));

    // illustration of atomic_with_ghost!

    atomic_with_ghost!(ato => fetch_or(19); ghost g => {
        g = proof_int(g | 19);
    });

    atomic_with_ghost!(ato => fetch_or(23); update old_val -> new_val; ghost g => {
        assert(new_val == old_val | 23);
        assert(g == old_val);

        g = proof_int(g | 23);

        assert(g == new_val);
    });

    let res = atomic_with_ghost!(
        ato => compare_exchange(20, 25);
        update old_val -> new_val;
        returning ret;
        ghost g
    => {
        assert(imply(ret.is_Ok(), old_val == 20 && new_val == 25));
        assert(imply(ret.is_Err(), old_val != 20 && new_val == old_val
            && ret.get_Err_0() == old_val));

        g = if g == 20 { proof_int(25) } else { g };
    });

    let res = atomic_with_ghost!( ato => load();
        returning ret;
        ghost g
    => {
        assert(ret == g);
    });

    atomic_with_ghost!( ato => store(36);
        update old_val -> new_val;
        ghost g
    => {
        assert(old_val == g);
        assert(new_val == 36);
        g = proof_int(36);
    });

    atomic_with_ghost!( ato => store(36);
        update old_val -> new_val;
        ghost g
    => {
        assert(old_val == g);
        assert(new_val == 36);
        g = proof_int(36);
    });

    atomic_with_ghost!( ato => store(36);
        ghost g
    => {
        g = proof_int(36);
    });
    */
}

} // verus!

