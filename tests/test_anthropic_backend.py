import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

import src.infer as infer
from src.infer import LLM


class FakeResponse:
    def __init__(self, text):
        self.text = text

    def raise_for_status(self):
        return None

    def json(self):
        return {
            "content": [{"type": "text", "text": self.text}],
            "usage": {"input_tokens": 11, "output_tokens": 7},
        }


class AnthropicBackendTests(unittest.TestCase):
    def test_opus_4_6_backend_builds_anthropic_messages_payload(self):
        calls = []

        def fake_post(url, headers, json, timeout):
            calls.append(
                {
                    "url": url,
                    "headers": headers,
                    "payload": json,
                    "timeout": timeout,
                }
            )
            return FakeResponse(f"anthropic answer {len(calls)}")

        with tempfile.TemporaryDirectory() as cache_dir:
            config = {
                "platform": "anthropic",
                "anthropic_api_key": "sk-ant-test",
                "anthropic_generation_model": "claude-opus-4-6",
                "cache_dir": str(Path(cache_dir) / "cache"),
                "always_write_cache": False,
            }
            llm = LLM(config, Mock(), use_cache=False)

            with patch.object(infer, "requests", SimpleNamespace(post=fake_post)):
                answers = llm.infer_llm(
                    engine="ignored-for-anthropic",
                    instruction="Follow the verification style.",
                    exemplars=[],
                    query="Fix this Verus function.",
                    system_info="You are a Verus assistant.",
                    answer_num=2,
                    max_tokens=128,
                    temp=0.2,
                    use_cache=False,
                )

        self.assertEqual(answers, ["anthropic answer 1", "anthropic answer 2"])
        self.assertEqual(len(calls), 2)
        captured = calls[0]
        self.assertEqual(captured["url"], "https://api.anthropic.com/v1/messages")
        self.assertEqual(captured["headers"]["x-api-key"], "sk-ant-test")
        self.assertEqual(captured["payload"]["model"], "claude-opus-4-6")
        self.assertEqual(captured["payload"]["max_tokens"], 128)
        self.assertEqual(captured["payload"]["temperature"], 0.2)
        self.assertNotIn("n", captured["payload"])
        self.assertEqual(captured["payload"]["system"], "You are a Verus assistant.")
        self.assertNotIn(
            "system",
            {message["role"] for message in captured["payload"]["messages"]},
        )
        self.assertEqual(calls[1]["payload"], captured["payload"])

    def test_legacy_anthropic_model_key_and_list_api_key_are_supported(self):
        llm = LLM(
            {
                "platform": "anthropic",
                "anthropic_api_key": ["sk-ant-list"],
                "anthropic_model": "claude-opus-4-6",
            },
            Mock(),
            use_cache=False,
        )

        self.assertEqual(llm._get_anthropic_api_key(), "sk-ant-list")
        self.assertEqual(llm._get_anthropic_model(), "claude-opus-4-6")


if __name__ == "__main__":
    unittest.main()
