from typing import List, Dict, Optional
from modules.base import BaseModule
from modules.veval import VEval
from configs.sconfig import config
from prompts.template import fill_template
import os, subprocess
from dataclasses import dataclass

class Trial:
    def __init__(self, trial_id: int, eval: VEval,
                 code_loc: Optional[str] = None,
                 logger = None):
        self.id = trial_id
        self.eval = eval
        self.code_loc = code_loc
        self.eval.eval(100, True, None)
        self.logger = logger

    @property
    def code(self):
        return self.eval.code
    
    @property
    def rustc_out(self):
        return self.eval.rustc_out
    
    def desc(self, diff_loc: Optional[str]=None,
             output_rustc_out = True):
        ans = ''
        if (diff_loc is None) or (self.code_loc is None):
            ans += "#### Verus Code\n\n"
            ans += '```verus\n'
            ans += self.code
            ans += '```\n\n'
            if output_rustc_out:
                ans += "#### Compilation Result\n\n"
                ans += self.rustc_out
        else:
            ans += "#### Verus Code (Modified Part)\n\n"
            result = subprocess.run(
                ['git', 'diff', self.code_loc, diff_loc],
                text=True,
                capture_output=True,
                check=True
            )
            ans += result.stdout + '\n\n'
            if output_rustc_out:
                ans += "#### Compilation Result\n\n"
                ans += self.rustc_out

        return ans

@dataclass
class HyperParams:
    trial_fetch_mode: str = 'naive'
    max_prev_trial: int = 4


class Context:
    """
    Context class to store the trials and modules.
    """
    def __init__(self, raw_code: str, params: HyperParams, logger):
        self.trials: List[Trial] = []
        self.modules: Dict[str, BaseModule] = {}
        self.knowledge: Dict[str, str] = {}
        self.logger = logger
        self.raw_code = raw_code
        self.params = params

        raw_code_loc = os.path.join(config['tmp_dir'], 'raw.rs')
        self.raw_code_loc = raw_code_loc
        with open(raw_code_loc, 'w') as f:
            f.write(raw_code)

        self.add_trial(raw_code)

    def add_trial(self, code: str) -> None:
        """
        Add a result generate by LLM to the context.
        """
        trial_id = len(self.trials)
        path = os.path.join(config['tmp_dir'], f'trial_{trial_id}.rs')
        with open(path , 'w') as f: f.write(code)
        eval = VEval(code, self.logger)
        self.trials.append(Trial(trial_id, eval, path, self.logger))

    def get_trial(self, id: int): return self.trials[id]

    def register_modoule(self, name: str, module: BaseModule) -> None:
        """
        Register a module to the context.
        """
        self.modules[name] = module

    def get_knowledge(self, id: str):
        return self.knowledge.get(id, '')

    def add_knowledge(self, id: str, knowledge: str, 
                      append = False):
        """
        Add knowledge to the context.
        """
        if append:
            self.knowledge[id] += knowledge
        else:
            self.knowledge[id] = knowledge

    def gen_knowledge(self):
        """
        Generate the knowledge for the context.
        """
        knowledge = ''
        for name, desc in self.knowledge.items():
            knowledge += f"### {name}\n\n"
            knowledge += desc
            knowledge += '\n\n'
        return knowledge

    def gen_task_desc(self):
        """
        Generate the task description for the context.
        """
    
        if self.params.trial_fetch_mode == 'naive':
            # Naive mode: use the last trial
            trial = self.trials[-1]
            prevs = self.trials[-1-self.params.max_prev_trial:-1]
        else:
            # Other mode: TODO
            trial = None
            prevs = []

        rloc = self.raw_code_loc
        verus_code = trial.code
        rustc_out = trial.rustc_out
        knowledge = self.gen_knowledge()
        prev_descs = [f'### Failure {i}\n\n' + ptrail.desc(rloc)
                      for i, ptrail in enumerate(prevs)]
        
        return fill_template("task_desc", {
            'verus_code': verus_code,
            'rustc_out': rustc_out,
            'knowledge': knowledge,
            'failures': '\n\n'.join(prev_descs),
        })