from context import Context

class Planner:
    def __init__(self, logger):
        self.logger = logger

    def plan_prompt(self, ctx: Context):
        modules_desc = ""
        for module in ctx.modules.values():
            modules_desc += f"- {module.name}: {module.desc}\n"

    