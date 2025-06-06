"""
Module for repairing arithmetic errors in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import (
    clean_code,
    evaluate_samples,
    get_examples,
    get_nonlinear_lines,
)
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval
from src.utils.path_utils import samples_dir, best_dir, debug_dir


class RepairArithmeticModule(BaseRepairModule):
    """
    Module for repairing arithmetic errors.
    Handles both arithmetic overflow/underflow errors and nonlinear arithmetic proof issues.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_arithmetic",
            desc="Repair arithmetic failures including overflow/underflow and nonlinear proofs",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the arithmetic repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific arithmetic VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair arithmetic error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.ArithmeticFlow
            )
            if not failures:
                self.logger.warning("No arithmetic failures found in the last trial.")
                return code  # Return original code if no arithmetic error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is an arithmetic error
        if failure_to_fix.error != VerusErrorType.ArithmeticFlow:
            self.logger.warning(
                f"Received non-arithmetic error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        # Try repairing nonlinear arithmetic issues first
        nonlinear_result = self.repair_nonlinear_arith_error(context, failure_to_fix)
        if nonlinear_result and nonlinear_result != code:
            return nonlinear_result

        # If nonlinear repair didn't work or wasn't applicable, try general arithmetic repair
        return self.repair_arithmetic_flow(context, failure_to_fix)

    def repair_nonlinear_arith_error(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair nonlinear arithmetic errors by adding appropriate assertions.

        Args:
            context: The current execution context
            failure_to_fix: The specific arithmetic VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        # Check if there are nonlinear expressions in the code
        nl_lines = get_nonlinear_lines(code, self.logger)
        if not nl_lines:
            return ""  # No nonlinear lines found, skip this repair

        # Filter nonlinear lines that are relevant to the error
        filtered_nl_lines = []
        for i, (st, ed, text) in enumerate(nl_lines):
            if text in failure_to_fix.get_text():
                filtered_nl_lines.append((st, ed, text))

        if not filtered_nl_lines:
            return ""  # No relevant nonlinear lines found

        instruction = """Your mission is to add assert statements into the given Rust function to help Verus prove non-linear properties.

Here are some principles that you have to follow:
Response with the Rust code only, do not include any explanation.
You should only add assertions with non-linear property if necessary in the following ways, and you should not make any other changes to the program.

#### 1. Nonlinear Arithmetic
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

#### 2. Nonlinear Arithmetic Overflow
Verus cannot prove that a non-linear expression does not overflow unless you tell it the range of the expression.
For example, if a non-linear expression x*x*x is used in the program, only tell Verus 0 <= x <= 10 is not enough, we have to write the following statement to help Verus prove no arithmetic overflow for x*x*x:

    assert(0 < x*x*x <= 10 * 10 * 10) by (nonlinear_arith)
        requires
            0 < x,
            x <= 10,
            {}

In this example, the `nonlinear_arith' keyword enables Verus to use its non-linear reasoning, and
the `requires' statements should include all the variable bound information needed to prove no-arithmetic overflow.

#### Task
Please check the given program, and add nonlinear_arith assertion for the following assertions:
"""

        # Add the identified nonlinear expressions to the instruction
        for i, (st, ed, text) in enumerate(filtered_nl_lines):
            instruction += "{}. Lines {}-{}:\n{}\n".format(i + 1, st, ed, text)

        # Load examples
        examples = get_examples(self.config, "nonlin", self.logger)

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"),
            instruction=instruction,
            exemplars=examples,
            query=code,
            system_info=self.default_system,
            answer_num=3,
            max_tokens=8192,
            temp=1.0,
        )

        # Evaluate samples and get the best one
        output_dir = samples_dir()
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_nonlinear_arith",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_arithmetic_flow(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair arithmetic overflow/underflow errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific arithmetic VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        error_trace = failure_to_fix.trace[0]
        error_highlight = (
            error_trace.get_highlights()[0] if error_trace.get_highlights() else ""
        )

        instruction = f"""Your mission is to fix the arithmetic underflow/overflow error for the following code.
Basically, for each variable involved in the expression `{error_highlight}' in line `{error_trace.get_text().strip()}' of the program, there are several general ways to fix the error:

0. Make sure the value of EVERY variable involved in this expression is specified as a loop invariant.
1. Add a bound for the whole expression `{error_highlight}' as a loop invariant or as an assert. This bound can be a constant value, or another expression whose bound has been specified through loop invariants or asserts.
2. Or, add BOTH a lower bound (i.e. x > ..., x >= ...) AND an upper bound (i.e., x < ..., x <= ...) as an assertion or a loop invariant if they are in a loop body for EACH variable involved in the expression {error_highlight}. If the variable is a loop index variable, make sure that its lower bound (e.g., its initial value at the beginning of the loop) and upper bound (based on the loop-exit condition) are specified as loop invariants. You may use the loop index variable in the invariant.

Do not miss any variable in `{error_highlight}', and do NOT add bound information related to any other variables. Please do not change function post-conditions.
"""

        instruction += """Response requirements:
Respond with the verus code only, do not include any explanation.
You should only add loop invariants, and you should NOT make any other changes to the program.

Hint for the upper bound:
1. For the lower/upper bound, you don't always need to find the exact or strict value. Your mission is to find a provable bound for Verus, which is usually based on the loop index, like `car <= CONSTANT * index`.
2. If the expression involves the loop index or is updated during each loop iteration, use the loop index variable as the upper or lower bound in the invariant instead of using the CONSTANT alone!
3. If there is a non-linear upper bound, you can use a constant to represent part of the expression (e.g., a * CONSTANT_RELATED_TO_b) to make it linear. However, ensure that at least one variable remains (DO NOT USE A CONSTANT TO REPLACE THE WHOLE NON-LINEAR). This approach makes it easier to prove.
4. You may use conditional loop invariants to specify the upper bound based on the loop index. For example, `i > 0 ==> x < 10 * i` means that if `i` is greater than 0, then `x` is less than 10 times `i`.
"""

        # Load examples
        examples = get_examples(self.config, "aritherr", self.logger)

        query_template = "Arithmetic underflow/overflow \n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        line_info = f"Line {error_trace.lines[0]}-{error_trace.lines[1]}:\n"
        inv_info = line_info + error_trace.get_text() + "\n"
        query = query_template.format(inv_info, code)

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_generation_model", "gpt-4"),
            instruction=instruction,
            exemplars=examples,
            query=query,
            system_info=self.default_system,
            answer_num=3,
            max_tokens=8192,
            temp=1.0,
        )

        # Evaluate samples and get the best one
        output_dir = samples_dir()
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_arithmetic_flow",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
