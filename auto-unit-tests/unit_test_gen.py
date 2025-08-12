import os, sys, json
import openai
import generator_util as util 

COVERAGE_JSON_PATH = "coverage_results.json" 
TEST_STRING = "/* TEST CODE BELOW */" 
MAX_ITERATIONS = 6 # change as needed 

def initialize_tests(rust_file_path, base=False):
    """
    This function takes a rust_file with Verus specs and test cases. It removes Verus specs, test cases, and ghost code, and generates a new 
    Rust-only file with automatically generated unit tests. This is stored in test-cases folder. A copy of the file with Verus specs
    but no test cases is stored in verus-test-cases folder. If base is True, the function returns the path to the baseline Rust file without automatic generation, 
    the Rust file with tests, the total number of lines in the file, the number of lines, total amount of lines covered by tests, and a list of uncovered lines, used for evaluation and revision of the tests.
    Note: The base_path is None if base is False, otherwise it is the path to the baseline Rust file.
    """
    code = util.read_rust_file(rust_file_path)
    spec_code = code[:code.find(TEST_STRING) + len(TEST_STRING)]
    rust_code = util.strip_verus(spec_code[:-len(TEST_STRING)])
    if base:
        baseline = util.strip_verus(code, keep=True)
    
    # test code generation
    prompt = f"""
    You are an expert Rust developer.

    Given the following Rust file, generate a set of idiomatic Rust unit tests using the #[test] attribute. 
    Make sure tests cover all cases, including edge cases, and error handling if applicable. Do not call any library 
    not already imported in the file.

    Here is the Rust file:

    {rust_code}
    
    Provide only the Rust test code (inside a #[cfg(test)] mod tests {{ ... }} block). Ensure the output is valid and 
    compatible with cargo tarpaulin for coverage measurement:
    - Inside the test module, include use super::*; at the top
    - Each test function must return nothing (i.e., not -> i32, etc.)
    - All tests must actually call functions from the main code
    - No functions should be declared but unused
    - Make sure all edge cases (including those with very specific preconditions) are covered
    - Do not include ```rust fences or any explanation — only output code
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=5000,
    )
    test_code = response.choices[0].message.content.strip()
    
    # rust code cleanup 
    prompt = f"""
    You are an expert Rust developer.

    Given the following Rust file, provide the same code without lines which can never be run.
    Keep code which may seem unnecessary but could be used for error handling or verification, 
    and keep every line of the code exactly as is, with no modifications to the logic or 
    structure of the code. Only remove lines which truly cannot be tested, which should be very few 
    if any lines, such as the one line
    empty 
    ```rust
    pub fn main() {{
    }}
    ```
    function. 

    Here is the Rust file:

    {rust_code}

    Provide only the Rust code (and only the Rust code). In particular, do not ever include ```rust fences. 
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=5000,
    )
    rust_code = response.choices[0].message.content.strip()
    combined_code = rust_code.strip() + "\n\n" + test_code
    
    # prepare to store rust code with tests in test-cases folder
    os.makedirs("test-cases", exist_ok=True)
    base_filename = os.path.basename(rust_file_path)
    base_no_ext = os.path.splitext(base_filename)[0]
    output_path = os.path.join("test-cases", f"{base_no_ext}_tests.rs")
    
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(combined_code)
    
    # store spec file for transcompilation
    os.makedirs("verus-test-cases", exist_ok=True)
    spec_path = os.path.join("verus-test-cases", f"{base_no_ext}_spec.rs")
    with open(spec_path, "w", encoding="utf-8") as f:
        f.write(spec_code)
        print(f"Specs stored for transcompilation.")
        
    if base:
        # store baseline for comparison
        os.makedirs("baseline-test-cases", exist_ok=True)
        base_path = os.path.join("baseline-test-cases", f"{base_no_ext}_base.rs")
        with open(base_path, "w", encoding="utf-8") as f:
            f.write(baseline)
    else:
        base_path = None
        
    total, covered, uncovered_lines = util.get_rust_coverage(output_path)
    return base_path, output_path, total, covered, uncovered_lines

def revise_errors(output_path, error_info):
    """
    Takes a rust file at output_path and an error_info string, which is the relevant error information extracted from cargo tarpaulin.
    uses the error information to automatically fix the errors within the Rust file, and returns the new total and covered line counts,
    as well as coverage information (error or uncovered lines list).
    """
    rust_code = util.read_rust_file(output_path)
    prompt = f"""
    You are an expert Rust developer.
    
    The following Rust file has unit tests, but due to mistakes from removing Verus from the original file with both Verus and Rust, 
    the original file or generating unit tests, this file cannot be run with cargo tarpaulin for coverage measurement without 
    raising an error. 
    
    The following error information was extracted from cargo tarpaulin:
    {error_info}
    This is what is breaking the code and must be fixed. Using your knowledge of Rust, provide an explanation of what is wrong 
    with the code, as if critiquing a friend.
    Here is the Rust/Verus file:

    {rust_code}
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=8000,
    )
    error_analysis = response.choices[0].message.content.strip() 
    
    prompt = f"""
    You are roleplaying as a Rust compiler. 
    
    The following Rust file has unit tests, but due to mistakes from removing Verus from the original file with both Verus and Rust, 
    the original file or generating unit tests, this file cannot be run with cargo tarpaulin for coverage measurement without 
    raising an error. 
    
    The following error information was extracted from cargo tarpaulin:
    {error_info}
    This is what is breaking the code and must be fixed. Using your knowledge of Rust, pretend you are the compiler running the "cargo tarpaulin"
    command on this file and trace through the code until you hit the error outputted above. Add brief explanations between each step of the trace, using the trace to derive how the
    code is incorrect. Output the full trace, as if you were the compiler. Only output the trace, without returning contents from the original
    code file directly, and make sure you roleplaying that you are running the cargo tarpaulin command specifically on this file.
    Here is the Rust/Verus file:

    {rust_code}
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=8000,
    )
    error_trace = response.choices[0].message.content.strip() 
    
    prompt = f"""
    You are an expert Rust developer.
    
    The following Rust file has unit tests, but due to mistakes from removing Verus from the original file with both Verus and Rust, 
    the original file or generating unit tests, this file cannot be run with cargo tarpaulin for coverage measurement without 
    raising an error. 
    
    Here is the Rust with unit tests file:

    {rust_code} 
    
    Your friend has provided guidance on exactly what is wrong with the file, which is given below:
    
    {error_analysis}

    Your friend has also provided a trace of the code, if needed for further context, which is given below:
    {error_trace}
    
    Fix only the error(s) preventing cargo-tarpaulin from running, and leave all existing code, tests, imports, and structure unchanged.
    Return just the string of file contents, without ```rust fences or extra text, and return the entire file, not just 
    what you changed. To reiterate, no matter what, do not alter pre-existing tests or code beyond what is necessary to fix the error(s) 
    preventing cargo tarpaulin from running.
    """ 
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    rust_code = response.choices[0].message.content.strip()

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(rust_code)
    
    return util.get_rust_coverage(output_path)
    
def revise_tests(output_path, uncovered_lines):
    """
    Generates additional tests to cover uncovered lines in the Rust file.
    Returns new total and covered line counts, as well as the uncovered lines list.
    This function should be called only if total != covered and total != -1.
    """
    rust_code = util.read_rust_file(output_path)
    for line_idxs in uncovered_lines:
        trace, context_block = util.get_trace_from_lines(rust_code, line_idxs) 
        prompt = f"""
        You are an expert Rust developer.
        
        The following Rust file has unit tests, and but the unit tests miss some line(s) of code. In particular, the lines at indices
        {line_idxs[0]} (inclusive) to {line_idxs[1]} (exclusive) are not covered by any unit tests. 
        
        Those lines, along with up to two lines of surrounding context, are provided below:
        {context_block}
        
        Moreover, the how to reach those lines is given by the following trace:
        {trace}
        This trace is derived from the Rust file, and you should this understanding of how to reach those lines to consider which 
        if/else/specific preconditions and/or edge cases are not covered by the existing tests which has led to those uncovered line(s)
        at indices {line_idxs[0]} (inclusive) to {line_idxs[1]} (exclusive).
        
        **Append** new unit test(s) to cover exactly those line(s), but only if they are valid Rust code and compatible with cargo tarpaulin for coverage measurement:
        If you cannot add new test(s) without introducing errors, do not add any new test(s) and keep the file as is.
        
        Return the entire rust file with the new test(s) included. Keep everything else the same, including 
        the existing tests, imports, and structure of the file. Do not remove any existing tests or code.
        Return just the string of file contents, without ```rust fences or extra text. 
        Return the **full** Rust file with your additions in place. Do not alter or remove any existing tests or code, and omit any ```rust``` fences or 
        commentary, and make sure to only focus on the uncovered lines provided above. To reiterate, no matter what, do not alter
        pre-existing tests or code. You are only allowed to add new tests to cover the uncovered line(s). 
        
        Here is the Rust file:

        {rust_code} 
        
        As a reminder, the output must still be valid and compatible with cargo tarpaulin for coverage measurement:
        - Inside the test module, include use super::*; at the top
        - Each test function must return nothing (i.e., not -> i32, etc.)
        - All tests must actually call functions from the main code
        - No functions should be declared but unused
        - Make sure all edge cases (including those with very specific preconditions) are covered
        - Do not include ```rust fences or any explanation — only output code
        """
        response = openai.ChatCompletion.create(
            deployment_id=util.deployment_name,
            messages=[{"role": "user", "content": prompt}],
            max_completion_tokens=16000,
        )
            
        rust_code = response.choices[0].message.content.strip()

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(rust_code)
    
    return util.get_rust_coverage(output_path)

def save_coverage_result(file_id, coverage_percent):
    """
    Saves the coverage result of the current rust file to a JSON file.
    """
    if os.path.exists(COVERAGE_JSON_PATH):
        with open(COVERAGE_JSON_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
    else:
        data = {}

    # Update coverage
    data[file_id] = {"coverage": round(coverage_percent, 2)}
    with open(COVERAGE_JSON_PATH, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        
if __name__ == "__main__":
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print("Usage: python generate_tests.py <path_to_rust_file> [-b]")
        sys.exit(1)

    rust_file_path = sys.argv[1]
    run_baseline = len(sys.argv) == 3 and sys.argv[2] == "-b"

    if not os.path.isfile(rust_file_path):
        print(f"Error: File not found at {rust_file_path}")
        sys.exit(1)

    base_path, output_path, total, covered, line_info = initialize_tests(rust_file_path, base=run_baseline)
    
    # revise as needed 
    iter = 0 
    while (total > covered or total == -1) and iter < MAX_ITERATIONS:
        temp_covered = covered
        penalty = 1
        if total != -1:
            total, covered, line_info = revise_tests(output_path, line_info)
            if temp_covered == covered: 
                penalty = MAX_ITERATIONS // 3 # penalty for not improving coverage
            iter += penalty
        else:
            total, covered, line_info = revise_errors(output_path, line_info) 
            iter += penalty
        
    coverage = (covered / total) * 100
    print(f"Test coverage: {coverage:.2f}%")
    
    file_id = os.path.splitext(os.path.basename(output_path))[0]
    save_coverage_result(file_id, coverage)
    
    # baseline
    if run_baseline:
        base_total, base_covered, line_info = util.get_rust_coverage(base_path) 
        iter = 0 
        while base_total == -1 and iter < MAX_ITERATIONS:
            base_total, base_covered, line_info = revise_errors(base_path, line_info)
            iter += 1 
        
        coverage = (base_covered / base_total) * 100
        print(f"Baseline coverage: {coverage:.2f}%")
        file_id += "_baseline"
        save_coverage_result(file_id, coverage)