#!/usr/bin/env python
"""
Simple test script to verify the LLM caching functionality in the run.sh environment.
This script simulates a basic workflow and checks cache functionality.
"""

import os
import sys
import time
import json
import logging
from pathlib import Path

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("run_cache_test")

# Make sure the current directory is in the path
sys.path.insert(0, os.path.abspath("."))

try:
    # Load the Azure configuration
    from src.configs.sconfig import config, reset_config
    
    # Reset to Azure configuration
    reset_config('config-azure')
    
    # Import cache and LLM
    from src.llm_cache import LLMCache
    from src.infer import LLM
    
    logger.info(f"Successfully loaded configuration with platform: {config.get('platform')}")
except ImportError as e:
    logger.error(f"Import error: {e}")
    sys.exit(1)

def verify_cache_in_run_context():
    """Simulate a basic workflow and verify cache functionality."""
    # Get environment variables (should be set by run.sh)
    python_path = os.environ.get('PYTHONPATH', 'Not set')
    enable_llm = os.environ.get('ENABLE_LLM_INFERENCE', 'Not set')
    cache_enabled = os.environ.get('LLM_CACHE_ENABLED', 'Not set')
    cache_dir = os.environ.get('LLM_CACHE_DIR', 'Not set')
    
    # Print environment configuration
    print(f"\nEnvironment Configuration:")
    print(f"- PYTHONPATH: {python_path}")
    print(f"- ENABLE_LLM_INFERENCE: {enable_llm}")
    print(f"- LLM_CACHE_ENABLED: {cache_enabled}")
    print(f"- LLM_CACHE_DIR: {cache_dir}")
    
    # Configure LLM with caching
    config['use_cache'] = cache_enabled == '1'
    config['cache_dir'] = cache_dir
    config['cache_max_age_days'] = int(os.environ.get('LLM_CACHE_MAX_AGE_DAYS', '7'))
    
    # Create cache directory if it doesn't exist
    os.makedirs(cache_dir, exist_ok=True)
    
    # Initialize LLM
    llm = LLM(config, logger)
    
    # Print LLM configuration
    print(f"\nLLM Configuration:")
    print(f"- Platform: {config.get('platform')}")
    print(f"- Model: {config.get('aoai_generation_model')}")
    print(f"- Cache enabled: {llm.cache.enabled}")
    print(f"- Cache directory: {llm.cache.cache_dir}")
    
    # Define a fixed query for testing
    engine = config.get('aoai_generation_model', 'gpt-4')
    instruction = "Summarize the following Verus code:"
    query = "function max(a: int, b: int) -> int\n    ensures result >= a && result >= b\n    ensures result == a || result == b\n{\n    if a >= b {\n        return a;\n    } else {\n        return b;\n    }\n}"
    system_info = "You are a helpful AI assistant specializing in formal verification with Verus."
    
    # First call (cache miss expected)
    print(f"\nFirst call (cache miss expected)...")
    start_time = time.time()
    responses = llm.infer_llm(
        engine=engine,
        instruction=instruction,
        exemplars=[],
        query=query,
        system_info=system_info,
        answer_num=1,
    )
    elapsed = time.time() - start_time
    
    print(f"Response time: {elapsed:.4f}s")
    print(f"Response length: {len(responses[0])} characters")
    print(f"Cache stats: {json.dumps(llm.cache.get_stats(), indent=2)}")
    
    # Wait a moment to ensure the cache file is written
    time.sleep(0.5)
    
    # Second call (cache hit expected)
    print(f"\nSecond call (cache hit expected)...")
    start_time = time.time()
    responses2 = llm.infer_llm(
        engine=engine,
        instruction=instruction,
        exemplars=[],
        query=query,
        system_info=system_info,
        answer_num=1,
    )
    elapsed = time.time() - start_time
    
    print(f"Response time: {elapsed:.4f}s")
    print(f"Response length: {len(responses2[0])} characters")
    print(f"Cache stats: {json.dumps(llm.cache.get_stats(), indent=2)}")
    
    # Check if the responses are identical
    identical = responses[0] == responses2[0]
    print(f"\nResponses are identical: {identical}")
    
    # Check if the second response was significantly faster (indicating a cache hit)
    cache_hit_confirmed = elapsed < 0.1  # Cache hits typically take a few milliseconds
    print(f"Cache hit confirmed by timing: {cache_hit_confirmed}")
    
    # Show summary of cache files
    cache_path = Path(llm.cache.cache_dir)
    cache_files = list(cache_path.glob("*.json"))
    print(f"\nFound {len(cache_files)} cache files")
    
    return identical and cache_hit_confirmed

if __name__ == "__main__":
    print("\n===== Testing LLM Caching in run.sh Environment =====")
    
    # Verify cache functionality
    success = verify_cache_in_run_context()
    
    # Print result
    if success:
        print("\n✅ Cache test passed! Caching is working correctly in the run.sh environment.")
    else:
        print("\n❌ Cache test failed! Check the logs for details.")
        sys.exit(1) 