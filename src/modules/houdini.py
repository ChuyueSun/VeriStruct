# Copyright (c) Microsoft Corporation. #
# Licensed under the MIT license.      #

import tempfile

from veval import VerusError, VerusErrorType, VEval

from src.modules.lynette import lynette
from utils import compress_nl_assertion


class houdini:
    def __init__(self, config, immutable_funcs=[]):
        self.config = config
        self.verification_path = config.verus_path
        self.immutable_funcs = immutable_funcs

    def merge_invariant(self, code1, code2):
        with tempfile.NamedTemporaryFile(
            mode="w", prefix="merge_inv_orig", suffix=".rs"
        ) as f1, tempfile.NamedTemporaryFile(mode="w", prefix="merge_new_inv", suffix=".rs") as f2:
            f1.write(code1)
            f1.flush()
            f2.write(code2)
            f2.flush()

            path1 = f1.name
            path2 = f2.name

            m = lynette.code_merge_invariant(path1, path2)

        if m.returncode == 0:
            return m.stdout
        else:
            raise Exception(f"Error in merging invariants:{m.stderr}")

    # the input is a list of Veval list[VerusError]
    def get_error_line(self, failures: list[VerusError], considerassert=True):
        ret = []
        for f in failures:
            # if we don't want Houdini to remove assert, we skip assert errors
            if considerassert and f.error == VerusErrorType.AssertFail:
                ret.append(f.trace[0].lines[0])
            elif f.error == VerusErrorType.InvFailEnd or f.error == VerusErrorType.InvFailFront:
                ret.append(f.trace[0].lines[0])
            elif f.error == VerusErrorType.PostCondFail:
                st, ed = f.trace[1].lines
                for i in range(st, ed + 1):
                    ret.append(i)
            else:
                continue
        return ret

    def print_verus_errors(self, errors: list[VerusError]):
        for err in errors:
            print(f"Error Type: {err.error}")
            print(f"Message: {err.error_text}")
            print("Trace:")
            for idx, trace in enumerate(err.trace, start=1):
                print(f"  {idx}. {trace.get_text(snippet=False)}")
            print()

    def run(self, code, verbose=False):
        """Run Houdini invariant inference algorithm.

        Args:
            code: Source code to analyze
            verbose: Whether to print verbose output

        Returns:
            Tuple of (failures, modified_code) where failures are verification errors
            and modified_code has problematic assertions commented out
        """
        code = compress_nl_assertion(code)
        immutable_areas = self._get_immutable_areas(code)
        # print(f"immutable_areas: {immutable_areas}")
        for _ in range(100):
            # Run verifier
            veval = VEval(code)
            veval.eval()
            failures = veval.get_failures()
            self.print_verus_errors(failures)
            if not failures:
                break

            # Get lines with errors
            error_lines = self.get_error_line(failures)
            if not error_lines:
                break

            # Comment out problematic assertions
            code_lines = code.split("\n")
            all_immutable = True

            for line in error_lines:
                if line == 0:
                    continue

                # Skip assertions in immutable areas
                if self._is_in_immutable_area(line, immutable_areas):
                    continue

                all_immutable = False
                code_lines[line - 1] = "// // //" + code_lines[line - 1]

            code = "\n".join(code_lines)

            if all_immutable:
                print("All errors are in immutable areas, stop removing")
                break
        print("final code:")
        self.print_verus_errors(failures)
        return failures, code

    def _get_immutable_areas(self, code):
        """Get line ranges of immutable functions that should not be modified."""
        immutable_areas = []

        with tempfile.NamedTemporaryFile(mode="w", prefix="immutable_area", suffix=".rs") as f:
            f.write(code)
            f.flush()

            for func in self.immutable_funcs:
                try:
                    res = lynette.func_code_extract(f.name, func)
                    if res.returncode != 0:
                        print(f"Warning: Failed to extract function {func}: {res.stderr}")
                        continue

                    func_code = res.stdout.strip()
                    if not func_code:
                        print(f"Warning: Empty function code for {func}")
                        continue

                    # Find function location
                    code_lines = code.splitlines()
                    func_lines = func_code.splitlines()

                    if not func_lines:
                        print(f"Warning: No lines found for function {func}")
                        continue

                    # Find start line of function
                    start_line = self._find_function_start(code_lines, func_lines)
                    if start_line is not None:
                        immutable_areas.append((start_line, start_line + len(func_lines) - 1))
                    else:
                        print(f"Warning: Could not find function {func} in code")
                except Exception as e:
                    print(f"Error processing function {func}: {str(e)}")
                    continue

        return immutable_areas

    def _find_function_start(self, code_lines, func_lines):
        """Find starting line number of function in code."""
        for i, line in enumerate(code_lines):
            if line.strip() == func_lines[0].strip():
                # Verify full function match
                if all(
                    i + j < len(code_lines) and code_lines[i + j].strip() == func_lines[j].strip()
                    for j in range(len(func_lines))
                ):
                    return i + 1  # Convert to 1-based index
        return None

    def _is_in_immutable_area(self, line, immutable_areas):
        """Check if line number falls within any immutable area."""
        return any(start <= line <= end for start, end in immutable_areas)
