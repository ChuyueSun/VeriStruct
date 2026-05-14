"""Tests for RepairRegistry._should_rollback (backlog #11).

Background: the rb_type_invariant_todo run on 2026-05-13 regressed from 1 to 9
Verus errors in a repair round. Both before and after had `compilation_error=True`,
so the old rollback condition — which only fired when a *new* compilation error
was introduced — did not pop the worse trial. This test exercises the corrected
behavior: any strictly-worse EvalScore should trigger rollback.
"""

import unittest
from unittest.mock import Mock

from src.modules.repair_registry import RepairRegistry
from src.modules.veval import EvalScore


def _score(verified=0, errors=0, compilation_error=False, verus_errors=0):
    return EvalScore(verified, errors, compilation_error, verus_errors)


class ShouldRollbackTests(unittest.TestCase):
    def setUp(self):
        # _should_rollback is a method but doesn't touch instance state beyond
        # __init__. We bind it to a real (mock-configured) registry to be safe.
        self.registry = RepairRegistry.__new__(RepairRegistry)
        self.registry.logger = Mock()

    # --- New-compilation-error case (existing behavior) ---

    def test_rollback_when_new_compilation_error_is_introduced(self):
        before = _score(verified=5, errors=0, compilation_error=False, verus_errors=0)
        after = _score(verified=-1, errors=999, compilation_error=True, verus_errors=2)
        self.assertTrue(self.registry._should_rollback(before, after))

    # --- New case from backlog #11 ---

    def test_rollback_when_both_fail_compilation_but_after_has_more_verus_errors(self):
        # Exact shape of the rb regression: 1 verus error -> 9 verus errors,
        # both compile-failing.
        before = _score(verified=-1, errors=999, compilation_error=True, verus_errors=1)
        after = _score(verified=-1, errors=999, compilation_error=True, verus_errors=9)
        self.assertTrue(self.registry._should_rollback(before, after))

    def test_rollback_when_both_compile_but_after_has_fewer_verified(self):
        before = _score(verified=10, errors=0, compilation_error=False, verus_errors=0)
        after = _score(verified=5, errors=0, compilation_error=False, verus_errors=0)
        self.assertTrue(self.registry._should_rollback(before, after))

    def test_rollback_when_both_compile_but_after_has_more_errors(self):
        before = _score(verified=5, errors=2, compilation_error=False, verus_errors=0)
        after = _score(verified=5, errors=5, compilation_error=False, verus_errors=0)
        self.assertTrue(self.registry._should_rollback(before, after))

    # --- No-rollback cases ---

    def test_no_rollback_when_after_is_strictly_better(self):
        before = _score(verified=-1, errors=999, compilation_error=True, verus_errors=9)
        after = _score(verified=-1, errors=999, compilation_error=True, verus_errors=1)
        self.assertFalse(self.registry._should_rollback(before, after))

    def test_no_rollback_when_fixing_compilation_error(self):
        before = _score(verified=-1, errors=999, compilation_error=True, verus_errors=5)
        after = _score(verified=8, errors=0, compilation_error=False, verus_errors=0)
        self.assertFalse(self.registry._should_rollback(before, after))

    def test_no_rollback_when_scores_are_equal(self):
        # Same score in, same score out — repair was a no-op. We don't roll back
        # because the repair did no harm; the surrounding "did not improve"
        # warning at repair_registry.py:686 captures the lack of progress.
        before = _score(verified=-1, errors=999, compilation_error=True, verus_errors=3)
        after = _score(verified=-1, errors=999, compilation_error=True, verus_errors=3)
        self.assertFalse(self.registry._should_rollback(before, after))

    # --- None-safety ---

    def test_no_rollback_when_before_score_is_none(self):
        after = _score(verified=-1, errors=999, compilation_error=True, verus_errors=9)
        self.assertFalse(self.registry._should_rollback(None, after))

    def test_no_rollback_when_after_score_is_none(self):
        before = _score(verified=10, errors=0, compilation_error=False, verus_errors=0)
        self.assertFalse(self.registry._should_rollback(before, None))


if __name__ == "__main__":
    unittest.main()
