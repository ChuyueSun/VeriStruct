# LLM Cache

The `LLMCache` module provides a lightweight disk-based cache for language model
queries. It avoids redundant API calls by storing responses keyed on the full
set of query parameters and recording basic hit/miss statistics.

## Key features

- **Configurable storage** – cache entries live in a dedicated directory
  (`llm_cache` by default) and are automatically created when caching is enabled.
- **Environment flags** – `ENABLE_LLM_CACHE` (default `"1"`) toggles caching
  globally; the deprecated `LLM_CACHE_ENABLED` is still honored when set to
  `"0"` for backwards compatibility.
- **Deterministic keys** – the cache key is an MD5 hash of the engine,
  instruction, query, max tokens, exemplar list and system information, ensuring
  distinct entries for different parameter sets.
- **Age-based eviction** – each entry carries a timestamp; lookups ignore files
  older than the configured maximum age (seven days by default), and `clear()`
  prunes expired files from the cache directory.

### Cache key generation

`LLMCache` forms a cache key by serializing the full set of query parameters and
hashing the result:

1. A dictionary is assembled with `engine`, `instruction`, `query`,
   `max_tokens`, the `exemplars` list (or `[]` if none), and `system_info`
   (or `""`).
2. `json.dumps(..., sort_keys=True)` converts this dictionary to a canonical
   string with keys sorted to guarantee deterministic ordering.
3. The MD5 digest of that string is computed and rendered as a 32-character
   hexadecimal value.

The resulting digest becomes both the in-memory identifier and the filename of
the cache entry (`<cache_dir>/<digest>.json`). Any change to the parameters or
their order yields a different key.

## Reading from the cache

`get()` computes a cache key from the incoming request and returns the stored
responses if a fresh entry exists. When the cache is disabled, missing, or
outdated, the method records a miss and explains the reason via the optional
logger.

## Writing responses

`save()` persists new responses to the cache when caching is enabled or when in
write-only mode (`always_write=True`). Each cache file contains the timestamp,
engine, response list and original query parameters, enabling later validation
and debugging.

## Statistics

The `get_stats()` method reports the counts of cache hits, misses and the overall
hit rate, helping monitor cache effectiveness over time.
