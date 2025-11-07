# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

import json as json_lib
import logging
import os
import random
import time
import warnings
from pathlib import Path
from typing import List, Tuple, Union

import requests

# Import our new cache
from src.llm_cache import LLMCache
from src.utils.path_utils import get_output_dir

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
        Initialize the LLM client with API configuration.
        Supports OpenAI, Azure OpenAI, and Anthropic platforms.
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
                self.logger.warning("Disabling cache due to deprecated LLM_CACHE_ENABLED=0 setting")
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
        platform_type_log = self.config.get("platform", "openai")
        if platform_type_log in ["openai", "xai", "azure"]:
            self.logger.info(f"Config base URLs: {self.config.get('aoai_api_base')}")
        else:
            self.logger.info("Config: using non-OpenAI platform; base URL list not applicable")

        # Log which platform we are going to initialize
        self.logger.info(f"LLM initializing for platform: {self.config.get('platform', 'openai')}")

        if self.dummy_mode:
            self.logger.warning("LLM in dummy mode. Will return placeholder responses.")

        # Pick a random backend index
        self.client_id = 0

    def _extract_responses_api_answers(self, response_json: dict, final_answers: List[str]):
        """Extract answers from OpenAI Responses API format."""
        out = response_json.get("output") or response_json.get("choices")
        if isinstance(out, list) and out:
            try:
                for item in out:
                    if isinstance(item, dict) and item.get("type") == "message":
                        content = item.get("content", [])
                        for c in content:
                            if isinstance(c, dict) and c.get("type") in (
                                "output_text",
                                "output_text.delta",
                                "text",
                            ):
                                txt = c.get("text")
                                if isinstance(txt, str):
                                    final_answers.append(txt)
                # Fallback to generic text field
                if not final_answers and isinstance(out[0], dict):
                    maybe = out[0].get("content") or out[0].get("text")
                    if isinstance(maybe, str):
                        final_answers.append(maybe)
            except Exception:
                pass
        # Try top-level text fields
        if not final_answers:
            maybe_txt = response_json.get("output_text") or response_json.get("text")
            if isinstance(maybe_txt, str):
                final_answers.append(maybe_txt)

    def infer_llm(
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
        return_usage_meta: bool = False,
    ) -> Union[
        List[str],
        Tuple[List[str], List[dict]],
        Tuple[List[str], dict],
        Tuple[List[str], List[dict], dict],
    ]:
        """
        Calls LLM API to generate responses. Returns a list of strings
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
                dummy_response = "// This is a placeholder response from dummy mode.\n" + query
            else:
                dummy_response = "This is a placeholder response from dummy mode."

            if return_msg and return_usage_meta:
                return (
                    [dummy_response] * answer_num,
                    [],
                    {"input_tokens": None, "output_tokens": None},
                )
            if return_msg:
                return [dummy_response] * answer_num, []
            if return_usage_meta:
                return [dummy_response] * answer_num, {
                    "input_tokens": None,
                    "output_tokens": None,
                }
            return [dummy_response] * answer_num

        # Check cache if enabled
        if use_cache and self.cache.enabled:
            # Double-check environment variable in case it changed after the call started
            if os.environ.get("ENABLE_LLM_CACHE", "1") == "0":
                self.logger.debug("Cache disabled by environment variable for this call")
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

                    if return_msg and return_usage_meta:
                        dummy_messages = [
                            {
                                "role": "system",
                                "content": system_info or "You are a helpful assistant",
                            },
                            {"role": "user", "content": query},
                            {"role": "assistant", "content": result[0]},
                        ]
                        return (
                            result,
                            dummy_messages,
                            {"input_tokens": None, "output_tokens": None},
                        )
                    if return_msg:
                        dummy_messages = [
                            {
                                "role": "system",
                                "content": system_info or "You are a helpful assistant",
                            },
                            {"role": "user", "content": query},
                            {"role": "assistant", "content": result[0]},
                        ]
                        return result, dummy_messages
                    if return_usage_meta:
                        return result, {"input_tokens": None, "output_tokens": None}
                    return result

        # Get the cache key (MD5) that will be used
        cache_key = self.cache._get_cache_key(
            engine, instruction, query, max_tokens, exemplars, system_info
        )

        # Save prompts in the output/prompts directory
        output_dir = get_output_dir()
        prompt_dir = output_dir / "prompts"
        prompt_dir.mkdir(parents=True, exist_ok=True)

        # Create a descriptive name for the prompt file
        timestamp = time.strftime("%Y%m%d_%H%M%S")

        # Extract module/task type from instruction
        module_type = "unknown"
        if instruction:
            # Check repair types first (more specific patterns)
            if "fix the syntax error" in instruction.lower():
                module_type = "syntax"
            elif "fix the type" in instruction.lower() or "mismatched type" in instruction.lower():
                module_type = "type"
            elif "fix the precondition not satisfied" in instruction.lower():
                module_type = "repair_precond"
            elif "fix the postcondition" in instruction.lower():
                module_type = "repair_postcond"
            elif (
                "fix the assertion" in instruction.lower()
                or "test assertion" in instruction.lower()
            ):
                module_type = "repair_assertion"
            elif "fix the" in instruction.lower() and "invariant" in instruction.lower():
                module_type = "repair_invariant"
            # Then check generation types (broader patterns)
            elif "add.*requires.*and.*ensures" in instruction.lower() or (
                "requires" in instruction.lower()
                and "ensures" in instruction.lower()
                and "add" in instruction.lower()
            ):
                module_type = "spec"
            elif "todo.*proof" in instruction.lower() or "add proof" in instruction.lower():
                module_type = "proof"
            elif "invariant" in instruction.lower() and "implement" in instruction.lower():
                module_type = "inv"
            elif "view" in instruction.lower() and (
                "generate" in instruction.lower() or "implement" in instruction.lower()
            ):
                module_type = "view"

        # Create the prompt file path with timestamp and module type
        prompt_file = prompt_dir / f"{module_type}_{timestamp}_{cache_key[:8]}.md"

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

        # Build chat messages (system/instruction/exemplars/query)
        messages = []
        if system_info:
            messages.append({"role": "system", "content": system_info})

        # Embed examples in instruction for cleaner message structure
        full_instruction = instruction if instruction else ""
        if exemplars:
            # Check if using answer-only format (query is just a title)
            is_answer_only = exemplars and all(
                ex.get("query", "").startswith("Example ") and len(ex.get("query", "")) < 100
                for ex in exemplars[:3]  # Check first 3
            )

            if is_answer_only:
                # Embed examples directly in instruction
                full_instruction += "\n\n## Examples of Completed Code\n\n"
                for i, ex in enumerate(exemplars):
                    title = ex.get("query", f"Example {i+1}")
                    code = ex.get("answer", "")
                    full_instruction += f"### {title}\n```rust\n{code}\n```\n\n"
                # Don't add to messages separately
            else:
                # Traditional format: user/assistant pairs
                if full_instruction:
                    messages.append({"role": "user", "content": full_instruction})
                    full_instruction = None  # Already added
                for ex in exemplars:
                    messages.append({"role": "user", "content": ex.get("query", "")})
                    messages.append({"role": "assistant", "content": ex.get("answer", "")})

        if full_instruction:
            messages.append({"role": "user", "content": full_instruction})
        if query:
            messages.append({"role": "user", "content": query})

        # Make API call to LLM provider
        start_time = time.time()
        response_json = None
        input_tokens = None
        output_tokens = None
        reasoning_tokens = None

        try:
            platform_type = self.config.get("platform", "openai")
            model = self.config.get("aoai_generation_model", "gpt-4o")
            is_reasoning = model.startswith("o1") or model.startswith("o3")

            # Reasoning models (o1/o3) require temperature=1.0
            if is_reasoning and temp != 1.0:
                self.logger.debug(
                    f"Adjusting temperature from {temp} to 1.0 for reasoning model {model}"
                )
                temp = 1.0

            # Set timeout based on model type
            # Reasoning models (o1/o3) need much longer timeout due to thinking time
            api_timeout = (
                600 if is_reasoning else 120
            )  # 10 minutes for reasoning, 2 minutes for others
            self.logger.debug(f"Using API timeout: {api_timeout}s for model {model}")

            # Build request
            headers = {"Content-Type": "application/json"}
            payload = {"messages": messages, "temperature": temp, "n": answer_num}

            if platform_type == "azure":
                # Azure OpenAI
                base = self.config["aoai_api_base"][0]
                api_version = self.config.get("aoai_api_version", "2024-12-01-preview")
                headers["api-key"] = self.config["aoai_api_key"][0]
                url = f"{base}openai/deployments/{model}/chat/completions?api-version={api_version}"
                # Use max_completion_tokens for reasoning models, max_tokens for others
                payload["max_completion_tokens" if is_reasoning else "max_tokens"] = max_tokens
            elif platform_type == "anthropic":
                # Anthropic Claude API
                anthropic_model = self.config.get("anthropic_generation_model", "claude-sonnet-4-5")
                anthropic_key = self.config.get("anthropic_api_key", [""])[0]
                headers = {
                    "x-api-key": anthropic_key,
                    "anthropic-version": "2023-06-01",
                    "content-type": "application/json",
                }
                url = "https://api.anthropic.com/v1/messages"
                # Anthropic uses different format
                payload = {
                    "model": anthropic_model,
                    "max_tokens": max_tokens,
                    "temperature": temp,
                    "messages": messages,
                }
            else:
                # Standard OpenAI/XAI
                key = self.config.get("aoai_api_key", [os.environ.get("OPENAI_API_KEY", "")])[0]
                if key:
                    headers["Authorization"] = f"Bearer {key}"
                if is_reasoning:
                    # OpenAI Responses API
                    url = "https://api.openai.com/v1/responses"
                    joined = "\n\n".join([f"{m['role']}: {m['content']}" for m in messages])
                    payload = {
                        "model": model,
                        "input": joined,
                        "temperature": temp,
                        "max_output_tokens": max_tokens,
                    }
                else:
                    # OpenAI Chat Completions API
                    url = "https://api.openai.com/v1/chat/completions"
                    payload["model"] = model
                    payload["max_tokens"] = max_tokens

            # Make request with appropriate timeout
            resp = requests.post(url, headers=headers, json=payload, timeout=api_timeout)
            resp.raise_for_status()
            response_json = resp.json()

            # Extract token usage
            usage = response_json.get("usage", {}) if isinstance(response_json, dict) else {}
            input_tokens = usage.get("input_tokens") or usage.get("prompt_tokens")
            output_tokens = usage.get("output_tokens") or usage.get("completion_tokens")

            # Extract reasoning tokens (for o1/o3 models)
            reasoning_tokens = usage.get("reasoning_tokens")
            if not reasoning_tokens and isinstance(usage, dict):
                # Check completion_tokens_details (Azure) or output_tokens_details (OpenAI)
                details = (
                    usage.get("completion_tokens_details")
                    or usage.get("output_tokens_details")
                    or {}
                )
                reasoning_tokens = (
                    details.get("reasoning_tokens") if isinstance(details, dict) else None
                )

            # Log token usage
            if input_tokens or output_tokens:
                log_msg = f"Token usage - Input: {input_tokens}, Output: {output_tokens}"
                if reasoning_tokens:
                    log_msg += f", Reasoning: {reasoning_tokens}"
                self.logger.debug(log_msg)

            # Extract answers from response
            final_answers: List[str] = []
            if platform_type == "anthropic":
                # Anthropic Claude API format
                content_blocks = response_json.get("content", [])
                for block in content_blocks:
                    if isinstance(block, dict) and block.get("type") == "text":
                        text = block.get("text", "")
                        if text:
                            final_answers.append(text)
                # Anthropic only returns one response, but we requested answer_num
                # Duplicate the response to match expected count
                if final_answers and answer_num > 1:
                    single_answer = final_answers[0]
                    final_answers = [single_answer] * min(answer_num, 1)
            elif is_reasoning and platform_type != "azure":
                # OpenAI Responses API format
                self._extract_responses_api_answers(response_json, final_answers)
            else:
                # Chat Completions API format (includes Azure o1)
                choices = response_json.get("choices", [])
                for ch in choices:
                    content = ch.get("message", {}).get("content")
                    if isinstance(content, str):
                        final_answers.append(content)

            # Ensure we have at least one answer
            if not final_answers:
                final_answers = [""]

            # Repeat answers if needed to satisfy answer_num
            while len(final_answers) < answer_num:
                final_answers.extend(final_answers[: answer_num - len(final_answers)])

            infer_time = time.time() - start_time
            self.logger.info(f"Infer time: {infer_time:.2f}s")

        except Exception as e:
            self.logger.error(f"Direct LLM call failed: {e}")
            if return_msg and return_usage_meta:
                return [], [], {"input_tokens": None, "output_tokens": None}
            if return_msg:
                return [], []
            if return_usage_meta:
                return [], {"input_tokens": None, "output_tokens": None}
            return []

        # Cache the result if caching is enabled or always_write is enabled
        cache_saving_enabled = (use_cache and self.cache.enabled) or self.cache.always_write
        if cache_saving_enabled:
            # Double-check environment variable in case it changed during the call
            if os.environ.get("ENABLE_LLM_CACHE", "1") == "0" and not self.cache.always_write:
                self.logger.debug("Cache save skipped - disabled by environment variable")
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
                    self.logger.debug(f"Saved response to cache (time: {infer_time:.2f}s)")
                else:
                    self.logger.debug(
                        f"Saved response to cache in write-only mode (time: {infer_time:.2f}s)"
                    )

        # Build usage metadata
        usage_meta = {"input_tokens": input_tokens, "output_tokens": output_tokens}

        # Add optional token fields if available
        if reasoning_tokens is not None:
            usage_meta["reasoning_tokens"] = reasoning_tokens

        try:
            usage = response_json.get("usage", {}) if isinstance(response_json, dict) else {}
            total_tokens = usage.get("total_tokens")
            if total_tokens is not None:
                usage_meta["total_tokens"] = total_tokens
        except Exception:
            pass

        # Log usage metadata
        if return_usage_meta and (input_tokens or output_tokens):
            self.logger.info(f"Usage metadata: {usage_meta}")

        # Build return value based on requested metadata
        if return_msg:
            returned_messages = messages + (
                [{"role": "assistant", "content": final_answers[0]}] if final_answers else []
            )
            if return_usage_meta:
                return final_answers, returned_messages, usage_meta
            return final_answers, returned_messages

        if return_usage_meta:
            return final_answers, usage_meta

        return final_answers
