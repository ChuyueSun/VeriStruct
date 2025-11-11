# GitHub Configuration

This directory contains GitHub-specific configuration files for the VerusAgent repository.

## Pre-commit Hooks

This repository uses [pre-commit](https://pre-commit.com/) to ensure code quality and consistency.

### Setup

1. **Install pre-commit:**

```bash
pip install pre-commit
```

2. **Install the git hooks:**

```bash
pre-commit install
```

3. **Run manually (optional):**

```bash
# Run on all files
pre-commit run --all-files

# Run on staged files only
pre-commit run
```

### What Gets Checked

The pre-commit hooks run the following checks:

- **General:** Trailing whitespace, end-of-file fixes, YAML/JSON/TOML validation
- **Python:** Code formatting (black), import sorting (isort), linting (flake8)
- **Rust:** Code formatting (rustfmt), linting (clippy)
- **Shell:** Script linting (shellcheck)
- **Markdown:** Linting and formatting
- **Security:** Detect private keys, large files

### Skipping Hooks

If you absolutely need to skip the pre-commit hooks (not recommended):

```bash
git commit --no-verify
```

## GitHub Actions

### Pre-commit Workflow

The `.github/workflows/pre-commit.yml` workflow runs on every push and pull request to ensure all code meets quality standards. This workflow:

- Runs all pre-commit hooks
- Fails if any checks don't pass
- Provides detailed error messages

## Troubleshooting

### Pre-commit failing on existing files

If pre-commit fails on files you didn't modify:

```bash
# Auto-fix what can be fixed
pre-commit run --all-files

# Commit the fixes
git add -u
git commit -m "Apply pre-commit fixes"
```

### Updating pre-commit hooks

```bash
pre-commit autoupdate
```

### Rust tools not found

Install Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy
```
