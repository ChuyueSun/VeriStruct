import unittest

from src.modules.veval import VEval


def make_veval(success: bool, compilation_error: bool = False) -> VEval:
    veval = VEval("")
    veval.verus_result = {"verification-results": {"success": success}}
    veval.compilation_error = compilation_error
    return veval


class VEvalStatusTests(unittest.TestCase):
    def test_verus_succeed_requires_successful_verification_without_compilation_error(self):
        self.assertTrue(make_veval(success=True).verus_succeed())
        self.assertFalse(make_veval(success=False).verus_succeed())
        self.assertFalse(
            make_veval(success=True, compilation_error=True).verus_succeed()
        )

    def test_result_accessors_raise_without_verus_result(self):
        veval = VEval("")

        with self.assertRaisesRegex(RuntimeError, "No Verus result"):
            veval.verus_succeed()
        with self.assertRaisesRegex(RuntimeError, "No Verus result"):
            veval.get_failed_postconds()
        with self.assertRaisesRegex(RuntimeError, "No Verus result"):
            veval.get_failures()
        with self.assertRaisesRegex(RuntimeError, "No Verus result"):
            veval.get_vstd_errors()


if __name__ == "__main__":
    unittest.main()
