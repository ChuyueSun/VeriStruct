# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

import random
import time
import warnings
from typing import List, Tuple, Union

# sglang
import sglang as sgl

try:
    from azure.identity import DefaultAzureCredential, get_bearer_token_provider
except ImportError:
    # If you only sometimes need azure.identity, it's OK to handle missing packages
    pass


class LLM:
    def __init__(self, config, logger):
        """
        Initialize the LLM with SGL backends. If multiple keys/backends exist,
        you could store them in self.backends and set one as the default.
        """
        self.config = config
        self.logger = logger

        # Prepare a list of potential SGL backends
        self.backends = []

        if getattr(config, "platform", "openai") == "openai":
            for i in range(len(config.aoai_api_key)):
                self.backends.append(
                    sgl.OpenAI(
                        model_name=config.aoai_generation_model,
                        api_key=config.aoai_api_key[i],
                        base_url=config.aoai_api_base[i],
                    )
                )
        elif getattr(config, "platform", "openai") == "azure":
            for i in range(len(config.aoai_api_key)):
                self.backends.append(
                    sgl.OpenAI(
                        model_name=config.aoai_generation_model,
                        api_version=config.aoai_api_version,
                        azure_endpoint=config.aoai_api_base[i],
                        api_key=config.aoai_api_key[i],
                        is_azure=True,
                    )
                )
        elif getattr(config, "platform", "openai") == "anthropic":
            for i in range(len(config.anthropic_api_key)):
                self.backends.append(
                    sgl.Anthropic(
                        model_name=config.anthropic_generation_model,
                        api_key=config.anthropic_api_key[i],
                    )
                )
        else:
            raise ValueError("Unknown platform")

        # Pick a random backend index
        self.client_id = 0

    @sgl.function
    def _build_prompt(
        s, system_info, instruction, exemplars, query, answer_num, max_tokens=8192
    ):
        """
        Internal sgl.function to build the conversation flow for a single infer_llm call.
        """
        # system message
        s += sgl.system(system_info or "You are a helpful AI assistant.")

        # instruction, if provided
        if instruction is not None:
            s += sgl.user(instruction)
        # s += sgl.assistant("OK, I'm ready to help.")

        # exemplars
        if exemplars:
            for ex in exemplars:
                # ex["query"], ex["answer"]
                s += sgl.user(ex["query"])
                s += sgl.assistant(ex["answer"])
        if query:
            # final query from user
            s += sgl.user(query)

        # We'll store the final response in a variable named "final_answer".
        # Now using max_completion_tokens=8192 by default.
        forks = s.fork(answer_num)
        forks += sgl.assistant(
            sgl.gen(
                f"final_answer",
                max_tokens=max_tokens,
            )
        )
        forks.join()

    def infer_llm(
        self,
        engine: str,
        instruction: str,
        exemplars: list,
        query: str,
        system_info: str = None,
        answer_num: int = 3,
        max_tokens: int = 8192,
        temp: float = 0.7,
        json: bool = False,
        return_msg: bool = False,
        verbose: bool = False,
    ) -> Union[List[str], Tuple[List[str], List[dict]]]:
        """
        Calls SGL to build and run an LLM prompt. Returns a list of strings
        (the final answers). If return_msg=True, returns a tuple of
        (list of strings, conversation messages).

        :param engine: Model or engine name (currently unused in the snippet).
        :param instruction: User instruction or system directive.
        :param exemplars: Example (prompt, answer) pairs for few-shot prompting.
        :param query: The user query or code snippet to process.
        :param system_info: Additional system instructions or context.
        :param answer_num: Number of answers to generate.
        :param max_tokens: Token limit for each completion.
        :param temp: Sampling temperature for the LLM.
        :param json: Whether to parse the output as JSON (not fully shown).
        :param return_msg: If True, also return the entire conversation messages.
        :param verbose: If True, log debug info.

        :return: Either a list of answer strings, or (list of answers, list of messages).
        """

        if verbose:
            self.logger.info(f"Using backend #{self.client_id}")

        # Select the backend
        sgl.set_default_backend(self.backends[self.client_id])

        start_time = time.time()
        try:
            # Build the prompt using the SGL function `_build_prompt`
            state = self._build_prompt.run(
                system_info=system_info,
                instruction=instruction,
                exemplars=exemplars,
                query=query,
                answer_num=answer_num,
                max_tokens=max_tokens,
            )
        except Exception as e:
            self.logger.error(f"SGL error: {e}")
            if return_msg:
                return [], []
            else:
                return []

        # Extract the final answer(s)
        final_answers = state["final_answer"]
        infer_time = time.time() - start_time
        self.logger.info(f"Infer time: {infer_time:.2f}s")
        # Ensure final_answers is always a list of strings
        if isinstance(final_answers, str):
            final_answers = [final_answers]
        elif not isinstance(final_answers, list):
            # In case it's neither a string nor a list, coerce it to a list
            final_answers = [str(final_answers)]
        else:
            # If it's already a list, ensure each item is a string
            final_answers = [
                ans if isinstance(ans, str) else str(ans) for ans in final_answers
            ]

        if return_msg:
            # state.messages() presumably returns a list of conversation messages
            return final_answers, state.messages()
        else:
            return final_answers
