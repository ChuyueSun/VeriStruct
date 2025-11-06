# Parallel Benchmark Run Guide

## 🚀 Quick Start

The parallel run has been launched! Here's how to monitor and analyze it.

---

## 📊 Monitoring Tools

### 1. **Quick Status Check**
```bash
./check_benchmark_status.sh
```
Shows:
- Whether run is active
- Number of processes
- Latest output
- Log files created
- Output directories

### 2. **Live Monitoring**
```bash
# Monitor overall progress
tail -f run_all_benchmarks.out

# Monitor specific benchmark
tail -f logs/bitmap_2_todo_*.log
tail -f logs/bst_map_todo_*.log
```

### 3. **Results Analysis** (when complete)
```bash
python3 analyze_results.py
```
Shows:
- Success/failure summary
- Verification scores
- Detailed results table

---

## 📁 File Locations

| File/Directory | Description |
|---------------|-------------|
| `run_all_benchmarks.out` | Main output from parallel runner |
| `logs/*.log` | Individual benchmark logs |
| `output/<benchmark>/azure_*/` | Detailed results per benchmark |
| `output/<benchmark>/azure_*/best/` | Best results for each benchmark |
| `benchmark_summary_*.txt` | Final summary (created when complete) |

---

## 🎯 What's Running

**13 Benchmarks in Parallel:**

| # | Benchmark | View Pattern | Expected Modules |
|---|-----------|--------------|------------------|
| 1 | `atomics_todo` | ❌ No View | inv → spec → proof |
| 2 | `bitmap_2_todo` | ✅ spec fn | view → spec → proof |
| 3 | `bitmap_todo` | ✅ spec fn | view → spec → proof |
| 4 | `bst_map_todo` | ✅ View trait + TODO | view → inv → spec → proof |
| 5 | `invariants_todo` | ❌ No View | spec only |
| 6 | `node_todo` | ❌ No View | inv → spec → proof |
| 7 | `option_todo` | ❌ No View | spec only |
| 8 | `rb_type_invariant_todo` | ✅ Empty View trait | view → refine → inv → spec → proof |
| 9 | `rwlock_vstd_todo` | ❌ No View | spec only |
| 10 | `set_from_vec_todo` | ✅ closed spec fn | view → spec → proof |
| 11 | `transfer_todo` | ❌ No View | spec → proof |
| 12 | `treemap_todo` | ✅ View trait + TODO | view → inv → spec → proof |
| 13 | `vectors_todo` | ❌ No View | spec → proof |

**View Coverage:**
- ✅ **6 benchmarks** use View inference (all patterns covered!)
- ❌ **7 benchmarks** don't need View (correct!)

---

## ⏱️ Timing

- **Started:** 2025-11-05 13:31:42
- **Parallel workers:** 12
- **Expected duration:** 1-2 hours
- **Timeout per benchmark:** 2 hours

---

## 🔍 Key Tests

This run validates:

### 1. **View Inference Improvements** ✅
- spec fn view (bitmap_2_todo, bitmap_todo, set_from_vec_todo)
- View trait with TODO (bst_map_todo, treemap_todo)
- Empty View trait (rb_type_invariant_todo)

### 2. **No False Positives** ✅
- Benchmarks without View should skip view_inference
- No unnecessary module runs

### 3. **Surgical Insertion** ✅
- No spec keyword deletion
- No nested impl blocks
- Correct code structure preservation

---

## 📈 Checking Progress

### While Running:
```bash
# Check status
./check_benchmark_status.sh

# See which benchmarks started
ls output/

# Count completed (approximate)
ls output/*/best/ 2>/dev/null | wc -l
```

### When Complete:
```bash
# Full analysis
python3 analyze_results.py

# Check final summary
cat benchmark_summary_*.txt

# View specific result
cat output/bitmap_2_todo/azure_*/best/best.rs
```

---

## 🎯 Success Criteria

A benchmark is considered **successful** if:
- ✅ Verified > 0
- ✅ Errors = 0
- ✅ Verus Errors = 0
- ✅ Compilation Error = False

Expected success rate: **60-80%** (some benchmarks are inherently difficult)

---

## 🛑 Stopping the Run

If needed:
```bash
# Find main process
ps aux | grep run_all_benchmarks.py | grep -v grep

# Kill it (replace PID)
kill <PID>

# Or force kill all
pkill -f run_all_benchmarks.py
```

---

## 💡 Tips

1. **Don't panic if some fail** - Some benchmarks are challenging
2. **Check individual logs** for detailed error messages
3. **View inference benchmarks** (6 of them) are the most important for this test
4. **Compare with previous runs** in output/ directory

---

## 🎁 After Completion

The run will automatically create:
1. `benchmark_summary_YYYYMMDD_HHMMSS.txt` - Overall results
2. Individual result files in `output/<benchmark>/azure_*/`
3. Best results in `output/<benchmark>/azure_*/best/`

Check these for:
- Verification success/failure
- Code quality
- Error patterns
- View inference correctness

---

## 📞 Help

Run stuck? Check:
```bash
# Is it actually running?
ps aux | grep run_all_benchmarks

# Any errors in main output?
tail -100 run_all_benchmarks.out

# Any disk space issues?
df -h

# Any memory issues?
free -h
```

Good luck! 🍀
