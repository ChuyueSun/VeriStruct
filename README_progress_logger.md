# VerusAgent Progress Logger

The Progress Logger is a utility component added to VerusAgent that tracks and records detailed information about the execution process, helping you analyze performance, debug issues, and understand the repair process.

## Features

- Records timing information for each step and repair operation
- Tracks VEval results after each step
- Logs repair information for each repair round
- Produces both JSON logs and a human-readable summary
- Maintains detailed statistics about the execution

## Output Files

The Progress Logger generates the following files in the `output/progress_logs` directory:

- `progress.json`: A detailed JSON log of the entire execution process
- `summary.txt`: A human-readable summary of the execution with key statistics

## JSON Format

The `progress.json` file contains the following structure:

```json
{
  "start_time": "2025-04-24T16:54:05.776",
  "steps": [
    {
      "name": "view_inference",
      "number": 1,
      "start_time": "2025-04-24T16:54:05.776",
      "result": {
        "compilation_error": false,
        "verified": 4,
        "errors": 5,
        "verus_errors": 9,
        "is_correct": false,
        "code_length": 7832
      },
      "execution_time": 189.19,
      "end_time": "2025-04-24T16:57:15.646"
    },
    // ... other steps
  ],
  "repair_rounds": [
    {
      "round_number": 1,
      "start_time": "2025-04-24T16:57:17.841",
      "repairs": [
        {
          "error_type": "ConstructorFailTypeInvariant",
          "repair_module": "repair_type",
          "before_score": {
            "compilation_error": false,
            "verified": 6,
            "errors": 3,
            "verus_errors": 3
          },
          "after_score": {
            "compilation_error": false,
            "verified": 7,
            "errors": 2,
            "verus_errors": 2
          },
          "success": true,
          "execution_time": 66.03
        },
        // ... other repairs
      ],
      "end_time": "2025-04-24T17:00:32.184",
      "execution_time": 194.34
    },
    // ... other repair rounds
  ],
  "final_result": {
    "compilation_error": false,
    "verified": 9,
    "errors": 0,
    "verus_errors": 0,
    "is_correct": true
  },
  "end_time": "2025-04-24T17:06:35.048",
  "total_execution_time": 749.27
}
```

## Summary Format

The `summary.txt` file provides a human-readable overview of the execution:

```
# VerusAgent Execution Summary

Start time: 2025-04-24T16:54:05.776
End time: 2025-04-24T17:06:35.048
Total execution time: 749.27s

## Final Result

Verified: 9
Errors: 0
Verus Errors: 0
Compilation Error: false
Is Correct: true

## Statistics

Total steps: 4
Total repair rounds: 2
Total repairs attempted: 3
Successful repairs: 3
Average step time: 117.26s
Average repair time: 82.63s

## Steps

Step 1: view_inference
  Time: 189.19s
  Score: Verified=4, Errors=5, Verus Errors=9

Step 2: view_refinement
  Time: 127.52s
  Score: Verified=4, Errors=5, Verus Errors=9

Step 3: inv_inference
  Time: 62.33s
  Score: Verified=6, Errors=3, Verus Errors=7

Step 4: spec_inference
  Time: 108.20s
  Score: Verified=6, Errors=3, Verus Errors=3

## Repair Rounds

Round 1
  Time: 194.34s
  repair_type for ConstructorFailTypeInvariant
    Before: Verified=6, Errors=3, Verus Errors=3
    After: Verified=7, Errors=2, Verus Errors=2
    Time: 66.03s
  repair_postcond for PostCondFail
    Before: Verified=7, Errors=2, Verus Errors=2
    After: Verified=8, Errors=1, Verus Errors=1
    Time: 92.00s

Round 2
  Time: 90.61s
  repair_postcond for PostCondFail
    Before: Verified=8, Errors=1, Verus Errors=1
    After: Verified=9, Errors=0, Verus Errors=0
    Time: 89.86s
```

## Usage

The Progress Logger is automatically initialized and used in the VerusAgent main flow. You don't need to do anything special to enable it - it will automatically track all steps and repairs.

### Viewing Results

After running VerusAgent, check the `output/progress_logs` directory to view:

1. `progress.json` - For detailed analysis and programmatic processing
2. `summary.txt` - For a quick overview of the execution

### Advanced Usage

For custom analysis, you can parse the JSON log file to extract specific information:

```python
import json
from pathlib import Path

# Load the progress log
with open('output/progress_logs/progress.json', 'r') as f:
    progress_data = json.load(f)

# Get total execution time
total_time = progress_data['total_execution_time']
print(f"Total execution time: {total_time:.2f}s")

# Find the longest step
longest_step = max(progress_data['steps'], key=lambda x: x['execution_time'])
print(f"Longest step: {longest_step['name']} - {longest_step['execution_time']:.2f}s")

# Count successful repairs
successful_repairs = 0
for round in progress_data['repair_rounds']:
    for repair in round['repairs']:
        if repair['success']:
            successful_repairs += 1
print(f"Successful repairs: {successful_repairs}")
```

## Implementation

The Progress Logger has been integrated into the VerusAgent main flow by adding:

1. A new module: `src/modules/progress_logger.py` with the `ProgressLogger` class
2. Integration points in `main.py` to track steps and repair rounds
3. Integration in `repair_registry.py` to track individual repairs

The changes ensure minimal impact on existing code while providing comprehensive tracking.
