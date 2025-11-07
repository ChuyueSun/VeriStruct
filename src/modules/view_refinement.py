import os
import re
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    code_change_is_safe,
    debug_type_error,
    evaluate_samples,
    get_examples,
    save_selection_info,
    update_checkpoint_best,
)
from src.modules.veval import EvalScore, VEval
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, prompt_dir, samples_dir


class ViewRefinementModule(BaseModule):
    """
    Module for refining View functions in Verus code.

    This module improves a View function by ensuring it provides
    a proper abstraction of the data structure.
    """

    def __init__(self, config, logger):
        """
        Initialize the ViewRefinementModule.

        Args:
            config: Configuration object
            logger: Logger object
        """
        super().__init__(
            name="view_refinement",
            desc="Refine an existing View function to improve its mathematical abstraction",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for view refinement
        self.refinement_instruction = """
You are a highly experienced expert in Verus (the verifier for Rust). Your task is to refine the "View" function within the given Verus file. The "View" function is the mathematical abstraction for a data structure, capturing the minimal information needed for its specification in Verus.

It is perfectly acceptable to leave the code unchanged if the current abstraction is already appropriate; modify the "View" function only when necessary.

**CRITICAL: DO NOT "refine" sequential/indexed data into sets!**
  - If the current view is `Seq<T>` (e.g., `Seq<bool>` for bitmaps), **KEEP IT AS Seq<T>**
  - DO NOT change `Seq<bool>` to `(nat, Set<nat>)` - this makes verification extremely difficult
  - Sequential data structures (bitmaps, arrays, buffers) REQUIRE Seq for indexed access and update operations
  - **"More succinct" does NOT mean "better for verification"**

Your responsibilities:
  1. Analyze the current "View" function to determine if its tuple (or other structure) adequately represents the module.
  2. Evaluate whether the abstraction can be improved. (Hint: If the tuple is identical to the internal fields, that is likely not an ideal abstraction.)
  3. **EXCEPTION**: If the current view is `Seq<T>` for indexed/sequential data (bitmaps, arrays, vectors), DO NOT change it - `Seq<T>` is the correct abstraction!
  4. Modify only the "View" function to improve its abstraction while leaving all other parts of the file unchanged.
  5. Any refined view must convey at least the same amount of information. For non-sequential data, aim to use a flattened tuple that is shorter than the original.
  6. **IMPORTANT**: Do not remove any "// TODO" markers in the code. Preserve all TODO comments exactly as they appear.
  7. Return the **entire updated Verus file** with your refined "View" function (or the original file if no changes were necessary) and nothing else changed.

**Common patterns to PRESERVE (do not change these)**:
  - `Seq<bool>` for bitmaps → Keep as `Seq<bool>`
  - `Seq<T>` for Vec<T> or Array<T> → Keep as `Seq<T>`
  - `Seq<T>` for ring buffers, queues → Keep as `Seq<T>`

**Common patterns that CAN be refined**:
  - Tuple of raw fields → Abstract mathematical properties
  - Redundant information → Remove duplicates
  - Complex nested tuples → Flatten when possible

Please provide only the complete Rust code of the file with no additional commentary.
"""

    def _load_examples(self) -> List[Dict[str, str]]:
        """Load example files for view refinement."""
        examples = []
        try:
            example_path = Path(self.config.get("example_path", "examples")) / "input-view-refine"
            if example_path.exists():
                for f in sorted(example_path.iterdir()):
                    if f.suffix == ".rs":
                        input_content = f.read_text()
                        answer_path = (
                            Path(self.config.get("example_path", "examples"))
                            / "output-view-refine"
                            / f.name
                        )
                        answer = answer_path.read_text() if answer_path.exists() else ""
                        examples.append({"query": input_content, "answer": answer})
            else:
                self.logger.warning("Example path does not exist - proceeding without examples")
        except Exception as e:
            self.logger.error(f"Error loading examples: {e}")
        return examples

    def _is_trivial_view(self, code: str) -> bool:
        """
        Detect if a view is trivially exposing all internal fields without abstraction.

        A view is considered trivial if:
        1. **PRIMARY**: View tuple size equals struct field count (N fields → N-tuple)
           - This indicates no abstraction - just wrapping all fields in a tuple
        2. **SECONDARY**: Most fields (≥80%) are directly accessed without meaningful computation
           - Catches non-tuple views that still expose fields trivially

        Examples:
            TRIVIAL: struct {a, b, c} → type V = (T1, T2, T3)
            GOOD:    struct {a, b, c} → type V = (Seq<T>, usize)  [abstraction!]
            GOOD:    struct {a, b, c} → type V = Seq<T>          [single abstraction!]

        Args:
            code: The Verus code to analyze

        Returns:
            True if the view appears trivial, False otherwise
        """
        try:
            # Extract struct fields (looking for pub/private fields in struct definitions)
            # Pattern: struct Name<...> { field1: Type1, field2: Type2, ... }
            struct_pattern = r"(?:pub\s+)?struct\s+\w+[^{]*\{([^}]+)\}"
            struct_matches = re.findall(struct_pattern, code, re.DOTALL)

            if not struct_matches:
                return False

            # Count non-comment, non-empty field declarations
            struct_fields = []
            for struct_body in struct_matches:
                # Remove comments
                cleaned = re.sub(r"//.*?$", "", struct_body, flags=re.MULTILINE)
                cleaned = re.sub(r"/\*.*?\*/", "", cleaned, flags=re.DOTALL)
                # Find field declarations (name: type)
                fields = re.findall(r"(\w+)\s*:\s*[^,}]+", cleaned)
                struct_fields.extend([f.strip() for f in fields if f.strip()])

            if not struct_fields:
                return False

            struct_field_count = len(struct_fields)

            # Extract View type definition
            # Pattern: type V = ...;
            view_type_pattern = r"type\s+V\s*=\s*([^;]+);"
            view_type_match = re.search(view_type_pattern, code)

            if not view_type_match:
                return False

            view_type = view_type_match.group(1).strip()

            # Extract view function body
            # Pattern: closed spec fn view(&self) -> Self::V { ... }
            view_fn_pattern = (
                r"(?:closed\s+)?spec\s+fn\s+view\s*\([^)]*\)[^{]*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}"
            )
            view_fn_match = re.search(view_fn_pattern, code, re.DOTALL)

            if not view_fn_match:
                return False

            view_body = view_fn_match.group(1).strip()

            # PRIMARY HEURISTIC: If view tuple size == struct field count, it's trivial
            # This indicates the view is just exposing all fields without abstraction
            if view_type.startswith("(") and view_type.endswith(")"):
                # Count tuple elements (handle nested generics properly)
                tuple_elem_count = self._count_top_level_elements(view_type[1:-1])

                # If tuple size matches struct field count, it's trivial - no abstraction is happening
                if tuple_elem_count == struct_field_count:
                    self.logger.debug(
                        f"Trivial view detected: tuple size ({tuple_elem_count}) "
                        f"equals struct field count ({struct_field_count}) - no abstraction"
                    )
                    return True

            # SECONDARY HEURISTIC: Check if most fields are directly exposed without computation
            # This catches cases where the view type might be different but still trivially exposes fields
            field_access_count = 0
            for field_name in struct_fields:
                # Look for self.field_name with optional @ or as conversions
                field_access_pattern = rf"\bself\.{field_name}\b(?:@|\s+as\s+\w+)?"
                if re.search(field_access_pattern, view_body):
                    field_access_count += 1

            # If most fields are directly accessed without meaningful computation, likely trivial
            if field_access_count >= struct_field_count * 0.8:  # 80% threshold
                if not self._has_meaningful_computation(view_body):
                    self.logger.debug(
                        f"Trivial view detected: {field_access_count}/{struct_field_count} "
                        f"fields directly accessed without computation"
                    )
                    return True

            return False

        except Exception as e:
            self.logger.warning(f"Error analyzing view triviality: {e}")
            return False

    def _count_top_level_elements(self, type_str: str) -> int:
        """
        Count top-level comma-separated elements in a type string.
        Handles nested generics and tuples.

        Examples:
            "(Seq<T>, nat, nat)" → 3
            "(Map<K, V>, usize)" → 2
            "Seq<T>" → 0 (not a tuple)
        """
        depth = 0
        count = 1  # Start with 1 for first element

        for char in type_str:
            if char in "<([":
                depth += 1
            elif char in ">)]":
                depth -= 1
            elif char == "," and depth == 0:
                count += 1

        return count if type_str.strip() else 0

    def _has_meaningful_computation(self, view_body: str) -> bool:
        """
        Check if view body contains meaningful computation or abstraction.

        Meaningful computation includes:
        - Conditional logic (if/else, match)
        - Method calls (subrange, add, filter, map, etc.)
        - Complex expressions
        - Let bindings with computation
        """
        # Remove comments
        cleaned = re.sub(r"//.*?$", "", view_body, flags=re.MULTILINE)
        cleaned = re.sub(r"/\*.*?\*/", "", cleaned, flags=re.DOTALL)

        # Check for control flow
        if re.search(r"\b(if|else|match)\b", cleaned):
            return True

        # Check for let bindings (indicates intermediate computation)
        if re.search(r"\blet\s+\w+\s*=", cleaned):
            return True

        # Check for method calls that indicate data transformation
        # subrange, add, filter, map, union, insert, etc.
        meaningful_methods = [
            r"\.subrange\s*\(",
            r"\.add\s*\(",
            r"\.filter\s*\(",
            r"\.map\s*\(",
            r"\.union\s*\(",
            r"\.insert\s*\(",
            r"\.remove\s*\(",
            r"\.take\s*\(",
            r"\.skip\s*\(",
        ]

        for pattern in meaningful_methods:
            if re.search(pattern, cleaned):
                return True

        return False

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
            # Add retry marker to instruction to ensure cache differentiation
            if retry_attempt > 0:
                instruction = f"{instruction}\n[Retry Attempt: {retry_attempt}]"
                # Keep use_cache as is - use cache at all times

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
                    max_tokens=self.config.get("max_token", 8192),
                    temp=temp,
                    use_cache=use_cache,
                    stage="view_refinement",
                    module="view_refinement",
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
                    max_tokens=self.config.get("max_token", 8192),
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
            fixed_response, _ = debug_type_error(response, logger=self.logger)
            final_response = fixed_response if fixed_response else response
            if self.check_code_safety(original_code, final_response):
                safe_responses.append(final_response)
                self.logger.info(f"Generated code passed safety check{context_msg}")
            else:
                self.logger.warning(f"Generated code failed safety check{context_msg}")
        return safe_responses

    def _handle_compilation_retry(
        self,
        code: str,
        original_code: str,
        compile_attempt: int,
        max_compile_attempts: int,
        context,  # Add context parameter
    ) -> List[str]:
        """Handle compilation retry by getting fresh responses for compilation errors."""
        self.logger.info(
            f"Getting fresh responses for compilation errors (attempt {compile_attempt + 1}/{max_compile_attempts})"
        )
        try:
            retry_instruction = build_instruction(
                base_instruction=self.refinement_instruction
                + "\n\nIMPORTANT: Previous attempts resulted in compilation errors. Please ensure the code compiles correctly.",
                add_common=True,
                add_view=True,
                add_match=True,
                code=code,
                knowledge=context.gen_knowledge(),
            )

            responses = self._get_llm_responses(
                retry_instruction,
                code,
                temperature_boost=0.3,
                retry_attempt=compile_attempt,
                use_cache=True,  # Use cache at all times
                context=context,  # Pass context for tracking
            )

            return self._process_responses(
                responses, original_code, context_msg=" in compilation retry"
            )
        except Exception as e:
            self.logger.error(f"Error in compilation retry: {e}")
            return []

    def _save_best_code(
        self,
        best_code: str,
        best_score: EvalScore,
        output_dir: Path,
    ) -> None:
        """Save the best code to a file."""
        module_best_path = output_dir / "02_view_refinement_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best view refinement to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best view refinement: {e}")

    def exec(self, context) -> str:
        """
        Execute the view refinement module with the given context.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with refined View implementation
        """
        self.logger.info("View Refinement ...")

        # Get the latest trial code
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Build instruction and load examples
        instruction = build_instruction(
            base_instruction=self.refinement_instruction,
            add_common=True,
            add_view=True,
            add_match=True,
            code=code,
            knowledge=context.gen_knowledge(),
        )
        examples = self._load_examples()

        # Initial safety check retries
        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(f"View refinement attempt {retry_attempt + 1}/{max_retries}")

            # Save prompt for debugging
            prompt_path = prompt_dir()
            prompt_file = prompt_path / f"view_refinement_{retry_attempt + 1}.txt"
            prompt_file.write_text(instruction)
            self.logger.info(f"Saved view refinement prompt to {prompt_file}")

            # Use cache at all times
            responses = self._get_llm_responses(
                instruction,
                code,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,  # Always use cache
                context=context,  # Pass context for tracking
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
                instruction += f"\n\nIMPORTANT: Previous attempt failed safety checks. Please ensure your View refinement does not modify immutable functions and maintains semantic equivalence. Attempt {retry_attempt + 2}/{max_retries}."

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning("No safe responses found after all retries, using original code")
            safe_responses = [original_code]

        # Setup directories
        output_dir = samples_dir()
        global_dir = best_dir()

        # Compilation retry loop
        max_compile_attempts = 3
        compile_attempt = 0
        skip_compilation_retry = False  # Flag to skip retry when we just did trivial view retry

        while compile_attempt < max_compile_attempts:
            if compile_attempt > 0 and not skip_compilation_retry:
                new_responses = self._handle_compilation_retry(
                    code,
                    original_code,
                    compile_attempt,
                    max_compile_attempts,
                    context,  # Pass context to the method
                )
                if new_responses:
                    safe_responses = new_responses
                else:
                    break

            # Reset flag after using it
            skip_compilation_retry = False

            # Evaluate the samples and get the best one
            best_code, best_score, _ = evaluate_samples(
                samples=safe_responses,
                output_dir=output_dir,
                prefix=f"02_view_refinement_compile_attempt_{compile_attempt + 1}",
                logger=self.logger,
            )

            # Check if there's a compilation error
            if not best_score.compilation_error:
                self.logger.info(f"Found compiling code on attempt {compile_attempt + 1}")

                # CRITICAL CHECK: Detect trivial views and reject them
                if self._is_trivial_view(best_code):
                    self.logger.warning(
                        "Detected TRIVIAL view (tuple size = field count) - "
                        "no semantic abstraction provided. View just wraps all fields."
                    )

                    # Try to get better responses with specific feedback
                    if compile_attempt < max_compile_attempts - 1:
                        self.logger.info("Calling LLM again with feedback about trivial view issue")

                        # Build instruction with trivial view feedback
                        trivial_view_feedback = """
CRITICAL ISSUE: Your previous view was TRIVIAL - it just wrapped all struct fields without providing semantic abstraction.

**Problem Detected:**
- View tuple size equals struct field count (N fields → N-tuple)
- This means NO abstraction - just exposing implementation details as-is
- Example BAD: struct {a, b, c} → type V = (T1, T2, T3)  [just wrapping all 3 fields!]

**What You MUST Do:**
1. Provide a SEMANTIC abstraction representing WHAT the data means, not HOW it's stored
2. REDUCE tuple size by combining related fields into logical abstractions
3. Use collections (Seq, Set, Map) to represent logical contents, not raw implementation
4. Think: "What does a user of this data structure care about?" NOT "What are the fields?"

**Concrete Example for Ring Buffer:**
❌ BAD:  struct {ring: Vec<T>, head: usize, tail: usize} → type V = (Seq<T>, nat, nat)
         [Just exposing all 3 fields - no abstraction!]

✅ GOOD: struct {ring: Vec<T>, head: usize, tail: usize} → type V = (Seq<T>, usize)
         [Combines ring/head/tail into LOGICAL sequence of active elements + capacity]
         [2-tuple instead of 3 - showing true abstraction!]

**Critical Question:**
Is your tuple size STRICTLY LESS than the field count? If not, you're not abstracting!
"""

                        retry_instruction = build_instruction(
                            base_instruction=self.refinement_instruction + trivial_view_feedback,
                            add_common=True,
                            add_view=True,
                            add_match=False,
                            code=best_code,  # Use best_code with trivial view, not original code
                            knowledge=context.gen_knowledge(),
                        )

                        # Get fresh responses with the feedback (with error handling)
                        try:
                            new_responses = self._get_llm_responses(
                                retry_instruction,
                                best_code,  # Use best_code with trivial view, not original code
                                examples,
                                temperature_boost=0.3,
                                retry_attempt=compile_attempt + 1,
                                use_cache=True,  # Use cache at all times
                                context=context,
                            )

                            if new_responses and len(new_responses) > 0:
                                safe_responses = self._process_responses(
                                    new_responses,
                                    original_code,
                                    context_msg=" in trivial view retry",
                                )
                                if safe_responses and len(safe_responses) > 0:
                                    # Continue loop with new responses
                                    # Set flag to skip compilation retry on next iteration
                                    skip_compilation_retry = True
                                    compile_attempt += 1
                                    continue
                                else:
                                    self.logger.warning(
                                        "No safe responses after processing trivial view retry"
                                    )
                            else:
                                self.logger.warning(
                                    "No responses received from LLM for trivial view retry"
                                )
                        except Exception as e:
                            self.logger.error(f"Error during trivial view retry LLM call: {e}")

                        # If we couldn't get new responses, fall through to fallback
                        self.logger.warning(
                            "Could not generate new responses for trivial view - keeping original"
                        )
                    else:
                        # Max attempts reached with trivial view
                        self.logger.warning(
                            "Trivial view persists after max attempts - keeping original code"
                        )

                    # Fall back to original code
                    best_code = original_code
                    from src.modules.veval import VEval

                    veval = VEval(best_code, logger=self.logger)
                    best_score = veval.eval()

                break

            compile_attempt += 1
            if compile_attempt < max_compile_attempts:
                self.logger.warning(
                    f"Best code has compilation error, will try new responses... (attempt {compile_attempt + 1}/{max_compile_attempts})"
                )
            else:
                self.logger.warning(
                    f"Max compilation attempts reached, falling back to previous best code"
                )
                best_code = context.get_best_code()
                best_score = context.get_best_score()

        # Handle global best tracking
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save results
        self._save_best_code(best_code, best_score, output_dir)

        # Update context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)
        context.add_trial(best_code)  # Always use the best sample from this step

        return best_code
