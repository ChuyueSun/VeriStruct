from context import Context
from prompts.template import fill_template


class Planner:
    def __init__(self, logger):
        self.logger = logger

    def gen_prompt(self, ctx: Context):
        modules_desc = ""
        for module in ctx.modules.values():
            modules_desc += f"- {module.name}: {module.desc}\n"

        
            

    def exec(self, ctx: Context):

    