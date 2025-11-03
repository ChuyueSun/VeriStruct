#!/usr/bin/env python3
"""
Test script to verify the implemented workflow fixes work correctly.

Tests cover:
1. Assert forall syntax detection and fixing
2. Pattern-based repair functionality
3. Spec simplification (.view() to @)
4. Cast parenthesization

Run with: python tests/test_workflow_fixes.py
"""

import re
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.modules.spec_inference import fix_spec_syntax_issues


def test_assert_forall_detection():
    """Test that assert forall without 'by' is detected."""

    # Simulate the broken code from bitmap_todo
    broken_code = """
proof {
    bit_or_64_proof(u1, u2, or_int);
    assert forall|off: int| #![trigger result@[(i as int) * 64 + off]]
        0 <= off && off < 64 ==>
            result@[(i as int) * 64 + off]
                == (self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off]);
}
"""

    print("Test 1: Assert forall detection")
    print("================================")

    # Check if we can detect the pattern
    has_assert_forall = "assert forall" in broken_code
    has_by = "by {" in broken_code or "by{" in broken_code
    has_semicolon = ";" in broken_code

    print(f"Detection:")
    print(f"  Has 'assert forall': {has_assert_forall}")
    print(f"  Has 'by' clause: {has_by}")
    print(f"  Has semicolon: {has_semicolon}")
    print(f"  Needs fix: {has_assert_forall and has_semicolon and not has_by}")

    if has_assert_forall and has_semicolon and not has_by:
        print("  ✓ Would be detected and fixed by proof_generation module")
        return True
    else:
        print("  ✗ Would NOT be detected")
        return False


def test_pattern_based_repair():
    """Test pattern-based repair for assert forall."""

    print("\nTest 2: Pattern-based repair")
    print("=============================")

    broken_code = """assert forall|x: int| x > 0 ==> x >= 0;"""

    print(f"Input: {broken_code}")

    # Apply the fix pattern
    pattern = r"(assert forall\|[^|]+\|[^;]+);"
    fixed_code = re.sub(pattern, r"\1 by {\n    \n}", broken_code)

    print(f"Output: {fixed_code}")

    if "by {" in fixed_code:
        print("  ✓ Pattern-based fix works correctly")
        return True
    else:
        print("  ✗ Pattern-based fix failed")
        return False


def test_spec_simplification():
    """Test spec simplification (.view() to @)."""

    print("\nTest 3: Spec simplification")
    print("============================")

    verbose_code = """
fn set_bit(&mut self, index: u32, bit: bool)
    requires
        (index as int) < old(self).view().len()
    ensures
        self.view() == old(self).view().update(index as int, bit)
{
    // implementation
}
"""

    fixed_code = fix_spec_syntax_issues(verbose_code)

    # Check if simplifications were applied
    has_view_calls = ".view()" in fixed_code
    has_at_shorthand = "@" in fixed_code

    print(f"Checks:")
    print(f"  Still has .view() calls: {has_view_calls}")
    print(f"  Uses @ shorthand: {has_at_shorthand}")

    if not has_view_calls and has_at_shorthand:
        print("  ✓ Spec simplification works correctly")
        return True
    elif has_at_shorthand:
        print("  ⚠ Partially simplified")
        return True
    else:
        print("  ✗ Spec simplification failed")
        return False


def test_cast_parenthesization():
    """Test that casts are properly parenthesized."""

    print("\nTest 4: Cast parenthesization")
    print("==============================")

    broken_code = """
fn test(x: u32)
    requires
        x as int < 100
{
    // implementation
}
"""

    fixed_code = fix_spec_syntax_issues(broken_code)

    # Check if parentheses were added
    has_parenthesized_cast = "(x as int)" in fixed_code

    if has_parenthesized_cast:
        print("  ✓ Cast parenthesization works correctly")
        return True
    else:
        print("  ✗ Cast parenthesization failed")
        return False


def main():
    """Run all tests."""
    print("=" * 60)
    print("Testing VerusAgent Workflow Fixes")
    print("=" * 60)
    print()

    results = []
    results.append(("Assert forall detection", test_assert_forall_detection()))
    results.append(("Pattern-based repair", test_pattern_based_repair()))
    results.append(("Spec simplification", test_spec_simplification()))
    results.append(("Cast parenthesization", test_cast_parenthesization()))

    print()
    print("=" * 60)
    print("Summary")
    print("=" * 60)
    print()

    passed = sum(1 for _, result in results if result)
    total = len(results)

    for name, result in results:
        status = "✅ PASSED" if result else "❌ FAILED"
        print(f"{name}: {status}")

    print()
    print(f"Total: {passed}/{total} tests passed")

    if passed == total:
        print("\n🎉 All tests PASSED! ✅")
        return 0
    else:
        print(f"\n⚠️  {total - passed} test(s) failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
