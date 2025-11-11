#!/usr/bin/env bash
# Setup script for pre-commit hooks

set -e

echo "=========================================="
echo "  Setting up Pre-commit Hooks"
echo "=========================================="
echo ""

# Check if pip is available
if ! command -v pip &> /dev/null; then
    echo "❌ Error: pip is not installed. Please install Python and pip first."
    exit 1
fi

# Install pre-commit
echo "📦 Installing pre-commit..."
pip install pre-commit

# Check if Rust is available
if ! command -v rustc &> /dev/null; then
    echo "⚠️  Warning: Rust is not installed."
    echo "   Some pre-commit hooks for Rust formatting won't work."
    echo "   Install Rust from: https://rustup.rs/"
    echo ""
else
    # Ensure rustfmt and clippy are installed
    echo "🦀 Setting up Rust tools..."
    rustup component add rustfmt clippy 2>/dev/null || true
fi

# Install git hooks
echo "🔧 Installing git hooks..."
pre-commit install

# Run pre-commit on all files to see current status
echo ""
echo "🔍 Running pre-commit checks on all files..."
echo "   (This may take a while on first run)"
echo ""

if pre-commit run --all-files; then
    echo ""
    echo "✅ All pre-commit checks passed!"
else
    echo ""
    echo "⚠️  Some pre-commit checks failed or made changes."
    echo "   Review the changes and commit them if appropriate."
    echo ""
    echo "   To commit the auto-fixes:"
    echo "   git add -u"
    echo "   git commit -m 'Apply pre-commit auto-fixes'"
fi

echo ""
echo "=========================================="
echo "  Pre-commit Setup Complete!"
echo "=========================================="
echo ""
echo "Your pre-commit hooks are now active."
echo "They will run automatically on 'git commit'."
echo ""
echo "Useful commands:"
echo "  - Run manually:    pre-commit run --all-files"
echo "  - Update hooks:    pre-commit autoupdate"
echo "  - Skip hooks:      git commit --no-verify (not recommended)"
echo ""
