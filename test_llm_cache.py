import json
import logging
import time

from src.llm_cache import LLMCache

# Set up logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("cache_test")

# Initialize cache
cache = LLMCache(enabled=True, logger=logger)


def test_cache():
    """Test basic cache functionality"""
    print("\n1. Testing basic cache functionality:")

    # Test parameters
    engine = "gpt-4"
    instruction = "Translate the following to French"
    query = "Hello world"
    max_tokens = 100
    responses = ["Bonjour le monde"]

    # First call - should be a cache miss
    cached_response = cache.get(engine, instruction, query, max_tokens)
    print(f"Cache miss expected: {'Miss' if cached_response is None else 'Hit'}")

    # Save to cache
    cache.save(engine, instruction, query, max_tokens, responses)
    print(f"Saved response to cache")

    # Second call - should be a cache hit
    cached_response = cache.get(engine, instruction, query, max_tokens)
    print(f"Cache hit expected: {'Hit' if cached_response is not None else 'Miss'}")
    print(f"Cached response: {cached_response}")

    # Check cache stats
    stats = cache.get_stats()
    print(f"Cache stats: {json.dumps(stats, indent=2)}")


def test_cache_expiry():
    """Test cache expiry functionality with a very short timeout"""
    print("\n2. Testing cache expiry (with 2-second timeout):")

    # Create a new cache with very short expiry for testing
    test_cache = LLMCache(
        enabled=True, max_age_days=0.00002, logger=logger
    )  # ~2 seconds

    # Test parameters
    engine = "gpt-4"
    instruction = "What is the capital of France?"
    query = "Capital of France"
    max_tokens = 100
    responses = ["The capital of France is Paris."]

    # Save to cache
    test_cache.save(engine, instruction, query, max_tokens, responses)
    print(f"Saved response to test cache")

    # Should be a cache hit immediately
    cached_response = test_cache.get(engine, instruction, query, max_tokens)
    print(
        f"Immediate retrieval - expected hit: {'Hit' if cached_response is not None else 'Miss'}"
    )

    # Wait for expiry
    print("Waiting 3 seconds for cache entry to expire...")
    time.sleep(3)

    # Should be a cache miss after waiting
    cached_response = test_cache.get(engine, instruction, query, max_tokens)
    print(
        f"After waiting - expected miss: {'Miss' if cached_response is None else 'Hit'}"
    )


def test_cache_clearing():
    """Test cache clearing functionality"""
    print("\n3. Testing cache clearing:")

    # Create test cache
    test_cache = LLMCache(enabled=True, logger=logger)

    # Add multiple entries
    examples = [
        {
            "engine": "gpt-4",
            "instruction": "Translate to Spanish",
            "query": "Hello",
            "response": ["Hola"],
        },
        {
            "engine": "gpt-4",
            "instruction": "Translate to German",
            "query": "Hello",
            "response": ["Hallo"],
        },
        {
            "engine": "gpt-4",
            "instruction": "Translate to Italian",
            "query": "Hello",
            "response": ["Ciao"],
        },
    ]

    for ex in examples:
        test_cache.save(
            ex["engine"], ex["instruction"], ex["query"], 100, ex["response"]
        )

    print(f"Added {len(examples)} entries to cache")

    # Clear cache with 0 max age (clears all)
    cleared = test_cache.clear(max_age_days=0)
    print(f"Cleared {cleared} entries from cache")


# Run the tests
if __name__ == "__main__":
    print("Starting LLM Cache tests...")
    test_cache()
    test_cache_expiry()
    test_cache_clearing()
    print("\nAll tests completed!")
