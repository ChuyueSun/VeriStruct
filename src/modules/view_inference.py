import re
from pathlib import Path
from typing import Dict, List

from src.context import Context
from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    code_change_is_safe,
    debug_type_error,
    evaluate_samples,
    get_examples,
    parse_llm_response,
    update_checkpoint_best,
)
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, prompt_dir, samples_dir


class ViewInferenceModule(BaseModule):
    """
    Module for View function inference in Verus code.

    This module generates a View function that provides a mathematical abstraction
    for the given data structure, which is used in Verus specifications.
    """

    def __init__(self, config, logger):
        """
        Initialize the ViewInferenceModule.

        Args:
            config: Configuration object
            logger: Logger object
        """
        super().__init__(
            name="view_inference",
            desc="Generate a View function for the data structure's mathematical abstraction",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for View inference
        self.view_instruction = """
You are an expert in Verus (verifier for rust). Your task is to generate a View function for the given module.

The View is the mathematical abstraction of a data structure - it captures the ESSENTIAL LOGICAL STATE,
not just the implementation details.

**STEP 1: Identify the Data Structure's Purpose**

Ask yourself: What does this structure represent logically?
- A collection of elements? → Likely Seq<T>, Set<T>, or Map<K,V>
- A value with constraints? → Might need tuple to track both
- Multiple independent concepts? → Definitely needs tuple

**STEP 2: Count Independent Logical Aspects**

Analyze the struct fields to identify CONCEPTUAL aspects (not just field count):

Example: Counter with fields { value: u64, max: u64 }
- value, max = 2 fields
- Logically:
  * Current value (changes with operations)
  * Maximum allowed (constraint, doesn't change)
- = 2 independent aspects → need 2-tuple: (nat, nat)

Example: Stack with fields { data: Vec<T>, max_capacity: usize }
- data, max_capacity = 2 fields
- Logically:
  * Stack contents (the actual data stored)
  * Capacity limit (maximum size allowed)
- = 2 independent aspects → need 2-tuple: (Seq<T>, nat)

Example: SimpleList with fields { data: Vec<T> }
- data = 1 field
- Logically: just a sequence of elements
- = 1 aspect → simple Seq<T>

Example: Circular structure with fields { storage: Vec<T>, start_index: usize, end_index: usize }
- storage, start_index, end_index = 3 fields
- Logically:
  * The actual content stored (derived from storage + indices together)
  * Total capacity of the storage (fixed property)
- = 2 independent aspects → need 2-tuple: (Seq<T>, usize)

**STEP 3: Choose View Type Based on Analysis**

Single Type (not a tuple):
- Use when the structure represents ONE logical concept
- Examples:
  * List/Array/Vector → Seq<T>
  * Set operations → Set<T>
  * Key-value mapping → Map<K,V>
  * Bitmap → Seq<bool>

Tuple Type (T1, T2):
- Use when the structure has MULTIPLE independent logical aspects
- Common patterns:
  * Content + Capacity: (Seq<T>, nat) or (Seq<T>, usize)
    - When: Structure stores elements with a capacity limit
    - Examples: BoundedStack, bounded buffers/queues
  * Value + Constraint: (nat, nat)
    - When: Tracking a value with a maximum/minimum
    - Examples: BoundedCounter, indices with bounds
  * Data + Metadata: (MainType, MetadataType)
    - When: Primary data plus auxiliary information
    - First element: the main data (Seq, Map, Set, etc.)
    - Second element: size/capacity/constraints/properties

**STEP 4: Implement the View**

Based on your analysis:

```rust
impl<T: Copy> View for StructName<T> {
    type V = // Your chosen type from Step 3

    closed spec fn view(&self) -> Self::V {
        // Implementation that extracts the logical state
        // - For Seq<T>: convert storage to sequence
        // - For tuples: construct tuple with each aspect
        // - Use @ to get spec view of Vec/other types
    }
}
```

**CRITICAL RULES:**
- The View should be SIMPLER than the implementation (abstraction!)
- Fill in only the `/* TODO: part of view */` or empty View trait implementations
- Tuple size should generally be LESS than field count (true abstraction)
- Don't include derivable information (e.g., length can be computed from sequence)
- For `Vec` type variables, use `vec@` to get their Seq<T> view
- Do NOT use `reveal` keyword in the View implementation
- Think about what information is ESSENTIAL for specifications

**REASONING FRAMEWORK:**

Before generating, explicitly consider:
1. "This structure has [N] fields"
2. "Logically, it represents [X] independent concepts"
3. "The essential information is: [list the aspects]"
4. "Therefore, View type should be: [your choice]"

Mathematical types in Verus:
- Scalars: bool, int, nat
- Collections: Seq<T>, Set<T>, Map<K,V>
- Tuples: (T1, T2), (T1, T2, T3), etc.
- Combinations allowed: (Seq<T>, nat), (Map<K,V>, Set<K>), etc.

**CRITICAL: Ensure ALL delimiters are properly balanced:**
- Every opening brace { must have a matching closing brace }
- Every opening parenthesis ( must have a matching closing parenthesis )
- Every opening bracket [ must have a matching closing bracket ]
- Every impl block must be properly closed

**OUTPUT FORMAT:**

Return ONLY the view implementation, nothing else. Choose one of these formats:

**Format A: If code has existing `spec fn view` - return just the function body:**
```rust
let total_bits = self.bits@.len() * 64;
Seq::new(total_bits, |i: int| {
    let chunk_i = i / 64;
    let bit_i = i % 64;
    let chunk = self.bits@[chunk_i];
    get_bit64!(chunk, bit_i as u64)
})
```

**Format B: If code needs View trait - return the complete impl block:**
```rust
impl View for StructName {
    type V = Seq<bool>;

    closed spec fn view(&self) -> Self::V {
        // implementation
    }
}
```

DO NOT return the entire file. ONLY return the view implementation as shown above."""

    @staticmethod
    def _find_matching_brace(code: str, start_pos: int) -> int:
        """
        Find the position of the closing brace that matches the opening brace at start_pos.

        Args:
            code: The code string
            start_pos: Position of the opening brace

        Returns:
            Position of the matching closing brace, or -1 if not found
        """
        if start_pos >= len(code) or code[start_pos] != "{":
            return -1

        brace_count = 1
        i = start_pos + 1

        while i < len(code) and brace_count > 0:
            # Skip string literals and character literals to avoid counting braces inside them
            if code[i] == '"':
                i += 1
                while i < len(code):
                    if code[i] == "\\":
                        i += 2  # Skip escaped character
                    elif code[i] == '"':
                        i += 1
                        break
                    else:
                        i += 1
                continue
            elif code[i] == "'":
                i += 1
                while i < len(code):
                    if code[i] == "\\":
                        i += 2  # Skip escaped character
                    elif code[i] == "'":
                        i += 1
                        break
                    else:
                        i += 1
                continue
            # Skip single-line comments
            elif i + 1 < len(code) and code[i : i + 2] == "//":
                while i < len(code) and code[i] != "\n":
                    i += 1
                continue
            # Skip multi-line comments
            elif i + 1 < len(code) and code[i : i + 2] == "/*":
                i += 2
                while i + 1 < len(code):
                    if code[i : i + 2] == "*/":
                        i += 2
                        break
                    i += 1
                continue
            # Count braces
            elif code[i] == "{":
                brace_count += 1
            elif code[i] == "}":
                brace_count -= 1
                if brace_count == 0:
                    return i

            i += 1

        return -1

    @staticmethod
    def has_spec_fn_view(code: str) -> tuple[bool, str, int, int]:
        """
        Check if code already has a spec fn view declaration.

        Detects patterns:
        1. spec fn view(&self)
        2. pub spec fn view(&self)
        3. closed spec fn view(&self)
        4. pub closed spec fn view(&self)
        5. open spec fn view(&self)

        Returns:
            (has_spec_fn, struct_name, start_pos, end_pos)
            where start_pos and end_pos define the TODO region to replace
        """
        # Search for impl blocks that contain spec fn view
        # This is more robust than requiring struct definition and impl to be adjacent

        # Pattern to find impl blocks: impl StructName<...> {
        impl_pattern = r"impl\s+(\w+)\s*(?:<[^>]*>)?\s*\{"

        # Pattern to find spec fn view within an impl block
        spec_fn_pattern = r"((?:pub\s+)?(?:open\s+|closed\s+)?spec\s+fn\s+view\s*\(\s*&\s*self\s*\)\s*->\s*[^{]+)\{"

        # Find all impl blocks
        for impl_match in re.finditer(impl_pattern, code):
            struct_name = impl_match.group(1)
            impl_start = impl_match.end() - 1  # Position of opening brace

            # Find the end of this impl block
            impl_end = ViewInferenceModule._find_matching_brace(code, impl_start)
            if impl_end == -1:
                continue

            # Extract the impl block body
            impl_body = code[impl_start : impl_end + 1]

            # Search for spec fn view within this impl block
            spec_fn_match = re.search(spec_fn_pattern, impl_body)
            if spec_fn_match:
                # Found spec fn view in this impl block
                # Calculate absolute position in original code
                opening_brace_pos = impl_start + spec_fn_match.end() - 1

                # Find the matching closing brace for the spec fn view
                closing_brace_pos = ViewInferenceModule._find_matching_brace(
                    code, opening_brace_pos
                )

                if closing_brace_pos == -1:
                    continue

                # The body is between the opening and closing braces
                start_pos = opening_brace_pos + 1
                end_pos = closing_brace_pos

                return True, struct_name, start_pos, end_pos

        return False, "", -1, -1

    @staticmethod
    def has_view_trait_with_todo(code: str) -> tuple[bool, str, int, int]:
        """
        Check if code has impl View for with a TODO in the view function.

        Detects patterns:
        1. impl View for StructName { type V = ...; open spec fn view(...) { // TODO } }
        2. impl View for StructName { type V = ...; closed spec fn view(...) { // TODO } }

        Returns:
            (has_view_trait, struct_name, start_pos, end_pos)
            where start_pos and end_pos define the view function body to replace
        """
        # Look for impl View for with a view function
        # Note: We now only match up to the opening brace, then use _find_matching_brace
        pattern = r"impl\s*(?:<[^>]*>)?\s*View\s+for\s+(\w+)\s*(?:<[^>]*>)?\s*\{.*?type\s+V\s*=[^;]+;.*?((?:open\s+|closed\s+)?spec\s+fn\s+view\s*\([^)]*\)[^{]*)\{"

        match = re.search(pattern, code, re.DOTALL)
        if match:
            struct_name = match.group(1)
            # Find the opening brace position (right after the match)
            opening_brace_pos = match.end() - 1

            # Find the matching closing brace
            closing_brace_pos = ViewInferenceModule._find_matching_brace(code, opening_brace_pos)

            if closing_brace_pos == -1:
                return False, "", -1, -1

            # The body is between the opening and closing braces
            start_pos = opening_brace_pos + 1
            end_pos = closing_brace_pos
            body = code[start_pos:end_pos]

            # Only consider it a TODO case if:
            # 1. Body explicitly contains TODO comment
            # 2. Body is empty or only whitespace/comments
            body_stripped = body.strip()
            is_todo = (
                "TODO" in body
                or len(body_stripped) == 0
                or (len(body_stripped) < 20 and "//" in body_stripped)  # Just a comment
            )
            if is_todo:
                return True, struct_name, start_pos, end_pos

        return False, "", -1, -1

    @staticmethod
    def extract_view_implementation(response: str, is_spec_fn: bool) -> str:
        """
        Extract the view implementation from LLM response.

        Args:
            response: LLM response text
            is_spec_fn: If True, extract function body only; if False, extract impl block

        Returns:
            Extracted implementation
        """
        # Parse code blocks from response
        code = parse_llm_response(response)

        if is_spec_fn:
            # For spec fn, we want just the function body
            # Look for the code between the first { and last } that isn't part of impl View
            # Remove any impl View for or spec fn view wrappers

            # If LLM returned full function, extract body
            # Pattern matches up to the opening brace (non-greedy)
            fn_pattern = r"spec\s+fn\s+view\s*\([^)]*\)[^{]*\{"
            match = re.search(fn_pattern, code, re.DOTALL)
            if match:
                # Find the opening brace position
                opening_brace_pos = match.end() - 1

                # Use _find_matching_brace to find the proper closing brace
                closing_brace_pos = ViewInferenceModule._find_matching_brace(
                    code, opening_brace_pos
                )

                if closing_brace_pos != -1:
                    # Extract only the content between the braces (the function body)
                    return code[opening_brace_pos + 1 : closing_brace_pos].strip()

            # Otherwise, assume it's already just the body
            return code.strip()
        else:
            # For View trait, we want the complete impl block
            # Pattern matches up to the opening brace, then use _find_matching_brace
            impl_pattern = r"(impl\s*(?:<[^>]*>)?\s*View\s+for\s+\w+.*?)\{"
            match = re.search(impl_pattern, code, re.DOTALL)
            if match:
                # Find the opening brace position
                opening_brace_pos = match.end() - 1

                # Use _find_matching_brace to find the proper closing brace
                closing_brace_pos = ViewInferenceModule._find_matching_brace(
                    code, opening_brace_pos
                )

                if closing_brace_pos != -1:
                    # Extract the entire impl block including braces
                    return code[match.start() : closing_brace_pos + 1].strip()

            return code.strip()

    @staticmethod
    def insert_view_body(original_code: str, view_body: str, start_pos: int, end_pos: int) -> str:
        """
        Insert view function body into the original code.

        Args:
            original_code: Original source code
            view_body: The view function body to insert
            start_pos: Start position to replace
            end_pos: End position to replace

        Returns:
            Modified code with view body inserted
        """
        # Normalize indentation: detect minimum indentation and strip it, then add 8 spaces
        lines = view_body.split("\n")

        # Find minimum indentation level (excluding empty lines)
        min_indent = float("inf")
        for line in lines:
            if line.strip():  # Only consider non-empty lines
                leading_spaces = len(line) - len(line.lstrip())
                min_indent = min(min_indent, leading_spaces)

        # If all lines were empty, set min_indent to 0
        if min_indent == float("inf"):
            min_indent = 0

        # Strip the minimum indentation and add 8 spaces
        indented_lines = []
        for line in lines:
            if line.strip():  # Don't indent empty lines
                # Strip min_indent spaces, then add 8 spaces
                stripped_line = line[min_indent:] if len(line) >= min_indent else line.lstrip()
                indented_lines.append("        " + stripped_line)
            else:
                indented_lines.append(line)
        indented_body = "\n".join(indented_lines)

        # Insert the body
        return original_code[:start_pos] + "\n" + indented_body + "\n    " + original_code[end_pos:]

    @staticmethod
    def insert_view_trait(original_code: str, view_impl: str, struct_name: str) -> str:
        """
        Insert View trait implementation into the original code.

        Args:
            original_code: Original source code
            view_impl: The View trait implementation
            struct_name: Name of the struct

        Returns:
            Modified code with View trait inserted
        """
        # Find the struct definition
        struct_pattern = rf"(pub\s+)?struct\s+{struct_name}\s*(?:<[^>]*>)?\s*\{{[^}}]*\}}"
        match = re.search(struct_pattern, original_code, re.DOTALL)

        if not match:
            # Fallback: insert before impl block
            impl_pattern = rf"impl\s*(?:<[^>]*>)?\s*{struct_name}"
            match = re.search(impl_pattern, original_code)
            if match:
                insert_pos = match.start()
                return original_code[:insert_pos] + view_impl + "\n\n" + original_code[insert_pos:]
        else:
            # Insert after struct definition
            insert_pos = match.end()
            return (
                original_code[:insert_pos] + "\n\n" + view_impl + "\n" + original_code[insert_pos:]
            )

        # Last resort: add at the end before closing verus! block
        verus_end = original_code.rfind("}")
        if verus_end > 0:
            return original_code[:verus_end] + "\n" + view_impl + "\n" + original_code[verus_end:]

        return original_code + "\n\n" + view_impl

    @staticmethod
    def check_balanced_delimiters(code: str) -> tuple[bool, str]:
        """
        Check if all delimiters (parentheses, braces, brackets) are balanced.

        Args:
            code: The code to check

        Returns:
            Tuple of (is_balanced, error_message)
        """
        stack = []
        pairs = {"(": ")", "[": "]", "{": "}"}
        line_num = 1
        char_pos = 0

        # Remove string literals and comments to avoid false positives
        # Simple approach: remove single-line and multi-line comments
        code_no_comments = re.sub(r"//.*?$", "", code, flags=re.MULTILINE)
        code_no_comments = re.sub(r"/\*.*?\*/", "", code_no_comments, flags=re.DOTALL)

        for i, char in enumerate(code_no_comments):
            if char == "\n":
                line_num += 1
                char_pos = 0
            else:
                char_pos += 1

            if char in pairs:
                stack.append((char, line_num, char_pos))
            elif char in pairs.values():
                if not stack:
                    return (
                        False,
                        f"Unmatched closing delimiter '{char}' at line {line_num}, position {char_pos}",
                    )
                opening, open_line, open_pos = stack.pop()
                if pairs[opening] != char:
                    return (
                        False,
                        f"Mismatched delimiters: '{opening}' at line {open_line}:{open_pos} closed with '{char}' at line {line_num}:{char_pos}",
                    )

        if stack:
            unclosed = [(char, line, pos) for char, line, pos in stack]
            return (
                False,
                f"Unclosed delimiters: {unclosed[0][0]} at line {unclosed[0][1]}, position {unclosed[0][2]}",
            )

        return True, ""

    def parse_view_response(self, response: str) -> str:
        """
        Parse the LLM response to extract and clean the View implementation.

        Args:
            response: Raw LLM response text

        Returns:
            Cleaned code with proper View implementation
        """
        self.logger.info("Parsing view inference response...")

        # Use the general parser first to extract all Rust code
        parsed_code = parse_llm_response(response, logger=self.logger)

        # If parsing failed or returned empty string, log warning and return original
        if not parsed_code:
            self.logger.warning("General parser couldn't extract code, using original response")
            return response

        # Check if the parser gave us a complete View implementation
        if "impl" in parsed_code and "View for" in parsed_code and "type V =" in parsed_code:
            self.logger.info("Successfully extracted View implementation")
            return parsed_code

        # If we don't have a View implementation yet, try to extract it specifically
        # Pattern matches up to the opening brace, then use _find_matching_brace
        view_impl_pattern = r"impl\s*<.*?>\s*View\s+for\s+\w+.*?\{"
        matches = list(re.finditer(view_impl_pattern, parsed_code, re.DOTALL))

        if matches:
            for match in matches:
                # Check if this impl block contains the required elements
                opening_brace_pos = match.end() - 1
                closing_brace_pos = ViewInferenceModule._find_matching_brace(
                    parsed_code, opening_brace_pos
                )

                if closing_brace_pos != -1:
                    impl_block = parsed_code[match.start() : closing_brace_pos + 1]
                    # Verify it contains the view function
                    if "type V =" in impl_block and "spec fn view" in impl_block:
                        self.logger.info("Extracted specific View implementation from parsed code")
                        return impl_block

        # If we still don't have a View implementation, try the original response
        matches = list(re.finditer(view_impl_pattern, response, re.DOTALL))
        if matches:
            for match in matches:
                opening_brace_pos = match.end() - 1
                closing_brace_pos = ViewInferenceModule._find_matching_brace(
                    response, opening_brace_pos
                )

                if closing_brace_pos != -1:
                    impl_block = response[match.start() : closing_brace_pos + 1]
                    # Verify it contains the view function
                    if "type V =" in impl_block and "spec fn view" in impl_block:
                        self.logger.info("Extracted View implementation from original response")
                        return impl_block

        # If nothing worked, return the parsed code anyway
        self.logger.warning(
            "Could not find specific View implementation, returning general parsed code"
        )
        return parsed_code

    def _get_llm_responses(
        self,
        instruction: str,
        code: str,
        examples: List[Dict[str, str]] = None,
        temperature_boost: float = 0.2,
        retry_attempt: int = 0,
        use_cache: bool = True,
        context=None,
    ) -> List[str]:
        """Get responses from LLM with error handling."""
        try:
            # Add retry marker to instruction to ensure cache miss
            if retry_attempt > 0:
                instruction = f"{instruction}\n[Retry Attempt: {retry_attempt}]"
                use_cache = False  # Disable cache for retries

            # Log the complete query content for debugging
            self.logger.debug("=== LLM Query Content ===")
            self.logger.debug(f"Retry Attempt: {retry_attempt}")
            self.logger.debug(f"Temperature: {1.0 + (retry_attempt * temperature_boost)}")
            self.logger.debug(f"Cache Enabled: {use_cache}")
            self.logger.debug("\n=== Instruction ===\n" + instruction)
            self.logger.debug("\n=== Code ===\n" + code)
            if examples:
                self.logger.debug("\n=== Examples ===")
                for i, ex in enumerate(examples):
                    self.logger.debug(f"\nExample {i+1} Query:\n" + ex["query"])
                    self.logger.debug(f"\nExample {i+1} Answer:\n" + ex["answer"])
            self.logger.debug("=====================")

            engine = self.config.get("aoai_generation_model", "gpt-4")
            temp = 1.0 + (retry_attempt * temperature_boost)

            # Use tracking wrapper if context is available
            if context is not None and hasattr(context, "infer_llm_with_tracking"):
                result = context.infer_llm_with_tracking(
                    engine=engine,
                    instruction=instruction,
                    exemplars=examples or [],
                    query=code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 20000),
                    temp=temp,
                    use_cache=use_cache,
                    stage="view_inference",
                    module="view_inference",
                )
                # Unwrap if tuple returned
                if isinstance(result, tuple):
                    result = result[0]
                return result
            else:
                return self.llm.infer_llm(
                    engine,
                    instruction,
                    examples or [],
                    code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 20000),
                    temp=temp,
                    use_cache=use_cache,
                )
        except Exception as e:
            self.logger.error(f"Error during LLM inference: {e}")
            return []

    def _process_responses(
        self, responses: List[str], original_code: str, context_msg: str = ""
    ) -> List[str]:
        """Process and validate LLM responses, inserting view implementation into original code."""
        safe_responses = []

        # Detect which pattern we have
        # Pattern 1-2: spec fn view (with optional pub/open/closed modifiers)
        has_spec_fn, struct_name, start_pos, end_pos = self.has_spec_fn_view(original_code)

        # Pattern 4: impl View for with TODO in view function
        (
            has_view_trait_todo,
            view_trait_struct,
            view_start,
            view_end,
        ) = self.has_view_trait_with_todo(original_code)

        if has_spec_fn:
            self.logger.info(f"Pattern: spec fn view for {struct_name}, will fill in body only")
            is_spec_fn = True
        elif has_view_trait_todo:
            self.logger.info(
                f"Pattern: impl View for {view_trait_struct} with TODO, will fill in view function body"
            )
            is_spec_fn = True  # Treat similar to spec fn - just fill in body
            struct_name = view_trait_struct
            start_pos = view_start
            end_pos = view_end
        else:
            self.logger.info(
                "Pattern: Empty or no View, will insert complete View trait implementation"
            )
            is_spec_fn = False

        for response in responses:
            try:
                # Extract just the view implementation from response
                view_impl = self.extract_view_implementation(response, is_spec_fn=is_spec_fn)

                if not view_impl:
                    self.logger.warning(
                        f"Could not extract view implementation from response{context_msg}"
                    )
                    continue

                # Check for balanced delimiters in the extracted implementation
                is_balanced, error_msg = self.check_balanced_delimiters(view_impl)
                if not is_balanced:
                    self.logger.warning(
                        f"Generated view implementation has unbalanced delimiters: {error_msg}{context_msg}"
                    )
                    continue

                # Apply type error fixes to the view implementation
                fixed_impl, _ = debug_type_error(view_impl, logger=self.logger)
                view_impl = fixed_impl if fixed_impl else view_impl

                # Apply regex-based syntax fixes
                from src.modules.repair_regex import fix_common_syntax_errors

                view_impl, was_changed = fix_common_syntax_errors(view_impl, self.logger)
                if was_changed:
                    self.logger.info("Applied regex syntax fixes to view implementation")

                # Now insert the view implementation into the original code
                if is_spec_fn:
                    # Insert function body into existing spec fn view or View trait view function
                    final_code = self.insert_view_body(original_code, view_impl, start_pos, end_pos)
                else:
                    # Insert complete View trait implementation
                    # Try to detect struct name from original code
                    struct_match = re.search(r"(?:pub\s+)?struct\s+(\w+)", original_code)
                    if struct_match:
                        struct_name = struct_match.group(1)
                    else:
                        self.logger.warning(
                            f"Could not detect struct name from code for View trait insertion{context_msg}"
                        )
                        continue
                    final_code = self.insert_view_trait(original_code, view_impl, struct_name)

                # Validate the final assembled code
                is_balanced, error_msg = self.check_balanced_delimiters(final_code)
                if not is_balanced:
                    self.logger.warning(
                        f"Final code has unbalanced delimiters after insertion: {error_msg}{context_msg}"
                    )
                    continue

                # Check if the generated code is safe
                if self.check_code_safety(original_code, final_code):
                    safe_responses.append(final_code)
                    self.logger.info(
                        f"View implementation successfully inserted and validated{context_msg}"
                    )
                else:
                    self.logger.warning(f"Final code failed safety check{context_msg}")
            except Exception as e:
                self.logger.error(f"Error processing response: {e}{context_msg}")
                continue

        return safe_responses

    def exec(self, context: Context) -> str:
        """
        Execute the view inference module with the given context.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with View function
        """
        self.logger.info("View Inference ...")

        # Get the latest trial code
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Build the complete instruction using the prompt system
        instruction = build_instruction(
            base_instruction=self.view_instruction,
            add_common=True,
            add_view=True,  # Include View guidelines
            code=code,
            knowledge=context.gen_knowledge(),
        )

        # Load examples showing completed View implementations (answer-only format)
        # This reduces redundancy - we only show the pattern, not the before/after
        raw_examples = get_examples(self.config, "view", self.logger, max_examples=10)

        # Convert to answer-only format: use 'answer' as both query and answer
        # This shows the LLM the correct pattern without redundant TODO version
        examples = []
        for i, ex in enumerate(raw_examples):
            if ex.get("answer"):
                examples.append(
                    {
                        "query": f"Example {i+1}: Pattern for implementing View trait",
                        "answer": ex["answer"],
                    }
                )

        self.logger.info(
            f"Using {len(examples)} answer-only examples from output-view (reduced redundancy)"
        )

        # Retry mechanism for safety checks
        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(f"View inference attempt {retry_attempt + 1}/{max_retries}")

            # Save prompt for debugging
            prompt_path = prompt_dir()
            prompt_file = prompt_path / f"view_inference_{retry_attempt + 1}.txt"
            prompt_file.write_text(instruction)
            self.logger.info(f"Saved view inference prompt to {prompt_file}")

            # Use cache only for first attempt
            responses = self._get_llm_responses(
                instruction,
                code,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                context=context,  # Pass context for tracking
                #   use_cache=(retry_attempt == 0)
            )
            if not responses and retry_attempt == max_retries - 1:
                return code

            safe_responses.extend(self._process_responses(responses, original_code))

            if safe_responses:
                self.logger.info(
                    f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed validation checks. Common issues:\n"
                )
                instruction += (
                    f"1. Unbalanced delimiters - ensure ALL {{ }} ( ) [ ] are properly matched\n"
                )
                instruction += f"2. Unclosed impl blocks - every 'impl' must have a closing }}\n"
                instruction += f"3. Code safety - do not modify immutable functions\n"
                instruction += (
                    f"Please fix these issues. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning("No safe responses found after all retries, using original code")
            safe_responses = [original_code]

        # Save all generated samples
        output_dir = samples_dir()
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = best_dir()
        global_dir.mkdir(exist_ok=True, parents=True)

        # Evaluate processed samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=safe_responses,
            output_dir=output_dir,
            prefix="01_view_inference",
            logger=self.logger,
        )

        # Initialize and update global best
        checkpoint_best_score = (
            context.get_best_score() if hasattr(context, "get_best_score") else None
        )
        checkpoint_best_code = (
            context.get_best_code() if hasattr(context, "get_best_code") else None
        )

        # Compare and update checkpoint best if current is better
        if checkpoint_best_code is None:
            # First time: initialize with current best
            self.logger.debug(
                f"ViewInference - Initial checkpoint_best_code is None: {checkpoint_best_code is None}"
            )
            self.logger.debug(
                f"ViewInference - Initial checkpoint_best_score: {checkpoint_best_score}"
            )
            self.logger.debug(f"ViewInference - Current best_score: {best_score}")
            self.logger.info("ViewInference - Initializing checkpoint best with current best")
            checkpoint_best_code = best_code
            checkpoint_best_score = best_score
        elif best_score > checkpoint_best_score:
            # Current result is better: update checkpoint best
            self.logger.info(
                f"ViewInference - Found better result: {best_score} > {checkpoint_best_score}"
            )
            self.logger.info("ViewInference - Updating checkpoint best with current best")
            checkpoint_best_code = best_code
            checkpoint_best_score = best_score
        else:
            # Previous checkpoint was better: keep it
            self.logger.info(
                f"ViewInference - Keeping previous checkpoint best: {checkpoint_best_score} >= {best_score}"
            )

        # Save the module-specific best from this step
        module_best_path = output_dir / "01_view_inference_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best view inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best view inference: {e}")

        # Update context's global best tracking
        context.set_best_code(checkpoint_best_code)
        context.set_best_score(checkpoint_best_score)

        # Add the best sample from current step to context
        context.add_trial(best_code)

        return best_code
