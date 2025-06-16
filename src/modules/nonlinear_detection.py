# Copyright (c) Microsoft Corporation. #
# Licensed under the MIT license.      #

import tempfile
import os
from typing import List, Set, Tuple

from src.modules.base import BaseModule
from src.modules.lynette import lynette
from src.context import Context


class NonlinearDetectionModule(BaseModule):
    """
    Module that uses Lynette to detect nonlinear arithmetic operations
    and suggests appropriate fixes or annotations.
    """

    def __init__(self, config, logger):
        super().__init__(config, logger)
        self.name = "nonlinear_detection"
        self.desc = "Detects nonlinear arithmetic and suggests fixes using Lynette"

    def exec(self, context: Context) -> str:
        """
        Detect nonlinear arithmetic in the current code and suggest fixes.
        """
        current_code = context.get_latest_code()
        
        self.logger.info("Detecting nonlinear arithmetic using Lynette...")
        
        nonlinear_lines = self._detect_nonlinear_lines(current_code)
        
        if not nonlinear_lines:
            self.logger.info("No nonlinear arithmetic detected")
            return current_code
        
        self.logger.info(f"Found nonlinear arithmetic on lines: {nonlinear_lines}")
        
        # Try to fix nonlinear arithmetic by adding appropriate annotations
        fixed_code = self._fix_nonlinear_arithmetic(current_code, nonlinear_lines)
        
        if fixed_code != current_code:
            self.logger.info("Applied nonlinear arithmetic fixes")
            context.add_trial(fixed_code)
            return fixed_code
        else:
            self.logger.info("No fixes applied for nonlinear arithmetic")
            return current_code

    def _detect_nonlinear_lines(self, code: str) -> List[int]:
        """
        Use Lynette to detect lines with nonlinear arithmetic.
        
        Args:
            code: Source code to analyze
            
        Returns:
            List of line numbers containing nonlinear arithmetic
        """
        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, prefix="nonlinear_detect_", suffix=".rs"
            ) as temp_f:
                temp_f.write(code)
                temp_f.flush()

                result = lynette.code_detect_nonlinear(temp_f.name)
                
                if result.returncode != 0:
                    self.logger.warning(f"Lynette nonlinear detection failed: {result.stderr}")
                    return []

                # Parse the output to extract line numbers
                lines = []
                for line in result.stdout.strip().split('\n'):
                    if line.strip() and line.strip().isdigit():
                        lines.append(int(line.strip()))
                
                return lines

        except Exception as e:
            self.logger.error(f"Exception during nonlinear detection: {e}")
            return []
        finally:
            if 'temp_f' in locals() and os.path.exists(temp_f.name):
                os.unlink(temp_f.name)

    def _fix_nonlinear_arithmetic(self, code: str, nonlinear_lines: List[int]) -> str:
        """
        Attempt to fix nonlinear arithmetic by adding appropriate annotations.
        
        Args:
            code: Source code
            nonlinear_lines: Line numbers with nonlinear arithmetic
            
        Returns:
            Fixed code with nonlinear arithmetic annotations
        """
        if not nonlinear_lines:
            return code
            
        lines = code.split('\n')
        modified = False
        
        for line_num in nonlinear_lines:
            if 1 <= line_num <= len(lines):
                line_idx = line_num - 1  # Convert to 0-based index
                original_line = lines[line_idx]
                
                # Check if this line is inside an assert or requires/ensures block
                if self._is_in_specification_context(lines, line_idx):
                    # Add nonlinear_arith annotation to assert statements
                    if 'assert' in original_line and 'by' not in original_line:
                        # Add nonlinear_arith annotation
                        indent = len(original_line) - len(original_line.lstrip())
                        fixed_line = original_line.rstrip()
                        if not fixed_line.endswith(';'):
                            fixed_line += ' by { nonlinear_arith(); }'
                        else:
                            fixed_line = fixed_line[:-1] + ' by { nonlinear_arith(); };'
                        lines[line_idx] = fixed_line
                        modified = True
                        self.logger.info(f"Added nonlinear_arith annotation to line {line_num}")
                    
                    # For requires/ensures, we might need to add assume statements
                    elif any(keyword in original_line for keyword in ['requires', 'ensures']):
                        # This is more complex - might need to restructure the specification
                        self.logger.info(f"Nonlinear arithmetic in specification at line {line_num} - manual review needed")
        
        return '\n'.join(lines) if modified else code

    def _is_in_specification_context(self, lines: List[str], line_idx: int) -> bool:
        """
        Check if a line is within a specification context (assert, requires, ensures).
        
        Args:
            lines: All lines of code
            line_idx: Index of the line to check
            
        Returns:
            True if line is in specification context
        """
        if line_idx < 0 or line_idx >= len(lines):
            return False
            
        current_line = lines[line_idx].strip()
        
        # Direct check for assert
        if current_line.startswith('assert'):
            return True
            
        # Check for requires/ensures context by looking backwards
        for i in range(line_idx, -1, -1):
            line = lines[i].strip()
            if line.startswith('fn ') or line.startswith('pub fn '):
                # Found function definition, check if we're in requires/ensures
                for j in range(i, min(line_idx + 1, len(lines))):
                    check_line = lines[j].strip()
                    if any(keyword in check_line for keyword in ['requires', 'ensures']):
                        return True
                break
                
        return False

    def get_nonlinear_lines_for_context(self, context: Context) -> List[int]:
        """
        Get nonlinear arithmetic lines for the current context.
        This can be used by other modules to understand nonlinear arithmetic locations.
        
        Args:
            context: Current execution context
            
        Returns:
            List of line numbers with nonlinear arithmetic
        """
        current_code = context.get_latest_code()
        return self._detect_nonlinear_lines(current_code) 