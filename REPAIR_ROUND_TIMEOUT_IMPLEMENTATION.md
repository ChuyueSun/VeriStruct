# Repair Round Timeout Implementation

## Summary

Implemented a repair round timeout mechanism to prevent repair rounds from running indefinitely. This addresses the issue observed in `azure_20251105_133142` where Repair Round 3 took 822 seconds with zero completed repairs.

## Changes Made

### 1. Configuration (`src/configs/config-azure.json`)

Added new configuration parameter:

```json
"repair_round_timeout": 900
```

**Default:** 900 seconds (15 minutes)
**Purpose:** Maximum time allowed for a single repair round

### 2. Main Loop (`src/main.py`)

Modified the repair round loop to:

1. **Extract timeout from config:**
   ```python
   repair_round_timeout = config.get("repair_round_timeout", 900)
   ```

2. **Pass timeout to repair_all:**
   ```python
   repair_results = repair_registry.repair_all(
       context,
       failures,
       output_dir,
       progress_logger,
       round_timeout=repair_round_timeout,
       round_start_time=repair_round_start
   )
   ```

3. **Log timeout warnings:**
   ```python
   if repair_round_time > repair_round_timeout:
       logger.warning(
           f"⏱️ Repair round {current_round} exceeded timeout: "
           f"{repair_round_time:.2f}s / {repair_round_timeout:.2f}s"
       )
   ```

### 3. Repair Registry (`src/modules/repair_registry.py`)

Enhanced `repair_all()` method with timeout support:

1. **New Parameters:**
   - `round_timeout: Optional[float]` - Max time for the round
   - `round_start_time: Optional[float]` - When the round started

2. **Timeout Check Helper:**
   ```python
   def check_round_timeout():
       if round_timeout and round_start_time:
           elapsed = time.time() - round_start_time
           if elapsed > round_timeout:
               logger.warning(f"⏱️ Repair round timeout reached: {elapsed:.2f}s / {round_timeout:.2f}s")
               return True
       return False
   ```

3. **Strategic Timeout Checks:**
   - ✅ Before LLM-based syntax repair
   - ✅ After compilation error handling
   - ✅ Before processing each error type
   - ✅ After each repair completes

4. **Graceful Termination:**
   When timeout is detected, the method:
   - Logs an error with 🚨 emoji
   - Returns immediately with partial results
   - Allows fallback logic to handle the incomplete round

## How It Works

```
Repair Round Start (t=0s)
    ↓
Compilation Error Handling
    ├─ Regex fixes (fast)
    ├─ [TIMEOUT CHECK]
    └─ LLM-based syntax repair
    ↓
[TIMEOUT CHECK]
    ↓
Process Each Error Type (prioritized)
    ├─ [TIMEOUT CHECK] ← Before each error type
    ├─ Attempt repair (with per-repair timeouts)
    ├─ [TIMEOUT CHECK] ← After each repair
    └─ Next error type...
    ↓
Return Results
```

## Example Behavior

### Without Timeout (Old Behavior)
```
Round 3: Starting...
  - Attempting syntax repair... (600s)
  - Attempting postcond repair... (180s)
  - Attempting syntax repair... (42s)
  - Total: 822s ✗ (No results)
```

### With Timeout (New Behavior)
```
Round 3: Starting...
  - Attempting syntax repair... (600s)
  - ⏱️ Repair round timeout reached: 905.23s / 900.00s
  - 🚨 Repair round timed out before processing PostCondFail
  - Total: 900s ✓ (Early termination)
  - Fallback to best checkpoint
```

## Testing

Created test suite in `tests/test_repair_round_timeout.py`:

- ✅ Test 1: Basic timeout check
- ✅ Test 2: Timeout in repair_all (integration)
- ✅ Test 3: No timeout when disabled
- ✅ Test 4: Partial results on timeout

All tests pass successfully.

## Impact on Existing Runs

### Before (Issue Case)
- **Round 3:** 822.12s, 0 repairs, compilation error persists
- Wasted 13+ minutes with no progress
- LLM calls timing out at 600+ seconds

### After (Expected Behavior)
- **Round 3:** Max 900s, early termination on timeout
- Clear logging: "🚨 Repair round timed out..."
- Graceful fallback to previous checkpoint
- Better resource utilization

## Configuration Guidelines

| Timeout Value | Use Case | Trade-off |
|--------------|----------|-----------|
| 300s (5 min) | Development/testing | Fast feedback, may miss some repairs |
| 600s (10 min) | Aggressive optimization | Balanced speed vs completeness |
| 900s (15 min) | **Default** - Production | Good balance for most cases |
| 1200s (20 min) | Complex benchmarks | More thorough, slower rounds |
| null/None | Debugging | No timeout, may hang indefinitely |

## Monitoring

Watch for these log indicators:

- ⏱️ = Timeout warning (approaching or exceeded)
- 🚨 = Critical timeout error (round terminated)
- ⏭️ = Skip action due to timeout

## Future Enhancements

1. **Adaptive Timeout:** Adjust based on error count
   ```python
   timeout = base_timeout + (num_errors * 60)  # 1 min per error
   ```

2. **Budget Allocation:** Distribute timeout across error types
   ```python
   per_error_budget = round_timeout / len(error_types)
   ```

3. **Predictive Timeout:** Use historical data
   ```python
   if avg_repair_time > (remaining_time / remaining_errors):
       skip_repair()
   ```

4. **Partial Checkpointing:** Save intermediate progress
   ```python
   if elapsed > checkpoint_interval:
       save_partial_checkpoint()
   ```

## Compatibility

- ✅ Backward compatible (timeout is optional)
- ✅ Existing configs work without changes
- ✅ No breaking changes to API
- ✅ Graceful degradation when timeout not specified

## Rollback

If issues arise, disable by setting:

```json
{
  "repair_round_timeout": null
}
```

Or remove the parameter entirely (defaults to None, effectively no timeout).
