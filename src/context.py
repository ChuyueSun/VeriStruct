import os
import subprocess
import sys
import time
import warnings
from dataclasses import dataclass
from typing import Dict, List, Optional

from src.configs.sconfig import config
from src.doc.naive_reader import get_content
from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.veval import EvalScore, VEval
from src.prompts.template import fill_template


class Trial:
    def __init__(self, trial_id: int, eval: VEval, code_loc: Optional[str] = None, logger=None):
        self.id = trial_id
        self.eval = eval
        self.code_loc = code_loc
        self.logger = logger

        # Call eval with the correct parameters
        try:
            # The eval method signature has changed, update parameters accordingly
            self.eval.eval(max_errs=100, json_mode=True)
        except Exception as e:
            if self.logger:
                self.logger.error(f"Error evaluating trial: {e}")
                # Provide a short excerpt of rustc stderr to aid debugging
                try:
                    stderr = getattr(self.eval, "rustc_out", "") or ""
                    if stderr:
                        lines = stderr.splitlines()
                        excerpt = "\n".join(lines[:30])  # first 30 lines
                        self.logger.error("rustc stderr excerpt (first 30 lines):\n" + excerpt)
                except Exception as _:
                    # Best‑effort logging; ignore secondary failures
                    pass

    @property
    def code(self):
        return self.eval.code

    @property
    def rustc_out(self):
        return self.eval.rustc_out

    def desc(self, diff_loc: Optional[str] = None, output_rustc_out=True):
        ans = ""
        if (diff_loc is None) or (self.code_loc is None):
            ans += "#### Verus Code\n\n"
            ans += "```verus\n"
            ans += self.code
            ans += "```\n\n"
            if output_rustc_out:
                ans += "#### Compilation Result\n\n"
                ans += self.rustc_out
        else:
            ans += "#### Verus Code (Modified Part)\n\n"
            result = subprocess.run(
                ["git", "diff", self.code_loc, diff_loc],
                text=True,
                capture_output=True,
                check=True,
            )
            ans += result.stdout + "\n\n"
            if output_rustc_out:
                ans += "#### Compilation Result\n\n"
                ans += self.rustc_out

        return ans


@dataclass
class HyperParams:
    trial_fetch_mode: str = "naive"
    max_prev_trial: int = 4


class Context:
    """
    Context class to store the trials and modules.
    """

    def __init__(self, raw_code: str, params: HyperParams, logger, progress_logger=None):
        self.trials: List[Trial] = []
        self.modules: Dict[str, BaseModule] = {}
        self.knowledge: Dict[str, str] = {}
        self.logger = logger
        self.raw_code = raw_code
        self.params = params
        self.llm = LLM(config, logger)
        self.progress_logger = progress_logger

        # Global best tracking
        self.best_code = None
        self.best_score = None

        # Use a default tmp directory if not specified in config
        tmp_dir = config.get("tmp_dir", "tmp")
        raw_code_loc = os.path.join(tmp_dir, "raw.rs")
        self.raw_code_loc = raw_code_loc

        # Ensure tmp directory exists
        os.makedirs(tmp_dir, exist_ok=True)

        with open(raw_code_loc, "w") as f:
            f.write(raw_code)

        self.add_trial(raw_code)

        # Process use statements and add knowledge
        self.logger.info("=" * 60)
        self.logger.info("CONTEXT INITIALIZATION - PROCESSING KNOWLEDGE")
        self.logger.info("=" * 60)

        knowledge_added = False
        for line in raw_code.split("\n"):
            if line.startswith("use"):
                if line.strip() == "use vstd::prelude::*;":
                    continue
                lib_name = line.split(" ")[1].strip()
                self.logger.info(f"Found use statement: {line.strip()}")
                self.logger.debug(f"Extracting library name: {lib_name}")
                content = get_content(lib_name)
                if len(content) > 0:
                    self.add_knowledge(lib_name, content, append=False)
                    self.logger.info(
                        f"✓ Added knowledge for '{lib_name}' ({len(content)} characters)"
                    )
                    knowledge_added = True
                else:
                    self.logger.info(f"✗ No content found for '{lib_name}'")

        # Following is for logging only
        if knowledge_added:
            self.logger.info("\n" + "=" * 60)
            self.logger.info("FINAL KNOWLEDGE SUMMARY")
            self.logger.info("=" * 60)
            total_knowledge = self.gen_knowledge()
            self.logger.info(f"Total knowledge entries: {len(self.knowledge)}")
            self.logger.info(f"Total knowledge length: {len(total_knowledge)} characters")
            self.logger.debug("\nFormatted knowledge preview:")
            self.logger.debug("-" * 40)
            # Print first 500 characters of the formatted knowledge
            preview = total_knowledge[:500]
            self.logger.debug(preview)
            self.logger.debug(str(self.knowledge.keys()))
            if len(total_knowledge) > 500:
                self.logger.debug(
                    f"... (truncated, showing first 500 of {len(total_knowledge)} characters)"
                )
            self.logger.debug("-" * 40)
        else:
            self.logger.info("No knowledge was added during initialization.")

        self.logger.info("=" * 60)

    def add_trial(self, code: str) -> None:
        """
        Add a result generate by LLM to the context.
        """
        trial_id = len(self.trials)
        # Use the same tmp directory as in __init__
        tmp_dir = config.get("tmp_dir", "tmp")
        path = os.path.join(tmp_dir, f"trial_{trial_id}.rs")
        with open(path, "w") as f:
            f.write(code)
        eval = VEval(code, self.logger)
        self.trials.append(Trial(trial_id, eval, path, self.logger))

    def get_trial(self, id: int):
        return self.trials[id]

    def get_best_code(self) -> Optional[str]:
        """Get the global best code tracked by this context."""
        return self.best_code

    def get_best_score(self) -> Optional[EvalScore]:
        """Get the global best score tracked by this context."""
        return self.best_score

    def set_best_code(self, code: str) -> None:
        """Set the global best code."""
        self.best_code = code

    def set_best_score(self, score: EvalScore) -> None:
        """Set the global best score."""
        self.best_score = score

    def register_module(self, name: str, module: BaseModule) -> None:
        """Register a module with the context."""
        self.modules[name] = module

    def register_modoule(self, name: str, module: BaseModule) -> None:
        """Deprecated. Use :meth:`register_module` instead."""
        warnings.warn(
            "register_modoule is deprecated; use register_module instead",
            DeprecationWarning,
            stacklevel=2,
        )
        self.register_module(name, module)

    def get_knowledge(self, id: str):
        return self.knowledge.get(id, "")

    def add_knowledge(self, id: str, knowledge: str, append=False):
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
        knowledge = "\n\n# relevant vstd lib knowledge\n\n"
        for name, desc in self.knowledge.items():
            knowledge += f"## {name}\n\n"
            knowledge += desc
            knowledge += "\n\n"
        return knowledge

    def gen_task_desc(self):
        """
        Generate the task description for the context.
        """

        if self.params.trial_fetch_mode == "naive":
            # Naive mode: use the last trial
            trial = self.trials[-1]
            prevs = self.trials[-1 - self.params.max_prev_trial : -1]
        else:
            raise NotImplementedError(
                f"Unsupported trial_fetch_mode: {self.params.trial_fetch_mode!r}"
            )

        rloc = self.raw_code_loc
        verus_code = trial.code
        rustc_out = trial.rustc_out
        knowledge = self.gen_knowledge()
        prev_descs = [f"### Failure {i}\n\n" + ptrail.desc(rloc) for i, ptrail in enumerate(prevs)]

        return fill_template(
            "task_desc",
            {
                "verus_code": verus_code,
                "rustc_out": rustc_out,
                "knowledge": knowledge,
                "failures": "\n\n".join(prev_descs),
            },
        )

    def infer_llm_with_tracking(
        self,
        engine: str,
        instruction: str,
        exemplars: list,
        query: str,
        system_info: str = None,
        answer_num: int = 5,
        max_tokens: int = 8192,
        temp: float = 0.7,
        json: bool = False,
        return_msg: bool = False,
        verbose: bool = False,
        use_cache: bool = True,
        stage: str = None,
        module: str = None,
    ):
        """
        Wrapper around LLM.infer_llm that tracks statistics.

        Args:
            Same as LLM.infer_llm, plus:
            stage: Stage name for tracking
            module: Module name for tracking

        Returns:
            Same as LLM.infer_llm
        """
        start_time = time.time()

        # Call the actual LLM
        result = self.llm.infer_llm(
            engine=engine,
            instruction=instruction,
            exemplars=exemplars,
            query=query,
            system_info=system_info,
            answer_num=answer_num,
            max_tokens=max_tokens,
            temp=temp,
            json=json,
            return_msg=return_msg,
            verbose=verbose,
            use_cache=use_cache,
            return_usage_meta=True,
        )

        # Track the call
        response_time = time.time() - start_time

        # Determine if it was a cache hit (approximate)
        cache_hit = response_time < 0.1  # If very fast, likely from cache

        if self.progress_logger:
            # Unpack usage metadata depending on return type
            input_tokens = None
            output_tokens = None
            try:
                if return_msg:
                    # Could be (answers, messages, usage)
                    if isinstance(result, tuple) and len(result) == 3:
                        _, _, usage = result
                        input_tokens = (
                            usage.get("input_tokens") if isinstance(usage, dict) else None
                        )
                        output_tokens = (
                            usage.get("output_tokens") if isinstance(usage, dict) else None
                        )
                else:
                    # Could be (answers, usage)
                    if (
                        isinstance(result, tuple)
                        and len(result) == 2
                        and isinstance(result[1], dict)
                    ):
                        usage = result[1]
                        input_tokens = usage.get("input_tokens")
                        output_tokens = usage.get("output_tokens")
            except Exception:
                input_tokens = None
                output_tokens = None
            self.progress_logger.record_llm_call(
                stage=stage,
                module=module,
                response_time=response_time,
                cache_hit=cache_hit,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
            )

        return result
