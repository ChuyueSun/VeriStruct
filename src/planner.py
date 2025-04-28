import os
from pathlib import Path

from configs.sconfig import config
from context import Context
from prompts.template import fill_template

# Fix the file loading bug and make path resolution more robust
# Use the current script directory as a fallback if project_dir is not in config
current_dir = Path(os.path.dirname(os.path.abspath(__file__)))
project_dir = config.get("project_dir", current_dir.parent)
task_overview_path = Path(project_dir) / "prompts" / "task_overview.md"

# Fall back to a relative path if the absolute path doesn't exist
if not task_overview_path.exists():
    task_overview_path = current_dir.parent / "prompts" / "task_overview.md"

task_overview = task_overview_path.read_text() if task_overview_path.exists() else ""


class Planner:
    def __init__(self, logger):
        self.logger = logger

    def exec(self, ctx: Context):
        modules = ""
        for module in ctx.modules.values():
            modules += f"- **{module.name}**: {module.desc}\n"
            
        workflow_options = """
## Workflow Options
There are exactly two possible workflows for verifying Verus code:

1. **Full Sequence Workflow**: 
   - Step 1: View Inference - Generate a View function for the data structure
   - Step 2: View Refinement - Refine the generated View implementation
   - Step 3: Invariant Inference - Generate invariants for loops and data structures
   - Step 4: Specification Inference - Generate function specifications (requires/ensures)

2. **Specification-Only Workflow**:
   - Step 1: Specification Inference - Generate function specifications without implementing a View

Your task is to decide which workflow is most appropriate for the given Verus code.
Choose the Specification-Only workflow only if the code has no data structures needing a View implementation.
        """

        system = fill_template(
            "plan_system",
            {
                "task_overview": task_overview,
                "modules": modules,
                "workflow_options": workflow_options
            },
        )

        prompt = f"""
{ctx.gen_task_desc()}

Analyze the code and decide which of the two possible workflows is most appropriate:
1. Full Sequence Workflow (view_inference → view_refinement → inv_inference → spec_inference)
2. Specification-Only Workflow (spec_inference only)

Explain your choice in 2-3 sentences, then specify the exact workflow to use.
"""

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
