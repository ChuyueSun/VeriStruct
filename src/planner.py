import os
from pathlib import Path

from configs.sconfig import config
from context import Context
from prompts.template import fill_template

# Fix the file loading bug - changing from 'w' (write) to 'r' (read) mode
# Using Path for better file handling
task_overview_path = Path(config["project_dir"]) / "prompts" / "task_overview.md"
task_overview = task_overview_path.read_text() if task_overview_path.exists() else ""


class Planner:
    def __init__(self, logger):
        self.logger = logger

    def exec(self, ctx: Context):
        modules = ""
        for module in ctx.modules.values():
            modules += f"- **{module.name}**: {module.desc}\n"

        system = fill_template(
            "plan_system",
            {
                "task_overview": task_overview,
                "modules": modules,
            },
        )

        prompt = ctx.gen_task_desc()

        return ctx.llm.infer_llm(
            "",
            instruction=None,
            exemplars=[],
            query=prompt,
            system_info=system,
            answer_num=1,
            max_tokens=100000,
            json=False,
            return_msg=True,
        )
