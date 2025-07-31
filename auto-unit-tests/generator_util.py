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
    Returns a tuple (total_lines, covered_lines). Should there be an error running
    coverage results, (-1, 0) is returned instead.
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
        # extract covered and total 
        output = result.stdout + "\n" + result.stderr
        lines = output.strip().splitlines()
        line = next(l for l in reversed(lines) if "lines covered" in l)
        parts = line.split(",")[1].strip().split()[0].split("/")
        covered, total = int(parts[0]), int(parts[1])
        
        # extract uncovered_lines list 
        uncovered_line = next((l for l in lines if l.startswith("Uncovered Lines:")), "")
        after_colon = uncovered_line.split(":", 1)[1].strip() if ":" in uncovered_line else ""
        if not after_colon:
            uncovered_lines = []
        else:
            uncovered_lines = [int(x.strip()) for x in after_colon.split(",") if x.strip()]

        return total, covered, uncovered_lines
    except Exception:
        import traceback
        print("Coverage extraction failed:")
        traceback.print_exc()
        return -1, 0, [] # -1 indicates error, 0 gives 0% coverage 
    finally:
        # remove temp cargo project
        shutil.rmtree(temp_dir, ignore_errors=True)
        
def read_rust_file(file_path: str) -> str:
    with open(file_path, "r", encoding="utf-8") as f:
        return f.read()
    
def get_lines_with_context(file_content, line_numbers):
    """
    Given the contents of a file and a list of 1-indexed line numbers,
    returns a list of strings, each including the line and up to 2 lines
    of surrounding context (before and after).
    """
    lines = file_content.splitlines()
    num_lines = len(lines)
    result = []

    for idx in line_numbers:
        start = max(idx - 3, 0)     
        end = min(idx + 2, num_lines) 
        context_block = lines[start:end]
        result.append('\n'.join(context_block))

    return result
 
def strip_verus(rust_code, keep=False):
    """
    Strips the Verus-specific specs and units tests from the Rust file.
    Returns the Rust code without Verus.
    TODO: make sure this runs without error. 
    """
    if keep:
        diff_line = "This does NOT include the units tests, which should be directly translated into cargo tarpaulin friendly rust test cases without distortion of the original content."
    else: 
        diff_line = "All existing unit tests (which will be regenerated)"
        
    prompt = f"""
    You are an expert Verus and Rust developer.

    Given the following Rust file with Verus specs and unit tests, 
    remove all Verus-specific syntax, attributes, and comments. This includes:

    - Any #[verus::*] attributes
    - Any use of `vstd::*` libraries or modules
    - Any functions, macros, or syntax specific to the Verus verifier
    - Any Verus-style ghost code or proof constructs
    - All Verus-only types like `Loc`, `nat`, `seq`, or others not defined in standard Rust
    - All calls to Verus-only functions such as `assert_by_contradiction`, `requires`, `ensures`, `proof`, etc.
    - Any `proof`, `tracked`, `spec`, or `ghost` keywords
    - Any inline proof blocks (e.g. `proof {{ ... }}`)
    - Any Verus-style custom traits or special attributes not supported in Rust
    - {diff_line}
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

    {rust_code}
    
    Return only the Rust code with Verus removed. Do not add explanations, summaries, or ```rust tags.
    Output only the raw Rust code.
    """
    response = openai.ChatCompletion.create(
        deployment_id=deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    return response.choices[0].message.content.strip()