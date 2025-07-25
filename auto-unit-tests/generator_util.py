import os, subprocess
import shutil
import tempfile 
import openai
from dotenv import load_dotenv

load_dotenv()

openai.api_type = os.getenv("OPENAI_API_TYPE") 
openai.api_key = os.getenv("OPENAI_API_KEY")
openai.api_base = os.getenv("OPENAI_API_BASE")
openai.api_version = os.getenv("OPENAI_API_VERSION")

deployment_name = os.getenv("AOAI_REFINEMENT_MODEL", "o3-mini")

def get_rust_coverage(rust_test_file_path):
    """
    Runs cargo tarpaulin to get coverage for the given Rust test file.
    Returns a tuple (total_lines, covered_lines).
    File at path must be valid Rust code without Verus to run without error.
    """
    # temporary directory to hold the Cargo project
    temp_dir = tempfile.mkdtemp()
    crate_name = "temp_coverage"
    crate_path = os.path.join(temp_dir, crate_name)
    src_path = os.path.join(crate_path, "src")
    os.makedirs(src_path)

    # copy input file into src/lib.rs
    shutil.copy(rust_test_file_path, os.path.join(src_path, "lib.rs"))

    # minimal cargo toml
    cargo_toml = f"""
    [package]
    name = "{crate_name}"
    version = "0.1.0"
    edition = "2021"

    [lib]
    path = "src/lib.rs"
    """
    with open(os.path.join(crate_path, "Cargo.toml"), "w") as f:
        f.write(cargo_toml.strip())

    # cargo tarpaulin
    try:
        result = subprocess.run(
            ["cargo", "tarpaulin", "--verbose"],
            cwd=crate_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=60,
        )
        output = result.stdout + "\n" + result.stderr
        lines = output.strip().splitlines()
        line = next(l for l in reversed(lines) if "lines covered" in l)
        parts = line.split(",")[1].strip().split()[0].split("/")
        covered = int(parts[0])
        total = int(parts[1])
        return total, covered
    except Exception:
        import traceback
        print("Coverage extraction failed:")
        traceback.print_exc()
    finally:
        # remove temp cargo project
        shutil.rmtree(temp_dir, ignore_errors=True)
        
def read_rust_file(file_path: str) -> str:
    with open(file_path, "r", encoding="utf-8") as f:
        return f.read()
    
def strip_verus(rust_code):
    """
    Strips the Verus-specific specs and units tests from the Rust file.
    Returns the Rust code without Verus.
    TODO: make sure this runs without error. 
    """
    prompt = f"""
    You are an expert Verus and Rust developer.

    Given the following Rust file with Verus specs and unit tests, 
    remove all Verus-specific syntax, attributes, and comments. This includes:

    - Any #[verus::*] attributes
    - Any use of `vstd::*` libraries or modules
    - Any functions, macros, or syntax specific to the Verus verifier
    - Any Verus-style ghost code or proof constructs
    - All existing unit tests (which will be regenerated)
    - All Verus-only types like `Loc`, `nat`, `seq`, or others not defined in standard Rust
    - All calls to Verus-only functions such as `assert_by_contradiction`, `requires`, `ensures`, `proof`, etc.
    - Any `proof`, `tracked`, `spec`, or `ghost` keywords
    - Any inline proof blocks (e.g. `proof {{ ... }}`)
    - Any Verus-style custom traits or special attributes not supported in Rust

    Retain ONLY the real, executable Rust code. This includes:

    - Type and struct definitions
    - Public API and implementation logic
    - Any standard Rust libraries (e.g. `std::result::*`)
    - Generic types, trait bounds, and idiomatic patterns used in real-world Rust
    - Functions like `alloc`, `duplicate`, or `validate_2` if they represent real logic (but strip any Verus wrappers)
    - Any match, if-let, or control flow logic used in safe, regular Rust

    Do NOT:

    - Add any new imports or dependencies
    - Introduce any new helper crates or custom macros
    - Modify or "clean up" formatting — preserve spacing, indentation, and structure as-is
    - Include any `verus!` blocks or macro-style wrapping from Verus

    Here is the Rust with Verus file: 

    ```rust
    {rust_code}
    ```
    
    Return only the Rust code with Verus removed. Do not add explanations, summaries, or ```rust tags.
    Output only the raw Rust code.
    """
    response = openai.ChatCompletion.create(
        deployment_id=deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    return response.choices[0].message.content.strip()

REFERENCE = """ ```rust
// REFERENCE

// Verus: A High-Level Overview

// The 'verus!' macro: Your entry point for verification.
// Encapsulates Rust code to be formally verified by Verus.
// Anything inside 'verus! { ... }' is under Verus's scrutiny.
// 'use vstd::prelude::*;' is the standard import for Verus-specific utilities.
verus! {
    // Verified code here
}

// ---
// Ghost Code vs. Executable Code: The Core Distinction

// Verus distinguishes between code that runs ('exec') and code purely for verification ('ghost').
// Ghost code adds *zero* runtime overhead.

// Executable Code ('exec' mode - default for functions):
// - Standard Rust code that compiles and runs on your machine.
// - Manipulates "physical" values in memory (e.g., Rust's fixed-size integers like u32, i8).
// - Can call 'spec' functions and, within 'proof { ... }' blocks, 'proof' functions.

// Ghost Code: Never compiled, only for static verification.
// 1. 'spec' mode (Specification Code):
//    - Defines *what* the program should do (its mathematical properties, functional definitions).
//    - Purely functional; no side effects, no mutable state.
//    - Can use arbitrary-precision 'int' and 'nat' types.
//    - Examples: 'spec fn', 'spec const'.
// 2. 'proof' mode (Proof Code):
//    - Defines *how* to prove that 'exec' code satisfies its 'spec' properties.
//    - Contains logical steps and assertions to guide the SMT solver.
//    - Can call 'spec' functions and other 'proof' functions.
//    - Examples: 'proof fn', 'proof const', 'proof { ... }' blocks.

// Mode Interaction Rules:
// - 'spec' functions can call other 'spec' functions.
// - 'proof' functions can call 'spec' functions and other 'proof' functions.
// - 'exec' functions can call 'spec' functions directly (their definitions are "visible").
// - 'exec' functions can invoke 'proof' functions, but *only* from within a 'proof { ... }' block or via 'assert(...) by { ... }'.
// - Neither 'spec' nor 'proof' code can call 'exec' functions or contain 'exec' logic.

// ---
// Contracts: 'requires' and 'ensures'

// Define the behavioral contract of a function. Crucial for modular verification.

// 1. Preconditions ('requires' clauses):
//    - Promises the *caller* must uphold before calling the function.
//    - If 'requires' are met, the function guarantees its 'ensures'.
//    - Verus verifies that every call site satisfies the called function's 'requires'.
//    Example:
fn octuple(x1: i8) -> i8
    requires -16 <= x1, x1 < 16, // Caller must ensure x1 is in this range to avoid overflow.
{
    let x2 = x1 + x1;
    let x4 = x2 + x2;
    x4 + x4
}

// 2. Postconditions ('ensures' clauses):
//    - Promises the *function* makes upon successful completion, assuming 'requires' were met.
//    - Verus verifies that the function body satisfies its 'ensures'.
//    - Callers can then *assume* the 'ensures' holds after the call.
//    Example:
fn octuple_verified(x1: i8) -> (x8: i8) // 'x8' names the return value for 'ensures'
    requires -16 <= x1, x1 < 16,
    ensures x8 == 8 * x1, // Function guarantees the return value is 8 times the input.
{
    let x2 = x1 + x1;
    let x4 = x2 + x2;
    x4 + x4
}

// ---
// Proof Assistant Tools: 'assert' and 'assume'

// 1. 'assert(expression)':
//    - Command to the SMT solver: "Prove 'expression' is true *right here*."
//    - If Verus cannot prove it, it's a verification error.
//    - Useful for: guiding the solver, debugging proofs, documenting intermediate facts.
//    - Always safe: prevents incorrect proofs from succeeding.

// 2. 'assume(expression)':
//    - Command to the SMT solver: "Take 'expression' as true *without proof*."
//    - **DANGEROUS**: Assuming a false statement allows Verus to "prove" anything.
//    - Use with extreme caution, primarily for temporary proof development or when trusting external components (e.g., OS calls).
//    - A sound, complete proof generally avoids 'assume'.

// ---
// Integer Types: 'int' and 'nat' (Ghost Only)

// Verus provides infinite-precision mathematical integers for specifications.
// 1. 'int': Represents all mathematical integers ($\dots, -2, -1, 0, 1, 2, \dots$).
//    - No overflow/underflow. Ideal for abstract arithmetic in 'spec' code.
// 2. 'nat': Represents natural numbers (integers $\ge 0$).
//    - An 'int' with an additional non-negativity constraint. Useful for lengths, indices.
// 'int' and 'nat' cannot be used in 'exec' code; use Rust's fixed-width types (u32, i64, etc.) there.
// Verus bridges this by verifying that 'exec' code's fixed-width arithmetic is safe given 'int' specifications (e.g., via 'requires').

// ---
// Syntactic Sugar for Specifications

// 1. Chained inequalities:
//    - `0 <= i <= j < len` is equivalent to `0 <= i && i <= j && j < len`. Concise.

// 2. '==>' (Implication):
//    - Logical "if...then": `A ==> B` means "if A is true, then B must be true."
//    - Equivalent to `!A || B`. Common in 'forall' statements.

// 3. '&&&' and '|||' (Triple And/Or):
//    - Lower precedence than `&&` and `||`.
//    - Used for visually structuring lists of conditions in 'requires' or 'ensures'.
//    Example:
//    requires
//        &&& input > 0
//        &&& input < 100
//        &&& input % 2 == 0;

// ---
// Equality: '==' vs. '=~='

// 1. '==' (Mathematical Equality):
//    - In Verus ghost code, '==' always means true mathematical equality (reflexive, symmetric, transitive).
//    - For primitive types (int, bool): value equality.
//    - For 'struct's/'enum's: recursively checks field equality.
//    - For 'Seq', 'Set', 'Map': checks extensional equality (same elements/mappings).
//    - *Note*: By default, Verus often promotes '==' to '=~=' inside 'assert', 'ensures', 'invariant' for collections.

// 2. '=~=' (Extensional Equality):
//    - Explicitly asserts that two collections (like 'Seq', 'Set', 'Map') contain the exact same elements/mappings, regardless of how they were constructed or their memory location.
//    - Useful when the SMT solver needs an explicit hint that two differently constructed collections are logically equivalent.
//    Example:
//    assert(seq![1,2] + seq![3] =~= seq![1,2,3]); // forces extensional check

// ---
// Datatypes: 'struct's and 'enum's

// 1. 'struct': Collects a set of named fields into a custom data type.
//    - Defines a blueprint for a record.
//    Example:
struct Point {
    x: int,
    y: int,
}
//    - Fields are accessed with the '.' operator (e.g., 'my_point.x').
//    - Methods can be defined on 'struct's using 'impl' blocks.
impl Point {
    spec fn len2(&self) -> int {
        self.x * self.x + self.y * self.y
    }
}

// 2. 'enum': Defines a type that can be any one of several defined variants.
//    - Variants can have named fields, tuple-like fields, or no fields (unit variants).
//    Example:
enum Beverage {
    Coffee { creamers: nat, sugar: bool },
    Soda { flavor: Syrup },
    Water { ice: bool },
}
enum Syrup { // Unit variants (tags only)
    Cola,
    RootBeer,
}
//    - Query variants: 'is' operator (e.g., 'my_bev is Soda').
//    - Access fields (if distinct names across variants): '->' operator (e.g., 'bev->creamers').
//    - Access tuple-like fields: '->' with index (e.g., 'shape->1' for second field).
//    - 'match' statement: Traditional Rust-like pattern matching for exhaustive handling.
//    - 'matches' syntax with '&&', '==>', '&&&': Powerful for inline pattern matching and binding variables within expressions.
//      Example: `l matches Mammal { legs, .. } && legs == 4` (binds 'legs' for subsequent use).

// ---
// Standard Library ('vstd') and Collections

// 'vstd' provides common utilities and datatypes for proofs.
// 'use vstd::prelude::*;' imports default useful definitions.

// Specification Libraries: 'Seq<T>', 'Set<T>', 'Map<Key, Value>'
// - Mathematical abstractions for sequences, sets, and maps.
// - Can be of arbitrary size (including infinite for 'Set' and 'Map').
// - No memory constraints or overflow concerns.
// - Constructed using macros ('seq![...]', 'set![...]', 'map![...]') or functions with closures ('Seq::new', 'Set::new', 'Map::new').
// - Support standard operations (e.g., '.len()', '.contains()', '.union()', '.intersect()', '.dom()').
// - Proving properties often requires inductive proofs and explicit use of '=~=' for extensional equality.

// Executable Libraries: 'Vec'
// - Verus supports Rust's 'std::vec::Vec'.
// - Connects 'Vec' (runtime) to 'Seq' (spec) using the '@' (view) operator.
// - 'v@' returns a 'Seq' view of the 'Vec' 'v', allowing 'spec' methods to be used on 'exec' data for verification.
//   Example: `assert(my_vec@.len() == 5);`

// ---
// 'spec' Closures (Anonymous Functions)

// - Anonymous functions defined inline in ghost code.
// - Type: 'spec_fn(arg_types) -> return_type'.
// - Follow 'spec' mode rules: purely functional, no side effects, only call 'spec' functions.
// - Useful for defining custom logic for collection construction or higher-order specifications.
// Example:
spec fn adder(x: int) -> spec_fn(int) -> int {
    |y: int| x + y // Closure capturing 'x'
}

// ---
// 'pub open' vs. 'pub closed' for 'spec fn'

// Manages visibility and "unfolding" of 'spec' function definitions across modules.
// - 'pub open spec fn': Definition is fully visible and "unfolded" by the SMT solver in other modules that import it.
//   - Useful for fundamental, universally-trusted definitions.
// - 'pub closed spec fn': Definition is *hidden* from other modules.
//   - Only the signature and any properties proven by 'proof fn's within the *same module* are visible to callers.
//   - Essential for abstraction, modularity, and managing proof complexity in large projects. Callers must rely on 'proof fn's (lemmas) to learn about its properties.

// ---
// 'proof { ... }' blocks vs. 'assert(...) by { ... }'

// Both provide local proof contexts within 'exec' functions.
// - 'proof { ... }' block:
//   - A general context for ghost code.
//   - Can declare ghost variables.
//   - Any facts proven inside propagate to the surrounding 'exec' code.
// - 'assert(...) by { ... }':
//   - A localized proof for a *specific assertion*.
//   - Proof steps and facts established inside the 'by { ... }' block are *confined* to proving *that single assertion* and do not propagate globally.
//   - Useful for tightly scoped, self-contained proofs.

// ---
// 'const' Declarations and Modes

// Constants can also have modes, affecting their visibility and behavior.
// - 'spec const': Ghost only, for specifications.
// - 'proof const': Ghost only, for proofs.
// - 'exec const': Standard Rust const, compiled to runtime. Can have 'ensures' clauses.
// - Default mode (no explicit keyword): Dual-use (both 'spec' and 'exec').
//   - Must use compilable types (no 'int'/'nat').
//   - Cannot call 'exec' or 'proof' functions.
// - #[verifier::when_used_as_spec(SPEC_DEF)]: Allows an 'exec const' to have a specific 'spec' interpretation when used in 'spec' contexts, bridging concrete and abstract values.

// ---
// The Verus Mindset

// - **Think in Contracts**: Precisely define what your functions require and guarantee.
// - **Leverage Ghost Code**: Write rich specifications without runtime cost.
// - **Embrace Mathematical Integers**: Use 'int'/'nat' for ideal arithmetic in specs, then prove fixed-width 'exec' arithmetic safe.
// - **Proof is a Process**: Use 'assert' to guide, 'assume' for temporary help (to be removed).
// - **Connect Abstract to Concrete**: Use the '@' view operator to link runtime data to mathematical models.
"""