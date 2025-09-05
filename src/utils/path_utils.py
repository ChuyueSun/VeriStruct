from pathlib import Path
import os

_DEF = "output"


def get_output_dir() -> Path:
    """Return top-level output directory determined by VERUS_OUTPUT_DIR env."""
    return Path(os.environ.get("VERUS_OUTPUT_DIR", _DEF))


def _ensure_subdir(name: str) -> Path:
    d = get_output_dir() / name
    d.mkdir(parents=True, exist_ok=True)
    return d


def samples_dir() -> Path:
    return _ensure_subdir("samples")


def best_dir() -> Path:
    return _ensure_subdir("best")


def debug_dir() -> Path:
    return _ensure_subdir("debug")
