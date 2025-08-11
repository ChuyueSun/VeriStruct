"""
A simple caching mechanism for LLM API calls to avoid redundant API calls.
"""

import hashlib
import json
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union


class LLMCache:
    """
    A class to cache LLM API call results to disk.
    """

    def __init__(
        self,
        cache_dir: str = "llm_cache",
        enabled: bool = True,
        max_age_days: int = 7,
        always_write: bool = False,
        logger=None,
    ):
        """
        Initialize the cache.

        Args:
            cache_dir: Directory to store cache files
            enabled: Whether caching is enabled for reading
            max_age_days: Maximum age of cache entries in days
            always_write: Whether to always write to cache even if reading is disabled
            logger: Optional logger for cache operations
        """
        self.cache_dir = Path(cache_dir)
        self.always_write = always_write

        # Check environment variable to determine if caching is enabled
        enable_cache_env = os.environ.get("ENABLE_LLM_CACHE", "1")

        # Check for deprecated environment variable
        deprecated_cache_env = os.environ.get("LLM_CACHE_ENABLED")
        if deprecated_cache_env is not None:
            if logger:
                logger.warning(
                    "LLM_CACHE_ENABLED is deprecated. Please use ENABLE_LLM_CACHE instead."
                )
            # Still honor the deprecated variable if it's set to disable caching
            if deprecated_cache_env == "0":
                if logger:
                    logger.warning(
                        "Disabling cache due to deprecated LLM_CACHE_ENABLED=0 setting"
                    )
                enable_cache_env = "0"

        # Cache is enabled if passed parameter is True and environment variable is "1"
        self.enabled = enabled and enable_cache_env == "1"

        # Log the cache status
        if logger:
            if self.enabled:
                logger.info(
                    f"LLM cache enabled for reading and writing (from env: ENABLE_LLM_CACHE={enable_cache_env})"
                )
            elif self.always_write:
                logger.info(
                    f"LLM cache disabled for reading but enabled for writing (from env: ENABLE_LLM_CACHE={enable_cache_env})"
                )
            else:
                logger.info(
                    f"LLM cache disabled (from env: ENABLE_LLM_CACHE={enable_cache_env})"
                )

        self.max_age_seconds = max_age_days * 24 * 60 * 60
        self.logger = logger

        # Create cache directory if needed for writing
        if self.enabled or self.always_write:
            self.cache_dir.mkdir(exist_ok=True, parents=True)

        # Cache hit statistics
        self.hits = 0
        self.misses = 0

    def _get_cache_key(
        self,
        engine: str,
        instruction: str,
        query: str,
        max_tokens: int,
        exemplars: Optional[List] = None,
        system_info: Optional[str] = None,
    ) -> str:
        """Generate a unique cache key based on the query parameters."""
        # Create a dictionary of the parameters to hash
        params = {
            "engine": engine,
            "instruction": instruction,
            "query": query,
            "max_tokens": max_tokens,
            "exemplars": exemplars or [],
            "system_info": system_info or "",
        }

        # Convert to a string and hash
        param_str = json.dumps(params, sort_keys=True)
        return hashlib.md5(param_str.encode()).hexdigest()

    def _get_cache_file(self, cache_key: str) -> Path:
        """Get the cache file path for a given key."""
        return self.cache_dir / f"{cache_key}.json"

    def get(
        self,
        engine: str,
        instruction: str,
        query: str,
        max_tokens: int,
        exemplars: Optional[List] = None,
        system_info: Optional[str] = None,
    ) -> Optional[List[str]]:
        """
        Retrieve a cached result if it exists and is not too old.

        Returns:
            List of response strings if cache hit, None if cache miss
        """
        # Double-check environment variables in case they changed after initialization
        if os.environ.get("ENABLE_LLM_CACHE", "1") == "0":
            if self.logger:
                self.logger.warning("Cache miss: Cache disabled by environment variable")
            self.misses += 1
            return None

        if not self.enabled:
            if self.logger:
                self.logger.warning("Cache miss: Cache is not enabled")
            self.misses += 1
            return None

        cache_key = self._get_cache_key(
            engine, instruction, query, max_tokens, exemplars, system_info
        )
        cache_file = self._get_cache_file(cache_key)

        if not cache_file.exists():
            if self.logger:
                self.logger.warning(f"Cache miss: File not found for key {cache_key}")
                self.logger.debug(f"Cache miss details - Query: {query[:100]}...")
            self.misses += 1
            return None

        try:
            cache_data = json.loads(cache_file.read_text())

            # Check if cache is too old
            timestamp = cache_data.get("timestamp", 0)
            current_time = time.time()
            age_hours = (current_time - timestamp) / 3600  # Convert to hours

            if current_time - timestamp > self.max_age_seconds:
                if self.logger:
                    self.logger.warning(f"Cache miss: Entry expired for key {cache_key}")
                    self.logger.debug(f"Cache entry age: {age_hours:.2f} hours (max age: {self.max_age_seconds/3600:.2f} hours)")
                self.misses += 1
                return None

            # Return the cached responses
            responses = cache_data.get("responses", [])

            if self.logger:
                self.logger.debug(f"Cache hit: {cache_key}")
            self.hits += 1
            return responses

        except (json.JSONDecodeError, KeyError) as e:
            if self.logger:
                self.logger.error(f"Cache miss: Failed to read cache file {cache_key}")
                self.logger.error(f"Cache read error details: {str(e)}")
                self.logger.debug(f"Cache file path: {cache_file}")
            self.misses += 1
            return None

    def save(
        self,
        engine: str,
        instruction: str,
        query: str,
        max_tokens: int,
        responses: List[str],
        exemplars: Optional[List] = None,
        system_info: Optional[str] = None,
    ) -> None:
        """Save a response to the cache."""
        # Double-check environment variables in case they changed after initialization
        if os.environ.get("ENABLE_LLM_CACHE", "1") == "0" and not self.always_write:
            if self.logger:
                self.logger.debug(
                    "Cache save skipped - disabled by environment variable"
                )
            return

        # Only skip saving if both enabled and always_write are False
        if not self.enabled and not self.always_write:
            return

        cache_key = self._get_cache_key(
            engine, instruction, query, max_tokens, exemplars, system_info
        )
        cache_file = self._get_cache_file(cache_key)

        try:
            # Create a cache entry with timestamp and responses
            cache_data = {
                "timestamp": time.time(),
                "engine": engine,
                "responses": responses,
                # Include the original query parameters for reference
                "query_params": {
                    "instruction": instruction,
                    "query": query,
                    "max_tokens": max_tokens,
                    "system_info": system_info or "",
                },
            }

            # Ensure the cache directory exists (might have been created after initialization)
            if not self.cache_dir.exists():
                self.cache_dir.mkdir(exist_ok=True, parents=True)

            with open(cache_file, "w") as f:
                json.dump(cache_data, f, indent=2)

            if self.logger:
                if self.enabled:
                    self.logger.debug(f"Saved to cache: {cache_key}")
                else:
                    self.logger.debug(f"Saved to cache (write-only mode): {cache_key}")

        except Exception as e:
            if self.logger:
                self.logger.warning(f"Cache write error: {e}")

    def clear(self, max_age_days: Optional[int] = None) -> int:
        """
        Clear old cache entries.

        Args:
            max_age_days: Override the instance max_age_days

        Returns:
            Number of entries cleared
        """
        if not self.enabled or not self.cache_dir.exists():
            return 0

        max_age = (
            max_age_days * 24 * 60 * 60
            if max_age_days is not None
            else self.max_age_seconds
        )
        current_time = time.time()
        cleared_count = 0

        for cache_file in self.cache_dir.glob("*.json"):
            try:
                cache_data = json.loads(cache_file.read_text())
                timestamp = cache_data.get("timestamp", 0)

                if current_time - timestamp > max_age:
                    cache_file.unlink()
                    cleared_count += 1

            except (json.JSONDecodeError, KeyError):
                # Invalid cache file, just delete it
                cache_file.unlink()
                cleared_count += 1

        if self.logger and cleared_count > 0:
            self.logger.info(f"Cleared {cleared_count} cache entries")

        return cleared_count

    def get_stats(self) -> Dict[str, int]:
        """Get cache hit statistics."""
        return {
            "hits": self.hits,
            "misses": self.misses,
            "total": self.hits + self.misses,
            "hit_rate": (
                self.hits / (self.hits + self.misses)
                if (self.hits + self.misses) > 0
                else 0
            ),
        }
