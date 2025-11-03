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

Return the ENTIRE file with your changes integrated into the original code."""

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
            self.logger.warning(
                "General parser couldn't extract code, using original response"
            )
            return response

        # Check if the parser gave us a complete View implementation
        if (
            "impl" in parsed_code
            and "View for" in parsed_code
            and "type V =" in parsed_code
        ):
            self.logger.info("Successfully extracted View implementation")
            return parsed_code

        # If we don't have a View implementation yet, try to extract it specifically
        view_impl_pattern = r"impl\s*<.*?>\s*View\s+for\s+\w+.*?{.*?type\s+V\s*=.*?closed\s+spec\s+fn\s+view.*?}.*?}"
        view_impls = re.findall(view_impl_pattern, parsed_code, re.DOTALL)

        if view_impls:
            self.logger.info("Extracted specific View implementation from parsed code")
            return view_impls[0]

        # If we still don't have a View implementation, try the original response
        view_impls = re.findall(view_impl_pattern, response, re.DOTALL)
        if view_impls:
            self.logger.info("Extracted View implementation from original response")
            return view_impls[0]

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
            self.logger.debug(
                f"Temperature: {1.0 + (retry_attempt * temperature_boost)}"
            )
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
        """Process and validate LLM responses."""
        safe_responses = []
        for response in responses:
            # First parse the response to extract the View implementation
            final_response = parsed_response = parse_llm_response(response)

            # Check for balanced delimiters FIRST
            is_balanced, error_msg = self.check_balanced_delimiters(final_response)
            if not is_balanced:
                self.logger.warning(
                    f"Generated view code has unbalanced delimiters: {error_msg}{context_msg}"
                )
                continue

            # Then apply debug_type_error to fix any type errors
            fixed_response, _ = debug_type_error(parsed_response, logger=self.logger)
            temp_response = fixed_response if fixed_response else parsed_response

            # Apply regex-based syntax fixes
            from src.modules.repair_regex import fix_common_syntax_errors

            final_response, was_changed = fix_common_syntax_errors(
                temp_response, self.logger
            )
            if was_changed:
                self.logger.info(
                    "Applied regex syntax fixes to view inference response"
                )

            # Re-check balanced delimiters after fixing type errors
            is_balanced, error_msg = self.check_balanced_delimiters(final_response)
            if not is_balanced:
                self.logger.warning(
                    f"View code has unbalanced delimiters after type error fixes: {error_msg}{context_msg}"
                )
                continue

            # Check if the generated code is safe
            if self.check_code_safety(original_code, final_response):
                safe_responses.append(final_response)
                self.logger.info(
                    f"Generated view code passed all checks (delimiters + safety){context_msg}"
                )
            else:
                self.logger.warning(
                    f"Generated view code failed safety check{context_msg}"
                )
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
            self.logger.info(
                f"View inference attempt {retry_attempt + 1}/{max_retries}"
            )

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
                instruction += f"\n\nIMPORTANT: Previous attempt failed validation checks. Common issues:\n"
                instruction += f"1. Unbalanced delimiters - ensure ALL {{ }} ( ) [ ] are properly matched\n"
                instruction += (
                    f"2. Unclosed impl blocks - every 'impl' must have a closing }}\n"
                )
                instruction += f"3. Code safety - do not modify immutable functions\n"
                instruction += f"Please fix these issues. Attempt {retry_attempt + 2}/{max_retries}."

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
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

        # If this is the first checkpoint_best_code, initialize it
        if checkpoint_best_code is None:
            self.logger.debug(
                f"ViewInference - Initial checkpoint_best_code is None: {checkpoint_best_code is None}"
            )
            self.logger.debug(
                f"ViewInference - Initial checkpoint_best_score: {checkpoint_best_score}"
            )
            self.logger.debug(f"ViewInference - Current best_score: {best_score}")
            self.logger.info(
                "ViewInference - Initializing checkpoint best with current best"
            )
            checkpoint_best_code = best_code
            checkpoint_best_score = best_score

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
