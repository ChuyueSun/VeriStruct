from typing import List, Dict
from modules.base import BaseModule
from modules.veval import VEval

class Trial:
    def __init__(self, trial_id: int, eval: VEval, logger):
        self.id = trial_id
        self.eval = eval
        self.eval.eval(100, True, None)
        self.logger = logger

    @property
    def code(self):
        return self.eval.code
    
    @property
    def rustc_out(self):
        return self.eval.rustc_out

class Context:
    """
    Context class to store the trials and modules.
    """
    def __init__(self, logger):
        self.trials: List[Trial] = []
        self.modules: Dict[str, BaseModule] = {}
        self.knowledge: Dict[str, str] = {}
        self.logger = logger

    def add_trial(self, code: str) -> None:
        """
        Add a result generate by LLM to the context.
        """
        trial_id = len(self.evals) + 1
        eval = VEval(code, self.logger)
        self.trials.append(Trial(trial_id, eval, self.logger))

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
    
    

    