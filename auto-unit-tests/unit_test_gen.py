import os, sys, json
import openai
import generator_util as util 

COVERAGE_JSON_PATH = "coverage_results.json" 

def initialize_tests(rust_file_path):
    # remove verus
    rust_code = util.strip_verus(util.read_rust_file(rust_file_path))
    
    # test code generation
    prompt = f"""
    You are an expert Rust developer.

    Given the following Rust file, generate a set of idiomatic Rust unit tests using the #[test] attribute. 
    Make sure tests cover all cases, including edge cases, and error handling if applicable. Do not call any library 
    not already imported in the file.

    Here is the Rust file:

    ```rust
    {rust_code}
    Provide only the Rust test code (inside a #[cfg(test)] mod tests {{ ... }} block). Ensure the output is valid and 
    compatible with cargo tarpaulin for coverage measurement:
    - Inside the test module, include use super::*; at the top
    - Each test function must return nothing (i.e., not -> i32, etc.)
    - All tests must actually call functions from the main code
    - No functions should be declared but unused
    - Do not include ```rust tags or any explanation — only output code
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
    function. 

    Here is the Rust file:

    ```rust
    {rust_code}
    Provide only the Rust code (and only the Rust code). In particular, do not ever include ```rust tags. 
    """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=5000,
    )
    rust_code = response.choices[0].message.content.strip()
    combined_code = rust_code.strip() + "\n\n" + test_code
    
    # prepare to store rust code with tests in test_cases folder
    os.makedirs("test_cases", exist_ok=True)
    base_filename = os.path.basename(rust_file_path)
    base_no_ext = os.path.splitext(base_filename)[0]
    output_path = os.path.join("test_cases", f"{base_no_ext}_tests.rs")
    
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(combined_code)
    
    total, covered = util.get_rust_coverage(output_path)
    return output_path, total, covered

def revise_tests(output_path, total, covered):
    """
    Generates additional tests to cover uncovered lines in the Rust file.
    Returns new total and covered line counts. 
    This function should be called only if total != covered.
    Should total == -1 (there was an error in previous steps), this function will attempt to fix 
    the file.
    """
    rust_code = util.read_rust_file(output_path)
     # test code generation
    if total == -1:
            prompt = f"""
        You are an expert Rust developer.
        
        The following Rust file has unit tests, but due to mistakes from removing Verus from the original file with both Verus and Rust, 
        the original file or generating unit tests, this file cannot be run with cargo tarpaulin for coverage measurement without 
        raising an error. 

        Identify the lines of code which you suspect most are causing the error, and only fix those lines. Keep 
        everything else the same, including the existing tests, imports, and structure of the file. Do not remove any existing tests or code. 
        Return just the string of file contents, without ```rust tags or extra text, and return the entire file, not just 
        what you changed.

        ```rust
        {rust_code} 
        """
    else: 
        prompt = f"""
        You are an expert Rust developer.
        
        The following Rust file has unit tests, and out of the {total} lines of code, every line is covered by an unit tests
        aside from {total - covered} uncovered lines. Identify the {total - covered} uncovered lines and generate new unit tests to cover them.

        Return the entire rust file with the new tests included. Keep everything else the same, including 
        the existing tests, imports, and structure of the file. Do not remove any existing tests or code.
        Return just the string of file contents, without ```rust tags or extra text. 
        
        Here is the Rust file:

        ```rust
        
        Ensure the output is still valid and compatible with cargo tarpaulin for coverage measurement:
        - Inside the test module, include use super::*; at the top
        - Each test function must return nothing (i.e., not -> i32, etc.)
        - All tests must actually call functions from the main code
        - No functions should be declared but unused
        - Do not include ```rust tags or any explanation — only output code
        {rust_code} 
        """
    response = openai.ChatCompletion.create(
        deployment_id=util.deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=5000,
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
    if len(sys.argv) != 2:
        print("Usage: python generate_tests.py <path_to_rust_file>")
        sys.exit(1)

    rust_file_path = sys.argv[1]

    if not os.path.isfile(rust_file_path):
        print(f"Error: File not found at {rust_file_path}")
        sys.exit(1)

    output_path, total_lines, covered_lines = initialize_tests(rust_file_path)
    if total_lines > covered_lines or total_lines == -1:
        total_lines, covered_lines = revise_tests(output_path, total_lines, covered_lines)
        
    coverage = (covered_lines / total_lines) * 100
    print(f"Test coverage: {coverage:.2f}%")
    
    file_id = os.path.splitext(os.path.basename(output_path))[0]
    save_coverage_result(file_id, coverage)