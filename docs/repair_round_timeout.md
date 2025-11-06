# Repair Round Timeout Feature

## Overview

The repair round timeout feature prevents individual repair rounds from running indefinitely, addressing the issue where Round 3 in the bitmap_2_todo example took 822 seconds with no completed repairs.

## Problem Statement

Without timeout protection, repair rounds can get stuck in expensive LLM calls that:
- Take 10+ minutes per attempt
- Fail to produce usable results
- Waste computational resources and time
- Block progress in the verification pipeline

### Example from Real Logs

In `azure_20251105_133142` run:
- Round 1: 104s ✓ (1 successful repair)
- Round 2: 147s ✓ (2 attempted repairs)
- **Round 3: 822s ✗ (0 completed repairs - TIMEOUT ISSUE)**
- Round 4: 0.28s ✓ (fallback to checkpoint)
- Round 5: 0.20s ✓ (attempted repair)

Round 3 consumed 822 seconds (>13 minutes) with zero results.

## Solution

### Configuration

Added `repair_round_timeout` parameter to config files:

```json
{
  "repair_round_timeout": 900
}
```

**Default:** 900 seconds (15 minutes)

### Implementation

1. **Timeout Parameter Passing** (`src/main.py`):
   - Extract timeout from config
   - Pass to `repair_registry.repair_all()`
   - Log warnings when rounds exceed timeout

2. **Timeout Checks** (`src/modules/repair_registry.py`):
   - Added `round_timeout` and `round_start_time` parameters
   - Created `check_round_timeout()` helper function
   - Added timeout checks at strategic points:
     * Before LLM-based syntax repair
     * After compilation error handling
     * Before processing each error type
     * After each repair completes

3. **Graceful Termination**:
   - When timeout is detected, log error and return immediately
   - Return partial results if any repairs completed
   - Fallback logic in main.py handles incomplete rounds

## Usage

### Default Behavior

Timeout is automatically enabled with 900s (15 minutes) limit:

```python
# No changes needed - uses default from config
repair_results = repair_registry.repair_all(
    context, failures, output_dir, progress_logger,
    round_timeout=900,
    round_start_time=time.time()
)
```

### Custom Timeout

Override via configuration or environment:

```json
{
  "repair_round_timeout": 600  // 10 minutes
}
```

Or disable timeout entirely:

```json
{
  "repair_round_timeout": null  // No timeout
}
```

## Benefits

1. **Prevents Infinite Loops**: Rounds that would take 10+ minutes are terminated
2. **Resource Efficiency**: Avoids wasting time on unproductive repairs
3. **Better User Experience**: Provides predictable execution times
4. **Graceful Degradation**: Falls back to previous checkpoints when rounds timeout
5. **Detailed Logging**: Clear warnings when timeouts occur

## Logging Output

When a timeout occurs, you'll see:

```
⏱️ Repair round timeout reached: 905.23s / 900.00s
🚨 Repair round timed out before processing PostCondFail
⏱️ Repair round 3 exceeded timeout: 905.23s / 900.00s
```

## Monitoring

The timeout is tracked in:
- Console logs with emoji indicators (⏱️, 🚨)
- Progress logs (`progress_bitmap_2_todo_*.json`)
- Statistics reports showing round execution times

## Recommendations

- **Default (900s)**: Good for most cases
- **Aggressive (600s)**: For faster iteration, accept some incomplete rounds
- **Conservative (1200s)**: For complex repairs with many errors
- **Development (300s)**: Quick feedback during testing

## Future Improvements

1. Adaptive timeouts based on error count
2. Per-error-type timeout budgets
3. Early termination hints from LLM responses
4. Timeout prediction based on historical data
