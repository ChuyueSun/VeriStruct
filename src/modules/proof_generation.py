"""
Module for generating proof blocks in Verus code wherever `// TODO: add proof` appears.

This module is intended to run *after* `spec_inference` if the planner detects
that proof stubs remain.  It analyzes the code and replaces every occurrence of
`// TODO: add proof` (or similar) with a proper `proof { ... }` block that
helps Verus discharge the outstanding obligations.
"""

import re  # Added for regex detection of empty proof blocks
from pathlib import Path
from typing import Dict, List

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.lynette import lynette
from src.modules.utils import (
    code_change_is_safe,
    debug_type_error,
    evaluate_samples,
    get_examples,
    update_checkpoint_best,
)
from src.prompts.template import build_instruction, load_prompt
from src.utils.path_utils import best_dir, prompt_dir, samples_dir


class ProofGenerationModule(BaseModule):
    """Module that fills in proof blocks for Verus verification."""

    def __init__(self, config, logger):
        super().__init__(
            name="proof_generation",
            desc="Generate proofs for Verus functions",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for proof generation (loaded from prompts/verus_proof.md)
        self.proof_instruction = load_prompt("verus_proof")

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
                    use_cache=use_cache,  # Pass cache flag to LLM
                    stage="proof_generation",
                    module="proof_generation",
                )
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
                    temp=1.0 + (retry_attempt * temperature_boost),
                    use_cache=use_cache,  # Pass cache flag to LLM
                )
        except Exception as e:
            self.logger.error(f"Error during LLM inference: {e}")
            return []

    def _process_responses(
        self,
        responses: List[str],
        original_code: str,
        context_msg: str = "",
        verus_path: str = "verus",
    ) -> List[str]:
        """Process and validate LLM responses."""
        safe_responses = []

        def normalize_verus_syntax(code: str) -> str:
            """Normalize common Verus proof syntax issues to match canonical patterns.

            - Replace chained inequalities (e.g., 0 <= x < y) with conjunctions
            - Ensure 'assert forall' has 'by {}' clause (CRITICAL)
            - Replace 'implies' with '==>' inside forall assertions
            - Fix invalid 'let ... in' syntax in ensures clauses
            - Fix old() placement (old(*x) → *old(x))
            - Replace invalid @ notation when View not defined
            - Parenthesize casted ints in arithmetic (i as int) * 64
            """
            # 0) CRITICAL: Validate and fix assert forall syntax
            # Check if assert forall exists without 'by' clause
            def validate_and_fix_assert_forall(code_text: str) -> str:
                """Ensure all assert forall statements have 'by' clause."""
                lines = code_text.split("\n")
                result_lines = []
                i = 0

                while i < len(lines):
                    line = lines[i]
                    stripped = line.lstrip()

                    if stripped.startswith("assert forall"):
                        # Found assert forall - collect the complete statement
                        assert_lines = [line]
                        j = i + 1
                        found_by = "by" in line
                        found_semicolon = ";" in line

                        # Look ahead to find 'by' or ';'
                        while j < len(lines) and not found_semicolon and not found_by:
                            next_line = lines[j]
                            assert_lines.append(next_line)
                            if "by" in next_line:
                                found_by = True
                            if ";" in next_line:
                                found_semicolon = True
                            j += 1

                        # If we found semicolon but no 'by', we need to fix it
                        if found_semicolon and not found_by:
                            # Remove the trailing semicolon and add 'by {}'
                            fixed_lines = []
                            for al in assert_lines:
                                if ";" in al:
                                    # Replace semicolon with 'by { }'
                                    fixed_lines.append(
                                        al.replace(";", " by {\n    \n}")
                                    )
                                else:
                                    fixed_lines.append(al)

                            result_lines.extend(fixed_lines)
                            self.logger.warning(
                                f"Fixed assert forall without 'by' clause at line {i+1}. "
                                "This is a common syntax error in Verus."
                            )
                        else:
                            result_lines.extend(assert_lines)

                        i = j
                    else:
                        result_lines.append(line)
                        i += 1

                return "\n".join(result_lines)

            code = validate_and_fix_assert_forall(code)

            # 1) Replace 'implies' with '==>' in assert forall lines
            def replace_implies_in_forall(line: str) -> str:
                if line.lstrip().startswith("assert forall") and " implies" in line:
                    return line.replace(" implies", " ==>")
                return line

            code = "\n".join(replace_implies_in_forall(ln) for ln in code.splitlines())

            # 2) Replace chained inequalities: a <= b < c  ->  a <= b && b < c
            # Handle common patterns with variables/casts
            def fix_chained(match: re.Match) -> str:
                left = match.group(1).strip()
                mid = match.group(2).strip()
                right = match.group(3).strip()
                return f"{left} <= {mid} && {mid} < {right}"

            # Specific common case first: 0 <= i <= j < EXPR (triple chained with mixed <= and <)
            # Must handle <= properly to avoid creating split operators
            code = re.sub(
                r"0\s*<=\s*(\w+)\s*<=\s*(\w+)\s*<\s*([^\n,)+-/*]+)",
                r"0 <= \1 && \1 <= \2 && \2 < \3",
                code,
            )

            # Also handle: 0 <= n <= EXPR (double <= chain, no final <)
            # This prevents the bug where <= gets split into < =
            code = re.sub(
                r"0\s*<=\s*(\w+)\s*<=\s*([^\n,)+-/*<]+)", r"0 <= \1 && \1 <= \2", code
            )

            # Simpler case: 0 <= k < EXPR (double chained with < only)
            code = re.sub(
                r"0\s*<=\s*(\w+)\s*<\s*([^\n,)+-/*=]+)", r"0 <= \1 && \1 < \2", code
            )

            # General chained case: X <= Y < Z
            code = re.sub(
                r"([\w@()\s+\-*/]+?)<=\s*([\w@()]+)\s*<\s*([\w@()\s+\-*/]+)",
                fix_chained,
                code,
            )

            # 3) Parenthesize '(i as int) * 64' forms
            code = re.sub(r"(\w+)\s+as\s+int\s*\*\s*64", r"(\1 as int) * 64", code)

            # 4) CRITICAL: Add assert_seqs_equal import if macro is used
            if (
                "assert_seqs_equal!" in code
                and "use vstd::assert_seqs_equal" not in code
            ):
                self.logger.warning(
                    "Code uses assert_seqs_equal! but missing import, adding it"
                )
                # Add import after use vstd::prelude::*;
                code = code.replace(
                    "use vstd::prelude::*;",
                    "use vstd::prelude::*;\nuse vstd::assert_seqs_equal;",
                )

            # 5) CRITICAL: Remove duplicate main function if present
            # LLM sometimes adds boilerplate "fn main() {}" when code already has "pub fn main()"
            main_count = code.count("fn main(") + code.count("fn main {")
            if main_count > 1:
                self.logger.warning(
                    f"Found {main_count} main functions, removing duplicates"
                )
                lines = code.split("\n")
                result_lines = []
                for i, line in enumerate(lines):
                    # Remove simple boilerplate "fn main() {}" in first 10 lines
                    if i < 10 and "fn main() {}" in line:
                        self.logger.warning(f"Removing boilerplate main at line {i+1}")
                        continue
                    result_lines.append(line)
                code = "\n".join(result_lines)

            # 5) Prefer @ shorthand over .view()
            code = re.sub(r"([A-Za-z_][A-Za-z0-9_]*)\.view\(\)", r"\1@", code)

            # 6) Normalize result.view()[...] to result@[...]
            code = code.replace("result.view()[", "result@[")
            code = code.replace("self.view()[", "self@[")
            code = code.replace("bm.view()[", "bm@[")

            return code

        for response in responses:
            # Fix simple type errors
            fixed_response, _ = debug_type_error(response, logger=self.logger)
            final_response = fixed_response if fixed_response else response

            # Normalize Verus proof syntax to avoid common parser errors
            # NOTE: This MUST come before regex fixes to avoid introducing split operators
            final_response = normalize_verus_syntax(final_response)

            # Apply regex-based syntax fixes AFTER normalization to clean up any issues
            from src.modules.repair_regex import fix_common_syntax_errors

            final_response, was_changed = fix_common_syntax_errors(
                final_response, self.logger
            )
            if was_changed:
                self.logger.info(
                    "Applied regex syntax fixes to proof generation response"
                )

            # Check if the generated code is safe
            if code_change_is_safe(
                origin_code=original_code,
                changed_code=final_response,
                verus_path=verus_path,
                logger=self.logger,
            ):
                safe_responses.append(final_response)
                self.logger.info(
                    f"Generated proof code passed safety check{context_msg}"
                )
            else:
                self.logger.warning(
                    f"Generated proof code failed safety check{context_msg}"
                )
        return safe_responses

    # ---------------------------------------------------------------------
    # Helper
    # ---------------------------------------------------------------------

    def _detect_lemmas(self, code: str) -> List[str]:
        """Detect lemma definitions in the code and return list of lemma names."""
        lemmas = []

        # Pattern 1: proof fn lemma_name
        pattern1 = re.findall(r"proof\s+fn\s+(lemma_\w+)\s*\(", code)
        lemmas.extend(pattern1)

        # Pattern 2: pub proof fn lemma_name
        pattern2 = re.findall(r"pub\s+proof\s+fn\s+(lemma_\w+)\s*\(", code)
        lemmas.extend(pattern2)

        # Pattern 3: Any function with "lemma" in name
        pattern3 = re.findall(
            r"(?:pub\s+)?(?:proof\s+)?fn\s+(\w*lemma\w*)\s*\(", code, re.IGNORECASE
        )
        lemmas.extend(pattern3)

        # Remove duplicates while preserving order
        seen = set()
        unique_lemmas = []
        for lemma in lemmas:
            if lemma not in seen:
                seen.add(lemma)
                unique_lemmas.append(lemma)

        return unique_lemmas

    def _has_type_invariant(self, code: str) -> bool:
        """Check if code has type invariant attribute."""
        return "#[verifier::type_invariant]" in code

    def _should_skip(self, code: str) -> bool:
        """Return True only if there are no TODO markers or empty proof blocks."""

        # If any TODO markers or placeholders exist, we should not skip.
        if (
            ("TODO: add proof" in code)
            or ("TODO:add proof" in code)
            or ("TODO: add invariants" in code)
            or ("TODO: add invariant" in code)
            or ("TODO: add assert" in code)
            or ("TODO: add asserts" in code)
            or ("Proof body here if needed" in code)
        ):
            return False

        # Detect empty proof blocks such as `proof{}`, `proof {}`, or `proof {\n}`
        if re.search(r"proof\s*{\s*}\s*", code) or re.search(
            r"proof\s*{\s*//[^\n]*\n\s*}\s*", code
        ):  # Matches proof blocks with only comments
            return False

        return True

    def exec(self, context) -> str:  # type: ignore[override]
        """Run proof generation on the latest trial in *context*."""
        self.logger.info("Proof Generation ...")

        # Current code to operate on
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Early exit if no proof markers exist
        if self._should_skip(code):
            self.logger.info(
                "No '// TODO: add proof' markers found – skipping proof generation."
            )
            return code

        # Detect code features to customize instruction dynamically
        has_type_invariant = self._has_type_invariant(code)
        lemmas_in_code = self._detect_lemmas(code)

        if has_type_invariant:
            self.logger.info(
                "Detected #[verifier::type_invariant] - will add use_type_invariant guidance"
            )

        if lemmas_in_code:
            self.logger.info(
                f"Detected {len(lemmas_in_code)} lemma(s): {', '.join(lemmas_in_code[:5])}"
            )
            self.logger.info("Will add lemma invocation guidance to prompt")

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"Proof generation attempt {retry_attempt + 1}/{max_retries}"
            )

            # Build instruction with common Verus knowledge and match guidelines
            instruction = build_instruction(
                base_instruction=self.proof_instruction,
                add_common=True,
                add_match=True,
                code=code,
                knowledge=context.gen_knowledge(),
            )

            # Dynamically add lemma invocation guidance if lemmas detected
            if lemmas_in_code:
                lemma_guidance = f"\n\n**DETECTED LEMMAS IN THIS FILE**: {', '.join(lemmas_in_code)}\n\n"
                lemma_guidance += "**CRITICAL: You MUST invoke these lemmas in your proof blocks!**\n\n"
                lemma_guidance += "Call the relevant lemmas:\n"
                lemma_guidance += "```rust\n"
                lemma_guidance += "proof {\n"
                if has_type_invariant:
                    lemma_guidance += "    use_type_invariant(&*self);  // First\n"
                for lemma in lemmas_in_code[:3]:  # Show up to 3 examples
                    if "mod_auto" in lemma:
                        lemma_guidance += f"    {lemma}(self.ring.len() as int);  // For modulo operations\n"
                    else:
                        lemma_guidance += f"    {lemma}(...);  // Check lemma signature for parameters\n"
                lemma_guidance += "}\n```\n"
                lemma_guidance += f"\n**These lemmas establish properties** that help prove your assertions. Check each lemma's `ensures` clause to understand what it proves.\n"

                instruction += lemma_guidance

            # Load examples showing completed proofs/invariants (answer-only format)
            # Dynamic selection based on detected code features
            raw_examples = get_examples(
                self.config, "proof", self.logger, max_examples=20
            )

            # Prioritize examples based on code features
            scored_examples = []
            for ex in raw_examples:
                answer = ex.get("answer", "")
                score = 0

                # Critical: Type invariant examples (rb_type_invariant, invariants)
                if has_type_invariant and "type_invariant" in answer:
                    score += 100

                # Important: Lemma invocation examples
                if lemmas_in_code and "lemma_" in answer and "proof {" in answer:
                    score += 50

                # Structural patterns matching
                # Option<Box<>> patterns (node, bst_map, option)
                if "Option<Box<" in code and "Option<Box<" in answer:
                    score += 40

                # Tree/BST structures (bst_map, treemap, node)
                if any(kw in code for kw in ["left", "right", "Node<", "TreeNode"]):
                    if any(
                        kw in answer for kw in ["left", "right", "TreeNode", "tree"]
                    ):
                        score += 35

                # Map operations (bst_map, treemap)
                if "Map<" in code and ".insert" in code:
                    if "Map<" in answer and "to_map" in answer:
                        score += 35

                # Circular/modulo operations (rb_type_invariant)
                if "%" in code or "circular" in code.lower():
                    if "%" in answer or "mod_" in answer:
                        score += 40

                # Atomic/concurrency patterns (atomics, rwlock)
                if any(kw in code for kw in ["Atomic", "rwlock", "lock"]):
                    if any(kw in answer for kw in ["Atomic", "lock", "acquire"]):
                        score += 40

                # Bit operations/chunking (bitmap)
                if any(kw in code for kw in ["bit", "chunk", "u64", "Chunked"]):
                    if any(kw in answer for kw in ["chunk", "bit", "u64", "get_bit"]):
                        score += 35

                # View trait (most benchmarks)
                if "impl View" in code or "impl<" in code and "View" in code:
                    if "spec fn view" in answer:
                        score += 30

                # Loop invariants (most benchmarks)
                if "for " in code or "while " in code:
                    if "invariant" in answer:
                        score += 20

                # Sequence operations (vectors, bitmap, rb)
                if "Seq<" in code:
                    if "Seq<" in answer and ("subrange" in answer or "push" in answer):
                        score += 25

                # Penalize overly complex examples
                if len(answer) > 2000:
                    score -= 10

                scored_examples.append((score, ex))

            # Sort by score (highest first) and take top 6
            scored_examples.sort(key=lambda x: x[0], reverse=True)
            selected_examples = [ex for score, ex in scored_examples[:6]]

            # Convert to answer-only format
            examples = []
            for i, ex in enumerate(selected_examples):
                if ex.get("answer"):
                    examples.append(
                        {
                            "query": f"Example {i+1}: Pattern for writing proofs and loop invariants",
                            "answer": ex["answer"],
                        }
                    )

            self.logger.info(
                f"Selected {len(examples)} most relevant examples from {len(raw_examples)} available"
            )
            if has_type_invariant:
                self.logger.info("  - Prioritized type_invariant examples")
            if lemmas_in_code:
                self.logger.info("  - Prioritized lemma invocation examples")

            # Save prompt for debugging
            prompt_path = prompt_dir()
            prompt_file = prompt_path / f"proof_generation_{retry_attempt + 1}.txt"
            prompt_file.write_text(instruction)
            self.logger.info(f"Saved proof generation prompt to {prompt_file}")

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
            if not responses and retry_attempt == max_retries - 1:
                return code

            safe_responses.extend(
                self._process_responses(
                    responses,
                    original_code,
                    context_msg="",
                    verus_path=self.config.get("verus_path", "verus"),
                )
            )

            if safe_responses:
                self.logger.info(
                    f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                self.proof_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed safety checks. "
                    f"Please ensure your proof blocks do not modify any existing code "
                    f"and only add new proof blocks. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            return original_code

        # Evaluate samples and select the best one
        output_dir = samples_dir()
        global_dir = best_dir()

        best_code, best_score, _ = evaluate_samples(
            samples=safe_responses,
            output_dir=output_dir,
            prefix="05_proof_generation",
            logger=self.logger,
        )

        # Final safety check on the best code
        if not code_change_is_safe(
            origin_code=original_code,
            changed_code=best_code,
            verus_path=self.config.get("verus_path", "verus"),
            logger=self.logger,
        ):
            self.logger.warning(
                "Best generated code failed final safety check, falling back to original"
            )
            best_code = original_code

        # Update global checkpoint best (but don't overwrite current trial yet)
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save the best proof generation from this step to a module-specific file
        module_best_path = output_dir / "05_proof_generation_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best proof generation to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best proof generation: {e}")

        # Store the updated global best in context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from current step to context
        context.add_trial(best_code)

        return best_code
