#!/usr/bin/env python
"""
Test the LLM cache functionality using the actual Azure configuration.
"""

import os
import sys
import time
import json
import logging
import argparse
from pathlib import Path

# Parse command line arguments
parser = argparse.ArgumentParser(description="Test LLM cache with Azure config")
parser.add_argument("--fixed-query", action="store_true", 
                    help="Use a fixed query to test cache hits with repeated runs")
args = parser.parse_args()

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("cache_with_real_config")

# Make sure src is in the path
sys.path.insert(0, os.path.abspath("."))

# First load the Azure configuration
try:
    from src.configs.sconfig import config, reset_config
    
    # Reset to Azure configuration
    reset_config('config-azure')
    logger.info(f"Loaded Azure configuration with platform: {config.get('platform')}")
    
    # Import cache and LLM after config is loaded
    from src.llm_cache import LLMCache
    from src.infer import LLM
    
    logger.info("Successfully imported all required modules")
except ImportError as e:
    logger.error(f"Import error: {e}")
    sys.exit(1)

def test_cache_with_actual_config():
    """Test the cache using the actual Azure configuration."""
    
    # Get cache settings from environment or use defaults
    cache_dir = os.environ.get("LLM_CACHE_DIR", "llm_cache")
    cache_enabled = os.environ.get("LLM_CACHE_ENABLED", "1") == "1"
    cache_max_age = int(os.environ.get("LLM_CACHE_MAX_AGE_DAYS", "7"))
    
    # Add cache configuration to the config
    config["use_cache"] = cache_enabled
    config["cache_dir"] = cache_dir
    config["cache_max_age_days"] = cache_max_age
    
    logger.info(f"Cache configuration: enabled={cache_enabled}, dir={cache_dir}, max_age={cache_max_age}")
    
    # Initialize the LLM with the Azure configuration
    logger.info("Initializing LLM with Azure configuration")
    llm = LLM(config, logger)
    
    # Print cache and LLM status
    print(f"\nLLM and Cache Configuration:")
    print(f"- LLM platform: {config.get('platform')}")
    print(f"- LLM model: {config.get('aoai_generation_model')}")
    print(f"- API base: {config.get('aoai_api_base')}")
    print(f"- Cache enabled: {llm.cache.enabled}")
    print(f"- Cache directory: {llm.cache.cache_dir}")
    print(f"- Cache max age: {llm.cache.max_age_seconds / (24*60*60):.1f} days")
    print(f"- Using dummy mode: {llm.dummy_mode}")
    
    # Skip the actual API calls if in dummy mode
    if llm.dummy_mode:
        logger.warning("LLM is in dummy mode. Skipping actual API calls.")
        print("\nLLM is in dummy mode. This test will only verify the caching mechanism.")
    
    # Test parameters
    engine = config.get("aoai_generation_model", "gpt-4")
    instruction = "Translate this sentence to French:"
    
    # Use fixed query if specified, otherwise generate a unique one
    if args.fixed_query:
        query = "Hello world! This is a test with a fixed query."
        print(f"\nUsing fixed query for cache hit testing: '{query}'")
    else:
        query = f"Hello world! This is a test at {time.time()}"
        print(f"\nUsing unique query: '{query}'")
        
    system_info = "You are a helpful AI assistant that translates text accurately."
    
    # First call - should be a cache miss unless using fixed query that was previously cached
    print("\n1. First inference call:")
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
    
    print(f"Response (in {elapsed:.4f}s): {responses[0]}")
    print(f"Cache stats: {json.dumps(llm.cache.get_stats(), indent=2)}")
    
    # Wait a moment to ensure the cache file is written
    time.sleep(0.5)
    
    # Second call with the same parameters - should be a cache hit
    print("\n2. Second inference call (cache hit expected):")
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
    
    print(f"Response (in {elapsed:.4f}s): {responses[0]}")
    print(f"Cache stats: {json.dumps(llm.cache.get_stats(), indent=2)}")
    print(f"Hit rate: {llm.cache.get_stats()['hit_rate']:.2%}")
    
    # Check cache directory
    cache_path = Path(llm.cache.cache_dir)
    cache_files = list(cache_path.glob("*.json"))
    print(f"\nFound {len(cache_files)} cache files:")
    
    for file in cache_files[:5]:  # Limit to 5 files
        try:
            data = json.loads(file.read_text())
            query_text = data.get("query_params", {}).get("query", "N/A")
            timestamp = data.get("timestamp", 0)
            time_str = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(timestamp))
            print(f"- {file.name}: {time_str} - {query_text[:50]}...")
        except Exception as e:
            print(f"- {file.name}: Error reading file ({e})")
    
    if len(cache_files) > 5:
        print(f"... and {len(cache_files) - 5} more files")
    
    return llm

if __name__ == "__main__":
    print("\n===== Testing LLM Cache with Azure Configuration =====")
    try:
        llm = test_cache_with_actual_config()
        print("\nTest completed successfully!")
    except Exception as e:
        logger.error(f"Test failed: {e}", exc_info=True)
        sys.exit(1) 