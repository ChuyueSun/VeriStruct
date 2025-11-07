"""
Module for fixing common syntax errors using regex patterns.

This module provides fast, deterministic fixes for common syntax errors
that don't require LLM-based reasoning. It should run BEFORE expensive
LLM-based repairs to catch trivial issues.
"""

import logging
import re
from typing import Tuple


def fix_common_syntax_errors(code: str, logger: logging.Logger = None) -> Tuple[str, bool]:
    """
    Fix common syntax errors using regex patterns.

    This function handles trivial syntax errors that are:
    - Deterministic (same pattern always needs same fix)
    - Common (appear frequently in generated code)
    - Simple (don't require semantic understanding)

    Args:
        code: The Rust/Verus code to fix
        logger: Optional logger for reporting fixes

    Returns:
        Tuple of (fixed_code, was_changed)
        - fixed_code: The code with syntax errors fixed
        - was_changed: True if any fixes were applied
    """
    original_code = code
    fixes_applied = []

    # Pattern 1: Split operators (most common issue)
    # Fix: i < = j  →  i <= j
    patterns = [
        (r"< =", r"<=", "split_less_equal"),
        (r"> =", r">=", "split_greater_equal"),
        (r"= =", r"==", "split_equal_equal"),
        (r"! =", r"!=", "split_not_equal"),
        (r"< = =", r"<==", "split_iff_left"),
        (r"= = >", r"==>", "split_implies"),
    ]

    for pattern, replacement, fix_name in patterns:
        if re.search(pattern, code):
            code = re.sub(pattern, replacement, code)
            fixes_applied.append(fix_name)

    # Pattern 2: Missing spaces before multi-char operators
    # Fix: a==>b  →  a ==> b
    space_fixes = [
        (r"(\w+)==>", r"\1 ==>", "space_before_implies"),
        (r"(\w+)<==>", r"\1 <==>", "space_before_iff"),
        (r"(\w+)&&&", r"\1 &&&", "space_before_and"),
        (r"(\w+)\|\|\|", r"\1 |||", "space_before_or"),
    ]

    for pattern, replacement, fix_name in space_fixes:
        if re.search(pattern, code):
            code = re.sub(pattern, replacement, code)
            fixes_applied.append(fix_name)

    # Pattern 3: Missing spaces in forall/exists
    # Fix: forall|i: int|n  →  forall|i: int| n
    quantifier_fixes = [
        (r"forall\|([^|]+)\|(\w)", r"forall|\1| \2", "space_after_forall"),
        (r"exists\|([^|]+)\|(\w)", r"exists|\1| \2", "space_after_exists"),
    ]

    for pattern, replacement, fix_name in quantifier_fixes:
        if re.search(pattern, code):
            code = re.sub(pattern, replacement, code)
            fixes_applied.append(fix_name)

    # Pattern 4: Missing spaces around comparison in chained comparisons
    # Fix: 0<=i  →  0 <= i (but be careful not to break existing correct code)
    # Only fix if there's a clear pattern of missing spaces
    comparison_fixes = [
        (r"(\d)<(\w)", r"\1 < \2", "space_around_less"),
        (r"(\w)>(\d)", r"\1 > \2", "space_around_greater"),
    ]

    for pattern, replacement, fix_name in comparison_fixes:
        # Only apply if it looks like it's in a specification context
        matches = re.finditer(pattern, code)
        for match in matches:
            # Check if this is in a likely specification (has forall, invariant, requires, ensures nearby)
            context_start = max(0, match.start() - 100)
            context_end = min(len(code), match.end() + 100)
            context = code[context_start:context_end]

            if any(
                keyword in context
                for keyword in [
                    "forall",
                    "exists",
                    "invariant",
                    "requires",
                    "ensures",
                    "assert",
                ]
            ):
                code = (
                    code[: match.start()]
                    + re.sub(pattern, replacement, match.group())
                    + code[match.end() :]
                )
                if fix_name not in fixes_applied:
                    fixes_applied.append(fix_name)

    was_changed = code != original_code

    if logger and was_changed:
        logger.info(
            f"Regex syntax fixer applied {len(fixes_applied)} fix(es): {', '.join(fixes_applied)}"
        )
        # Log specific examples of what was fixed
        if "< =" in original_code and "<=" in code:
            count = original_code.count("< =")
            logger.info(f"  - Fixed {count} instance(s) of '< =' → '<='")
        if "- n==>" in original_code and "- n ==>" in code:
            logger.info(f"  - Fixed missing space before '==>'")

    return code, was_changed


def fix_syntax_errors_with_regex(code: str, logger: logging.Logger = None) -> Tuple[str, bool]:
    """
    Convenience wrapper for fix_common_syntax_errors.

    Args:
        code: The code to fix
        logger: Optional logger

    Returns:
        Tuple of (fixed_code, was_changed)
    """
    return fix_common_syntax_errors(code, logger)


# Additional utility function for more aggressive fixing
def fix_aggressive_syntax_errors(code: str, logger: logging.Logger = None) -> Tuple[str, bool]:
    """
    Apply more aggressive regex fixes that might have false positives.
    Use this only when standard fixes don't work.

    Args:
        code: The code to fix
        logger: Optional logger

    Returns:
        Tuple of (fixed_code, was_changed)
    """
    original_code = code

    # First apply standard fixes
    code, _ = fix_common_syntax_errors(code, logger)

    # Additional aggressive patterns
    # These might occasionally break valid code, so use with caution

    # Fix: Multiple spaces around operators
    code = re.sub(r"\s+<=\s+", r" <= ", code)
    code = re.sub(r"\s+>=\s+", r" >= ", code)
    code = re.sub(r"\s+==\s+", r" == ", code)

    # Fix: Missing semicolons (very aggressive, might break things)
    # Don't use this for now - too dangerous

    was_changed = code != original_code

    if logger and was_changed:
        logger.info("Applied aggressive regex fixes")

    return code, was_changed
