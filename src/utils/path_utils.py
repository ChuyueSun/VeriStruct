from pathlib import Path
import os

_DEF = "output"


def get_output_dir() -> Path:
    """Return top-level output directory determined by VERUS_OUTPUT_DIR env."""
    return Path(os.environ.get("VERUS_OUTPUT_DIR", _DEF))


def samples_dir() -> Path:
    d = get_output_dir() / "samples"
    d.mkdir(parents=True, exist_ok=True)
    return d


def best_dir() -> Path:
    d = get_output_dir() / "best"
    d.mkdir(parents=True, exist_ok=True)
    return d


def debug_dir() -> Path:
    d = get_output_dir() / "debug"
    d.mkdir(parents=True, exist_ok=True)
    return d 