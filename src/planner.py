import os
from context import Context
from prompts.template import fill_template
from configs.sconfig import config
from infer import 

task_overview = open(
    os.path.join(config['project_dir'], 'prompts', 'task_overview.md'),
    'w'
).read()

class Planner:
    def __init__(self, logger):
        self.logger = logger

    def exec(self, ctx: Context):
        modules = ""
        for module in ctx.modules.values():
            modules += f"- **{module.name}**: {module.desc}\n"
        
        system = fill_template('plan_system', {
            'task_overview': task_overview,
            'modules': modules,
        })

        prompt = ctx.gen_task_desc()

        return ctx.llm.infer_llm(
            '',
            instruction=None,
            exemplars=[],
            query=prompt,
            system_info=system,
            answer_num=1,
            max_tokens=100000,
            json=False,
            return_msg=True
        )


    