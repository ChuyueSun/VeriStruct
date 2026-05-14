"""Tests that LLM.infer_llm surfaces HTTP error response bodies in logs.

Background: in the 2026-05-14 bitmap_todo run we hit "400 Client Error:
Bad Request" 19 times but couldn't see Anthropic's actual error message
because requests.raise_for_status() only surfaces the status code. The
underlying body — e.g. "messages.0: user messages must have non-empty
content" or "max_tokens: 200000 > 128000" — was thrown away.

This test pins the contract: when an HTTP error is raised, the response
body is included in the logged error line.
"""

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

import requests

import src.infer as infer
from src.infer import LLM


class _FailingResponse:
    """A response that mimics a 400 Bad Request with a JSON error body."""

    status_code = 400
    text = (
        '{"type":"error","error":{"type":"invalid_request_error",'
        '"message":"messages.0: user messages must have non-empty content"}}'
    )

    def json(self):
        return {"error": {"message": "messages.0: user messages must have non-empty content"}}

    def raise_for_status(self):
        # Mirror what requests does: raise HTTPError with self attached as .response.
        err = requests.HTTPError("400 Client Error: Bad Request for url: ...")
        err.response = self
        raise err


class HttpErrorBodyLoggingTests(unittest.TestCase):
    def _build_llm(self, cache_dir: Path):
        config = {
            "platform": "anthropic",
            "anthropic_api_key": "sk-ant-test",
            "anthropic_generation_model": "claude-opus-4-6",
            "cache_dir": str(cache_dir),
        }
        return LLM(config, MagicMock(), use_cache=False)

    def test_400_response_body_is_logged_when_raise_for_status_fires(self):
        with tempfile.TemporaryDirectory() as cache_dir:
            llm = self._build_llm(Path(cache_dir) / "cache")
            llm.logger = MagicMock()

            def fake_post(url, headers, json, timeout):
                return _FailingResponse()

            with patch.object(infer, "requests", SimpleNamespace(post=fake_post, HTTPError=requests.HTTPError)):
                result = llm.infer_llm(
                    engine="ignored-for-anthropic",
                    instruction="dummy",
                    exemplars=[],
                    query="dummy",
                    system_info="dummy",
                    answer_num=1,
                    max_tokens=128,
                    temp=0.1,
                    use_cache=False,
                )

        self.assertEqual(result, [])  # error path returns []

        # The logger.error call must include the body content.
        error_logs = [
            call.args[0]
            for call in llm.logger.error.call_args_list
            if call.args and "Direct LLM call failed" in str(call.args[0])
        ]
        self.assertTrue(
            error_logs,
            "Expected an error log line starting with 'Direct LLM call failed'",
        )
        joined = " ".join(error_logs)
        self.assertIn(
            "user messages must have non-empty content",
            joined,
            f"Body text missing from error log. Got: {joined}",
        )
        # The status-code preamble should still be present
        self.assertIn("400", joined)

    def test_non_http_exception_does_not_break_logging(self):
        """Regression guard: when `requests.post` raises something *without*
        a .response attribute (e.g. a connection error), we still log cleanly."""

        with tempfile.TemporaryDirectory() as cache_dir:
            llm = self._build_llm(Path(cache_dir) / "cache")
            llm.logger = MagicMock()

            def fake_post(url, headers, json, timeout):
                raise requests.ConnectionError("DNS lookup failed")

            with patch.object(infer, "requests", SimpleNamespace(post=fake_post, ConnectionError=requests.ConnectionError, HTTPError=requests.HTTPError)):
                result = llm.infer_llm(
                    engine="ignored-for-anthropic",
                    instruction="dummy",
                    exemplars=[],
                    query="dummy",
                    system_info="dummy",
                    answer_num=1,
                    max_tokens=128,
                    temp=0.1,
                    use_cache=False,
                )

        self.assertEqual(result, [])
        error_logs = [
            call.args[0]
            for call in llm.logger.error.call_args_list
            if call.args and "Direct LLM call failed" in str(call.args[0])
        ]
        self.assertTrue(error_logs)
        # Must not have the body-suffix marker since there's no response object
        for log_line in error_logs:
            self.assertNotIn(" | body: ", log_line)


if __name__ == "__main__":
    unittest.main()
