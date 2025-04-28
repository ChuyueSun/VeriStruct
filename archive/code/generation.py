# Copyright (c) Microsoft Corporation. #
# Licensed under the MIT license.      #

import json
import os
import re
from pathlib import Path
from typing import Callable, List

from houdini import houdini
from refinement import Refinement
from veval import EvalScore, VEval

from infer import LLM
from utils import clean_code, code_change_is_safe, evaluate, get_nonlinear_lines

ANSWER_NUM = 5  # It needs to be 1 if using deepseek-reasoner


class Generation:
    def __init__(
        self,
        config,
        logger,
        phase1_examples=["3", "6", "7"],
        view_examples=["8", "9"],
        repair_uniform=False,
        test_repair=False,
        immutable_funcs=[],
    ):
        self.config = config
        self.llm = LLM(config, logger)
        global ANSWER_NUM
        if "deepseek-reasoner" in self.config.aoai_generation_model:
            ANSWER_NUM = 1
        self.logger = logger
        self.default_refine_funcs = [
            self.constantrefine_inference,
            self.arraylen_inference,
            self.arrayrefine_inference,
            self.condlooprefine_inference,
        ]
        self.hdn = houdini(config, immutable_funcs=immutable_funcs)
        self.refinement = Refinement(config, logger, immutable_funcs=immutable_funcs)
        self.phase1_examples = phase1_examples
        self.view_examples = view_examples
        self.repair_uniform = repair_uniform
        self.test_repair = test_repair
        self.immutable_funcs = immutable_funcs

        self.logger.info(
            "Generation initialized with phase1_examples: %s", self.phase1_examples
        )
        self.logger.info(
            "Generation initialized with view_examples: %s", self.view_examples
        )
        self.logger.info(
            "Generation initialized with repair_uniform: %s", self.repair_uniform
        )

        self.data_structure_funcs = [
            self.view_inference,
            self.view_refinement,
            self.inv_inference,
            # self.inv_refinement,
            self.requires_inference,
        ]
        self.system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."
        self.important_note = """**Important Notes**:
- Don't delete existing non-buggy `#[trigger]`!
- Don't change "unwind" to `(unwind) as bool`!
- Return the complete modified Rust code in your response without explanations."""
        self.require_ensure_knowledge = """**Formatting for `requires` and `ensures`**:
```rust
fn func(arg) -> rettype
    requires
        REQUIREMENT1,
        REQUIREMENT2,
        ...
    ensures
        ENSUREMENT1,
        ENSUREMENT2,
        if COND {
        ENSUREMENT3
        } else {
        ENSUREMENT4
        }
        ...
```
- In requires, use `old(self)` to refer to the pre-state of an &mut variable.
- When using the return value in an `ensures` clause, assign it a name if not already provided (change the return type of the function), e.g.:
```rust
fn func(arg) -> (retname: rettype)
```
"""
        self.spec_knowledge = """**Spec Functions**:
1. No Direct Method Calls:
In a spec function, you cannot directly call instance methods such as vector.is_full().
2. Use the @ Operator:
To invoke methods on a variable within a spec, first convert it to its specification-level representation View with @.
3. Always use vector.len() instead of vector@.len().
4. Simplify Boolean Conjunctions:
When combining multiple conditions, avoid excessive &&&. Fewer (or well-structured) conjunctions make the spec code easier to read and debug.
"""
        self.operators_knowledge = """Verus extends Rust logical operators with low-precedence forms that are especially helpful in specification code:
Standard Operators: &&, ||, ==>, <==>
Low-Precedence Variants: &&& and |||
The meaning of &&& is the same as && (logical AND), and ||| is the same as || (logical OR), but with lower precedence. This allows you to write conditions in a "bulleted list" style that remains grouped in a logical manner:
```&&& a ==> b
&&& c
&&& d <==> e && f```
is equivalent to:
```(a ==> b) && c && (d <==> (e && f))```
Note:
Implication (==>) and equivalence (<==>) bind more tightly than &&& and |||.
Using &&&/||| can make long specifications clearer by grouping logical clauses neatly."""

    def wrap_code(self, code, veval):
        code += "\n// " + str(veval.get_score())
        code += "\n// " + ("".join(str(e) for e in veval.verus_errors)).replace(
            "\n", "\n// "
        )
        code += "\n// " + str(veval.rustc_out).replace("\n", "\n// ")
        code += "\n// " + str(veval.verus_out).replace("\n", "\n// ")
        return code

    def generate_view(self, code, temp_dir, answer_num=3, retry=3):
        """
        Enhanced generation pipeline:
        1. Dynamically determine needed refinement functions using LLM.
        2. Iteratively generate and evaluate candidate code using only necessary refinement steps.
        3. Repairs best code if necessary.
        """
        temp_dir = Path(temp_dir)
        best_code_of_all = code
        best_score_of_all = EvalScore.get_worst_score()

        for iteration in range(retry):
            # Step 1: Dynamically decide necessary refinement steps
            selected_funcs = self.select_refinement_funcs_llm(code)

            # Log chosen refinement methods for debugging
            self.logger.info(
                f"Iteration {iteration}: Selected refinement functions: {', '.join(func.__name__ for func in selected_funcs)}"
            )

            # Step 2: Run the refinement iteration using selected functions
            (
                cand_code,
                iteration_best_code,
                iteration_best_score,
            ) = self.run_dynamic_refinement(
                code=code,
                selected_funcs=selected_funcs,
                iteration_idx=iteration,
                temp_dir=temp_dir,
            )

            # Check if the candidate code is correct
            if self.check_and_handle_success(cand_code):
                self.logger.info("Successfully verified candidate code!")
                return cand_code

            # Update global best candidate if necessary
            best_score_of_all, best_code_of_all = self.update_global_best(
                cand_code, best_score_of_all, best_code_of_all, temp_dir
            )

            # Write iteration result
            self.write_iteration_result(cand_code, iteration, temp_dir)

        # Step 3: Attempt repair if no fully correct solution is found
        if best_score_of_all.is_correct():
            self.logger.info("Final best code is already correct!")
            (temp_dir / "view-correct.rs").write_text(best_code_of_all)
            return best_code_of_all

        self.logger.info("Attempting final repair on best code found...")
        return self.repair_and_finalize(best_code_of_all, temp_dir)

    def select_refinement_funcs_llm(self, code: str) -> List[Callable]:
        """
        Use LLM.infer_llm from infer.py to dynamically select necessary refinement steps from `self.data_structure_funcs`.
        Returns a subset list of refinement functions.
        """
        instruction = f"""
        Given the following Rust code for verification:

        {code}

    You must select the necessary steps from the following list to assemble a verification pipeline, based on their descriptions:

    - view_inference (optional, recommended for complex modules): Generates an abstract mathematical representation of the data structure, capturing its minimal essential properties for verification. If this step is selected, 'view_refinement' and  'inv_inference' should also be chosen.
    - view_refinement (optional): Improves the abstraction of an existing 'View' function to ensure minimal yet sufficient representation.
    - inv_inference (optional, recommended for complex modules): Creates an invariant function ('inv') to capture all necessary conditions and constraints of the data structure.
    - requires_inference (MANDATORY, must be performed last): Determines and appends appropriate 'requires' and 'ensures' clauses for public methods based on semantic analysis.

    Important: Choose 'view_inference' and 'view_refinement' only if you can clearly define a mathematical abstraction for the data structure using types like bool, int, nat, Seq<T>, Set<T>, or Map<K, V>. If unsure or the structure does not lend itself to such abstraction, do not select these steps. Choose these steps carefully.

    Respond strictly as a JSON-formatted list containing the names of the selected generation steps:
    ["view_inference", "view_refinement", "inv_inference", "requires_inference"]
or ["requires_inference"]
    Ensure that 'requires_inference' is always the last step in the list.
        """

        response = self.llm.infer_llm(
            engine=self.config.aoai_generation_model,
            instruction=instruction,
            exemplars=[],
            query="",
            system_info="You are an assistant that determines necessary refinement steps.",
            answer_num=1,
            max_tokens=8192,
            temp=0.0,
            json=True,
        )
        try:
            needed_funcs_names = json.loads(response[0])
            selected_funcs = [
                func
                for func in self.data_structure_funcs
                if func.__name__ in needed_funcs_names
            ]
            return selected_funcs if selected_funcs else self.data_structure_funcs
        except (json.JSONDecodeError, KeyError, IndexError) as e:
            self.logger.error(f"LLM response parsing error: {str(e)}")
            # Fallback to all functions if LLM fails
            return self.data_structure_funcs

    def run_dynamic_refinement(self, code, selected_funcs, iteration_idx, temp_dir):
        """
        Executes a single iteration of refinement using only the selected refinement functions.
        Returns candidate code, best code for this iteration, and best score.
        """
        cand_code = code
        best_code = cand_code
        best_score = EvalScore.get_worst_score()

        for func in selected_funcs:
            candidates = func(cand_code, temp=1.0, answer_num=ANSWER_NUM)
            cand_code, best_code, best_score = self.evaluate_candidates(
                candidates, iteration_idx, func, best_code, best_score, temp_dir
            )

        cand_code, _ = self.refinement.debug_type_error(cand_code)

        return cand_code, best_code, best_score

    def evaluate_candidates(
        self, candidates, iteration_idx, func, last_best_code, last_best_score, temp_dir
    ):
        """
        Evaluates multiple candidate codes generated by a single data_structure_func.
        Updates and returns the best candidate code and score for this iteration.
        """
        best_score = EvalScore.get_worst_score()
        best_code = last_best_code
        for j, cand in enumerate(candidates):
            cand = clean_code(cand)
            cand, _ = self.refinement.debug_type_error(cand)

            veval = VEval(cand, self.logger)
            score = veval.eval_and_get_score()

            # If code is correct according to VEval
            if score.is_correct():
                self.logger.info("Found a correct proof!")
                return cand, cand, score

            # # If Houdini can prove the code
            # failures, houdini_code = self.hdn.run(cand)
            # if len(failures) == 0:
            #     self.logger.info("Found a correct proof!")
            #     return houdini_code, houdini_code, score

            # Update the best candidate if needed
            if not (score < best_score):
                best_score = score
                best_code = cand

            # Write each candidate's code to a temp file
            self.write_candidate_code(cand, veval, iteration_idx, func, j, temp_dir)

        # Return the best after evaluating all candidates of this func
        if best_score.is_good_code_next_phase(last_best_score):
            return best_code, best_code, best_score
        else:
            return last_best_code, last_best_code, last_best_score

    # -------------------------------------------------------------------------
    # Helper Methods
    # -------------------------------------------------------------------------

    def check_and_handle_success(self, code):
        """
        Checks if the given code is correct. If it is, returns True (success).
        Otherwise, returns False.
        """
        veval = VEval(code, self.logger)
        score = veval.eval_and_get_score()
        return score.is_correct()

    def update_global_best(
        self, cand_code, best_score_of_all, best_code_of_all, temp_dir
    ):
        """
        Compares cand_code's score with the global best. If cand_code is better,
        update the global best and write it to a file.
        """
        veval = VEval(cand_code, self.logger)
        score = veval.eval_and_get_score()

        if score > best_score_of_all:
            best_score_of_all = score
            best_code_of_all = cand_code
            (temp_dir / "view-best.rs").write_text(
                self.wrap_code(best_code_of_all, veval)
            )

        return best_score_of_all, best_code_of_all

    def write_iteration_result(self, cand_code, iteration_idx, temp_dir):
        """
        Wraps the candidate code with context (e.g., VEval results) and
        writes it out to a file for the given iteration index.
        """
        veval = VEval(cand_code, self.logger)
        veval.eval()
        wrapped_code = self.wrap_code(cand_code, veval)
        (temp_dir / f"view-{iteration_idx}.rs").write_text(wrapped_code)

    def write_candidate_code(
        self, cand, veval, iteration_idx, func, cand_idx, temp_dir
    ):
        """
        Writes an individual candidate code out to a file, including VEval metadata.
        """
        (temp_dir / f"view-{iteration_idx}-{func.__name__}-{cand_idx}.rs").write_text(
            self.wrap_code(cand, veval)
        )

    def repair_and_finalize(self, best_code_of_all, temp_dir):
        """
        Attempt to repair the best code of all and finalize if successful;
        otherwise, write out what we have after the repair attempt.
        """
        repair_temp_dir = temp_dir / "repair"
        repair_temp_dir.mkdir(parents=True, exist_ok=True)

        if self.test_repair:
            self.logger.info("Repairing parameterized tests ...")
            code = self.refinement.repair_test_veval(
                best_code_of_all, temp_dir=repair_temp_dir
            )
        else:
            code = self.refinement.repair_veval(
                best_code_of_all, temp_dir=repair_temp_dir
            )

        veval = VEval(code, self.logger)
        score = veval.eval_and_get_score()

        if score.is_correct():
            self.logger.info("Found a correct proof after repair!")
            (temp_dir / "view-correct-after-repair.rs").write_text(code)
            return code
        # If Houdini can prove the code
        failures, houdini_code = self.hdn.run(code)
        if len(failures) == 0:
            h_veval = VEval(houdini_code, self.logger)
            score = h_veval.eval_and_get_score()
            if score.is_correct():
                self.logger.info("Found a correct proof after Houdini!")
                (temp_dir / "view-correct-after-repair.rs").write_text(houdini_code)
                return houdini_code

        wrapped_code = self.wrap_code(code, veval)
        (temp_dir / "view-best-after-repair.rs").write_text(wrapped_code)
        return code

    def view_inference(self, code, temp=0, answer_num=1):
        self.logger.info("View Inference ...")
        instruction = """
You are an expert in Verus (verifier for rust). Your task is to generate a View function for the given module. View is the mathematical abstraction for the given data structure. It contains the minimal information to completely represent it. View is used strictly in Verus spec.
    - Add a `View` spec function that provides a mathematical abstraction for types used in the executable code.
    - For `Vec` type variables in the `View`, append "@" to their names.
    - Fill in `/* TODO: part of view */`.
Mathematical types in Verus include:
    - bool
    - int
    - nat
    - Seq<T>
    - Set<T>
    - Map<K, V>

Steps:
    1. Infer the information should be contained in the return type of the `View` function. It could be any of the mathematical types mentioned above or a combination (tuple) of them.
    2. Generate the view function based on the inferred information. Return it as part of the input file.


Format:
```verus

impl<T: Copy> View for RingBuffer<T> {
    type V = // your inferred View return type here that contain the minimal information to represent the class

    closed spec fn view(&self) -> Self::V {
        ... // your implementation here
    }
}
```"""
        instruction += "\n\n" + self.important_note
        instruction += "\n\n" + self.spec_knowledge
        instruction = "\n\n" + self.refinement.add_seq_knowledge(code, instruction)
        examples = []
        example_path = Path(self.config.example_path) / "input-view"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = (
                        Path(self.config.example_path) / "output-view" / f.name
                    )
                    answer = answer_path.read_text() if answer_path.exists() else ""

                    examples.append({"query": input_content, "answer": answer})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            self.system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=1.0,
        )

    def view_refinement(self, code, temp=0, answer_num=1):
        self.logger.info("View Refinement ...")
        instruction = """
You are a highly experienced expert in Verus (the verifier for Rust). Your task is to refine the "View" function within the given Verus file. The "View" function is the mathematical abstraction for a data structure, capturing the minimal information needed for its specification in Verus.

Your responsibilities:
  1. Analyze the current "View" function to determine if its tuple (or other structure) adequately represents the module.
  2. Evaluate whether the abstraction can be improved. (Hint: If the tuple is identical to the internal fields, that is likely not an ideal abstraction.)
  3. Modify only the "View" function to improve its abstraction while leaving all other parts of the file unchanged.
  4. Use a flattened tuple.
  5. Return the **entire updated Verus file** with your refined "View" function.

Please provide only the complete Rust code of the refined file with no additional commentary.
"""
        #         instruction += "\n\n" + self.important_note
        #         instruction += "\n\n" + self.spec_knowledge
        #         instruction = "\n\n" + self.refinement.add_seq_knowledge(code, instruction)
        examples = []
        example_path = Path(self.config.example_path) / "input-view-refine"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = (
                        Path(self.config.example_path) / "output-view-refine" / f.name
                    )
                    answer = answer_path.read_text() if answer_path.exists() else ""

                    examples.append({"query": input_content, "answer": answer})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            self.system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=1.0,
        )

    def inv_inference(self, code, temp=0, answer_num=1):
        self.logger.info("Inv Inference ...")
        instruction = """You are an expert in Verus (a Rust-based verification framework). Given the following Rust code that defines a data structure with private fields, create a closed spec function: `closed spec fn inv(&self) -> bool`. This function should capture all necessary invariants of the data structure. You are allowed to reference private fields directly (i.e., do not rely on "view" conversions unless absolutely necessary). Do not modify other parts of the code or add explanatory text—just provide the final inv function definition."""
        instruction += "\n" + self.important_note
        instruction += "\n" + self.spec_knowledge
        instruction = self.refinement.add_seq_knowledge(code, instruction)

        examples = []
        example_path = Path(self.config.example_path) / "input-inv"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = Path(self.config.example_path) / "output-inv" / f.name
                    answer = answer_path.read_text() if answer_path.exists() else ""

                    examples.append({"query": input_content, "answer": answer})

        # Get responses from LLM
        responses = self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            self.system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=1.0,
        )

        # Process each response to replace @.len() with .len() in type invariants
        processed_responses = []
        for response in responses:
            processed = self.replace_at_len_in_type_invariant(response)
            processed_responses.append(processed)

        return processed_responses

    def replace_at_len_in_type_invariant(self, content):
        """
        Replace all instances of "@.len()" with ".len()" but only within functions
        labeled with #[verifier::type_invariant].
        """
        # Define regex pattern to find type_invariant blocks
        type_invariant_pattern = (
            r"(#\[verifier::type_invariant\][^{]*{((?:[^{}]|(?:\{[^{}]*\}))*)})"
        )

        # Use re.DOTALL to make '.' match newlines as well
        matches = re.finditer(type_invariant_pattern, content, re.DOTALL)

        # Make a copy of the content to modify
        modified_content = content

        # For each match, replace "@.len()" with .len() in the function block
        for match in matches:
            full_match = match.group(
                1
            )  # The entire type_invariant function including the attribute
            function_block = match.group(2)  # Just the function body

            # Replace @.len() with .len() in the function block
            modified_block = re.sub(r"@\.len\(\)", r".len()", function_block)

            # Update the content
            modified_full_match = full_match.replace(function_block, modified_block)
            modified_content = modified_content.replace(full_match, modified_full_match)

        return modified_content

    def inv_refinement(self, code, temp=0, answer_num=1):
        self.logger.info("Inv Refinement ...")
        instruction = """
You are a highly experienced expert in Verus (the verifier for Rust). Your task is to refine the "inv" function within the given Verus file. The "inv" function is the invariant function for a data structure, defining the invariants for a data structure that must hold for all public methods.
Our objective is to get an `inv()` function which requires minimal proof obligation while still complete enough to capture all necessary constraints for the data structure.

Your responsibilities:
  1. Analyze the invariant function (inv) to determine if it properly captures all necessary constraints for the data structure.
  2. Improve the invariant where possible by using implementation fields/methods (self.var) instead of view (self.var@) fields/methods for certain operations like length checks.
  3. Specifically, look for places where using `self.field.len()` would be better than using `self.field@.len()` when checking bounds. This will reduce the number of proof obligations.
  4. Modify only the "inv" function to improve its abstraction while leaving all other parts of the file unchanged.
  5. Return the **entire updated Verus file** with your refined "inv" function.

Please provide only the complete Rust code of the refined file with no additional commentary.
"""
        examples = []
        example_path = Path(self.config.example_path) / "input-inv-refine"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = (
                        Path(self.config.example_path) / "output-inv-refine" / f.name
                    )
                    answer = answer_path.read_text() if answer_path.exists() else ""

                    examples.append({"query": input_content, "answer": answer})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            self.system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=1.0,
        )

    def ensures_inference(self, code, temp=0, answer_num=1):
        self.logger.info("Ensures Inference ...")
        instruction = """You are an expert in Verus (verifier for rust). Your task is to **Add `ensures` clauses for trait methods**:
   - Analyze the semantics of the functions and append appropriate `ensures` clauses to the trait method implementations.
   - DO NOT just copy the implementation code. You may use `self.view().XXX` in the `ensures` clauses.
   - DO NOT add the `requires` clause to the trait method implementations. This is not allowed: "trait method implementation cannot declare requires clauses; these can only be inherited from the trait declaration"
   - DO NOT use `old` without consideration: "only a variable binding is allowed as the argument to old".
   - DO NOT use `match` or `let` in the `ensures` clause.
   - DO NOT use `subrange` or concatenation in the `ensures` clause.
   - spec functions like View cannot have requires/ensures.
   """
        instruction += "\n\n" + self.important_note
        instruction += "\n\n" + self.require_ensure_knowledge
        instruction += "\n\n" + self.operators_knowledge
        # instruction += "\n" + self.spec_knowledge
        instruction = "\n\n" + self.refinement.add_seq_knowledge(code, instruction)

        examples = []
        example_path = Path(self.config.example_path) / "input-ensures"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = (
                        Path(self.config.example_path) / "output-ensures" / f.name
                    )
                    answer = answer_path.read_text() if answer_path.exists() else ""

                    examples.append({"query": input_content, "answer": answer})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            self.system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=1.0,
        )

    def requires_inference(self, code, temp=0, answer_num=3):
        self.logger.info("Requires Inference ...")
        instruction = """You are an expert in Verus (verifier for rust). Your task is **Add `requires` and `ensures` to public functions**:
   - Please change the return type of the function if it doesn't have a return type to `-> (retname: rettype)`.
   - Analyze the semantics of the functions and append appropriate `requires` and `ensures` clauses to the method implementations.
   - DO NOT just copy the implementation code. You may use `self.view().XXX` or `self@XXX` in the `ensures` clauses. If `self.view()` is a tuple, you can use `self@.i` to access the i-th element (zero index).
   - DO NOT use `old` without consideration: "only a variable binding is allowed as the argument to old".
   - DO NOT use `match` or `let` in the `ensures` clause.
   - DO NOT add anything to `fn main`.
   - You do not need to add `self.inv()` to the pre- and post-conditions of if `#[verifier::type_invariant]` is used before the `inv` definition.
   - spec functions like View cannot have requires/ensures."""
        instruction += "\n" + self.important_note
        instruction += "\n" + self.require_ensure_knowledge
        instruction = self.refinement.add_seq_knowledge(code, instruction)

        examples = []
        example_path = Path(self.config.example_path) / "input-requires"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = (
                        Path(self.config.example_path) / "output-requires" / f.name
                    )
                    answer = answer_path.read_text() if answer_path.exists() else ""

                    examples.append({"query": input_content, "answer": answer})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            self.system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=1.0,
        )

    def direct_view_inference(
        self,
        code,
        temp=0,
        answer_num=1,
        error="",
        use_misc_examples=True,
    ):
        self.logger.info("Direct View Inference ...")
        system = self.system
        instruction = """You are an expert in Verus, a verifier for Rust. Perform the following tasks for the provided Rust code:

1. **Generate a `View` function**:
   - Add a `View` function to the code that provides a mathematical abstraction for types used in the executable code.
   - For `Vec` type variables in the `View`, append "@" to their names.

2. **Implement `closed spec fn inv(&self) -> bool`**:
   - Define an `inv` function representing the invariant of the data structure.

3. **Add `ensures` clauses for trait methods**:
   - Analyze the semantics of the functions and append appropriate `ensures` clauses to the trait method implementations.
   - DO NOT just copy the implementation code. You may use `self.view().XXX` in the `ensures` clauses.

4. **Add `requires` and `ensures` to public functions**:
   - Include `requires self.inv(),` and `ensures self.inv(),` in the signatures of relevant public functions.
   - Note: Do not add `requires` to trait method implementations.

**Formatting for `requires` and `ensures`**:
```rust
fn func(arg) -> rettype
    requires
        REQUIREMENT1,
        REQUIREMENT2,
        ...
    ensures
        ENSUREMENT1,
        ENSUREMENT2,
        ...
```
- When using the return value in an `ensures` clause, assign it a name if not already provided, e.g.:
```rust
fn func(arg) -> (retname: rettype)
```

**Important Notes**:
- Do not modify anything under `pub trait Queue<T>: Sized { }`.
- Return the complete modified Rust code in your response without explanations.
"""
        # Integrate the Seq knowledge if needed
        instruction += self.refinement.add_seq_knowledge(code, instruction)

        examples = []

        for f in sorted(os.listdir(os.path.join(self.config.example_path, "input"))):
            if f.endswith(".rs") and f[2:-3] in self.view_examples:
                input_file = os.path.join(self.config.example_path, "input", f)
                output_file = os.path.join(self.config.example_path, "output", f)
                input_content = open(input_file).read()
                output_content = open(output_file).read()
                examples.append({"query": input_content, "answer": output_content})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    # This long prompt is used in the alternative design where proof generation is done in one shot
    # without further phases of refinement or repair
    def direct_full_inference(
        self,
        code,
        temp=0,
        answer_num=1,
        error="",
        use_simple=True,
        use_misc_examples=True,
    ):
        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        complex_instruction = """Your missions are to
1. Add loop invariants to the given Rust code, if there are loops in the code, so that Verus can verify the give function behaves exact what is described in the specifications
2. Add the proof blocks that could help Verus to prove the following code snippet. You need to analyze which locations in the code need to be proved and add the proof blocks to help Verus to prove the correctness of the code. You can insert multiple proof blocks in the code as long as they are necessary to prove the correctness of the code. You can also include new ghost variables that could help you to prove the correctness of the code.

Here are some principles that you have to follow:
Respond with the Rust code only, do not include any explanation.
If a function is marked with unimplemented!(), please leave it there and do NOT try to add new implementation.
You should never change or delete any existing code.
If this function contains no loop, feel free to leave it as it is without adding anything.

Please follow these steps in adding loop invariants for every loop:
1. You should identify every variable that is read in the loop  (e.g., x[k], y), particularly for array elements like x[k], and add an invariant about the initial value for EACH such variable and array;
2. You should identify every variable that is written (e.g., y = ..., x.set(..,..)) in every loop, and add an invariant about the value of that variable. Even if an invariant is already specified earlier in the program, please do repeat it in every loop suitable. Copy them in the response.
3. You should fully utilize the spec functions and proof functions in the invariant.

Here are some common locations where you can add proof blocks:
1. In the beginning of the function
2. Before the loop
3. In the beginning of the loop
4. In the end of the loop
5. Before the key operations
6. After the key operations

The proof block looks like this:
```
proof {
    // your proof code here
    // assert(...)
    // LEMMA_FUNCTION(...)
    // ...
} // Added by AI
```
The ghost variable looks like this:
```
let ghost ...; // Added by AI
```

If there is nothing to add for a function, that is OK.
"""
        simple_instruction = """Please generate loop invariants and proof blocks for the given Rust code, so that Verus can verify the give function behaves exact what is described in the specifications.

Respond with the Rust code only, do not include any explanation.
"""

        if use_simple:
            self.logger.warning("Using simple instruction ...")
            instruction = simple_instruction
        else:
            self.logger.warning("Using complex instruction ...")
            instruction = complex_instruction

        examples = []
        if use_misc_examples:
            for f in sorted(
                os.listdir(os.path.join(self.config.example_path, "input-temp"))
            ):
                if f.endswith(".rs") and f.startswith("ex"):
                    input_file = os.path.join(self.config.example_path, "input-temp", f)
                    output_file = os.path.join(
                        self.config.example_path, "output-temp", f
                    )
                    input_content = open(input_file).read()
                    output_content = open(output_file).read()
                    examples.append({"query": input_content, "answer": output_content})
        else:
            for f in sorted(
                os.listdir(os.path.join(self.config.example_path, "input"))
            ):
                if f.endswith(".rs") and f[2] in self.phase1_examples:
                    input_file = os.path.join(self.config.example_path, "input", f)
                    output_file = os.path.join(self.config.example_path, "output", f)
                    input_content = open(input_file).read()
                    output_content = open(output_file).read()
                    examples.append({"query": input_content, "answer": output_content})
        with open("example.log", "w") as f:
            for ex in examples:
                f.write(ex["query"] + "\n")
                f.write(ex["answer"] + "\n\n")

        self.logger.info("Direct Full Inference ...")
        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    # The default first-step of preliminary loop invariant generation
    def direct_inference(self, code, temp=0, answer_num=1, error=""):
        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """Your mission is to add loop invariants to the given Rust code, if there are loops in the code, so that Verus can verify the give function behaves exact what is described in the specifications.

Here are some principles that you have to follow:
Respond with Rust code only, do not include any explanation.
You should never change or delete existing Rust code.

Please follow these steps in adding loop invariants for every loop:
1. You should identify every variable that is read in the loop  (e.g., x[k], y), particularly for array elements like x[k], and add an invariant about the initial value for EACH such variable and array;
2. You should identify every variable that is written (e.g., y = ..., x.set(..,..)) in every loop, and add an invariant about the value of that variable. Even if an invariant is already specified earlier in the program, please do repeat it in every loop suitable.
3. You can leverage the spec functions and proof functions in the invariant.
"""
        # Integrate the Seq knowledge if needed
        instruction += self.refinement.add_seq_knowledge(code, instruction)

        examples = []

        for f in sorted(os.listdir(os.path.join(self.config.example_path, "input"))):
            if f.endswith(".rs") and f[2] in self.phase1_examples:
                input_file = os.path.join(self.config.example_path, "input", f)
                output_file = os.path.join(self.config.example_path, "output", f)
                input_content = open(input_file).read()
                output_content = open(output_file).read()
                examples.append({"query": input_content, "answer": output_content})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    # This is an alternative design where the generation phase and refinement phase are combined into one prompt
    def direct_inference_with_refinement(self, code, temp=0, answer_num=1, error=""):
        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """
## Step 1: Add Loop Invariants
Your mission is to add loop invariants to the given Rust code, if there are loops in the code, so that Verus can verify the give function behaves exact what is described in the specifications.

Here are some principles that you have to follow:
Respond with Rust code only, do not include any explanation.
You should never change or delete existing Rust code.

Please follow these steps in adding loop invariants for every loop:
1. You should identify every variable that is read in the loop  (e.g., x[k], y), particularly for array elements like x[k], and add an invariant about the initial value for EACH such variable and array;
2. You should identify every variable that is written (e.g., y = ..., x.set(..,..)) in every loop, and add an invariant about the value of that variable. Even if an invariant is already specified earlier in the program, please do repeat it in every loop suitable.
3. You can leverage the spec functions and proof functions in the invariant.

## Step 2: Constant propagation refinement

If an upper bound or a lower bound about a constant function parameter (e.g., X < ..., X > ...) is provided in the function pre-condition (i.e., in the `requires' code block at the beginning of the function),
please copy that (e.g., X < 10, X > 5) as a loop invariant to every loop in the function.
Even if an invariant is already specified earlier in the program, please do repeat it in every loop suitable.

## Step 3: Array length refinement

For every loop in the function, please identify every array that is read (e.g., x[k]) or written (e.g., x.set(..,..)) in it, and then add a loop invariant that specifies the length of the array (i.e., x.len() == ...).

## Step 4: Quantifier range refinement

Please take the following steps to check every loop invariant that involves an array (e.g., x[k]) in the given Rust code:
If this array x[k] has been modified in this loop through x.set(), leave this invariant as it is, do NOT make any changes, and move on to the next invariant.
Otherwise, when there is no x.set() in the loop, please make sure that the invariant covers every element in the array and hence has the form like `forall |k:int| 0<= k < x.len() ==> whatever-property'. When you make this change, please use a comment to explain why you believe the related array is never changed in the loop. Do NOT make any other changes to the code or the loop invariant!

## Step 5: Conditional loop invariant refinement

Your mission is to refine some loop invariants in the given Rust code only if the loop has special handling for the first iteration. This is what you should do: if an existing loop invariant P holds for all iterations of the loop except for the first iteration (e.g., some variable updates may only (not) occur during the first loop iteration), please leave P as it is and add another loop invariant conditioned on the loop index (e.g., index > 0 ==> P), following the example below.
Do not change P or any other loop invariants in any other way."""

        self.logger.warning("Direct Inference unified with Refinement ...")

        # Integrate the Seq knowledge if needed
        instruction += self.refinement.add_seq_knowledge(code, instruction)

        examples = []

        for f in sorted(os.listdir(os.path.join(self.config.example_path, "input"))):
            if f.endswith(".rs") and f[2] in self.phase1_examples:
                input_file = os.path.join(self.config.example_path, "input", f)
                output_file = os.path.join(self.config.example_path, "output", f)
                input_content = open(input_file).read()
                output_content = open(output_file).read()
                examples.append({"query": input_content, "answer": output_content})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    #############################################
    ###The next few are the refinement agents####
    #############################################

    def arraylen_inference(self, code, temp=0, answer_num=1, error=""):
        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """
        For every loop in the function, please identify every array that is read (e.g., x[k]) or written (e.g., x.set(..,..)) in it, and then add a loop invariant that specifies the length of the array (i.e., x.len() == ...).

Here are some principles that you have to follow:
 You should only response with Rust code, and not include any explanation.
 You should not make any other changes to the program.
"""
        examples = []

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    def condlooprefine_inference(self, code, temp=0, answer_num=1, error=""):
        """
        This one checks if any loop invariant should be made to be conditional on loop indx, particularly if the invariant holds for all but the first interation of the loop.

        In terms of error fixing:
        ** If Verus complains that an array-related loop invariant does not hold before the loop,
        we can try this refinement.
        """

        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """Your mission is to refine some loop invariants in the given Rust code only if the loop has special handling for the first iteration. This is what you should do: if an existing loop invariant P holds for all iterations of the loop except for the first iteration (e.g., some variable updates may only (not) occur during the first loop iteration), please leave P as it is and add another loop invariant conditioned on the loop index (e.g., index > 0 ==> P), following the example below.
Do not change P or any other loop invariants in any other way. """

        examples = []

        for f in sorted(
            os.listdir(os.path.join(self.config.example_path, "input-condinv"))
        ):
            if f.endswith(".rs"):
                input_file = os.path.join(self.config.example_path, "input-condinv", f)
                output_file = os.path.join(
                    self.config.example_path, "output-condinv", f
                )
                input_content = open(input_file).read()
                output_content = open(output_file).read()
                examples.append({"query": input_content, "answer": output_content})

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=1,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    # quantifier refinement
    def arrayrefine_inference(self, code, temp=0, answer_num=1, error=""):
        """
        This one checks if an array-related loop invariant has the right range clause:
        if the array was not modified in the loop, the range clause should be 0<= .. <array.len()
        otherwise, the range clause should be 0<= .. <i or i<= .. <array.len()

        In terms of error fixing:
        ** If Verus complains that an array-related loop invariant does not hold after the loop,
        we can check whether this array is actually not modified and hence should use [0, array.len) clause.
        or if this array is actually modified and hence should NOT use [0, array.len) clause.
        """

        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """Please take the following steps to check every loop invariant that involves an array (e.g., x[k]) in the given Rust code:
        If this array x[k] has been modified in this loop through x.set(), leave this invariant as it is, do NOT make any changes, and move on to the next invariant.
        Otherwise, when there is no x.set() in the loop, please make sure that the invariant covers every element in the array and hence has the form like `forall |k:int| 0<= k < x.len() ==> whatever-property'. When you make this change, please use a comment to explain why you believe the related array is never changed in the loop. Do NOT make any other changes to the code or the loop invariant!

You should only response with Rust code, and not include any explanation.
You should NEVER ever add new variables, NEVER!
You should only make changes to existing loop invariants in the following ways, and you should not make any other changes to the program.
"""
        examples = []

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=1,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    def constantrefine_inference(self, code, temp=0, answer_num=1, error=""):
        """
        This one checks if any constant parameter related invariant is missing.

        In terms of error fixing:
        ** If Verus complains about arithmetic overflow,
        we can run this refinement.
        """

        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """
If an upper bound or a lower bound about a constant function parameter (e.g., X < ..., X > ...) is provided in the function pre-condition (i.e., in the `requires' code block at the beginning of the function),
please copy that (e.g., X < 10, X > 5) as a loop invariant to every loop in the function.
Even if an invariant is already specified earlier in the program, please do repeat it in every loop suitable.

Here are some principles that you have to follow:
 You should only response with Rust code, and not include any explanation.
 You should not make any other changes to the program.
"""

        examples = []

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=1,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    def nonlinear_inference(self, code, temp=0, answer_num=1, error=""):
        """
        This one checks if any loop invariant is related to a non-linear property.

        In terms of error fixing:
        ** If any invariant is non-linear ...
        """

        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """Your mission is to add assert statements into the given Rust function to help Verus prove non-linear properties.

Here are some principles that you have to follow:
Response with the Rust code only, do not include any explanation.
You should only add assertions with non-linear property if necessary in the following ways, and you should not make any other changes to the program.

Nonlinear arithmetic involves equations that multiply, divide, or take the remainder of integer variables (e.g., x * (y * z) == (x * y) * z). Verus can reason about nonlinear arithmetic, but it needs to be told when to do so. To do this, you need to add a special keyword `nonlinear_arith' to the assert statement.
For example, if we know that variable X equals k*k+2*k and that variable Y equals (k+1)*(k+1), to prove that X+1 equals Y, we have to write the following statement to help Verus:

    assert(X+1 == Y) by (nonlinear_arith)
        requires
            X == k*k+2*k,
            Y == (k+1)*(k+1),
            0 < k,
            k < N,
            N <= 300,
            {}

In this example, the `nonlinear_arith' would enable Verus to use its non-linear reasoning to prove X+1 equals Y. The requires statements should include all the information that is needed to reason about the assert statement, including any variable bound information that is need to prove no-arithmetic overflow.

Please check the given program, and add nonlinear_arith assertion when Verus needs to reason about non-linear properties."""

        examples = []

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=1,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    def nonlbound_inference(self, code, temp=0, answer_num=1, error=""):
        """
        This one is to add bound for any nonlinear expressions.

        In terms of error fixing:
        ** arithmetic overflow
        """

        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """Your mission is to add assertions with `nonlinear_arith' keyword in the given Rust function to help Verus prove there is no arithmetic overflow for any non-linear expressions.

Here are some principles that you have to follow:
Response with the Rust code only, do not include any explanation.
You should only add assertions with non-linear property in the following ways, and you should not make any other changes to the program. Do not delete any existing assertions.

Verus cannot prove that a non-linear expression does not overflow unless you tell it the range of the expression.
For example, if a non-linear expression x*x*x is used in the program, only tell Verus 0 <= x <= 10 is not enough, we have to write the following statement to help Verus prove no arithmetic overflow for x*x*x:

    assert(0 < x*x*x <= 10 * 10 * 10) by (nonlinear_arith)
        requires
            0 < x,
            x <= 10,
            {}

In this example, the `nonlinear_arith' keyword enables Verus to use its non-linear reasoning, and
the `requires' statements should include all the variable bound information needed to prove no-arithmetic overflow.

Please check the given program, and add above nonlinear_arith assertions when needed. Note that both the lower bound and upper bound of the expression should be specified in the assertion.
"""

        examples = []

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=1,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    # This refinement agent is deprecated as loop-isolation false can largely solve break's problem
    def breakloop_inference(self, code, temp=0, answer_num=1, error=""):
        """
        This one should be applied to loops that have early breaks
        """

        system = "You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."

        instruction = """The break keyword serves as another way to prematurely exit a loop, but it carries a slight complication in terms of loop specifications. Unlike simple while loops whose loop conditions must only be false upon exiting, loops with a break command can exit regardless of whether the loop condition is true or not. Code including break commands are expected to explicitly specify post-loop conditions using ensures clause. Take a look at the example below about how to add `ensures` clause for a loop with break, and then add `ensures' clause for any loop that contains break in the provided code accordingly. If a loop does not have break in it, please do NOT make any changes.

You should only response with Rust code, and not include any explanation.
"""
        examples = self.refinement.get_examples("loopbreak")

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=1,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    # WARNING: This repair agent is **deprecated** (for now).
    # LLM is indeed capable of generating proof blocks, but doing it without error-guidance is not effective
    def proof_block_inference(self, code, temp=0, answer_num=1, error=""):
        system = """You are an experienced formal language programmer. You are very familiar with Verus, which is a tool for verifying the correctness of code written in Rust."""

        instruction = """Please add proof blocks to the given Rust code to help Verus prove the correctness of the code. You need to analyze which locations in the code need to be proved and add the proof blocks to help Verus to prove the correctness of the code. You can insert multiple proof blocks in the code as long as they are necessary to prove the correctness of the code. You can also include new ghost variables that could help you to prove the correctness of the code.

Here are some common locations where you can add proof blocks:
1. In the beginning of the function
2. Before the loop
3. In the beginning of the loop
4. In the end of the loop
5. Before the key operations
6. After the key operations

The proof block looks like this:
```
proof {
    // your proof code here
    // assert(...)
    // LEMMA_FUNCTION(...)
    // ...
} // Added by AI
```

The ghost variable looks like this:
```
let ghost ...; // Added by AI
```

Here are some principles that you have to follow:
 You should only response with Rust code, and not include any explanation."""
        examples = []
        for f in sorted(
            os.listdir(os.path.join(self.config.example_path, "input-proof"))
        ):
            if f.endswith(".rs"):
                input_file = os.path.join(self.config.example_path, "input-proof", f)
                output_file = os.path.join(self.config.example_path, "output-proof", f)
                input_content = open(input_file).read()
                output_content = open(output_file).read()
                examples.append({"query": input_content, "answer": output_content})
        with open("proof_block_example.txt", "w") as f:
            f.write("Query:\n" + examples[0]["query"])
            f.write("\n\nAnswer:\n" + examples[0]["answer"])

        return self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system,
            answer_num=answer_num,
            max_tokens=self.config.max_token,
            temp=temp,
        )

    # An alternative design where the proof is generated in one step (no refinement or repair)
    def generate_baseline(self, code, retry=25):
        """
        Generate the proof code.
        """
        temp = 1.0
        answer_num = 5

        best_code_of_all = code
        best_score_of_all = EvalScore.get_worst_score()
        for i in range(retry):
            self.logger.info("Direct inference with baseline attempt %d" % i)
            candidates = self.direct_full_inference(code, temp, answer_num)
            for cand_code in candidates:
                cand_code, _ = self.refinement.debug_type_error(cand_code)
                veval = VEval(cand_code, self.logger)
                score = veval.eval_and_get_score()
                if score.is_correct():
                    return cand_code
                if score > best_score_of_all:
                    best_score_of_all = score
                    best_code_of_all = cand_code
        return best_code_of_all

    def generate_with_proof_func(
        self,
        code,
        with_inference=True,
        with_refine=True,
        merge_cand=5,
        verbose=False,
        repair_steps=10,
        temp=1.0,
        temp_dir=Path("output-intermediate-temp"),
        disable_ranking=False,
    ):
        """
        Generate the proof code with the whole pipeline.
        This is the default pipeline for proof generation in AutoVerus.
        """
        temp_dir.mkdir(parents=True, exist_ok=True)
        answer_num = merge_cand
        original_code = code

        if with_inference:
            best_score_of_all = EvalScore.get_worst_score()
            best_score_of_valid = EvalScore.get_worst_score()
            code_pool = []
            best_code_of_all = original_code

            attempt = 0
            code_pool_stop_size = 4
            if disable_ranking:
                self.logger.warning("Disabled ranking")
                code_pool_stop_size = 1

            while attempt < 3:
                self.logger.info("Direct inference attempt {}".format(attempt))
                # Now use direct_inference.
                codes = self.direct_inference(
                    original_code, temp=temp, answer_num=answer_num
                )
                found = False
                has_unsafe = False
                for i, cand_code in enumerate(codes):
                    self.logger.info(f"Checking candidate {attempt}-{i}")
                    cand_code = clean_code(cand_code)
                    newcode, _ = self.refinement.debug_type_error(cand_code)
                    if newcode:
                        cand_code = newcode

                    veval = VEval(cand_code, self.logger)
                    score = veval.eval_and_get_score()

                    is_safe_code_change = code_change_is_safe(
                        original_code, cand_code, self.config.verus_path, self.logger
                    )

                    if not is_safe_code_change:
                        has_unsafe = True

                    if score.is_correct() and is_safe_code_change:
                        self.logger.info("Verus succeeded!!")
                        return cand_code

                    if score > best_score_of_all:
                        best_score_of_all = score
                        best_code_of_all = cand_code

                    (temp_dir / f"{attempt}-{i}.rs").write_text(
                        cand_code
                        + "\n// is safe: "
                        + str(is_safe_code_change)
                        + "\n// Score: "
                        + str(score)
                    )
                    # TODO: We could try to fix the code with compilation error, instead of directly rejecting it
                    if (
                        "verus!" in cand_code
                        and is_safe_code_change
                        and not score.compilation_error
                    ):
                        found = True
                        self.logger.info(f"{attempt}-{i}.rs in code pool")
                        code_pool.append(cand_code)
                        if not (score < best_score_of_valid):
                            best_score_of_valid = score
                            self.logger.info(
                                f"{attempt}-{i}.rs is now the best proof candidate"
                            )
                            code = cand_code
                        if len(code_pool) >= code_pool_stop_size:
                            break

                if found and not has_unsafe:
                    break

                # if unsafe code was generated or if no valid code is fine at all,
                # better try another invocation to get more candidates
                self.logger.info("Regenerate...")
                attempt += 1

            if best_score_of_valid == EvalScore.get_worst_score():
                self.logger.error("No valid code found!")
                code = best_code_of_all
                code_pool = [code]
            else:
                # Try merging the valid codes to see if there is a better one.

                # Will also try an all-together merge, which may or may not be helpful
                allmerged_code = code
                for i, cp in enumerate(code_pool):
                    self.logger.info(f"Working on merge-{i}.rs")
                    try:
                        merged_code = self.hdn.merge_invariant(code, cp)
                        allmerged_code = self.hdn.merge_invariant(allmerged_code, cp)
                    except Exception as e:
                        self.logger.error(f"Error in merging code at step {i}: {e}")
                        continue

                    # selectively merged
                    veval = VEval(merged_code, self.logger)
                    score = veval.eval_and_get_score()
                    (temp_dir / f"merged-{i}.rs").write_text(
                        merged_code + "\n// Score: " + str(score)
                    )
                    if score.is_correct():
                        self.logger.info("Merged code is correct.")
                        return merged_code

                    if disable_ranking:
                        if not score.compilation_error:
                            self.logger.info(
                                "Disabled ranking and the code is not correct."
                            )
                            code = merged_code
                    elif not (score < best_score_of_valid):
                        self.logger.info("Merged code is better.")
                        best_score_of_valid = score
                        best_code_of_all = merged_code
                        # Only change the current code when the score isn't worse.
                        code = merged_code

                    self.logger.info(f"Running houdini on merge-{i}.rs")
                    hdn_failures, merge_code = self.hdn.run(merged_code)
                    if len(hdn_failures) == 0:
                        self.logger.info("Merged code with hdn is correct.")
                        return merge_code

                    # allmerged version
                    am_veval = VEval(allmerged_code, self.logger)
                    am_score = am_veval.eval_and_get_score()
                    (temp_dir / f"allmerged-{i}.rs").write_text(
                        allmerged_code + "\n// Score: " + str(am_score)
                    )
                    if am_score.is_correct():
                        self.logger.info("All merged code is correct.")
                        return allmerged_code
                    hdn_failures, hdnam_code = self.hdn.run(allmerged_code)
                    if len(hdn_failures) == 0:
                        self.logger.info("All merged code with hdn is correct.")
                        return hdnam_code

        # the best candidate is `code' now
        # score is cur_score
        veval = VEval(code, self.logger)
        cur_score = veval.eval_and_get_score()

        if with_refine:
            refine_funcs = self.default_refine_funcs
            # If the code contains non-linear arithmetic
            nl_lines = get_nonlinear_lines(code, self.logger)
            if nl_lines:
                self.logger.warning("Non-linear arithmetic detected.")
                for _, _, text in nl_lines:
                    self.logger.warning(text)
                refine_funcs.append(self.nonlinear_inference)
                refine_funcs.append(self.nonlbound_inference)

            for i, func in enumerate(refine_funcs):
                self.logger.info("refining with %s" % func.__name__)
                attempt = 0
                original_code = code

                while attempt < 3:
                    # Only 1 refined candidate.
                    code = func(original_code, temp=temp)[0]
                    # simple filtering
                    code = clean_code(code)
                    newcode = self.refinement.debug_type_error(code)[0]
                    if newcode:
                        code = newcode
                    if verbose:
                        self.logger.info(code)
                    if not code_change_is_safe(
                        original_code, code, self.config.verus_path, self.logger
                    ):
                        self.logger.info("Unsafe code change")
                        code = original_code
                    if "verus!" in code:
                        break

                    self.logger.info("regenerate...")
                    attempt += 1
                if code == original_code:
                    self.logger.info("Refinement did not change the code")
                    continue

                veval = VEval(code, self.logger)
                new_score = veval.eval_and_get_score()
                if new_score.is_correct():
                    self.logger.info("Verus succeeded with refinement!!")
                    return code

                hdn_failures, hdn_code = self.hdn.run(code)
                if len(hdn_failures) == 0:
                    self.logger.info("Verus succeeded with refinement and Houdini!")
                    return hdn_code

                # still errors left, let's see if we should accept the new version
                if func.__name__ == "condlooprefine_inference":
                    # condloop-refine is not often helpful, so we need to be cautious here
                    self.logger.info("New refined code under condloop is not better")
                    code = original_code
                elif disable_ranking:
                    if not score.compilation_error:
                        self.logger.info(
                            "Disabled ranking and the code is not correct."
                        )
                        code = original_code
                elif new_score.is_good_repair(cur_score):
                    # Now we use a loose condition to accept the new code.
                    self.logger.info("New refined code is a good repair")
                    self.logger.info(code)
                    cur_score = new_score
                    (temp_dir / f"refine-{i}.rs").write_text(code)
                else:
                    self.logger.info("New refined code is worse")
                    code = original_code

        if repair_steps > 0:
            (temp_dir / "before-repair.rs").write_text(
                code + "\n// Score: " + str(cur_score).replace("\n", " ")
            )
            repair_temp_dir = temp_dir / "repair"
            repair_temp_dir.mkdir(parents=True, exist_ok=True)

            if self.repair_uniform:
                # Ablation study: repair with uniform strategy
                code = self.refinement.repair_veval_in_one(
                    code, max_attempt=repair_steps, temp_dir=repair_temp_dir, temp=temp
                )
            else:
                code = self.refinement.repair_veval(
                    code, max_attempt=repair_steps, temp_dir=repair_temp_dir, temp=temp
                )

            veval = VEval(code, self.logger)
            score = veval.eval_and_get_score()
            if score.is_correct():
                self.logger.info("Verus succeeded after repair!!")
                return code

        (temp_dir / "final.rs").write_text(
            code + "\n// Score: " + str(score).replace("\n", " ")
        )

        # run houdini
        hdn_code = self.hdn.run(code)[1]
        hdn_veval = VEval(hdn_code, self.logger)
        hdn_score = hdn_veval.eval_and_get_score()
        (temp_dir / "final-hdn.rs").write_text(
            hdn_code + "\n// Score: " + str(hdn_score).replace("\n", " ")
        )
        if hdn_score.is_correct():
            self.logger.info("Verus succeeded with hdn!!")
            return hdn_code
        elif hdn_score > score:
            self.logger.info("Houdini code is better")
        else:
            self.logger.info("Original code is better")
        return code

    def run(self, input_file, output_file, args: dict = {}):
        baseline = args.get("is_baseline", False)
        mode = args.get("mode", "gen")
        repair_steps = args.get("repair", 5)
        merge_cand = args.get("merge", 5)
        temp = args.get("temp", 1.0)
        phase_uniform = args.get("phase_uniform", False)
        disable_ranking = args.get("disable_ranking", False)
        direct_repair = args.get("direct_repair", False)
        disable_one_refinement = args.get("disable_one_refinement", -1)

        if disable_one_refinement >= 0 and disable_one_refinement < len(
            self.default_refine_funcs
        ):
            self.logger.warning(
                "Disable one refinement function: %s"
                % self.default_refine_funcs[disable_one_refinement].__name__
            )
            self.default_refine_funcs = (
                self.default_refine_funcs[:disable_one_refinement]
                + self.default_refine_funcs[disable_one_refinement + 1 :]
            )

        content = open(input_file).read()
        output_file = Path(output_file)
        output_dir = output_file.parent
        output_dir.mkdir(parents=True, exist_ok=True)
        temp_dir = Path(output_dir) / ("intermediate-" + output_file.stem)
        temp_dir.mkdir(parents=True, exist_ok=True)

        self.logger.info(
            "Generating proof code" + (" with baseline" if baseline else "")
        )
        self.logger.info("Temperature: " + str(temp))

        # Various alternate designs
        if mode == "gen_view":
            self.logger.info("Generate with view mode")
            code = self.generate_view(content, temp_dir=temp_dir, answer_num=3)
        elif mode == "repair_view":
            self.logger.info("Repair with view mode")
            code = self.repair_and_finalize(
                content,
                temp_dir=temp_dir,
            )
        elif baseline:
            self.logger.info("Generate with baseline mode")
            code = self.generate_baseline(content)
        elif phase_uniform:
            self.logger.info("Generate with uniform refinement mode")
            self.direct_inference = self.direct_inference_with_refinement
            code = self.generate_with_proof_func(
                content,
                with_refine=False,
                merge_cand=merge_cand,
                verbose=True,
                repair_steps=repair_steps,
                temp_dir=temp_dir,
                temp=temp,
                disable_ranking=disable_ranking,
            )
        elif direct_repair:
            self.logger.info("Generate with direct repair mode")
            code = self.generate_with_proof_func(
                content,
                with_inference=False,
                with_refine=False,
                merge_cand=merge_cand,
                verbose=True,
                repair_steps=repair_steps,
                temp_dir=temp_dir,
                temp=temp,
                disable_ranking=disable_ranking,
            )
        else:
            # default/recommended
            code = self.generate_with_proof_func(
                content,
                with_refine=True,
                merge_cand=merge_cand,
                verbose=True,
                repair_steps=repair_steps,
                temp_dir=temp_dir,
                temp=temp,
                disable_ranking=disable_ranking,
            )
        if mode != "gen_view":

            score, _ = evaluate(code, self.config.verus_path)
            is_safe = code_change_is_safe(
                content, code, self.config.verus_path, self.logger, debug=False
            )
            code += "\n// Score: " + str(score)
            code += "\n// Safe: " + str(is_safe)

        with open(output_file, "w") as wf:
            wf.write(code)
        self.logger.info("finished!")
