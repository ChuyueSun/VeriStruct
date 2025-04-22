"""
A simple caching mechanism for LLM API calls to avoid redundant API calls.
"""

import os
import json
import hashlib
import time
from pathlib import Path
from typing import Dict, List, Optional, Any, Union, Tuple

class LLMCache:
    """
    A class to cache LLM API call results to disk.
    """
    def __init__(self, cache_dir: str = "llm_cache", 
                 enabled: bool = True, 
                 max_age_days: int = 7,
                 logger=None):
        """
        Initialize the cache.
        
        Args:
            cache_dir: Directory to store cache files
            enabled: Whether caching is enabled
            max_age_days: Maximum age of cache entries in days
            logger: Optional logger for cache operations
        """
        self.cache_dir = Path(cache_dir)
        self.enabled = enabled
        self.max_age_seconds = max_age_days * 24 * 60 * 60
        self.logger = logger
        
        # Create cache directory if it doesn't exist
        if self.enabled:
            self.cache_dir.mkdir(exist_ok=True, parents=True)
            
        # Cache hit statistics
        self.hits = 0
        self.misses = 0
    
    def _get_cache_key(self, engine: str, instruction: str, 
                      query: str, max_tokens: int,
                      exemplars: Optional[List] = None,
                      system_info: Optional[str] = None) -> str:
        """Generate a unique cache key based on the query parameters."""
        # Create a dictionary of the parameters to hash
        params = {
            "engine": engine,
            "instruction": instruction,
            "query": query,
            "max_tokens": max_tokens,
            "exemplars": exemplars or [],
            "system_info": system_info or ""
        }
        
        # Convert to a string and hash
        param_str = json.dumps(params, sort_keys=True)
        return hashlib.md5(param_str.encode()).hexdigest()
    
    def _get_cache_file(self, cache_key: str) -> Path:
        """Get the cache file path for a given key."""
        return self.cache_dir / f"{cache_key}.json"
    
    def get(self, engine: str, instruction: str, query: str, 
            max_tokens: int, exemplars: Optional[List] = None,
            system_info: Optional[str] = None) -> Optional[List[str]]:
        """
        Retrieve a cached result if it exists and is not too old.
        
        Returns:
            List of response strings if cache hit, None if cache miss
        """
        if not self.enabled:
            self.misses += 1
            return None
            
        cache_key = self._get_cache_key(
            engine, instruction, query, max_tokens, exemplars, system_info
        )
        cache_file = self._get_cache_file(cache_key)
        
        if not cache_file.exists():
            if self.logger:
                self.logger.debug(f"Cache miss: {cache_key}")
            self.misses += 1
            return None
            
        try:
            cache_data = json.loads(cache_file.read_text())
            
            # Check if cache is too old
            timestamp = cache_data.get("timestamp", 0)
            current_time = time.time()
            
            if current_time - timestamp > self.max_age_seconds:
                if self.logger:
                    self.logger.debug(f"Cache expired: {cache_key}")
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
                self.logger.warning(f"Cache read error: {e}")
            self.misses += 1
            return None
    
    def save(self, engine: str, instruction: str, query: str, 
             max_tokens: int, responses: List[str],
             exemplars: Optional[List] = None,
             system_info: Optional[str] = None) -> None:
        """Save a response to the cache."""
        if not self.enabled:
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
                    "system_info": system_info or ""
                }
            }
            
            with open(cache_file, 'w') as f:
                json.dump(cache_data, f, indent=2)
                
            if self.logger:
                self.logger.debug(f"Saved to cache: {cache_key}")
                
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
            
        max_age = max_age_days * 24 * 60 * 60 if max_age_days is not None else self.max_age_seconds
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
            "hit_rate": self.hits / (self.hits + self.misses) if (self.hits + self.misses) > 0 else 0
        } 