"""Tests for BaseRepairModule.evaluate_repair_candidates (backlog #10).

Background: in the rb_type_invariant_todo run on 2026-05-13, the LLM correctly
identified that a duplicate ex_saturating_sub specification needed to be
removed, but its response was wrapped with a natural-language preamble
("Looking at the error, the issue is...") plus markdown fences. The repair
pipeline wrote the raw response — prose, fences, and all — directly as a
.rs file (repair_round_3_*.rs lines 1-3), regressing the error count from
1 to 9 and corrupting context.trials.

These tests pin the contract: evaluate_repair_candidates must extract code
from LLM responses before any safety check or downstream processing.
"""

import unittest
from pathlib import Path
from unittest.mock import patch

from src.modules.baserepair import BaseRepairModule


# A simplified version of the actual round-3 LLM response that caused the regression
PROSE_THEN_FENCE_RESPONSE = """Looking at the error, the issue is that `ex_saturating_sub` is a duplicate specification for `usize::saturating_sub`, which Verus already has a built-in specification for. I need to remove it.

```rust
use vstd::prelude::*;

verus! {

pub fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|i: int| 0 <= i < v.len() && v[i] == k,
    ensures
        r < v.len(),
        v[r as int] == k,
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            i1 <= i2,
            i2 < v.len(),
            exists|i: int| i1 <= i <= i2 && v[i] == k,
    {
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
    }
    i1
}

} // verus!
```
"""

ORIGINAL_CODE = """use vstd::prelude::*;

verus! {

pub fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
    {
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
    }
    i1
}

} // verus!
"""


class _StubRepairModule(BaseRepairModule):
    """Minimal subclass that bypasses LLM init and stubs out check_code_safety."""

    def __init__(self):
        # Bypass parent __init__ which would construct an LLM client.
        self.config = {}
        self.logger = _SilentLogger()
        self.immutable_funcs = []
        self.safety_calls = []
        self.name = "stub"
        self.desc = "stub repair for testing"

    def check_code_safety(self, original_code, new_code):
        self.safety_calls.append(new_code)
        return True


class _SilentLogger:
    def info(self, *args, **kwargs):
        pass

    def warning(self, *args, **kwargs):
        pass

    def error(self, *args, **kwargs):
        pass

    def debug(self, *args, **kwargs):
        pass


def _fake_evaluate_samples(samples, output_dir, prefix, logger):
    # Return the first sample so we can inspect what was forwarded.
    return samples[0] if samples else "", None, []


class CandidateExtractionTests(unittest.TestCase):
    def test_prose_and_fence_are_stripped_before_safety_and_eval(self):
        module = _StubRepairModule()

        with patch("src.modules.utils.evaluate_samples", side_effect=_fake_evaluate_samples):
            best = module.evaluate_repair_candidates(
                original_code=ORIGINAL_CODE,
                candidates=[PROSE_THEN_FENCE_RESPONSE],
                output_dir=Path("/tmp"),
                prefix="rb_regression",
            )

        # Safety check was called with extracted code, NOT the raw response
        self.assertEqual(len(module.safety_calls), 1)
        forwarded = module.safety_calls[0]
        self.assertNotIn("Looking at the error", forwarded)
        self.assertNotIn("```rust", forwarded)
        self.assertNotIn("```", forwarded.splitlines()[0] if forwarded.splitlines() else "")
        # And the actual code content survived
        self.assertIn("pub fn binary_search", forwarded)
        self.assertIn("verus!", forwarded)
        # And the returned best_code is similarly clean
        self.assertNotIn("Looking at the error", best)
        self.assertNotIn("```rust", best)

    def test_empty_response_is_excluded_rather_than_falling_back_to_raw(self):
        module = _StubRepairModule()

        with patch("src.modules.utils.evaluate_samples", side_effect=_fake_evaluate_samples):
            best = module.evaluate_repair_candidates(
                original_code=ORIGINAL_CODE,
                candidates=["", "   \n  "],
                output_dir=Path("/tmp"),
                prefix="empty",
            )

        # Neither empty candidate should have reached the safety check
        self.assertEqual(module.safety_calls, [])
        # And the function should fall back to original code
        self.assertEqual(best, ORIGINAL_CODE)

    def test_pure_code_response_passes_through_unchanged(self):
        # If the LLM behaves and just returns code, we shouldn't munge it.
        module = _StubRepairModule()
        pure_code = ORIGINAL_CODE  # no prose, no fences

        with patch("src.modules.utils.evaluate_samples", side_effect=_fake_evaluate_samples):
            best = module.evaluate_repair_candidates(
                original_code=ORIGINAL_CODE,
                candidates=[pure_code],
                output_dir=Path("/tmp"),
                prefix="pure",
            )

        self.assertEqual(len(module.safety_calls), 1)
        # parse_llm_response normalizes whitespace slightly via its keyword-count
        # path. Assert structural content, not exact equality.
        self.assertIn("pub fn binary_search", module.safety_calls[0])
        self.assertIn("verus!", best)


if __name__ == "__main__":
    unittest.main()
