# Copyright (c) Microsoft Corporation. #
# Licensed under the MIT license.      #


import os
import subprocess
import tempfile
from pathlib import Path


class Lynette:
    meta_command = [
        "cargo",
        "run",
        "--manifest-path="
        + os.path.abspath(
            os.path.join(
                os.path.dirname(__file__),
                "..",
                "..",
                "utils",
                "lynette",
                "source",
                "Cargo.toml",
            )
        ),
        "--",
    ]
    env = os.environ.copy()
    env["RUSTFLAGS"] = "-Awarnings"

    # Run a command
    # @command: a list of lynette commands arguemnts, e.g. ["compare", "foo.rs", "bar.rs"]
    # @return: a CompletedProcess object(returned by subprocess.run(...))
    def run(self, command):
        command = self.meta_command + command
        return subprocess.run(command, env=self.env, capture_output=True, text=True)

    def code_unimpl(self, file):
        return self.run(["code", "unimpl", file])

    def func_add(self, file1, file2, replace=False, funcs=[]):
        return self.run(
            ["func", "add", file1, file2, "--replace" if replace else ""]
            + ["--funcs"]
            + funcs
            if funcs
            else []
        )

    def code_merge(self, file1, file2, merge_mode):
        pass

    def code_merge_all(self, file1, file2):
        return self.run(["code", "merge", "--all", file1, file2])

    def code_merge_invariant(self, file1, file2, util_path="../../utils", logger=None):
        return self.run(["code", "merge", "--invariants", file1, file2])

    def code_detect_nonlinear(self, file):
        return self.run(["code", "detect-nl", file])

    def func_code_extract(self, file, func):
        return self.run(["func", "extract", "--function", func, file])


lynette = Lynette()

def get_nonlinear_lines(code, logger=None):
    """
    Detect nonlinear arithmetic lines in Verus code using Lynette.
    Returns a list of line numbers that contain nonlinear arithmetic.
    """
    try:
        # Create temporary file for the code
        with tempfile.NamedTemporaryFile(mode='w', suffix='.rs', delete=False) as f:
            f.write(code)
            f_path = f.name
        
        # Find Lynette cargo path
        cargopath = os.path.join("../../utils", "lynette/source/Cargo.toml")
        if not os.path.exists(cargopath):
            # Try relative path from current file location
            cargopath = (
                Path(__file__).parent.parent.parent
                / "utils"
                / "lynette"
                / "source"
                / "Cargo.toml"
            )
            if not cargopath.exists():
                if logger:
                    logger.warning(f"Could not find lynette Cargo.toml at {cargopath}")
                return []
            cargopath = str(cargopath.resolve())
        
        # Run Lynette nonlinear detection command
        detect_cmd = [
            "cargo", "run", "--manifest-path", cargopath, "--",
            "code", "detect-nl", f_path
        ]
        
        result = subprocess.run(
            detect_cmd, 
            capture_output=True, 
            text=True, 
            timeout=30
        )
        
        if result.returncode == 0:
            # Parse the output to extract line numbers
            lines = []
            for line in result.stdout.strip().split('\n'):
                if line.strip() and line.strip().isdigit():
                    lines.append(int(line.strip()))
            return lines
        else:
            if logger:
                logger.warning(f"Lynette nonlinear detection failed: {result.stderr}")
            return []
            
    except subprocess.TimeoutExpired:
        if logger:
            logger.warning("Lynette nonlinear detection timed out")
        return []
    except Exception as e:
        if logger:
            logger.warning(f"Exception during nonlinear detection: {e}")
        return []
    finally:
        # Clean up temporary file
        try:
            if 'f_path' in locals():
                os.unlink(f_path)
        except:
            pass  # Ignore cleanup errors
