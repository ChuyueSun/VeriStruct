from houdini import houdini

from infer import LLM


class Reader:
    def __init__(self, config, logger, immutable_funcs=[]):
        self.debug_mode = False
        self.config = config
        self.llm = LLM(config, logger)
        self.logger = logger
        self.hdn = houdini(config, immutable_funcs)
        self.immutable_funcs = immutable_funcs
        with open("verus-intro.txt", "r") as f:
            verus_knowledge = "".join(f.readlines())

        self.default_system = (
            "Verus is a tool for verifying the correctness of code written in Rust. Below, we provide some backgrounds on verus.\n\n## Verus Background\n"
            + verus_knowledge
            + "Now, you have been a wonderful expert at Verus, "
            + "and a good programmer that can write detailed comments to make the code more readable."
        )
        self.instruction = (
            "\n\n## Instruction\nBelow, there will be a verus code that lacks any comment.\n"
            "Please:"
            "- carefully read the code, \n"
            "- understand what it is doing, and\n"
            "- add a very detailed comment to the code at the following position:\n"
            "   - The pre- and post-condition of each function definition (tagged by `requires` and `ensures`)\n"
            "   - The proof function (tagged by `proof fn`)\n"
            "   - The specification function (tagged by `spec fn`)\n"
            "   - The loop invariant of each loop (tagged by `invariant`)\n"
            "   - Each program statement\n"
            "   - The proof code (wrapped by `proof {}`)\n\n"
            "### Note\n"
            "- Make sure that your comments are clear and detailed.\n"
            "- Do not modify the code.\n"
            "- Wrap the commented code using ```rust and ```.\n"
            "\n ## The Verus Code To Be Commented\n"
        )

    def set_debug_mode(self, val):
        self.debug_mode = val

    def show_prompt(self, f) -> None:
        f.write(self.default_system)
        f.write(self.instruction)

    def run_prompt(self, verus_code: str) -> str | None:
        if self.debug_mode:
            with open("../yican-trial/debug-prompt.md", "w") as f:
                self.show_prompt(f)
                f.write(verus_code)
            return None

        infer_result = self.llm.infer_llm(
            engine=None,
            instruction=self.instruction,
            exemplars=[],
            query=verus_code,
            system_info=self.default_system,
            answer_num=1,
            max_tokens=8192,
            temp=0,
            json=False,
            return_msg=False,
            verbose=True,
        )

        return infer_result[0]

    def run(self, input_path: str, output_path: str, args: dict) -> None:
        with open(input_path, "r") as f:
            verus_code = "".join(f.readlines())
            commented_code = self.run_prompt(verus_code)
            with open(output_path, "w") as f:
                f.write(commented_code)
