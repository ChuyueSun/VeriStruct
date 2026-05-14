"""Tests for RepairAssertionModule retry behavior (backlog #8).

Background: in the 2026-05-14 bitmap_todo run, repair_assertion was called
in repair rounds 3, 4, and 5. Each round took ~0.6s (cache hit) and served
the same non-improving LLM response three times. Root cause: repair_assert_fail
called self.llm.infer_llm directly, bypassing BaseRepairModule._get_llm_responses
— which is what adds "[Retry Attempt: N]" to the instruction (cache bypass)
and bumps temperature on retry.

These tests pin the new contract: repair_assert_fail routes through
_get_llm_responses, and exec retries with incrementing retry_attempt when
the previous attempt did not improve.
"""

import unittest
from unittest.mock import MagicMock, patch

from src.modules.repair_assertion import RepairAssertionModule
from src.modules.veval import VerusError, VerusErrorType


def _make_module():
    """Build a RepairAssertionModule with the LLM client mocked out."""
    config = {"aoai_debug_model": "claude-opus-4-6"}
    logger = MagicMock()
    with patch("src.modules.repair_assertion.LLM"):
        module = RepairAssertionModule(config=config, logger=logger)
    return module


def _make_failure(text="assert(x == y)"):
    """Minimal VerusError with one trace entry whose get_text returns `text`."""
    trace_entry = MagicMock()
    trace_entry.get_text = MagicMock(return_value=text)
    failure = MagicMock(spec=VerusError)
    failure.error = VerusErrorType.AssertFail
    failure.trace = [trace_entry]
    return failure


def _make_context(code="// dummy verus code\nverus! { }"):
    """Minimal context with one trial whose code is `code`."""
    trial = MagicMock()
    trial.code = code
    trial.eval = MagicMock()
    trial.eval.get_failures = MagicMock(return_value=[])
    ctx = MagicMock()
    ctx.trials = [trial]
    return ctx


class RepairAssertFailUsesGetLlmResponses(unittest.TestCase):
    def test_repair_assert_fail_calls_get_llm_responses_with_retry_attempt(self):
        module = _make_module()
        captured = {}

        def fake_get_llm_responses(**kwargs):
            captured.update(kwargs)
            return ["response 1", "response 2", "response 3"]

        # Patch on the instance — _get_llm_responses lives on BaseRepairModule.
        with patch.object(module, "_get_llm_responses", side_effect=fake_get_llm_responses):
            result = module.repair_assert_fail(
                context=_make_context(),
                failure_to_fix=_make_failure(),
                retry_attempt=2,
            )

        self.assertEqual(result, ["response 1", "response 2", "response 3"])
        self.assertEqual(captured["retry_attempt"], 2)
        self.assertTrue(captured["use_cache"])  # cache bypass is handled internally by retry_attempt > 0
        self.assertIsNotNone(captured["context"])  # context plumbed through for tracking
        self.assertIn("assertion error", captured["instruction"].lower())

    def test_repair_assert_fail_does_not_call_llm_directly(self):
        """The whole point of #8: stop bypassing _get_llm_responses."""
        module = _make_module()
        module.llm = MagicMock()
        module.llm.infer_llm = MagicMock(return_value=["should-not-be-called"])

        with patch.object(module, "_get_llm_responses", return_value=["via base helper"]):
            result = module.repair_assert_fail(
                context=_make_context(),
                failure_to_fix=_make_failure(),
            )

        self.assertEqual(result, ["via base helper"])
        module.llm.infer_llm.assert_not_called()


class ExecRetriesUntilImprovement(unittest.TestCase):
    def test_exec_retries_with_incrementing_retry_attempt_until_improvement(self):
        module = _make_module()
        seen_retry_attempts = []

        # First two attempts produce a candidate that doesn't improve the code;
        # the third attempt produces a candidate that does.
        def fake_repair_assert_fail(context, failure_to_fix, retry_attempt=0):
            seen_retry_attempts.append(retry_attempt)
            return [f"candidate from attempt {retry_attempt}"]

        original_code = "ORIGINAL"

        def fake_evaluate_repair_candidates(original_code, candidates, output_dir, prefix):
            # No improvement on the first two retries; the third retry's candidate is treated as improving.
            return candidates[0] if "attempt 2" in candidates[0] else original_code

        with patch.object(module, "repair_special_assertion_error", return_value=""):
            with patch.object(module, "repair_assert_fail", side_effect=fake_repair_assert_fail):
                with patch.object(module, "evaluate_repair_candidates", side_effect=fake_evaluate_repair_candidates):
                    ctx = _make_context(code=original_code)
                    result = module.exec(ctx, _make_failure())

        # We saw retry_attempts 0, 1, 2 in that order and stopped after improvement.
        self.assertEqual(seen_retry_attempts, [0, 1, 2])
        self.assertEqual(result, "candidate from attempt 2")

    def test_exec_returns_original_when_no_attempt_improves(self):
        """Bitmap_todo's failure shape: every attempt produces a non-improving candidate."""
        module = _make_module()
        original_code = "ORIGINAL"
        attempts = []

        def fake_repair_assert_fail(context, failure_to_fix, retry_attempt=0):
            attempts.append(retry_attempt)
            return [f"non-improving attempt {retry_attempt}"]

        def fake_evaluate(original_code, candidates, output_dir, prefix):
            return original_code  # never improves

        with patch.object(module, "repair_special_assertion_error", return_value=""):
            with patch.object(module, "repair_assert_fail", side_effect=fake_repair_assert_fail):
                with patch.object(module, "evaluate_repair_candidates", side_effect=fake_evaluate):
                    ctx = _make_context(code=original_code)
                    result = module.exec(ctx, _make_failure())

        # All three retry slots used; final result is original code.
        self.assertEqual(attempts, [0, 1, 2])
        self.assertEqual(result, original_code)

    def test_exec_returns_special_fix_immediately_when_it_improves(self):
        """Deterministic fast path short-circuits before any LLM call."""
        module = _make_module()
        original_code = "ORIGINAL"
        special_output = "SPECIAL_FIX"

        repair_assert_fail_mock = MagicMock()

        def fake_evaluate(original_code, candidates, output_dir, prefix):
            return candidates[0]  # treat the special fix as improving

        with patch.object(module, "repair_special_assertion_error", return_value=special_output):
            with patch.object(module, "repair_assert_fail", repair_assert_fail_mock):
                with patch.object(module, "evaluate_repair_candidates", side_effect=fake_evaluate):
                    ctx = _make_context(code=original_code)
                    result = module.exec(ctx, _make_failure())

        self.assertEqual(result, special_output)
        repair_assert_fail_mock.assert_not_called()


if __name__ == "__main__":
    unittest.main()
