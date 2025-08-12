import os, sys
import openai
import generator_util as util 

REFERENCE = """
// =====================================
//   Verus Reference: Human-Tuned Guide
// =====================================

This comprehensive guide ensures Verus code is **correct**, **verifiable**, and **idiomatic**, distilled from independent examples and best practices. It equips you with:

* Precise syntax and structural rules
* Verification design patterns and heuristics
* Detailed explanations of Verus modes, constructs, and proof strategies

---

## 🔁 File Layout

1.  **Global Imports** go **outside** `verus! {}` blocks. These imports provide access to Verus's standard library (`vstd`) and other necessary modules.

    ```rust
    use vstd::prelude::*; // Essential for most Verus projects, brings in common definitions
    use vstd::set::*;     // Example: Importing the Set data type
    // other `use` statements as needed for specific functionalities
    ```

2.  **Verified code** lives inside one or more `verus! {}` blocks. These blocks signal to the Verus verifier that the enclosed Rust code should be formally analyzed.

    ```rust
    verus! {
        // All Verus-verified definitions, functions, specifications (specs),
        // and proofs are placed within these blocks.
    }
    ```

3.  **No Rust-style tests** (`#[test]`, `#[cfg(test)]`, `assert_eq!`, `dbg!`, etc.) are used for formal verification. Instead, all correctness is established via Verus's ghost code (`assert`, `ensures`, `proof fn`, etc.).
    For demonstrating or running specific scenarios, a single `pub fn main()` is typically used, often placed under a special comment `/* TEST CODE BELOW */`. This `main` function should primarily contain calls to `exec` functions and `assert` statements that are proven by Verus.

---

## 🔤 Verus Code Modes

Verus divides code into three rigorously enforced modes, dictating what kind of logic they can contain, what types they can use, and which other functions they can call. Understanding these modes is fundamental to structuring Verus programs.

| Mode      | Description                                                 | Can Call (and use types from)      | Can Contain Types                                         | Runtime Presence |
| --------- | ----------------------------------------------------------- | ---------------------------------- | --------------------------------------------------------- | ---------------- |
| **spec** | Pure mathematical definitions (`spec fn`, `spec const`). Expresses "what" a program does, abstractly. | Only `spec` functions              | `int`, `nat`, `Seq<T>`, `Set<T>`, `Map<K,V>`, `Ghost<T>`, `Tracked<T>`, `spec_fn` closures, and *view* types of `exec` data. | **None** (Ghost) |
| **proof** | Code for constructing proofs (`proof fn`, `proof { ... }`). Explains "how" a property is proven. | `spec` functions & `proof` functions | Same as `spec` mode, as `proof` code is also ghost.      | **None** (Ghost) |
| **exec** | Regular Rust logic (`fn`, `struct`, `enum`, `impl`). This is the "runnable reality" of the program. | `spec` functions directly; `proof` functions only inside `proof { ... }` blocks or `assert(...) by { ... }`. | Standard Rust primitive types (`u8`, `u32`, `usize`, `bool`, `char`, etc.), Rust collections (`Vec`), references (`&`), Boxes, Rcs, Arcs. | **Yes** (Compiled) |

**Ghost-only types**: `int`, `nat`, `Seq<T>`, `Set<T>`, `Map<K,V>`, `Ghost<T>`, `Tracked<T>`, `spec_fn(Args) -> Ret`. These types exist solely for verification and cannot appear in `exec` mode code.

---

## 📐 Function Contracts

Verus uses function contracts to establish clear interfaces and enable modular verification.

### Requirements & Guarantees

* **Preconditions** (`requires`): Specify conditions that **must be true** about the inputs and state *before* the function body executes. If a caller satisfies these, the callee guarantees its postconditions.
    * Syntactically, `requires` clauses are **indented** on new lines, not parenthesized, for readability.

    ```rust
    pub fn foo(x: int) -> (y: int)
        requires
            // Each condition on a new, indented line
            x >= 0,
            x < 100, // Multiple conditions are separated by commas
        ensures
            y == x + 1,
    {
        x + 1
    }
    ```

* **Postconditions** (`ensures`): Specify conditions that **will be true** about the outputs and state *after* the function body successfully executes, assuming its `requires` were met.
    * `ensures` clauses follow the same indented style as `requires`.

### Naming Return Values

To refer to the function's return value within the `ensures` clause, explicitly name it in the function signature:

```rust
pub fn calculate_sum(a: int, b: int) -> (result_sum: int) // 'result_sum' is the name
    ensures
        result_sum == a + b, // Refer to the named return value
{
    a + b
}
````

-----

## 📏 Loop Verification

All loops (`while`, `for`) in Verus require explicit verification annotations to ensure correctness and termination.

  * **All loops** (`while`, `for`) **must** include:

      * `invariant` clauses
      * `decreases` metrics for termination

  * **`invariant`**: A list of ghost properties that are guaranteed to hold true *before* each iteration of the loop, and *after* each iteration (before checking the loop condition again). These are crucial for inductive proofs over loops.

    ```rust
    // Example: A loop summing numbers up to n
    fn sum_up_to_n(n: nat) -> (sum: nat)
        ensures sum == n * (n + 1) / 2
    {
        let mut i: nat = 0;
        let mut current_sum: nat = 0;

        while i <= n
            invariant // Indented list of invariants
                0 <= i <= n + 1, // Loop counter 'i' is within expected bounds
                current_sum == i * (i - 1) / 2, // 'current_sum' correctly represents sum of 0 to i-1
            decreases n - i // Metric that strictly decreases with each iteration
        {
            current_sum = current_sum + i;
            i = i + 1;
        }
        current_sum
    }
    ```

  * **`decreases`**: A metric (typically an `int` or `nat` expression) that must strictly decrease with each iteration of the loop, and must always be non-negative. This proves the loop terminates. Common `decreases` metrics include `n - i`, `len`, `height(tree)`, etc.

  * **Break-aware loops**: For loops that might exit early via a `break` statement, `invariant_except_break` can be used. If so, an explicit `ensures` clause on the loop block itself (rather than just on the function) is typically needed to specify the post-break state. This is a more advanced pattern.

-----

## 🧩 Data Types & Views

Verus allows you to define custom data types much like Rust, but with special considerations for verification.

### Structs and Enums

  * Define record types (`struct`) and variant types (`enum`) as you would in Rust.

  * Verus extends these with verification capabilities, allowing `spec` fields, `view` methods, etc.

  * **Important**: Verus-specific attributes like `#[verifier::ext_equal]` are placed directly on the `struct` or `enum` definition, *not* on `use` statements. All `use` statements are standard Rust `use` statements and must be top-level.

    ```rust
    #[verifier::ext_equal] // Example: Forcing extensional equality for this struct
    struct Point {
        x: int,
        y: int,
    }

    enum Shape {
        Circle(int),
        Rect(int, int),
    }
    ```

### View Pattern (`@` operator)

The view pattern is critical for bridging `exec` runtime data with its `spec` mathematical model. It allows you to reason about the abstract properties of concrete data structures.

1.  **Implement `View` trait**: For a Rust `exec` type `MyType`, you define how its "view" (`MyType@`) is represented as a `spec` type (e.g., `Seq<T>`, `Set<T>`, `int`).

    ```rust
    // Assume MyType wraps a Vec<T>
    use vstd::seq::Seq; // Import Seq for the view type

    impl View for MyType {
        type V = Seq<T>; // Define the associated view type
        // The 'closed' keyword here is important: it means the definition of 'view'
        // is internal to this module and its properties are exposed via 'proof fn's if needed.
        closed spec fn view(&self) -> Self::V {
            self.vec@ // Assuming MyType has a 'vec' field of type Vec<T>
                      // The `@` on `self.vec` extracts the Seq view of the Vec.
        }
    }
    ```

2.  **Use `self@` in specs**: Once the `View` trait is implemented, you can use `value@` (e.g., `my_instance@`) in `spec` contexts (like `requires`, `ensures`, `invariant`, `spec fn` bodies) to refer to the mathematical view of your runtime data.

    ```rust
    // In an exec function's ensures clause:
    fn modify_my_type(m: MyType) -> (ret: MyType)
        ensures
            ret@.len() == m@.len() + 1, // Reasoning about the length of the Seq view
    {
        // ... implementation that adds an element ...
        ret
    }
    ```

-----

## 🎯 PCM & Resource Algebras (Advanced Topic)

Programatic Concurrent Monoids (PCMs) and Resource Algebras are advanced concepts in Verus for reasoning about mutable state, ownership, and concurrency. They formalize how resources can be combined (`op`), split, and validated (`valid`).

1.  **Define `enum V`**: Represents the algebraic states or "permissions" for a resource. Each variant often corresponds to a distinct state or set of properties.

    ```rust
    enum MyResourcePermission {
        Empty,
        Chosen { value: int },
        Invalid,
    }
    ```

2.  **Implement `PCM` for `V`**: The `PCM` trait requires defining methods like `valid`, `op`, and `unit`. These methods specify the algebra's rules (e.g., how permissions combine, what constitutes a valid state).

    ```rust
    // `vstd::pervasive::*` often needed for PCM traits
    use vstd::pervasive::modes::*;
    use vstd::pervasive::eq::*;

    impl PCM for MyResourcePermission {
        open spec fn valid(self) -> bool {
            // Define what makes a permission state 'valid'
            !self.is_Invalid()
        }
        open spec fn op(self, other: Self) -> Self {
            // Define how two permissions combine (monoid operation)
            match (self, other) {
                (MyResourcePermission::Empty, p) => p,
                (p, MyResourcePermission::Empty) => p,
                (MyResourcePermission::Chosen { value: v1 }, MyResourcePermission::Chosen { value: v2 }) =>
                    if v1 == v2 { MyResourcePermission::Chosen { value: v1 } } else { MyResourcePermission::Invalid },
                _ => MyResourcePermission::Invalid, // Any other combination results in Invalid
            }
        }
        open spec fn unit() -> Self {
            // Define the identity element of the monoid
            MyResourcePermission::Empty
        }
    }
    ```

3.  **Use `Ghost<V>` or `Tracked<V>`**: `Ghost<V>` is for pure ghost state, while `Tracked<V>` is for ghost state tied to `exec` mutable memory. These types hold instances of your `PCM` `enum` and allow Verus to track and prove properties about resource ownership and invariants. Define a verified API in `exec` and `proof` code that manipulates these resources.

-----

## 🔎 Proof Tools

Verus provides specific keywords and constructs to assist in building and verifying proofs.

### `assert(expr)`

  * Command to the SMT solver: "Prove that `expr` is true *at this exact point* in the program, given all currently known facts."

  * If Verus cannot prove `expr`, it reports a verification error. This is your primary way to incrementally build proofs and debug why a larger proof might fail.

    ```rust
    let x: int = 5;
    assert(x > 0); // Verus verifies this is true
    assert(x % 2 == 1); // Verus verifies this is true
    ```

### `assume(expr)`

  * **Use sparingly**: Tells the SMT solver: "Assume `expr` is true without requiring proof."
  * This is a highly dangerous tool. If `expr` is actually false, `assume(false)` can lead to proving *any* statement (`assert(false)` would then succeed), fundamentally unsound proof.
  * **Primary use cases**:
      * **Temporary debugging**: To isolate proof failures, you might temporarily `assume` a complex lemma holds to allow verification of subsequent code. This `assume` **must be removed** for a final, sound proof.
      * **Trusting external contracts**: When interacting with unverified external code (e.g., OS calls), you might `assume` their documented postconditions. This transfers the burden of proof outside Verus.

### `proof { ... }` blocks

  * Introduces a block of **ghost code** (specifically `proof` mode) within an `exec` function.

  * Allows you to perform proof-specific operations without generating any runtime code.

  * Within a `proof { ... }` block, you can:

      * Declare ghost variables (`let g: int = ...;`).
      * Call `spec` functions and `proof` functions.
      * Write `assert` statements to guide the SMT solver.

  * **Fact Propagation**: Any new facts established via `assert` or by calling `proof fn`s (which have `ensures`) inside a `proof { ... }` block **propagate outwards** to the surrounding `exec` code's verification context. This means the `exec` code can then rely on these newly proven facts.

    ```rust
    fn my_exec_function(a: u32) {
        let x: u32 = a + 1; // Executable code

        proof { // Enter ghost context
            let ghost_val: int = x as int * 2; // Declare a ghost variable
            assert(ghost_val >= 0); // Assert a property of the ghost variable
            // call_some_lemma(ghost_val); // Call a proof function
        } // Exit ghost context; facts proven about 'x' might carry over
        assert(x >= a); // This assertion might be proven due to facts propagated from the proof block
    }
    ```

### `assert(expr) by { ... }`

  * Provides a **localized proof** for a **single `assert(expr)`** statement.

  * The code within the `by { ... }` block is a `proof` mode context whose **sole purpose** is to satisfy that specific `assert(expr)`.

  * **Crucial Difference from `proof { ... }`**: Facts established within the `by { ... }` block (e.g., by calling lemmas or adding assertions) **do not propagate** to the surrounding code outside that `assert` statement. They are purely for convincing the solver of `expr`.

    ```rust
    mod MyModule {
        use vstd::prelude::*;
        verus! {
            spec fn is_even(x: int) -> bool { x % 2 == 0 }
            proof fn lemma_even_plus_even_is_even(a: int, b: int)
                requires is_even(a), is_even(b),
                ensures is_even(a + b),
            {}
            fn test_function(a_exec: u32, b_exec: u32) {
                let a: int = a_exec as int;
                let b: int = b_exec as int;

                assert(is_even(a + b)) by { // Proof for *this specific assertion*
                    // The 'by' block context
                    lemma_even_plus_even_is_even(a, b); // This lemma call helps prove 'is_even(a+b)'
                    assert(a + b > 0); // This assertion's fact *does NOT* propagate outside this 'by' block
                }
                assert(is_even(a + b)); // This assertion passes because the previous 'assert by' proved it.
                // assert(a + b > 0); // THIS WOULD FAIL if not provable otherwise, because the fact from inside 'by' did not propagate.
            }
        }
    }
    ```

-----

## 📚 Recursion & Termination

Verus requires explicit proof of termination for recursive functions and loops.

### `decreases` Clauses

  * **All recursive functions** (whether `spec`, `proof`, or `exec`) **must** have a `decreases` clause.

  * **Tail-recursive `exec` functions** also require `decreases`.

  * The `decreases` clause specifies a well-founded metric (e.g., a natural number, a tuple of natural numbers) that strictly decreases with each recursive call or loop iteration, and remains non-negative. This guarantees termination and prevents infinite recursion/loops, which is essential for soundness.

    ```rust
    spec fn factorial(n: nat) -> nat
        decreases n // 'n' decreases with each recursive call
    {
        if n == 0 { 1 } else { n * factorial(n - 1) }
    }
    ```

### Fuel & `reveal_with_fuel`

  * The SMT solver works with limited computational resources, including a concept called **"fuel"** for unfolding recursive `spec fn` definitions.

  * By default, the solver has very little fuel (typically 1). This means it can only "unfold" the base case of a recursive `spec fn` automatically.

  * To enable the solver to reason about recursive calls beyond the base case, you might need to explicitly increase the fuel in your `proof` code using `reveal_with_fuel`. This effectively tells the solver to expand the function definition more times.

    ```rust
    spec fn triangle(n: nat) -> nat
        decreases n
    {
        if n == 0 { 0 } else { n + triangle(n - 1) }
    }

    proof fn test_triangle() {
        assert(triangle(3) == 6); // This might fail without enough fuel
        proof { reveal_with_fuel(triangle, 4); } // Give enough fuel to unfold triangle(3), triangle(2), triangle(1), triangle(0)
        assert(triangle(3) == 6); // Now it succeeds
    }
    ```

    This is generally used when you are proving properties about a `spec fn` itself, not just calling it.

-----

## 🧮 Integer Semantics

Verus carefully handles integer types to allow both precise mathematical reasoning and safe execution.

  * **Spec-level integers**:

      * `int`: Represents the set of all mathematical integers ($..., -2, -1, 0, 1, 2, ...$) with arbitrary precision. Operations on `int` never overflow or underflow. This is the default for numeric types in `spec` and `proof` code.
      * `nat`: Represents the set of natural numbers ($0, 1, 2, ...$). It is an `int` with an implicit `x >= 0` constraint. Useful for quantities that are inherently non-negative (e.g., lengths, counts).

  * **Exec-level integers**:

      * Standard Rust primitive integer types (`i8`, `u32`, `usize`, `i64`, etc.). These are fixed-bit-width and **can overflow or underflow** at runtime, leading to incorrect results or panics.
      * When performing arithmetic with `exec` integers, Verus will flag potential overflows/underflows unless you explicitly prove safety using `requires` clauses or `assert` statements that constrain the input values to safe ranges.
      * Verus uses `int` arithmetic in `spec` contexts (like `requires`, `ensures`) even when dealing with `exec` arguments, allowing you to precisely specify bounds.
        Example: `fn add(a: u8, b: u8) ensures (a as int + b as int) < 256 { ... }`

-----

## 🔗 Constants & Modes

Constants in Verus also follow the mode system, affecting their visibility and usage.

  * `spec const`: A constant that exists only in `spec` mode. It can be used in `spec` and `proof` code.

    ```rust
    spec const MAX_VALUE_SPEC: int = 1000;
    spec fn check_max(x: int) -> bool { x <= MAX_VALUE_SPEC }
    ```

  * `proof const`: A constant used exclusively for proofs, also existing only in ghost mode.

  * `exec const`: A standard Rust `const` that gets compiled into the runtime executable. It can have `ensures` clauses to link its runtime value to `spec` properties.

    ```rust
    exec const BUFFER_SIZE: usize
        ensures BUFFER_SIZE == 4096, // Verus proves this exec constant's value
    {
        4096
    }
    ```

  * **Default (no explicit mode)**: A `const` declared without `spec`, `proof`, or `exec` keywords is treated as **dual-use**. It can be used in both `exec` and `spec` contexts.

      * **Restriction**: Its type must be a compilable Rust type (no `int` or `nat`).
      * **Restriction**: It cannot call `exec` or `proof` functions (as it must be evaluable in both contexts).

    <!-- end list -->

    ```rust
    const DEFAULT_TIMEOUT_SECONDS: u64 = 30; // Dual-use
    fn do_something_with_timeout(timeout: u64)
        ensures timeout <= DEFAULT_TIMEOUT_SECONDS, // Used in spec context
    {
        // ...
        if timeout > DEFAULT_TIMEOUT_SECONDS { panic!("Timeout too long"); } // Used in exec context
    }
    ```

  * **Bridge `exec` to `spec`**: `#[verifier::when_used_as_spec(SPEC_DEF)]`
    If you have an `exec const` that is *not* dual-use (e.g., it has a complex type or calls an `exec fn` to initialize), but you need to refer to its value in a `spec` context, this attribute provides a mapping. It tells Verus: "When this `exec const` is encountered in a `spec` context, use the value/definition of `SPEC_DEF` (which must be a `spec const` or `spec fn`) instead."

    ```rust
    // A pure mathematical definition of usize::MAX
    spec fn spec_usize_max() -> int { 0xFFFF_FFFF_FFFF_FFFFu64 as int } // Example: Assuming 64-bit platform

    #[verifier::when_used_as_spec(spec_usize_max)] // Maps the exec const to its spec equivalent
    exec const USIZE_MAX_VALUE: usize = usize::MAX; // Standard Rust constant

    // Now, in a spec context, `USIZE_MAX_VALUE` will be treated as `spec_usize_max()`
    spec fn is_max_usize(val: int) -> bool { val == USIZE_MAX_VALUE }
    ```

-----

## 🧠 Verification Patterns & Strategies

Developing verified code in Verus involves specific architectural and proving strategies.

1.  **Contracts First (Design by Contract)**:

      * Before writing function bodies, define precise `requires` (preconditions) and `ensures` (postconditions). This forces clear thinking about inputs, outputs, and side effects.
      * This approach makes verification modular: you verify the function body against its own contract, and callers verify their inputs against the callee's contract.

2.  **Ghost Specifications (`spec fn`, `Seq`, `Set`, `Map`)**:

      * Use `spec fn`s to define abstract, mathematical properties and helper functions that describe program behavior at a high level.
      * Leverage `Seq`, `Set`, and `Map` (from `vstd`) for defining properties of collections in an abstract, unbounded way, independent of runtime representation.

3.  **Proof Organization**:

      * **Encapsulate lemmas**: For complex or reusable proof steps, define them as separate `proof fn`s. This modularizes your proofs and keeps your main function bodies cleaner.
      * **Inline steps**: Use `proof { ... }` blocks within `exec` functions for short, localized proof steps that benefit from visibility of `exec` variables.
      * **Scoped proofs**: Use `assert(expr) by { ... }` for very specific, self-contained proofs of individual assertions, preventing side effects on the surrounding verification context.

4.  **Loop Verification Discipline**:

      * Always state strong `invariant` clauses that capture the loop's progress and maintained properties.
      * Always provide a valid `decreases` metric to prove termination.

5.  **Modular Verification**:

      * Break down large verification problems into smaller, independent sub-problems (functions, lemmas).
      * Verus's contract system allows the verifier to trust a function's `ensures` if its `requires` are met, without needing to re-verify the function's internals at every call site. This drastically improves verification scalability.

-----

## ✅ Minimal Verus Test Pattern

For projects not relying on Rust's `#[test]` harness, use this structure to encapsulate your verification test cases. This structure ensures all properties are verified via Verus's static analysis.

```rust
/* TEST CODE BELOW */

// Typically, a single pub fn main() (or similar top-level exec function)
// is used to instantiate and demonstrate properties of your verified code.
pub fn main() {
    // 1. Instantiate ghost-enabled types and exec data.
    // Example: Using a Tracked<Permission> for a resource
    let tracked my_resource_perm = MyResource::allocate_permission(42); // Assumes `allocate_permission` is an exec fn returning Tracked<MyResourcePermission>
    let tracked duplicated_perm = my_resource_perm.duplicate(); // Assuming a method to duplicate permissions

    // 2. Use `assert` statements to verify properties of your data at various points.
    // Use `@` to get the `spec` view of `exec` or `Tracked` data.
    assert(my_resource_perm@.is_valid()); // Check a spec property of the resource's view
    assert(my_resource_perm@ == duplicated_perm@); // Verify equality of their spec views

    // 3. Call `proof` functions within `proof { ... }` blocks to bring lemmas into scope.
    // These calls help the SMT solver establish facts needed for subsequent assertions.
    proof {
        // Assuming MyResourcePermission has a lemma that helps reason about duplication
        MyResourcePermission::lemma_duplication_preserves_validity(my_resource_perm@);
        // This lemma call proves facts about `my_resource_perm@` that might be needed later.
    }

    // 4. Continue with more assertions, now benefiting from the facts established in the proof block.
    assert(my_resource_perm@.is_chosen()); // This might now be provable thanks to the lemma
}
```

**Final Checklist for a Verified Verus Module:**

  * [x] All `use` statements are placed **outside** any `verus! {}` blocks, at the top of the file.
  * [x] `requires` and `ensures` clauses consistently use the indented, comma-separated block style.
  * [x] All `while` and `for` loops have both `invariant` clauses and a `decreases` metric.
  * [x] No `#[test]`, `assert_eq!`, `dbg!`, or other Rust runtime test macros are used for verification purposes.
  * [x] All recursive functions (including tail-recursive `exec` functions) have `decreases` clauses to prove termination. Proofs of correctness for recursive functions are typically done via lemmas (`proof fn`).
  * [x] Abstract definitions and high-level properties are captured using `spec fn`.
  * [x] Proof steps and lemmas are encapsulated in `proof fn`.
  * [x] `exec` code interacts with `proof` code strictly within `proof { ... }` blocks or `assert(...) by { ... }` constructs.
  * [x] A clear `pub fn main()` (or similar top-level `exec` function) under `/* TEST CODE BELOW */` exists to instantiate scenarios and demonstrate verified properties via `assert` statements. This replaces traditional Rust testing.

Apply this guide rigorously to transform a Rust file into a fully verified Verus module. Happy verifying\!
"""

def generate_verus_tests(rust_file_path, spec_file_path):
    """
    Converts a Rust with Verus specs but Rust only units tests file to a complete Verus-annotated file with the corresponding unit tests.
    The Verus code overwrites the provided file at the spec_file_path in the `verus-test-cases' folder. 
    This function requires that be a /* TEST CODE BELOW */ line within the spec_file_path file. 
    Returns the filepath to the generated Verus file.
    """
    test_code = util.read_rust_file(rust_file_path) 
    test_idx = test_code.find("#[cfg(test)]")
    if test_idx != -1:
        test_code = test_code[:test_idx].rstrip()
    rust_code = util.read_rust_file(spec_file_path) + test_code + "\n} // verus!"
    
    if "/* TEST CODE BELOW */" not in rust_code:
        raise ValueError("Spec file must contain '/* TEST CODE BELOW */' marker.")

    prompt = f"""
    You are an expert Verus and Rust developer.

    Given the following Rust/Verus file with Verus specs and Rust unit tests, generate the file with the corresponding Verus versions of the rust unit tests, evaluating 
    the correctness of the functions in the file given the specs. All your edits should consist of removing the old Rust unit tests entirely and replacing 
    them with Verus unit tests in Verus syntax, which are assert statement based. Do so below the `/* TEST CODE BELOW */` comment which is guranteed to be in the file
    and change nothing else in the file.
    
    Here is the Rust file:

    {rust_code}
    
    Do not add explanations, summaries, or ```rust fences. Output only the raw Rust with Verus code. 
    
    If needed, see the provided Verus reference for details on how to transform Rust units tests into Verus from the specs and provided rust code: 
    
    Reference: 
    {REFERENCE} 
    
    (Use your own training knowledge to fill in any gaps in the Verus reference and to understand the Verus concepts. Any
    syntax related information in the guide must be followed strictly with no exception.) 
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=8000,
    )
    final_code = response.choices[0].message.content.strip()
    
    os.makedirs("verus-test-cases", exist_ok=True)
    base_filename = os.path.basename(rust_file_path)
    base_no_ext = os.path.splitext(base_filename)[0].split("_")[0] 
    output_path = os.path.join("verus-test-cases", f"{base_no_ext}_verus.rs")
    
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(final_code)
        
    print(f"Wrote Verus‑annotated file to {output_path}")
    return output_path

def check_syntax(output_path):
    """
    Accepts the filepath to a Rust file with Verus specs and unit tests and identifies lines with syntax errors. Stores the auto generated,
    error corrected file to the same filepath, overwriting the original file.
    """
    code = util.read_rust_file(output_path) 

    prompt = f"""
    You are an expert Verus and Rust developer.
    
    The code below is a Rust file with Verus specs and unit tests. Using your knowledge of Verus, identify any syntax errors in the code 
    and output an explanation of the lines with syntax errors, if any. If there are no Verus-specific syntax errors, output "No syntax errors found." Only
    return an explanations of the lines with syntax errors if there are genuine syntax errors, not just warnings or stylistic issues. 
    Write as if you are the friend of the developer who wrote this code.   
    
    Here is the Rust/Verus file:

    {code}
    
    Output only the desired text. 
    
    If needed, and only if needed, see the provided Verus reference for details on how to transform Rust units tests into Verus from the specs and provided rust code: 
    
    Reference: 
    {REFERENCE} 
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    error_analysis = response.choices[0].message.content.strip()
    
    prompt = f"""
    You are an expert Verus and Rust developer.
    
    The code below is a Rust file with Verus specs and unit tests.
    
    Here is the Rust/Verus file:

    {code}
    
    A friend has looked over the file and provided the following feedback on the Verus syntax errors present in the file:
    
    {error_analysis}
    
    Using your knowledge of Verus, make the appropriate changes, if any. Only make changes to the code if there are genuine Verus-specific syntax errors, 
    not just warnings or stylistic issues, and otherwise keep the rest of the code unchanged. If there are no Verus-specific syntax errors,
    do not make any changes to the code and simply return the original code as is. 
    
    IMPORTANT: Do not add explanations, summaries, or ```rust fences. Output only the raw Rust with Verus code. 
   
    If needed, and only if needed, see the provided Verus reference for details on how to transform Rust units tests into Verus from the specs and provided rust code: 
    
    Reference: 
    {REFERENCE} 
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    final_code = response.choices[0].message.content.strip()
    
    os.makedirs("verus-test-cases", exist_ok=True)
    base_filename = os.path.basename(output_path)
    base_no_ext = os.path.splitext(base_filename)[0] 
    output_path = os.path.join("verus-test-cases", f"{base_no_ext}_verus.rs")
    
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(final_code)
        
    print(f"Wrote Revised Verus‑annotated file to {output_path}")
    
if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python generate_tests.py <path_to_rust_file> <path_to_spec_file>")
        sys.exit(1)

    rust_file_path = sys.argv[1]
    spec_file_path = sys.argv[2]

    if not os.path.isfile(rust_file_path):
        print(f"Error: Rust file not found at {rust_file_path}")
        sys.exit(1)

    if not os.path.isfile(spec_file_path):
        print(f"Error: Spec file not found at {spec_file_path}")
        sys.exit(1)

    output_path = generate_verus_tests(rust_file_path, spec_file_path)
    check_syntax(output_path) 