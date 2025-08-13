import os
import openai
from dotenv import load_dotenv
import re, unicodedata

load_dotenv()

openai.api_type = os.getenv("OPENAI_API_TYPE") 
openai.api_key = os.getenv("OPENAI_API_KEY")
openai.api_base = os.getenv("OPENAI_API_BASE")
openai.api_version = os.getenv("OPENAI_API_VERSION")

deployment_name = os.getenv("AOAI_REFINEMENT_MODEL", "o3-mini")

HEADER_RE = re.compile(r'^(?:requires|ensures|invariant)\b$')
VERUS_REMOVE_PROMPT = """
You are an expert Verus and Rust developer.

Given the following Rust file possibly with Verus syntax/code, your task is to identify and remove any remaining Verus 
syntax. Note that the file has already gone through several rounds of Verus removal, so there may be very few or zero 
lines of Verus code left. This is a light double check to ensure nothing is missed. You must **not** remove anything 
unnecessary, create empty functions, or delete lines for no reason. If there are leftover Verus constructs/syntax, remove them
or translate them minimally and precisely. You may consider converting rare special cases like `assert` to `assert!`, but do 
not remove or alter anything else unless it is Verus-specific syntax that has been missed in the previous checks. 

The original code structure and functionality must remain fully intact. Even if they may seem useless to you, so long
as the code is valid Rust, it must remain unchanged. Once again, do not remove anything unless you know it is 
Verus syntax that either needs to be removed or translated to a Rust equivalent, depending on context. More likely than not,
Verus ONLY syntax must be entirely removed, unless truly essential to the Rust ONLY structure/function of the code, such as assert() tp
assert!(). It is critical that Rust code part of the Rust ONLY structure/function of the file is kept as is without removal or alteration, 
and Verus-specific constructs are either removed (most likely) or translated to their closest Rust equivalent if part of the Rust ONLY structure/function 
of the file. 

The rule is to pretty much completely remove Verus code except for the following exceptions in the 
:Translate instead of removing List::
- assert(expr)             → assert!(expr)
- assert(expr, by { ... }) → assert!(expr)  # drop proof block
- assume(expr)             → debug_assert!(expr)  # optional, for runtime checks
- pub fn f() -> (ret: T)   → pub fn f() -> T   # remove the `(ret: ...)` syntax
- pub fn f(...) ensures ... → pub fn f(...)    # remove ensures clauses
- old(var)                 → var.clone() or var as needed for runtime
which are Verus constructs that are simply Rust syntax part of the Rust code structure/function of the file being verified but 
written in Verus syntax due to using Verus. Any exceptions made to removing Verus and translating to Rust instead must refer to
the above list and any example outside of the list closely following the pattern of examples in the list. Of course, do not
remove any code that is Rust and not on this list, as Rust code must remain intact.

If needed for reference, Verus syntax includes:

1. Verus container and imports  
   - `verus! {{ ... }}` containers (including start and matching closing lines like `}} // verus!`)  
   - Any `use vstd::*` imports or references
   - Any leftover middle of the file uses of functions or macros from one of the vstd modules which are not Rust

2. Verus function qualifiers and signatures  
   - `spec fn`, `closed spec fn`, `open spec fn` function definitions and bodies  
   - `proof fn` and `pub proof fn` definitions and bodies  
   - `tracked fn`, `ghost fn` if any  
   - Remove full function bodies of these or translate to minimal placeholders (e.g., `unimplemented!()`), preserving API shape if needed

3. Function clauses and contracts  
   - `requires`, `ensures`, `invariant` clauses attached to function signatures or immediately before function bodies  
   - Inline clauses like `requires(...)` or `ensures(...)` on the same line as `fn`  
   - Multi-line clause blocks preceding `{{`  

4. Verus attributes and macros  
   - All `#[verifier::...]` attributes (e.g., `#[verifier::external_body]`, `#[verifier::external_fn_specification]`, `#[verifier::type_invariant]`)  
   - Verus-specific macros like `admit()`, `assert_by_contradiction`, `proof`, `tracked`, `ghost` blocks and inline blocks

5. Keywords and blocks  
   - `proof {{ ... }}`, `tracked {{ ... }}`, `ghost {{ ... }}` blocks anywhere in code, including inline and nested  
   - `tracked var => {{ ... }}` and `ghost var => {{ ... }}` forms  
   - Inline `proof {{ ... }}` blocks inside functions or methods

6. Verus-only types, traits, and quantifiers
   - Types like `int`, `nat`, `seq`, `Loc`, `Spec`, and custom Verus traits such as `View` if they cannot be translated or removed  
   - 'forall' quantifiers, `exists` expressions, or any other quantifier in Verus context
   - Replace with standard Rust types if possible or remove if purely Verus-specific

7. Placeholders and proof helpers  
   - Replace or remove `admit()` calls, ideally converting to `unimplemented!()` or removing if not needed  
   - Remove or translate `lemma_*` proof functions

Additional notes:  
- Preserve valid Rust syntax and idiomatic code  
- This includes real functions representing logic (even if named after Verus concepts)
- Do not remove any standard Rust functions, types, or valid code  
- When removing blocks or functions, ensure no dangling braces or syntax errors remain  
- Minimize impact on code readability and structure; prefer minimal stub replacements over deletion when possible  

This checklist ensures **every Verus-specific construct** mentioned is accounted for in your final Verus removal pass.

Return only the Rust code with any leftover Verus syntax, if at all, removed or translated, depending on which
provides the most faithul direct Rust equivalent. Do not add explanations, summaries, or Rust ''' code fences.

Once again, if no Verus code remains, return the original input unchanged. Output only the raw Rust code.
"""
    
def remove_container(rust_code):
    """verus! { ... } container removal, along with any vstd imports. 
    If baseline flag is off, only the left side of the container is removed. 
    Returns the Rust without the Verus container. 
    """
    start_idx = rust_code.find("verus! {")
    prefix = rust_code[:start_idx + len("verus! {")]
    rest = rust_code[start_idx + len("verus! {"):]
    lines = prefix.splitlines()
    res_lines = [] 
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("use vstd::") or stripped == "verus! {" or stripped == "verus!{":
            continue
        res_lines.append(line)
    prefix = "\n".join(res_lines)

    if "} // verus!" in rest:
        end_idx = rest.find("} // verus!")
        rest = rest[:end_idx]
    return prefix + rest.strip()

import re

def parse_functions(rust_code):
    lines = rust_code.splitlines(keepends=True)  # keep line endings for exact concat
    chunks = []
    current_non_func_lines = []

    def is_header_line(line):
        # Remove strings inside quotes
        line_no_str = re.sub(r'"(\\.|[^"\\])*"', '', line)
        # Remove inline comments starting with //
        line_no_comment = line_no_str.split('//')[0].strip()
        if not line_no_comment:
            return False
        # Check if 'fn' is present as a whole word outside strings/comments
        return bool(re.search(r'\bfn\b', line_no_comment))

    i = 0
    n = len(lines)

    while i < n:
        line = lines[i]

        if is_header_line(line):
            # Flush current non-func chunk if any
            if current_non_func_lines:
                chunks.append(''.join(current_non_func_lines))
                current_non_func_lines = []

            func_lines = [line]
            i += 1

            # Case 1: '{' is on the same line as header
            if '{' in line:
                chunks.append(''.join(func_lines))
                continue

            # Case 2: accumulate lines until a line that is exactly '{'
            while i < n:
                l = lines[i]
                if l.strip() == '{':
                    func_lines.append(l)
                    i += 1
                    break
                func_lines.append(l)
                i += 1

            chunks.append(''.join(func_lines))

        else:
            current_non_func_lines.append(line)
            i += 1

    # Flush last non-func chunk if any
    if current_non_func_lines:
        chunks.append(''.join(current_non_func_lines))

    return chunks

def remove_specs_from_chunk(func_chunk):
    spec_keywords = ['requires', 'ensures', 'invariant', 'recommends']

    def clean_text_for_spec_check(text: str) -> str:
        # Remove strings
        text = re.sub(r'"(\\.|[^"\\])*"', '', text)
        # Remove comments
        text = re.sub(r'//.*', '', text)
        text = re.sub(r'/\*.*?\*/', '', text, flags=re.DOTALL)
        # Replace all non-alpha characters with space
        text = re.sub(r'[^a-zA-Z]+', ' ', text)
        return text.strip()

    lines = func_chunk.splitlines(keepends=True)

    # Find the line with only '{' (after stripping spaces)
    brace_line_idx = None
    for idx, line in enumerate(lines):
        if line.strip() == '{':
            brace_line_idx = idx
            break

    if brace_line_idx is None or brace_line_idx <= 0:
        # No '{' found or nothing between header and brace
        return func_chunk

    # Separate lines between header and brace line
    between_lines = lines[1:brace_line_idx]

    comment_lines = []
    non_comment_lines = []

    for line in between_lines:
        stripped = line.strip()
        if stripped.startswith('//') or stripped.startswith('///') or stripped.startswith('/*') or stripped.endswith('*/'):
            comment_lines.append(line)
        else:
            non_comment_lines.append(line)

    combined_text = ' '.join(non_comment_lines)
    cleaned_text = clean_text_for_spec_check(combined_text)
    first_word = cleaned_text.split()[0] if cleaned_text else ''

    if first_word in spec_keywords:
        # Remove all non-comment lines between header and '{'
        new_chunk = ''.join([lines[0]] + comment_lines + [lines[brace_line_idx]] + lines[brace_line_idx+1:])
        return new_chunk
    else:
        return func_chunk
    
def remove_specs(rust_code):
    chunks = parse_functions(rust_code)
    output_chunks = []

    for chunk in chunks:
        first_line = chunk.splitlines(keepends=False)[0]

        # Strict function detection: remove strings and inline comments first
        line_no_str = re.sub(r'"(\\.|[^"\\])*"', '', first_line)
        line_no_comment = line_no_str.split('//')[0]

        # Only if 'fn' is anywhere outside strings/comments, it's a function chunk
        if 'fn' in line_no_comment:
            # Process the function chunk with the helper
            new_chunk = remove_specs_from_chunk(chunk)
            output_chunks.append(new_chunk)
        else:
            # Non-function chunk, leave as is
            output_chunks.append(chunk)

    return ''.join(output_chunks)

def remove_tracked_ghost_proof_blocks(rust_code):
    n = len(rust_code)
    i = 0
    out = []
    
    in_string = False
    string_delim = ''
    in_line_comment = False
    in_block_comment = False

    def is_id_char(c):
        return c.isalnum() or c == '_'

    def skip_whitespace(idx):
        while idx < n and rust_code[idx].isspace():
            idx += 1
        return idx

    def match_keyword_at(idx, kw):
        # must start with kw
        if not rust_code.startswith(kw, idx):
            return False
        start_ok = (idx == 0) or (not is_id_char(rust_code[idx-1]))
        end_idx = idx + len(kw)
        end_ok = (end_idx == n) or (not is_id_char(rust_code[end_idx]))
        return start_ok and end_ok

    def prev_backslashes_count(pos):
        cnt = 0
        k = pos - 1
        while k >= 0 and rust_code[k] == '\\':
            cnt += 1
            k -= 1
        return cnt

    while i < n:
        c = rust_code[i]
        
        # Handle string literals
        if in_string:
            out.append(c)
            if c == string_delim and (prev_backslashes_count(i) % 2 == 0):
                in_string = False
            i += 1
            continue
        
        # Handle line comments
        if in_line_comment:
            out.append(c)
            if c == '\n':
                in_line_comment = False
            i += 1
            continue
        
        # Handle block comments
        if in_block_comment:
            out.append(c)
            if c == '*' and i+1 < n and rust_code[i+1] == '/':
                out.append(rust_code[i+1])
                in_block_comment = False
                i += 2
            else:
                i += 1
            continue
        
        # Outside string/comment: detect start of strings or comments
        if c == '"' or c == "'":
            in_string = True
            string_delim = c
            out.append(c)
            i += 1
            continue
        
        if c == '/' and i+1 < n:
            if rust_code[i+1] == '/':
                in_line_comment = True
                out.append(c)
                out.append(rust_code[i+1])
                i += 2
                continue
            elif rust_code[i+1] == '*':
                in_block_comment = True
                out.append(c)
                out.append(rust_code[i+1])
                i += 2
                continue
        
        # Now check for tracked, ghost, proof keywords:
        for kw in ("tracked", "ghost", "proof"):
            if match_keyword_at(i, kw):
                # We've found a block start candidate.
                # Check what's after the keyword:
                j = i + len(kw)
                j = skip_whitespace(j)
                
                # For tracked and ghost, optionally consume var => 
                if kw in ("tracked", "ghost"):
                    # if next tokens are id and =>, consume them
                    start_j = j
                    # parse an optional identifier
                    while j < n and is_id_char(rust_code[j]):
                        j += 1
                    j = skip_whitespace(j)
                    if j+1 < n and rust_code[j] == '=' and rust_code[j+1] == '>':
                        j += 2
                        j = skip_whitespace(j)
                    else:
                        j = start_j  # rollback, no var =>
                
                # Next token should be {
                if j < n and rust_code[j] == '{':
                    # Skip entire block from i to matching }
                    brace_level = 1
                    j += 1
                    while j < n and brace_level > 0:
                        if rust_code[j] == '"' or rust_code[j] == "'":
                            # skip string inside block
                            delim = rust_code[j]
                            j += 1
                            while j < n:
                                if rust_code[j] == delim and ( (j == 0) or (prev_backslashes_count(j) % 2 == 0) ):
                                    j += 1
                                    break
                                j += 1
                            continue
                        if rust_code[j] == '/' and j+1 < n:
                            if rust_code[j+1] == '/':
                                # skip line comment inside block
                                j += 2
                                while j < n and rust_code[j] != '\n':
                                    j += 1
                                continue
                            elif rust_code[j+1] == '*':
                                # skip block comment inside block (defensive for unterminated comments)
                                j += 2
                                while j+1 < n and not (rust_code[j] == '*' and rust_code[j+1] == '/'):
                                    j += 1
                                if j+1 >= n:
                                    # unterminated block comment — move to end and break
                                    j = n
                                    break
                                j += 2
                                continue
                        if rust_code[j] == '{':
                            brace_level += 1
                        elif rust_code[j] == '}':
                            brace_level -= 1
                        j += 1
                    i = j
                    break
                else:
                    # No brace after keyword: output the whole keyword and move on
                    out.append(rust_code[i:i+len(kw)])
                    i += len(kw)
                    break
        else:
            # No keyword matched here
            out.append(c)
            i += 1

    return "".join(out)

def remove_spec_functions(rust_code):
    """
    Removes spec functions from the Rust code.
    Returns the cleaned up Rust code with spec functions removed.
    """
    code = rust_code

    # States for parsing
    IN_CODE, IN_LINE_COMMENT, IN_BLOCK_COMMENT, IN_STRING = 0, 1, 2, 3

    def is_keyword_spec_in_signature(line):
        return re.search(r"\b(?:\w+\s+)*(?:spec|proof)\s+fn\b", line) is not None

    result_lines = []
    lines = code.splitlines()
    n = len(lines)

    brace_stack = []
    skip_mode = None  # None or 'spec_func'

    # Track raw-string state across lines for strip_strings_comments
    raw_open = False
    raw_hashes = 0

    i = 0
    while i < n:
        line = lines[i]

        def strip_strings_comments(s):
            nonlocal raw_open, raw_hashes
            out = []
            st = IN_CODE
            i_c = 0

            def backslashes_before(sts, pos):
                cnt = 0
                k = pos - 1
                while k >= 0 and sts[k] == '\\':
                    cnt += 1
                    k -= 1
                return cnt

            if raw_open:
                end_search = 0
                found_end = False
                while end_search < len(s):
                    if s[end_search] == '"':
                        m = end_search + 1
                        matched_hashes = 0
                        while m < len(s) and matched_hashes < raw_hashes and s[m] == '#':
                            matched_hashes += 1
                            m += 1
                        if matched_hashes == raw_hashes:
                            span_len = m
                            out.append(' ' * span_len)
                            i_c = m
                            raw_open = False
                            raw_hashes = 0
                            found_end = True
                            break
                        else:
                            end_search = m
                    else:
                        end_search += 1
                if not found_end:
                    out.append(' ' * len(s))
                    return ''.join(out)
            while i_c < len(s):
                c = s[i_c]

                if st == IN_CODE:
                    if c == 'r' and i_c + 1 < len(s) and (s[i_c+1] == '"' or s[i_c+1] == '#'):
                        k = i_c + 1
                        hash_count = 0
                        while k < len(s) and s[k] == '#':
                            hash_count += 1
                            k += 1
                        if k < len(s) and s[k] == '"':
                            end_search = k + 1
                            found_end = False
                            while end_search < len(s):
                                if s[end_search] == '"':
                                    m = end_search + 1
                                    matched_hashes = 0
                                    while m < len(s) and matched_hashes < hash_count and s[m] == '#':
                                        matched_hashes += 1
                                        m += 1
                                    if matched_hashes == hash_count:
                                        span_len = m - i_c
                                        out.append(' ' * span_len)
                                        i_c = m
                                        found_end = True
                                        break
                                    else:
                                        end_search = m
                                else:
                                    end_search += 1
                            if not found_end:
                                raw_open = True
                                raw_hashes = hash_count
                                out.append(' ' * (len(s) - i_c))
                                return ''.join(out)
                            continue
                    if c == '"' or c == "'":
                        out.append(' ')
                        st = IN_STRING
                        string_delim = c
                        i_c += 1
                        continue
                    if c == '/' and i_c + 1 < len(s) and s[i_c+1] == '/':
                        break
                    if c == '/' and i_c + 1 < len(s) and s[i_c+1] == '*':
                        st = IN_BLOCK_COMMENT
                        out.append('  ')
                        i_c += 2
                        continue
                    out.append(c)
                elif st == IN_STRING:
                    if c == '\\' and i_c + 1 < len(s):
                        out.append('  ')
                        i_c += 2
                        continue
                    if c == '"' or c == "'":
                        if backslashes_before(s, i_c) % 2 == 0:
                            st = IN_CODE
                            out.append(' ')
                            i_c += 1
                            continue
                    out.append(' ')
                elif st == IN_BLOCK_COMMENT:
                    if c == '*' and i_c + 1 < len(s) and s[i_c+1] == '/':
                        out.append('  ')
                        i_c += 2
                        st = IN_CODE
                        continue
                    out.append(' ')
                i_c += 1
            return ''.join(out)

        stripped_for_check = strip_strings_comments(line).strip()

        if skip_mode is None:
            if is_keyword_spec_in_signature(stripped_for_check):
                skip_mode = 'spec_func'

                # Remove preceding comment lines
                while result_lines and result_lines[-1].strip().startswith("//"):
                    result_lines.pop()

                clean = strip_strings_comments(line)
                open_braces = clean.count('{')
                close_braces = clean.count('}')
                net_braces = open_braces - close_braces

                brace_stack = [net_braces] if net_braces > 0 else []

                i += 1
                continue

            result_lines.append(line)
            i += 1

        else:
            clean = strip_strings_comments(line)
            open_braces = clean.count('{')
            close_braces = clean.count('}')
            net_braces = open_braces - close_braces

            if brace_stack:
                brace_stack[0] += net_braces
            else:
                if net_braces > 0:
                    brace_stack = [net_braces]

            if brace_stack and brace_stack[0] <= 0:
                skip_mode = None
                brace_stack = []

            i += 1

    return "\n".join(result_lines)

def final_check(rust_code):
    """
    Final double check for leftover Verus syntax in the Rust file.
    At this point, there is very little work left to be done, so this is just a safety check.
    Returns the Rust code with any last minimal cleanup applied.
    """
    prompt = VERUS_REMOVE_PROMPT + f"""
    The code is provided below: 
    
    {rust_code}
    """
    response = openai.ChatCompletion.create(
        deployment_id=deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=16000,
    )
    return response.choices[0].message.content.strip()

def remove_empty(rust_code):
    """
    Removes empty functions, structs, and other empty constructs from Rust code. 
    Returns the cleaned up Rust code with empty functions, structs, enums, traits, impls, 
    and extra empty lines removed.
    """
    prompt = f"""
    The Rust code below is provided below: 
    
    {rust_code}
    
    Return only the Rust code with any empty functions, structs, enums, traits, or impls removed, following the rules below. 
    Do not alter anything else, and preserve all other code, comments, and structure.
    
    Rules:
    1) Only remove constructs that are truly empty - meaning either they contain no code, OR only comments,
       OR do contain `unimplemented!()`, `todo!()`, or any other constructs within indicating emptiness.
       This includes empty functions, empty structs, empty enums, empty traits, or empty impls.
    2) If a construct is deemed empty, remove any comments immediately above it.
    3) Remove unnecessary blank lines that appear within the body of non-empty constructs, or
       between the bodies of non-empty constructs, i.e., extra blank lines that serve no purpose. 

    Do not add explanations, summaries, or Rust ''' code fences.

    If no empty constructs exist, return the original input unchanged. Output only the raw Rust code.
    """
    response = openai.ChatCompletion.create(
        deployment_id=deployment_name,
        messages=[{"role": "user", "content": prompt}],
        max_completion_tokens=8000,
    )
    return response.choices[0].message.content.strip()

def strip_verus(rust_code):
    manual_clean = remove_tracked_ghost_proof_blocks(remove_specs(remove_container(rust_code)))
    manual_clean = remove_spec_functions(manual_clean)
    return remove_empty(final_check(manual_clean))

def pure_llm_baseline(rust_code):
    return remove_empty(final_check(rust_code))