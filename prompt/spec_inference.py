"""
Module for inferring requires and ensures clauses in Verus code.
"""

import re
from pathlib import Path
from typing import Dict, List

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    code_change_is_safe,
    debug_type_error,
    evaluate_samples,
    get_examples,
    update_checkpoint_best,
)
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, samples_dir


def fix_spec_syntax_issues(code: str) -> str:
    """
    Fix common syntax issues in spec clauses that cause compilation errors.

    Fixes applied:
    1. Wrap cast expressions in parentheses before comparison operators
       - BROKEN:  x as int < y  (causes "expected `,`" error)
       - FIXED:   (x as int) < y

    2. Remove trailing commas before ensures/recommends (when single condition)
       - BROKEN:  requires x > 0, \n ensures
       - FIXED:   requires x > 0 \n ensures

    3. Simplify .view() to @ shorthand
       - BROKEN:  self.view().len()
       - FIXED:   self@.len()

    4. Simplify old(self).view() to old(self)@
       - BROKEN:  old(self).view().field
       - FIXED:   old(self)@.field

    Args:
        code: The Rust/Verus code to fix

    Returns:
        Fixed code with syntax issues resolved
    """
    # Fix 1: Wrap cast expressions in parentheses
    # Pattern: Match any 'expr as type' that appears in comparisons
    # Do this in multiple passes to catch all cases
    fixed_code = code

    # Wrap casts followed by comparison operators (left side of comparison)
    cast_before_op = r"\b(\w+)\s+as\s+(int|nat|u\d+|i\d+|usize|isize)\s+([<>=!]+)"
    fixed_code = re.sub(cast_before_op, r"(\1 as \2) \3", fixed_code)

    # Wrap casts preceded by comparison operators (right side of comparison)
    # Pattern: comparison operator followed by cast
    op_before_cast = r"([<>=!]+)\s+(\w+)\s+as\s+(int|nat|u\d+|i\d+|usize|isize)\b"
    fixed_code = re.sub(op_before_cast, r"\1 (\2 as \3)", fixed_code)

    # Fix 2: Remove trailing commas before ensures/recommends/opening brace
    # Only when it's a single condition (no more conditions follow)
    lines = fixed_code.split("\n")
    fixed_lines = []
    in_spec_clause = False
    spec_clause_type = None

    for i, line in enumerate(lines):
        stripped = line.strip()

        # Track when we enter a spec clause
        if stripped.startswith("requires"):
            in_spec_clause = True
            spec_clause_type = "requires"
        elif stripped.startswith("ensures"):
            in_spec_clause = True
            spec_clause_type = "ensures"
        elif stripped.startswith("recommends"):
            in_spec_clause = True
            spec_clause_type = "recommends"
        elif (
            stripped.startswith("{")
            or stripped.startswith("fn ")
            or stripped.startswith("pub fn")
        ):
            in_spec_clause = False
            spec_clause_type = None

        # Check for problematic trailing commas
        if in_spec_clause and stripped.endswith(",") and not stripped.startswith("//"):
            # Look ahead to see what's next
            next_starts_new_clause = False
            for j in range(i + 1, min(i + 5, len(lines))):
                next_stripped = lines[j].strip()
                if next_stripped and not next_stripped.startswith("//"):
                    # Check if next line starts a new clause or function body
                    if (
                        next_stripped.startswith("ensures")
                        or next_stripped.startswith("recommends")
                        or next_stripped.startswith("{")
                    ):
                        next_starts_new_clause = True
                    break

            # Remove trailing comma only if next line starts a new clause
            if next_starts_new_clause:
                fixed_lines.append(line.rstrip(",").rstrip())
                continue

        fixed_lines.append(line)

    fixed_code = "\n".join(fixed_lines)

    # Fix 3: Simplify .view() to @ shorthand in specifications
    # Only in requires/ensures/recommends clauses and spec function bodies
    # Use @ for: self.view(), ret.view(), old(self).view(), var.view()

    # Pattern 1: old(self).view() -> old(self)@
    fixed_code = re.sub(r"old\((\w+)\)\.view\(\)", r"old(\1)@", fixed_code)

    # Pattern 2: Simple variable.view() -> variable@
    # Be careful not to replace in function definitions or non-spec contexts
    # Look for .view() followed by field access or method calls
    fixed_code = re.sub(r"(\w+)\.view\(\)\.len\(\)", r"\1@.len()", fixed_code)
    fixed_code = re.sub(r"(\w+)\.view\(\)\[", r"\1@[", fixed_code)
    fixed_code = re.sub(r"(\w+)\.view\(\)\.(\w+)", r"\1@.\2", fixed_code)

    # Pattern 3: In spec contexts, simplify standalone .view() calls
    # This is more aggressive and should only apply in spec clauses
    lines = fixed_code.split("\n")
    in_spec_clause = False
    result_lines = []

    for line in lines:
        stripped = line.strip()

        # Track spec clause context
        if any(
            stripped.startswith(kw)
            for kw in ["requires", "ensures", "recommends", "invariant"]
        ):
            in_spec_clause = True
        elif stripped.startswith("{") or (
            stripped.startswith("fn ") and "spec fn" not in line
        ):
            in_spec_clause = False

        # In spec clauses, aggressively replace .view() with @
        if in_spec_clause and ".view()" in line:
            # Replace remaining .view() occurrences
            line = re.sub(r"([a-zA-Z_]\w*)\.view\(\)", r"\1@", line)

        result_lines.append(line)

    return "\n".join(result_lines)


class SpecInferenceModule(BaseModule):
    """
    Module for inferring requires and ensures clauses for Verus functions.

    This module analyzes the code and adds appropriate preconditions and
    postconditions to functions based on their behavior.
    """

    def __init__(self, config, logger, immutable_funcs=None):
        super().__init__(
            name="spec_inference",
            desc="Infer requires and ensures clauses for functions",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)
        self.immutable_funcs = immutable_funcs or []

        # Main instruction for spec inference (will be augmented with invariant-specific guidance)
        self.inference_instruction = (
            "You are an expert in Verus (verifier for rust). Your task is to add requires and ensures clauses to functions.\n\n"
            "The examples provided show COMPLETED code with proper specifications. Study these patterns and apply them to the code with TODO markers.\n\n"
            "**CRITICAL CONSTRAINTS:**\n"
            "   - DO NOT modify function signatures or headers (pub, open, closed, spec, fn keywords)\n"
            "   - DO NOT add or remove `pub`, `open`, or `closed` keywords to any function\n"
            "   - DO NOT change `spec fn view` visibility - keep it exactly as is\n"
            "   - ONLY add `requires` and `ensures` clauses, nothing else\n"
            "1. **Add `requires` and `ensures` to functions**:\n"
            "   - For functions that return a value: Change signatures to `-> (retname: rettype)`\n"
            "   - For functions that return unit/nothing: DO NOT add `-> ()`, leave signature as is\n"
            "   - Add appropriate `requires` and `ensures` clauses based on function semantics\n"
            "   - **CRITICAL: For types with spec fn view(), use @ shorthand in specifications:**\n"
            "     * ALWAYS use `self@` instead of `self.view()` in requires/ensures\n"
            "     * ALWAYS use `ret@` instead of `ret.view()` in ensures\n"
            "     * ALWAYS use `old(self)@` instead of `old(self).view()` in ensures\n"
            "     * Examples: `self@.len()`, `self@.field`, `ret@[i]`, `old(self)@[i]`\n"
            "     * For tuples: if view() returns (A, B), use `self@.0`, `self@.1`\n"
            "     * NEVER write `self.view()` directly - it causes syntax errors\n"
            "   - **CRITICAL: old() Usage - What IS Allowed:**\n"
            "     * ✅ ALLOWED: old(node).unwrap().well_formed() for Option types\n"
            "     * ✅ ALLOWED: old(node).is_some(), old(node).is_none() checks\n"
            "     * ✅ ALLOWED: Calling methods on old(x) - old(self).method(), old(ptr).unwrap().field\n"
            "     * ✅ CORRECT PATTERN for Option parameters:\n"
            "       requires old(node).is_some() ==> old(node).unwrap().well_formed(),\n"
            "       ensures node.is_some() ==> node.unwrap().well_formed(),\n"
            "     * ❌ AVOID meaningless tautologies like: old(x).is_some() ==> true\n"
            "     * ❌ AVOID always-true conditions like: x.is_none() || true\n"
            "   - DO NOT use `match` or `let` in `requires`/`ensures` clauses (they are NOT allowed in specifications)\n"
            "   - NEVER write `let x = value in expression` in ensures - this will cause compilation errors\n"
            "   - Keep quantifier expressions (forall/exists) simple - avoid complex dereferences like `*ptr.method()` in quantifier bodies\n"
            "   - DO NOT modify `fn main()`\n"
            "   - Skip `self.inv()` in specs when `#[verifier::type_invariant]` is present\n"
            "   - Spec functions (e.g., View) cannot have requires/ensures\n\n"
            "2. **Add `ensures` clauses to trait method implementations**:\n"
            "   - Add appropriate `ensures` clauses based on method semantics\n"
            "   - State conditions that determine the return value\n\n"
            "   - For field access, follow the same rules as above:\n"
            "     * If type implements View: use `self@.field` (NOT `self.view().field`)\n"
            "     * Otherwise: use direct field access `self.field`\n"
            "   - DO NOT add `requires` clauses to trait implementations (only allowed in trait declarations)\n\n"
            "3. **Implement `spec fn` functions**:\n"
            "   - Write implementation based on function name and context\n"
            "   - Follow field access rules as above for View trait\n"
            "   - You MAY use `match` and `let` inside `spec fn` bodies\n\n"
            "**ADDITIONAL CONSTRAINTS:**\n"
            "   - DO NOT copy implementation code into specifications\n"
            "   - DO NOT delete `// TODO: add proof` or `// TODO: add loop invariant` markers\n"
            "   - DO NOT add loop invariants (leave for proof-generation stage)\n"
            "   - DO NOT add vector length requirements without careful consideration\n"
            "   - DO NOT use AtomicBool::load in requires/ensures clauses\n"
            "   - DO NOT directly compare atomic load with boolean (e.g. atomic.load() == false)\n\n"
            "**Type System Rules:**\n"
            "   - Use `None::<T>` instead of bare `None` for type inference\n"
            "     * CORRECT: `ret == None::<T>`\n"
            "     * INCORRECT: `ret == None`\n\n"
            "**Field Access Rules:**\n"
            "   - Check if type implements View (has `spec fn view()`) before using @\n"
            "   - For types without View: use direct field access `self.field`\n"
            "   - For types with View: use `self@.field` (the @ is shorthand for .view())\n"
            "   - For tuple views: use `self@.0`, `self@.1`, etc.\n"
            "     * CRITICAL: When using tuple access with comparison operators (e.g., `<`, `>`), wrap BOTH sides in parentheses\n"
            "     * CORRECT: `(x as nat) < (self@.0)`\n"
            "     * INCORRECT: `x as nat < self@.0` (causes parser error 'expected `,`')\n"
            "     * This applies to any comparison with casts or complex expressions\n\n"
            "**Other rules**:\n"
            "   - NO match/let in requires/ensures (but allowed in spec fn bodies)\n"
            "   - Spec functions cannot have requires/ensures\n\n"
            "**RETURN FORMAT:**\n"
            "   - Return the ENTIRE file with your changes, not just modified parts"
        )

    def _build_invariant_instruction(self, has_type_invariant: bool) -> str:
        """Build invariant-specific instruction based on code features."""
        if has_type_invariant:
            return (
                "\n**INVARIANT HANDLING**:\n"
                "This code has `#[verifier::type_invariant]` - type invariants are AUTOMATIC:\n"
                "- DO NOT add `self.inv()` or `old(self).inv()` to requires clauses (already implicit)\n"
                "- DO NOT add `self.inv()` to ensures clauses (already checked by Verus)\n"
                "- Adding them causes error: 'cannot refer to private function'\n"
                "- Focus on functional postconditions: return values, state relationships, etc.\n\n"
                "Example:\n"
                "```rust\n"
                "pub fn enqueue(&mut self, val: T) -> (ret: bool)\n"
                "    // NO requires self.inv() needed!\n"
                "    ensures\n"
                "        ret ==> self@.len() == old(self)@.len() + 1,  // Functional postcondition\n"
                "        ret ==> self@.last() == val\n"
                "```\n"
            )
        else:
            return (
                "\n**INVARIANT HANDLING**:\n"
                "This code uses spec function invariants (well_formed(), inv() without attribute):\n"
                "- MUST explicitly add to requires: `old(self).well_formed()` or `old(self).inv()`\n"
                "- MUST explicitly add to ensures: `self.well_formed()` or `self.inv()`\n"
                "- Verus does NOT automatically enforce these - explicit inclusion required\n\n"
                "Example:\n"
                "```rust\n"
                "pub fn insert(&mut self, key: u64, value: V)\n"
                "    requires\n"
                "        old(self).well_formed(),  // Must add!\n"
                "    ensures\n"
                "        self.well_formed(),  // Must add!\n"
                "        self.as_map() == old(self).as_map().insert(key, value)\n"
                "```\n"
            )

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
                use_cache = True
                # use_cache = False  # Disable cache for retries

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
            self.logger.info(
                f"Calling LLM engine: {engine}, answer_num: 3, use_cache: {use_cache}"
            )

            if context is not None:
                result = context.infer_llm_with_tracking(
                    engine=engine,
                    instruction=instruction,
                    exemplars=examples or [],
                    query=code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 8192),
                    temp=1.0 + (retry_attempt * temperature_boost),
                    use_cache=use_cache,
                    stage="spec_inference",
                    module="spec_inference",
                )

                # CRITICAL DEBUG: Log what we got back
                self.logger.info(f"LLM returned result type: {type(result)}")

                # Unwrap answers if messages/usage tuple returned
                if isinstance(result, tuple):
                    self.logger.info(f"Result is tuple with {len(result)} elements")
                    result = result[0]

                self.logger.info(
                    f"After unwrap - result type: {type(result)}, length: {len(result) if isinstance(result, list) else 'N/A'}"
                )

                if not result:
                    self.logger.error(
                        "CRITICAL: LLM returned empty result after unwrapping!"
                    )
                elif isinstance(result, list) and len(result) == 0:
                    self.logger.error("CRITICAL: LLM returned empty list!")

                return result
            else:
                result = self.llm.infer_llm(
                    engine,
                    instruction,
                    examples or [],
                    code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 8192),
                    temp=1.0 + (retry_attempt * temperature_boost),
                    use_cache=use_cache,  # Pass cache flag to LLM
                )

                self.logger.info(
                    f"LLM returned (no context) - type: {type(result)}, length: {len(result) if isinstance(result, list) else 'N/A'}"
                )
                return result
        except Exception as e:
            self.logger.error(f"EXCEPTION during LLM inference: {e}")
            import traceback

            self.logger.error(f"Traceback: {traceback.format_exc()}")
            return []

    def _has_type_invariant(self, code: str) -> bool:
        """Check if code has type invariant attribute."""
        return "#[verifier::type_invariant]" in code

    def check_code_safety(self, original_code: str, generated_code: str) -> bool:
        """Check if the generated code is safe to use."""
        # First check if code changes are safe using existing function
        if not code_change_is_safe(
            origin_code=original_code,
            changed_code=generated_code,
            verus_path=self.config.get("verus_path", "verus"),
            logger=self.logger,
            immutable_funcs=self.immutable_funcs,
        ):
            return False

        # Check for preservation of TODO markers
        # NOTE: spec_inference should REPLACE "// TODO: add requires and ensures"
        # but should PRESERVE "// TODO: add proof" and "// TODO: add loop invariant"
        todo_markers_to_preserve = ["// TODO: add proof", "// TODO: add loop invariant"]

        for marker in todo_markers_to_preserve:
            original_count = original_code.count(marker)
            generated_count = generated_code.count(marker)

            if original_count > generated_count:
                self.logger.warning(
                    f"Generated code removed {marker} marker(s). "
                    f"Original had {original_count}, generated has {generated_count}. "
                    f"spec_inference should preserve these markers for later stages."
                )
                return False

        # Check that spec_inference is actually making changes (replacing spec TODOs)
        spec_todo_marker = "// TODO: add requires and ensures"
        orig_spec_todos = original_code.count(spec_todo_marker)
        gen_spec_todos = generated_code.count(spec_todo_marker)

        if orig_spec_todos > 0 and gen_spec_todos == orig_spec_todos:
            self.logger.warning(
                f"Generated code did not replace any '{spec_todo_marker}' markers. "
                f"Found {orig_spec_todos} in both original and generated code. "
                f"LLM may not be following instructions."
            )
            # Don't return False here - still accept the candidate
            # This is just a warning that LLM might not be working correctly

        # Enforce function header visibility/spec markers are preserved exactly
        # Build multisets of (visibility, is_spec, name) for all fn headers
        header_pattern = re.compile(
            r"^\s*(?:(pub|open|closed)\s+)?(?:(spec)\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
            re.MULTILINE,
        )

        def extract_headers(code: str):
            from collections import Counter

            items = []
            for m in header_pattern.finditer(code):
                vis = m.group(1) or ""
                is_spec = "spec" if m.group(2) else ""
                name = m.group(3)
                items.append((vis, is_spec, name))
            return Counter(items)

        orig_headers = extract_headers(original_code)
        gen_headers = extract_headers(generated_code)

        if orig_headers != gen_headers:
            self.logger.warning(
                "Generated code modified function/spec headers (visibility/spec/name mismatch). Rejecting candidate."
            )
            self.logger.debug(f"Original headers: {orig_headers}")
            self.logger.debug(f"Generated headers: {gen_headers}")
            return False

        return True

    def _process_responses(
        self, responses: List[str], original_code: str, context_msg: str = ""
    ) -> List[str]:
        """Process and validate LLM responses."""
        safe_responses = []
        for response in responses:
            # Apply debug_type_error to fix any type errors
            fixed_response, _ = debug_type_error(response, logger=self.logger)
            temp_response = fixed_response if fixed_response else response

            # Apply regex-based syntax fixes FIRST (fast, deterministic)
            from src.modules.repair_regex import fix_common_syntax_errors

            temp_response, was_changed = fix_common_syntax_errors(
                temp_response, self.logger
            )
            if was_changed:
                self.logger.info(
                    "Applied regex syntax fixes to spec inference response"
                )

            # Fix syntax issues in requires/ensures clauses (prevents syntax errors)
            final_response = fix_spec_syntax_issues(temp_response)

            # Log if we fixed syntax issues
            if final_response != temp_response:
                self.logger.info(
                    f"Fixed syntax issues in requires/ensures clauses{context_msg}"
                )

            # Check if the generated code is safe
            if self.check_code_safety(original_code, final_response):
                safe_responses.append(final_response)
                self.logger.info(
                    f"Generated spec code passed safety check{context_msg}"
                )
            else:
                self.logger.warning(
                    f"Generated spec code failed safety check{context_msg}"
                )
        return safe_responses

    def exec(self, context) -> str:
        """
        Execute the spec inference module with the given context.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with inferred requires and ensures clauses
        """
        self.logger.info("Spec Inference (requires/ensures) ...")

        # Get the latest trial code
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Detect if code has type invariant
        has_type_invariant = self._has_type_invariant(code)
        if has_type_invariant:
            self.logger.info(
                "Detected #[verifier::type_invariant] - will customize instruction"
            )

        max_retries = 3
        safe_responses = []
        all_candidates = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"Spec inference attempt {retry_attempt + 1}/{max_retries}"
            )

            # Build base instruction with invariant-specific guidance integrated
            invariant_instruction = self._build_invariant_instruction(
                has_type_invariant
            )
            full_base_instruction = self.inference_instruction + invariant_instruction

            # Build the complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=full_base_instruction,
                add_common=True,
                add_requires_ensures=True,  # Include requires/ensures formatting
                add_match=True,  # Include match syntax guidelines
                code=code,
                # knowledge="",
                knowledge=context.gen_knowledge(),
            )

            # Load examples showing completed specifications (answer-only format)
            # Dynamic selection based on detected code features
            raw_examples = get_examples(
                self.config, "requires", self.logger, max_examples=20
            )

            # Score and prioritize examples based on code features
            scored_examples = []
            for ex in raw_examples:
                answer = ex.get("answer", "")
                score = 0

                # Option<Box<>> patterns (node, bst_map, option, treemap)
                if "Option<Box<" in code and "Option<Box<" in answer:
                    score += 50

                # Tree/BST structures (node, bst_map, treemap)
                if any(kw in code for kw in ["left", "right", "Node<", "TreeNode"]):
                    if any(
                        kw in answer
                        for kw in ["left", "right", "TreeNode", "tree", "as_map"]
                    ):
                        score += 45

                # Map operations (bst_map, treemap)
                if "Map<" in code:
                    if "Map<" in answer and ("insert" in answer or "to_map" in answer):
                        score += 40

                # Type invariant (rb_type_invariant, invariants)
                if has_type_invariant and "type_invariant" in answer:
                    score += 35

                # Circular/modulo operations (rb_type_invariant)
                if "%" in code or "ring" in code.lower() or "circular" in code.lower():
                    if "%" in answer or "ring" in answer.lower():
                        score += 35

                # Atomic/concurrency (atomics, rwlock)
                if any(kw in code for kw in ["Atomic", "rwlock", "lock"]):
                    if any(kw in answer for kw in ["Atomic", "lock"]):
                        score += 40

                # Bit operations (bitmap)
                if any(kw in code for kw in ["bit", "BitMap", "u64"]):
                    if any(kw in answer for kw in ["bit", "BitMap"]):
                        score += 35

                # Vector/Set operations (vectors, set_from_vec)
                if "Vec<" in code or "Set<" in code:
                    if any(kw in answer for kw in ["Vec<", "Set<", "vector", "set"]):
                        score += 30

                # Sequence operations (most benchmarks with Seq)
                if "Seq<" in code:
                    if "Seq<" in answer and ("subrange" in answer or "@" in answer):
                        score += 25

                # Transfer/simple patterns
                if "transfer" in code.lower() or len(code) < 2000:
                    if len(answer) < 1000:
                        score += 15

                # Penalize overly complex examples
                if len(answer) > 2500:
                    score -= 10

                scored_examples.append((score, ex))

            # Sort by score (highest first) and take top 5
            scored_examples.sort(key=lambda x: x[0], reverse=True)
            selected_examples = [ex for score, ex in scored_examples[:5]]

            # Convert to answer-only format
            examples = []
            for i, ex in enumerate(selected_examples):
                if ex.get("answer"):
                    examples.append(
                        {
                            "query": f"Example {i+1}: Pattern for writing requires/ensures specifications",
                            "answer": ex["answer"],
                        }
                    )

            self.logger.info(
                f"Selected {len(examples)} most relevant spec examples from {len(raw_examples)} available"
            )
            if has_type_invariant:
                self.logger.info("  - Prioritized type_invariant examples")
            if "Option<Box<" in code:
                self.logger.info("  - Prioritized Option<Box<>> examples")
            if "Map<" in code:
                self.logger.info("  - Prioritized Map operations examples")

            # Use cache only for first attempt
            responses = self._get_llm_responses(
                instruction,
                code,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                context=context,
                # use_cache=(retry_attempt == 0)
            )

            # CRITICAL DEBUG: Log if we got responses
            if not responses:
                self.logger.error(
                    f"CRITICAL: _get_llm_responses returned EMPTY list on attempt {retry_attempt + 1}! "
                    f"Check LLM connectivity, API keys, or if LLM is timing out."
                )
            else:
                self.logger.info(
                    f"✓ Received {len(responses)} responses from LLM on attempt {retry_attempt + 1}"
                )

            # Collect all processed candidates (even if not safe)
            if responses:
                self.logger.info(f"Processing {len(responses)} responses...")
                identical_count = 0
                for idx, resp in enumerate(responses):
                    # First fix type errors
                    fixed_resp, _ = debug_type_error(resp, logger=self.logger)
                    temp_resp = fixed_resp if fixed_resp else resp

                    # Then fix syntax issues (cast parentheses, trailing commas)
                    final_resp = fix_spec_syntax_issues(temp_resp)

                    # Log if we fixed syntax issues
                    if final_resp != temp_resp:
                        self.logger.info(f"Fixed syntax issues in response {idx+1}")

                    # Check if response is actually different from original AFTER processing
                    if final_resp.strip() == original_code.strip():
                        self.logger.warning(
                            f"Response {idx+1} is identical to original code after processing - LLM made no changes"
                        )
                        identical_count += 1
                        # Still add it to candidates if we have nothing else
                        # Don't continue/skip - we want to evaluate it

                    all_candidates.append(final_resp)
                    self.logger.info(
                        f"Added response {idx+1} to candidates pool (total: {len(all_candidates)})"
                    )

                if identical_count == len(responses):
                    self.logger.error(
                        f"CRITICAL: All {len(responses)} responses are identical to input! "
                        f"LLM is not making any changes. Check cache or prompt."
                    )
            else:
                self.logger.warning(
                    f"LLM returned EMPTY responses on attempt {retry_attempt + 1}"
                )

            # Process responses for safety
            new_safe = self._process_responses(responses, original_code)
            self.logger.info(
                f"Safety check: {len(new_safe)}/{len(responses)} responses passed safety checks on attempt {retry_attempt + 1}"
            )
            safe_responses.extend(new_safe)

            # Log current state
            self.logger.info(
                f"Cumulative stats - All candidates: {len(all_candidates)}, Safe candidates: {len(safe_responses)}"
            )

            if safe_responses:
                self.logger.info(
                    f"✓ Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts - stopping retries"
                )
                break
            else:
                self.logger.warning(
                    f"✗ No safe responses yet after attempt {retry_attempt + 1}/{max_retries}"
                )

            if retry_attempt < max_retries - 1:
                self.inference_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed safety checks. "
                    f"Please ensure your specifications maintain semantic equivalence "
                    f"and do not modify immutable functions. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # CRITICAL DECISION POINT: Decide candidate pool for evaluation
        self.logger.info(f"=== CANDIDATE SELECTION ===")
        self.logger.info(f"Total safe responses: {len(safe_responses)}")
        self.logger.info(f"Total all candidates: {len(all_candidates)}")

        # ALWAYS keep at least one candidate even if safety checks fail
        if safe_responses:
            candidates_for_eval = safe_responses
            self.logger.info(
                f"✓ Using {len(safe_responses)} SAFE candidates for evaluation"
            )
        elif all_candidates:
            self.logger.warning(
                f"⚠ No safe responses found; proceeding with best of {len(all_candidates)} UNSAFE candidates"
            )
            candidates_for_eval = all_candidates
        else:
            # No candidates generated at all - LLM returned empty responses
            # This is unusual, log it and return original
            self.logger.error(
                "❌ CRITICAL: No candidates generated after all retries - LLM returned empty responses. "
                "Check LLM connectivity and prompts. Using original code."
            )
            self.logger.info(f"=== RETURNING ORIGINAL CODE UNCHANGED ===")
            return original_code

        self.logger.info(
            f"✓ Selected {len(candidates_for_eval)} candidates to evaluate"
        )

        # Save all generated samples
        output_dir = samples_dir()
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = best_dir()
        global_dir.mkdir(exist_ok=True, parents=True)

        # Evaluate selected candidates and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=candidates_for_eval,
            output_dir=output_dir,
            prefix="04_spec_inference",
            logger=self.logger,
        )

        # Final safety check on the best code (do not discard; keep one candidate)
        if not self.check_code_safety(original_code, best_code):
            self.logger.warning(
                "Best generated code failed final safety check; keeping generated code to satisfy keep-one policy"
            )

        # Check for compilation errors and attempt repair if needed
        context.add_trial(best_code)  # Add trial to get evaluation results
        latest_trial = context.trials[-1]
        self.logger.info("Latest trial eval:")
        self.logger.info(latest_trial.eval.compilation_error)
        if latest_trial.eval.compilation_error:
            self.logger.info("Detected compilation error, attempting repair...")
            from src.modules.repair_registry import RepairRegistry

            repair_registry = RepairRegistry(
                self.config, self.logger, self.immutable_funcs
            )
            repaired_code = repair_registry.repair_compilation_error(context)
            if repaired_code and repaired_code != best_code:
                self.logger.info("Successfully repaired compilation error")
                best_code = repaired_code
                context.add_trial(best_code)  # Add the repaired code as a new trial

        # Get the global best from context
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()

        # Update global best if current best is better, but don't use it for the current step
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save the best spec inference from this step to a module-specific file
        module_best_path = output_dir / "04_spec_inference_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best spec inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best spec inference: {e}")

        # Store the updated global best in context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from current step to context
        context.add_trial(best_code)

        return best_code
