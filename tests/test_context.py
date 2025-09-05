import logging
import os
import sys
from pathlib import Path

import pytest

# Ensure repository root is on the Python path
sys.path.append(str(Path(__file__).resolve().parents[1]))

from src.context import Context, HyperParams


def build_context(mode: str) -> Context:
    """Helper to create a Context with the given trial_fetch_mode."""
    # Disable external LLM calls for testing
    os.environ["ENABLE_LLM_INFERENCE"] = "0"
    logger = logging.getLogger("test")
    return Context("fn main() {}", HyperParams(trial_fetch_mode=mode), logger)


def test_gen_task_desc_unsupported_mode_raises():
    ctx = build_context("unsupported")
    with pytest.raises(NotImplementedError):
        ctx.gen_task_desc()
