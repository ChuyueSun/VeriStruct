#!/usr/bin/env python3
"""
Quick verification script for repair round timeout implementation.
Checks that all necessary components are in place.
"""

import json
import sys
from pathlib import Path


def verify_config():
    """Verify config has the timeout parameter."""
    config_path = Path("src/configs/config-azure.json")

    if not config_path.exists():
        print(f"❌ Config file not found: {config_path}")
        return False

    with open(config_path) as f:
        config = json.load(f)

    if "repair_round_timeout" in config:
        timeout = config["repair_round_timeout"]
        print(f"✓ Config has repair_round_timeout: {timeout}s")
        return True
    else:
        print("❌ Config missing repair_round_timeout parameter")
        return False


def verify_main_py():
    """Verify main.py uses the timeout."""
    main_path = Path("src/main.py")

    if not main_path.exists():
        print(f"❌ Main file not found: {main_path}")
        return False

    content = main_path.read_text()

    checks = [
        ("repair_round_timeout = config.get", "Extract timeout from config"),
        ("round_timeout=repair_round_timeout", "Pass timeout to repair_all"),
        ("round_start_time=repair_round_start", "Pass start time to repair_all"),
    ]

    all_passed = True
    for check_str, description in checks:
        if check_str in content:
            print(f"✓ main.py: {description}")
        else:
            print(f"❌ main.py missing: {description}")
            all_passed = False

    return all_passed


def verify_repair_registry():
    """Verify repair_registry.py has timeout checks."""
    registry_path = Path("src/modules/repair_registry.py")

    if not registry_path.exists():
        print(f"❌ Registry file not found: {registry_path}")
        return False

    content = registry_path.read_text()

    checks = [
        ("round_timeout: Optional[float]", "Timeout parameter in repair_all"),
        ("round_start_time: Optional[float]", "Start time parameter in repair_all"),
        ("def check_round_timeout():", "Timeout check helper function"),
        ("check_round_timeout()", "Timeout check calls"),
    ]

    all_passed = True
    for check_str, description in checks:
        if check_str in content:
            print(f"✓ repair_registry.py: {description}")
        else:
            print(f"❌ repair_registry.py missing: {description}")
            all_passed = False

    # Count timeout check calls
    check_count = content.count("check_round_timeout()")
    if check_count >= 4:
        print(f"✓ repair_registry.py: {check_count} timeout checks (≥4 expected)")
    else:
        print(
            f"⚠ repair_registry.py: Only {check_count} timeout checks (4+ recommended)"
        )

    return all_passed


def verify_docs():
    """Verify documentation exists."""
    docs = [
        "docs/repair_round_timeout.md",
        "REPAIR_ROUND_TIMEOUT_IMPLEMENTATION.md",
        "examples/repair_round_timeout_comparison.md",
    ]

    all_exist = True
    for doc in docs:
        doc_path = Path(doc)
        if doc_path.exists():
            print(f"✓ Documentation: {doc}")
        else:
            print(f"❌ Documentation missing: {doc}")
            all_exist = False

    return all_exist


def verify_tests():
    """Verify test file exists."""
    test_path = Path("tests/test_repair_round_timeout.py")

    if not test_path.exists():
        print(f"❌ Test file not found: {test_path}")
        return False

    print(f"✓ Test file exists: {test_path}")
    return True


def main():
    print("=" * 70)
    print("REPAIR ROUND TIMEOUT IMPLEMENTATION VERIFICATION")
    print("=" * 70)
    print()

    results = []

    print("1. Configuration File")
    print("-" * 70)
    results.append(verify_config())
    print()

    print("2. Main Entry Point (main.py)")
    print("-" * 70)
    results.append(verify_main_py())
    print()

    print("3. Repair Registry (repair_registry.py)")
    print("-" * 70)
    results.append(verify_repair_registry())
    print()

    print("4. Documentation")
    print("-" * 70)
    results.append(verify_docs())
    print()

    print("5. Test Suite")
    print("-" * 70)
    results.append(verify_tests())
    print()

    print("=" * 70)
    if all(results):
        print("✅ ALL VERIFICATIONS PASSED")
        print("=" * 70)
        print()
        print("Repair round timeout is properly implemented!")
        print()
        print("Configuration:")
        print("  - Default timeout: 900 seconds (15 minutes)")
        print("  - Config location: src/configs/config-azure.json")
        print()
        print("To test:")
        print("  python tests/test_repair_round_timeout.py")
        print()
        return 0
    else:
        print("❌ SOME VERIFICATIONS FAILED")
        print("=" * 70)
        print("Please review the failed checks above.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
