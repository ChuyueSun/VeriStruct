import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = REPO_ROOT / "src" / "modules" / "repair_registry.py"


def registered_error_owners():
    tree = ast.parse(REGISTRY_PATH.read_text())
    owners = {}

    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        if not isinstance(node.func, ast.Attribute):
            continue
        if node.func.attr != "register_module" or len(node.args) < 3:
            continue

        module_name_arg = node.args[0]
        error_types_arg = node.args[2]
        if not isinstance(module_name_arg, ast.Constant):
            continue
        if not isinstance(error_types_arg, ast.List):
            continue

        module_name = module_name_arg.value
        for error_node in error_types_arg.elts:
            if not isinstance(error_node, ast.Attribute):
                continue
            error_name = error_node.attr
            if error_name in owners:
                raise AssertionError(
                    f"{error_name} registered to both {owners[error_name]} and {module_name}"
                )
            owners[error_name] = module_name

    return owners


class RepairRegistryRegistrationTests(unittest.TestCase):
    def test_error_type_registrations_are_unique(self):
        registered_error_owners()

    def test_private_clause_errors_have_explicit_owners(self):
        owners = registered_error_owners()

        self.assertEqual(owners["ensure_private"], "repair_postcond")
        self.assertEqual(owners["require_private"], "repair_remove_inv")


if __name__ == "__main__":
    unittest.main()
