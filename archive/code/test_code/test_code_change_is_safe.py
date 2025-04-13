import unittest
import tempfile
import os
from unittest.mock import patch, MagicMock

# Adjust this import to your actual code location, e.g.,:
# from my_module.file_name import get_func_body
from utils import get_func_body

class TestGetFuncBody(unittest.TestCase):
    def test_get_func_body_success(self):
        """
        Test that get_func_body returns the stdout from the subprocess when returncode == 0.
        """
        # Sample Rust code; doesn't matter much, since we're going to mock the subprocess anyway.
        test_code = r"""fn test_function() {
    let x = 42;
}

#[verifier::loop_isolation(false)]
fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)
    requires
        len < usize::MAX - 1,
        iterations * 2 < usize::MAX,
{
    let mut ring: Vec<i32> = Vec::new();

    if len == 0 {
        return;
    }

    for i in 0..(len + 1)
        invariant
            ring.len() == i,
    {
        ring.push(0);
    }

    assert(ring.len() > 1);
    let mut buf = RingBuffer::new(ring);
    assert(buf@.1 > 1);

    for _ in 0..2 * iterations
        invariant
            buf@.0.len() == 0,
            buf@.1 > 1
    {
        let enqueue_res = buf.enqueue(value);
        assert(enqueue_res);

        let buf_len = buf.len();
        assert(buf_len == 1);

        let has_elements = buf.has_elements();
        assert(has_elements);

        let dequeue_res = buf.dequeue();
        assert(dequeue_res =~= Some(value));

        let buf_len = buf.len();
        assert(buf_len == 0);

        let has_elements = buf.has_elements();
        assert(!has_elements);
    }
}
"""
        func_name = "test_enqueue_dequeue_generic"
        # func_name = "test_function"

        # We'll simulate Cargo returning some extracted body text.
        mock_stdout = "fn test_function() {\n    let y = 99;\n}\n"

        # Patch subprocess.run so we don't actually call Cargo.
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(
                returncode=0,
                stdout=mock_stdout,
                stderr=""
            )

            # Call the function
            result = get_func_body(test_code, func_name, util_path="/fake/util/path")

            # Verify the output is exactly what the mock says
            self.assertEqual(result, mock_stdout.strip())

            # Verify subprocess.run was called with the correct arguments
            self.assertTrue(mock_run.called, "subprocess.run should have been called.")
            args, kwargs = mock_run.call_args
            # The first argument to run is the command list
            cmd_list = args[0]

            # Check some key parts of the command
            self.assertIn("cargo", cmd_list, "Should call cargo in the command")
            self.assertIn("func", cmd_list, "Should call the 'func' subcommand")
            self.assertIn("extract", cmd_list, "Should use the 'extract' option")
            self.assertIn("-f", cmd_list, "Should pass the function name argument")
            self.assertIn(func_name, cmd_list, "Should pass the correct function name")

    def test_get_func_body_failure(self):
        """
        Test that get_func_body returns an empty string if returncode != 0.
        """
        test_code = "fn another_function() { let x = 123; }"
        func_name = "another_function"

        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(
                returncode=1,
                stdout="Some error text",
                stderr="Cargo error"
            )

            result = get_func_body(test_code, func_name, util_path="/fake/util/path")
            self.assertEqual(result, "", "Should return empty string on failure")

    def test_temp_file_cleanup(self):
        """
        Test that the temporary file is created and then removed by get_func_body.
        """
        test_code = "fn some_function() { }"
        func_name = "some_function"

        # We'll check that the temp file is created and then removed.
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0, stdout="extracted", stderr="")

            # We'll also patch the NamedTemporaryFile to capture the filename
            with patch("tempfile.NamedTemporaryFile", wraps=tempfile.NamedTemporaryFile) as mock_tmp:
                result = get_func_body(test_code, func_name, util_path="/fake/util/path")
                
                # The function should return "extracted"
                self.assertEqual(result, "extracted")

                # Ensure a temp file was indeed created
                self.assertTrue(mock_tmp.called, "NamedTemporaryFile should be called")

                # The call should remove the file via os.unlink
                # If you'd like, you can also patch 'os.unlink' to verify it was invoked.
                # But typically verifying no leftover file is enough in practice.

    # Additional tests might include edge cases such as an empty code string,
    # a non-existent function name, etc.

if __name__ == "__main__":
    unittest.main()
