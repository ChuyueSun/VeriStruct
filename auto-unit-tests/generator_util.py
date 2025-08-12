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

def extract_uncovered_lines(s):
    """
    Helper function taking a string of comma-separated integers and ranges (e.g. "1, 2, 4-6")
    which is returned by cargo tarpaulin, and returns a list of [start, end] pairs (zero-based indices),
    each representing a slice range for lines.
    
    Examples:
    - "6"     -> [[5, 6]]   # corresponds to line 6 (0-based slice lines[5:6])
    - "9-12"  -> [[8, 12]]  # corresponds to lines 9,10,11,12 (slice lines[8:12])
    """
    res = []
    for part in s.split(','):
        part = part.strip()
        if not part:
            continue
        if '-' in part:
            a_str, b_str = part.split('-', 1)
            a, b = int(a_str), int(b_str)
            res.append([a - 1, b])  
        else:
            n = int(part)
            res.append([n - 1, n]) # single line as [start, end)
    return res

def get_rust_coverage(rust_test_file_path):
    """
    Runs cargo tarpaulin to get coverage for the given Rust test file.
    Returns a tuple (total_lines, covered_lines, uncovered_lines). Should there be an error running
    coverage results, (-1, 0, stderr output) is returned instead. Uncovered_lines is a list of lists of length 2 representing line number 
    ranges that were not covered by tests should the coverage tool run successfully. Otherwise, uncovered_lines is a string containing the relevant stderr output from the 
    coverage tool.
    File at path must be valid Rust code (no Verus) to run without error.
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
        
        # extract covered and total 
        output = result.stdout + "\n" + result.stderr
        lines = output.strip().splitlines()
        line = next(l for l in reversed(lines) if "lines covered" in l)
        parts = line.split(",")[1].strip().split()[0].split("/")
        covered, total = int(parts[0]), int(parts[1])
        
        # extract uncovered_lines list 
        uncovered_line = next((l for l in lines if l.startswith("Uncovered Lines:")), "")
        after_colon = uncovered_line.split(":", 1)[1].strip() if ":" in uncovered_line else ""
        uncovered_lines = extract_uncovered_lines(after_colon)

        return total, covered, uncovered_lines
    except Exception as e:
        import traceback
        print("Coverage extraction failed:") 
        traceback.print_exc()
        
        # convert exception to string
        if hasattr(e, "stderr") and e.stderr:
            stderr_text = e.stderr
        else:
            stderr_text = str(e)  
        
        # extract error information
        start_idx = stderr_text.find("error[")
        end_idx = stderr_text.find('Error: "', start_idx)
        if start_idx != -1 and end_idx != -1:
            extracted = stderr_text[start_idx:end_idx]
        else: # in unexpected case stderr is not properly formatted, return everything 
            extracted = stderr_text
        return -1, 0, extracted # -1 indicates error, 0 gives 0% coverage 
    finally:
        # remove temp cargo project
        shutil.rmtree(temp_dir, ignore_errors=True)
        
def read_rust_file(file_path: str) -> str:
    with open(file_path, "r", encoding="utf-8") as f:
        return f.read()
    
def get_trace_from_lines(file_content, line_idxs):
    """
    Given the contents of a file and a list of length 2 representing a line number range,
    returns an automatically trace (string) for how to reach those lines, as well the line(s) itself.
    """
    lines = file_content.splitlines()
    
    start_index, end_index  = max(0, line_idxs[0] - 1), min(len(lines), line_idxs[1] + 1)
    context_block = lines[start_index:end_index]
    
    prompt = f"""
    You are roleplaying as a Rust compiler. 
    
    The following Rust file has unit tests, but the following lines(s) are uncovered by the tests:
    {context_block}
    
    Up to two lines of surrounding context are provided above, in addition to the uncovered lines themselves. 
    
    Imagine having to reach this block of lines in the code, and provide a trace of how you would reach these line(s). You are the 
    compiler running the Cargo tarpaulin coverage tool on this file. 
    The full file contents are given below, and you should use this to derive the trace:
    {file_content}
    
    Based on the trace and/or the specific preconditions required for the line(s) to execute, add a brief explanation for 
    exactly what the missing unit test must do. 
 
    Return this trace and the short explanation, and just this trace and the short explanation, without directly returning the 
    contents of the original file. Make sure to roleplay as a Rust compiler running the Cargo tarpaulin command (specifically)
    on this file. 
    """
    response = openai.ChatCompletion.create(
        deployment_id=deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    trace = response.choices[0].message.content.strip() 

    return trace, context_block
 
def strip_verus(rust_code):
    """
    Strips the Verus-specific specs and units tests from the Rust file.
    Returns the Rust code without Verus.
    TODO: make sure this runs without error. 
    """
    prompt = f"""
    You are an expert Verus and Rust developer.

    Given the following Rust file with Verus specs (and possibly unit tests), your task is to remove all Verus-specific syntax, attributes, and comments. 
    Remove only the Verus. Do not remove any part of the file that is that is Rust, even if the Rust code is not 
    executable. What should be removed includes:

    - Any #[verus::*] attributes
    - Any use of `vstd::*` libraries or modules
    - Any functions, macros, or syntax specific to the Verus verifier
    - Any Verus-style ghost code or proof constructs
    - All Verus-only types like `Loc`, `nat`, `seq`, or others not defined in standard Rust
    - All calls to Verus-only functions such as `assert_by_contradiction`, `requires`, `ensures`, `proof`, etc.
    - Any `proof`, `tracked`, `spec`, or `ghost` keywords
    - Any inline proof blocks (e.g. `proof {{ ... }}`)
    - Any Verus-style custom traits or special attributes not supported in Rust
    Retain ONLY real Rust code. This includes:

    - Type and struct definitions
    - Public API and implementation logic
    - Any standard Rust libraries (e.g. `std::result::*`)
    - Generic types, trait bounds, and idiomatic patterns used in real-world Rust
    - Functions like `alloc`, `duplicate`, or `validate_2` if they represent real logic (but strip any Verus wrappers)
    - Any match, if-let, or control flow logic used in safe, regular Rust
    - Verus syntax which is part of Rust function/structure of the code (i.e. not verification-specific), with a Rust equivalent that you must translate to
    - Rarely, Rust constructs with names which may be named after Verus names but are actually valid Rust identifiers

    Do NOT:

    - Add any new imports or dependencies
    - Introduce any new helper crates or custom macros
    - Modify or "clean up" formatting — preserve spacing, indentation, and structure as-is
    - Remove any real Rust, non-Verus code that is not an unit test
    - Include any `verus!` blocks or macro-style wrapping from Verus

    Here is the Rust with Verus file: 

    {rust_code}
    
    It is critical that your final code is a direct, faithful representation of the original Rust code, minus the Verus-specific syntax and constructs.
    You may not add, remove, or distort any part of the file which is essential to its function or original structure. Either remove additional Verus interspersed
    with the Rust or translate Verus versions of Rust syntax essential to the file's function or structure to their Rust equivalents.
    Return only the Rust code with each piece of Verus removed or translated to their Rust equivalent, as needed for a direct translation. 
    Do not add explanations, summaries, or ```rust fences.
    Output only the raw Rust code. Fix the spelling and change nothing else. 
    """
    response = openai.ChatCompletion.create(
        deployment_id=deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    rust_code = response.choices[0].message.content.strip()
    
    prompt = f"""
    You are an expert Rust developer.

    Here is the Rust file: 

    {rust_code}
    
    Identify any lines of code that cannot possibly in any way be useful to a human while at the same time cannot ever be reached
    through code. Primarily, remove empty functions that are never called. Do not remove anything else code unless absolutely, utterly
    confident it has no function. Otherwise, besides removing empty functions which are never called, keep the code exactly as is,
    including any comments, imports, and structure of the file.
    Output only raw Rust code. To reiterate, output only raw Rust code. Do not add explanations, summaries, or ```rust fences. 
    """
    response = openai.ChatCompletion.create(
        deployment_id=deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    
    return response.choices[0].message.content.strip()

CARGO_TARPAULIN_REFERENCE = """
Here are five common, general pitfalls that tend to trip up **cargo-tarpaulin** and cause it to either report zero coverage or even fail altogether:

1. **Panics or Early Exits in Test Code**
   Any `panic!()` (including via `unwrap()`) inside your tests will abort the test binary immediately—tarpaulin can’t collect coverage after that point. Make sure you either handle errors gracefully or isolate panicking cases so they don’t cut the whole harness short.

2. **Tests Outside of a `#[cfg(test)] mod tests { … }`**
   If you define test functions at the top level (or forget the `#[test]` attribute), tarpaulin won’t see or run them. Always wrap helper tests in a `#[cfg(test)] mod tests { … }` and annotate each test fn with `#[test]`.

3. **Multiple `main()` Functions / Binary Targets**
   When you have more than one `fn main()` (e.g. in multiple files under `src/`), tarpaulin may pick the wrong target or fail to instrument any of them. Keep a single binary entry point per crate, or explicitly tell tarpaulin which binary to run via `--bin`.

4. **Conditional Compilation Excluding Code Paths**
   If you guard code behind feature flags or `#[cfg(...)]` that aren’t enabled during tarpaulin’s build, that code will never be compiled or counted. Double-check that you’re not accidentally excluding large swaths of logic via `#[cfg(debug_assertions)]`, `#[cfg(feature = "...")]`, etc.

5. **Infinite Loops or Long-Running Tests**
   Any test that never returns (e.g. a `while true { … }` or waiting on I/O) will hang tarpaulin’s runner, causing a timeout or a misleading coverage report. Keep your tests deterministic and bounded in time.
"""