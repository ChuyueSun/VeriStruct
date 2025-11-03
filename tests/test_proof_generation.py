import logging
import os
import sys
from pathlib import Path

# Ensure repository root is on the Python path
sys.path.append(str(Path(__file__).resolve().parents[1]))

from src.modules.proof_generation import ProofGenerationModule


def build_module() -> ProofGenerationModule:
    """Helper to create ProofGenerationModule with LLM disabled."""
    os.environ["ENABLE_LLM_INFERENCE"] = "0"
    logger = logging.getLogger("test")
    return ProofGenerationModule({}, logger)


def test_should_skip_with_todo():
    module = build_module()
    code = "// TODO: add proof"
    assert module._should_skip(code) is False


def test_should_skip_with_empty_proof_block():
    module = build_module()
    code = "fn main() { proof { } }"
    assert module._should_skip(code) is False


def test_should_skip_when_clean():
    module = build_module()
    code = "fn main() { assert(true); }"
    assert module._should_skip(code) is True
