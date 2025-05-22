# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

import logging
import os
import random
import time
import warnings
from typing import List, Tuple, Union
import json as json_lib
from pathlib import Path

# Import our new cache
from src.llm_cache import LLMCache

# sglang
try:
    import sglang as sgl
except ImportError:
    # If you don't have sglang installed, create a dummy implementation
    class DummySGL:
        def function(self, func):
            return func

        def set_default_backend(self, backend):
            pass

        def system(self, text):
            return text

        def user(self, text):
            return text

        def assistant(self, text):
            return text

        def gen(self, name, **kwargs):
            return name

    sgl = DummySGL()
    print("Warning: sglang not installed, using dummy implementation")

try:
    from azure.identity import DefaultAzureCredential, get_bearer_token_provider
except ImportError:
    # If you only sometimes need azure.identity, it's OK to handle missing packages
    pass

# Add a flag to enable/disable LLM inference (for testing without API keys)
ENABLE_LLM_INFERENCE = os.environ.get("ENABLE_LLM_INFERENCE", "1") == "1"


class LLM:
    def __init__(self, config, logger, use_cache=True):
        """
        Initialize the LLM with SGL backends. If multiple keys/backends exist,
        you could store them in self.backends and set one as the default.
        """
        self.config = config
        self.logger = logger
        self.dummy_mode = False

        # Check if LLM inference is enabled via environment variable
        if os.environ.get("ENABLE_LLM_INFERENCE", "1") == "0" or self.config.get(
            "dummy_mode", False
        ):
            self.dummy_mode = True
            self.logger.warning("LLM in dummy mode. Will return placeholder responses.")
            return

        # Check environment variable to determine if caching is enabled
        enable_cache_env = os.environ.get("ENABLE_LLM_CACHE", "1")

        # Check for deprecated environment variable
        deprecated_cache_env = os.environ.get("LLM_CACHE_ENABLED")
        if deprecated_cache_env is not None:
            self.logger.warning(
                "LLM_CACHE_ENABLED is deprecated. Please use ENABLE_LLM_CACHE instead."
            )
            # Still honor the deprecated variable if it's set to disable caching
            if deprecated_cache_env == "0":
                self.logger.warning(
                    "Disabling cache due to deprecated LLM_CACHE_ENABLED=0 setting"
                )
                enable_cache_env = "0"

        # Cache is enabled if passed parameter is True and environment variable is "1"
        use_cache = use_cache and enable_cache_env == "1"

        cache_dir = self.config.get("cache_dir", "llm_cache")
        cache_max_age = self.config.get("cache_max_age_days", 7)

        # Get always_write option from config, default to True to always write to cache
        always_write = self.config.get("always_write_cache", True)

        self.logger.info(
            f"Cache status: {'enabled' if use_cache else 'disabled'} for reading (from env: ENABLE_LLM_CACHE={enable_cache_env})"
        )
        if always_write and not use_cache:
            self.logger.info("Cache writing enabled even though reading is disabled")

        self.cache = LLMCache(
            cache_dir=cache_dir,
            enabled=use_cache,
            max_age_days=cache_max_age,
            always_write=always_write,
            logger=self.logger,
        )

        # Log config for debugging
        self.logger.info(f"Config: {self.config.get('aoai_api_base')}")

        # Prepare a list of potential SGL backends
        self.backends = []

        if self.dummy_mode:
            self.logger.warning("LLM in dummy mode. Will return placeholder responses.")
            return

        try:
            if self.config.get("platform", "openai") == "openai":
                for i in range(len(self.config["aoai_api_key"])):
                    self.backends.append(
                        sgl.OpenAI(
                            model_name=self.config["aoai_generation_model"],
                            api_key=self.config["aoai_api_key"][i],
                            base_url=self.config["aoai_api_base"][i],
                        )
                    )
            elif self.config.get("platform", "openai") == "azure":
                for i in range(len(self.config["aoai_api_key"])):
                    self.backends.append(
                        sgl.OpenAI(
                            model_name=self.config["aoai_generation_model"],
                            api_version=self.config["aoai_api_version"],
                            azure_endpoint=self.config["aoai_api_base"][i],
                            api_key=self.config["aoai_api_key"][i],
                            is_azure=True,
                        )
                    )
            elif self.config.get("platform", "openai") == "anthropic":
                for i in range(len(self.config["anthropic_api_key"])):
                    self.backends.append(
                        sgl.Anthropic(
                            model_name=self.config["anthropic_generation_model"],
                            api_key=self.config["anthropic_api_key"][i],
                        )
                    )
            else:
                raise ValueError("Unknown platform")
        except Exception as e:
            self.logger.error(f"Error initializing LLM backends: {e}")
            self.dummy_mode = True
            self.logger.warning(
                "Falling back to dummy mode due to initialization error."
            )

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
        s += sgl.assistant(
            sgl.gen(
                f"final_answer",
                max_tokens=max_tokens,
                n=answer_num,
            )
        )

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
        use_cache: bool = True,
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
        :param use_cache: Whether to use cache for this specific call.

        :return: Either a list of answer strings, or (list of answers, list of messages).
        """

        if self.dummy_mode:
            self.logger.warning("LLM in dummy mode. Returning placeholder responses.")
            if query and len(query) > 100:
                dummy_response = (
                    "// This is a placeholder response from dummy mode.\n" + query
                )
            else:
                dummy_response = "This is a placeholder response from dummy mode."

            if return_msg:
                return [dummy_response] * answer_num, []
            else:
                return [dummy_response] * answer_num

        # Check cache if enabled
        if use_cache and self.cache.enabled:
            # Double-check environment variable in case it changed after the call started
            if os.environ.get("ENABLE_LLM_CACHE", "1") == "0":
                self.logger.debug(
                    "Cache disabled by environment variable for this call"
                )
            else:
                cached_responses = self.cache.get(
                    engine, instruction, query, max_tokens, exemplars, system_info
                )

                if cached_responses:
                    self.logger.info(
                        f"Using cached response (hit rate: {self.cache.get_stats()['hit_rate']:.2f})"
                    )

                    # Return the requested number of responses (up to what's available)
                    available_responses = min(len(cached_responses), answer_num)
                    result = cached_responses[:available_responses]

                    # If we don't have enough cached responses, add duplicates to meet the requested number
                    if available_responses < answer_num:
                        result.extend([result[0]] * (answer_num - available_responses))

                    if return_msg:
                        # Create a dummy message list when using cache
                        dummy_messages = [
                            {
                                "role": "system",
                                "content": system_info or "You are a helpful assistant",
                            },
                            {"role": "user", "content": query},
                            {"role": "assistant", "content": result[0]},
                        ]
                        return result, dummy_messages
                    else:
                        return result

        if verbose:
            self.logger.info(f"Using backend #{self.client_id}")

        # Select the backend
        try:
            sgl.set_default_backend(self.backends[self.client_id])
        except Exception as e:
            self.logger.error(f"Error setting backend: {e}")
            if return_msg:
                return [], []
            else:
                return []

        # Get the cache key (MD5) that will be used
        cache_key = self.cache._get_cache_key(
            engine, instruction, query, max_tokens, exemplars, system_info
        )
        
        # Save prompts in the same directory as the cache responses
        prompt_dir = Path(self.cache.cache_dir)
        
        # Create the prompt file path using the same MD5 but with a .md extension
        prompt_file = prompt_dir / f"{cache_key}.md"
        
        # Format the prompt components
        prompt_content = "# Prompt\n\n"
        if system_info:
            prompt_content += f"## System\n{system_info}\n\n"
        if instruction:
            prompt_content += f"## Instruction\n{instruction}\n\n"
        if exemplars:
            exemplar_content = "## Exemplars\n\n"
            for i, exemplar in enumerate(exemplars):
                exemplar_content += f"### Example {i+1}\n\n"
                exemplar_content += f"## Query\n{exemplar['query']}\n\n"
                exemplar_content += f"## Answer\n{exemplar['answer']}\n\n"
            prompt_content += exemplar_content
        if query:
            prompt_content += f"## Query\n{query}\n\n"
        
        # Save the prompt
        try:
            prompt_file.write_text(prompt_content)
            self.logger.debug(f"Saved prompt to {prompt_file}")
        except Exception as e:
            self.logger.warning(f"Failed to save prompt: {e}")

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

        # Cache the result if caching is enabled or always_write is enabled
        cache_saving_enabled = (
            use_cache and self.cache.enabled
        ) or self.cache.always_write
        if cache_saving_enabled:
            # Double-check environment variable in case it changed during the call
            if (
                os.environ.get("ENABLE_LLM_CACHE", "1") == "0"
                and not self.cache.always_write
            ):
                self.logger.debug(
                    "Cache save skipped - disabled by environment variable"
                )
            else:
                self.cache.save(
                    engine,
                    instruction,
                    query,
                    max_tokens,
                    final_answers,
                    exemplars,
                    system_info,
                )
                if self.cache.enabled:
                    self.logger.debug(
                        f"Saved response to cache (time: {infer_time:.2f}s)"
                    )
                else:
                    self.logger.debug(
                        f"Saved response to cache in write-only mode (time: {infer_time:.2f}s)"
                    )

        if return_msg:
            # state.messages() presumably returns a list of conversation messages
            return final_answers, state.messages()
        else:
            return final_answers
