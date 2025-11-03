import os
from pathlib import Path

from src.configs.sconfig import config
from src.context import Context
from src.prompts.template import fill_template

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
    """
    The Planner class is responsible for determining the verification workflow to use.
    It analyzes the code and decides which sequence of modules would be most effective
    for verification.
    """

    def __init__(self, logger):
        """Initialize the planner with a logger."""
        self.logger = logger

    def exec(self, ctx: Context):
        """
        Execute the planner to determine the verification workflow.

        Args:
            ctx: The context containing the code to analyze

        Returns:
            The LLM's response detailing the chosen workflow
        """
        # Create a list of available modules for the system prompt
        modules = ""
        for module in ctx.modules.values():
            modules += f"- **{module.name}**: {module.desc}\n"

        # Create the system prompt using the template
        system = fill_template(
            "plan_system",
            {
                "task_overview": task_overview,
                "modules": modules,
            },
        )

        # Create the user prompt with just the task description
        prompt = self.get_normalized_task_desc(ctx)

        # Call the LLM with tracking to make the decision
        return ctx.infer_llm_with_tracking(
            engine="",
            instruction=None,
            exemplars=[],
            query=prompt,
            system_info=system,
            answer_num=1,
            max_tokens=8192,
            json=False,
            return_msg=True,
            stage="planning",
            module="planner",
        )

    def get_normalized_task_desc(self, ctx: Context) -> str:
        """
        Generate a normalized task description without rustc_out to improve cache consistency.

        Args:
            ctx: The context containing the code to analyze

        Returns:
            A normalized task description with empty rustc_out for consistent caching
        """
        if ctx.params.trial_fetch_mode == "naive":
            # Naive mode: use the last trial
            trial = ctx.trials[-1]
            prevs = ctx.trials[-1 - ctx.params.max_prev_trial : -1]
        else:
            # Other mode: TODO
            trial = None
            prevs = []

        rloc = ctx.raw_code_loc
        verus_code = trial.code
        # Skip rustc_out to improve cache consistency
        knowledge = ctx.gen_knowledge()
        prev_descs = [
            f"### Failure {i}\n\n{ptrail.desc(rloc, output_rustc_out=False)}"
            for i, ptrail in enumerate(prevs)
        ]

        # Create the normalized description using the same template as Context.gen_task_desc()
        return fill_template(
            "task_desc",
            {
                "verus_code": verus_code,
                "rustc_out": "",  # Empty rustc_out for consistent caching
                "knowledge": knowledge,
                "failures": "\n\n".join(prev_descs),
            },
        )
