"""Tests for get_examples skipping empty exemplar pairs.

Background: on 2026-05-15 the full Opus 4.6 sweep failed 2 of 13 benchmarks
because src/examples/input-assert/ex{6,7,8,9}.rs and matching outputs were
zero-byte placeholder files committed in the repo's initial commit. They
were loaded as `{"query": "", "answer": ""}` and sent to Anthropic as
empty user/assistant messages, which the API rejects with HTTP 400:
"messages.N: user messages must have non-empty content".

The placeholder files have been removed; this test pins the defensive
filter so the bug doesn't recur if anyone re-introduces empty stubs.
"""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock

from src.modules.utils import get_examples


class GetExamplesEmptyFilterTests(unittest.TestCase):
    def _make_examples_dir(self, root: Path, kind: str, files: dict) -> None:
        """Build src/examples/input-<kind>/ and output-<kind>/ trees from a dict."""
        input_dir = root / f"input-{kind}"
        output_dir = root / f"output-{kind}"
        input_dir.mkdir(parents=True, exist_ok=True)
        output_dir.mkdir(parents=True, exist_ok=True)
        for name, (query, answer) in files.items():
            (input_dir / name).write_text(query)
            (output_dir / name).write_text(answer)

    def test_zero_byte_input_files_are_skipped(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "examples"
            self._make_examples_dir(
                root,
                "assert",
                {
                    "ex1.rs": ("real query 1", "real answer 1"),
                    "ex2.rs": ("real query 2", "real answer 2"),
                    "ex3.rs": ("", ""),  # zero-byte placeholder (the historical pattern)
                    "ex4.rs": ("", ""),
                },
            )
            examples = get_examples(
                {"example_path": str(root)}, "assert", MagicMock()
            )

        self.assertEqual(len(examples), 2)
        for ex in examples:
            self.assertTrue(ex["query"].strip(), "Filter let an empty query through")
            self.assertTrue(ex["answer"].strip(), "Filter let an empty answer through")

    def test_whitespace_only_files_are_skipped(self):
        """Anthropic rejects whitespace-only content the same way as empty."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "examples"
            self._make_examples_dir(
                root,
                "assert",
                {
                    "ex1.rs": ("real query", "real answer"),
                    "ex2.rs": ("   \n\t  ", "real answer"),  # whitespace input
                    "ex3.rs": ("real query", "\n  \n"),  # whitespace output
                },
            )
            examples = get_examples(
                {"example_path": str(root)}, "assert", MagicMock()
            )

        self.assertEqual(len(examples), 1)
        self.assertEqual(examples[0]["query"], "real query")

    def test_examples_with_substantial_content_pass_through(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "examples"
            self._make_examples_dir(
                root,
                "syntax",
                {
                    "ex1.rs": ("fn foo() {}", "fn foo() -> bool { true }"),
                    "ex2.rs": ("verus! { fn bar() {} }", "verus! { spec fn bar() -> bool { true } }"),
                },
            )
            examples = get_examples(
                {"example_path": str(root)}, "syntax", MagicMock()
            )

        self.assertEqual(len(examples), 2)
        self.assertIn("fn foo()", examples[0]["query"])

    def test_returns_empty_list_when_all_examples_are_empty(self):
        """Edge case: directory exists but every example is a zero-byte stub."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "examples"
            self._make_examples_dir(
                root,
                "assert",
                {
                    "ex1.rs": ("", ""),
                    "ex2.rs": ("", ""),
                },
            )
            examples = get_examples(
                {"example_path": str(root)}, "assert", MagicMock()
            )

        self.assertEqual(examples, [])


if __name__ == "__main__":
    unittest.main()
