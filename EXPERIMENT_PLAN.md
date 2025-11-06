# Comprehensive Experiment Plan for VerusAgent Workflow Testing

## Executive Summary

This document outlines a systematic experimental evaluation plan for the VerusAgent workflow, focusing on three key dimensions: **Robustness**, **Cost-Effectiveness**, and **Overall Effectiveness**. The plan includes quantitative metrics, diverse test scenarios, and statistical analysis methodologies.

---

## 1. Experimental Objectives

### Primary Research Questions
1. **Robustness**: How reliably does the workflow handle diverse code patterns and error scenarios?
2. **Cost**: What are the computational and financial costs (tokens, time, API calls)?
3. **Effectiveness**: How well does the generated code verify compared to baseline/manual approaches?

### Success Criteria
- **Robustness**: ≥80% success rate across diverse benchmarks
- **Cost**: Average cost per benchmark < $X (define threshold)
- **Effectiveness**: ≥70% verification success rate, reducing manual effort by ≥50%

---

## 2. Experimental Design

### 2.1 Test Corpus Design

#### A. Benchmark Categories (Stratified Sampling)

```
Category 1: Simple Data Structures (n=10)
- Single-field structs
- Basic array/vector operations
- Simple preconditions/postconditions
Example: simple_counter.rs, basic_queue.rs

Category 2: Complex Data Structures (n=10)
- Trees (BST, Red-Black trees)
- Hash maps
- Linked lists with invariants
Example: bst_map.rs, treemap.rs, bitmap_2.rs

Category 3: Algorithmic Patterns (n=10)
- Sorting algorithms
- Search algorithms
- Graph traversal
Example: binary_search.rs, quicksort.rs

Category 4: Concurrency & Atomics (n=5)
- Atomic operations
- Lock-based structures
- Concurrent data structures
Example: atomics.rs, rwlock.rs

Category 5: Edge Cases (n=10)
- Empty implementations (view functions with TODO)
- Large codebases (>1000 LOC)
- Deeply nested generics
- Option<Box<T>> patterns
Example: option_box_node.rs

Category 6: Error-Prone Patterns (n=5)
- Bit-manipulation (requires concrete specs)
- Modular arithmetic
- Unsafe/FFI boundaries
Example: bitmap with bit vectors

Total Benchmarks: 50
```

#### B. Controlled Variables
- **Fixed**: Verus version, LLM model (GPT-4/o1), timeout settings
- **Varied**: Code complexity, error types, pattern diversity

---

### 2.2 Metrics Definition

#### Robustness Metrics (R)

| Metric | Definition | Collection Method |
|--------|-----------|-------------------|
| **R1: Success Rate** | % of benchmarks that complete without fatal errors | Count successful runs / total runs |
| **R2: Module Completion** | % of workflow stages completed successfully | Track each module (view_inference, spec_inference, etc.) |
| **R3: Error Recovery Rate** | % of errors successfully repaired | (Errors fixed) / (Total errors encountered) |
| **R4: Stability Score** | Standard deviation of success across retries | Run each benchmark 3 times, measure variance |
| **R5: Safety Check Pass Rate** | % of LLM outputs passing safety checks | Safe responses / Total responses |
| **R6: Timeout Resilience** | % of runs completing within timeout budget | Successful completions within 30min threshold |

#### Cost Metrics (C)

| Metric | Definition | Collection Method |
|--------|-----------|-------------------|
| **C1: Total Tokens** | Sum of input + output tokens across all LLM calls | Parse usage tracking from context |
| **C2: API Call Count** | Number of LLM API calls per benchmark | Count infer_llm_with_tracking calls |
| **C3: Cache Hit Rate** | % of requests served from cache | Cache hits / Total requests |
| **C4: Time to Completion** | Wall-clock time per benchmark | Measure start to end time |
| **C5: Cost per Benchmark** | Estimated $ cost using pricing model | Tokens × pricing (GPT-4: $0.03/1K input, $0.06/1K output) |
| **C6: Retry Overhead** | Extra cost from retry attempts | (Total cost - First attempt cost) / Total cost |
| **C7: Module-wise Cost** | Token/time breakdown by module | Track separately for each stage |

#### Effectiveness Metrics (E)

| Metric | Definition | Collection Method |
|--------|-----------|-------------------|
| **E1: Verification Success** | % of benchmarks fully verified by Verus | Count benchmarks with 0 verification errors |
| **E2: Verification Progress** | Reduction in error count vs. initial TODO | (Initial errors - Final errors) / Initial errors |
| **E3: Code Quality Score** | Custom scoring: verified functions, coverage | VEval score analysis |
| **E4: Specification Correctness** | % of specs that are semantically correct | Manual review + Verus feedback |
| **E5: Proof Completeness** | % of required proofs successfully generated | Count TODO markers removed |
| **E6: Improvement over Baseline** | Comparison with baseline (no LLM) or human | Side-by-side comparison on subset |

---

## 3. Experimental Procedures

### 3.1 Baseline Establishment

**Baseline 1: No-LLM Baseline**
- Run Verus on TODO-marked code without VerusAgent
- Record initial error counts and types

**Baseline 2: Human Expert (Gold Standard)**
- Select 10 representative benchmarks
- Have expert manually add specifications
- Track time, LOC, final verification status

### 3.2 Experimental Runs

#### Phase 1: Standard Workflow Test (All 50 benchmarks)

```bash
# Configuration
- Model: GPT-4 (default), O1 (for complex cases)
- Cache: Enabled (default)
- Repair rounds: 5
- Timeout: 30 minutes per benchmark

# For each benchmark:
for benchmark in benchmarks/*.rs; do
  # Run with metrics collection
  python run_agent.py \
    --test-file $benchmark \
    --config config-azure \
    --repair-rounds 5 \
    --output-dir output/experiment_standard/ \
    --metrics-log metrics_standard.json
done
```

**Data Collection:**
- Progress logs (JSON) - track per-module timing and scores
- LLM usage tracking - tokens, API calls, cache hits
- VEval scores - compilation, verification status
- Error classifications - types and frequencies

#### Phase 2: Ablation Studies

**A. Module Ablation** (Test contribution of each module)
```python
# Test configurations:
configs = [
    "full_workflow",          # All modules
    "no_view_inference",      # Skip view inference
    "no_view_refinement",     # Skip view refinement
    "no_inv_inference",       # Skip invariant inference
    "no_repair",              # Skip repair modules
    "spec_only"               # Only spec_inference + proof_generation
]

# Run subset (n=20) on each config
```

**B. Repair Strategy Ablation**
```python
repair_strategies = [
    "no_repair",              # Baseline
    "syntax_only",            # Only syntax repairs
    "spec_errors_only",       # Only spec errors (priority 1)
    "all_except_proof",       # Skip proof errors (current skip list)
    "full_repair"             # Attempt all errors
]
```

**C. Example Selection Strategy**
```python
example_strategies = [
    "no_examples",            # No few-shot examples
    "random_3",               # Random 3 examples
    "scored_top5",            # Current scoring system
    "all_available"           # Max examples (up to 20)
]
```

#### Phase 3: Stress Testing

**A. Robustness Stress Tests**
1. **Empty Code Test**: Benchmarks with minimal TODO markers
2. **Large Code Test**: Benchmarks >1500 LOC
3. **Error Injection**: Deliberately introduce syntax errors to test repair
4. **Retry Sensitivity**: Vary max_retries (1, 3, 5, 10)
5. **Timeout Sensitivity**: Vary timeouts (10min, 30min, 60min)

**B. Cost Sensitivity Tests**
1. **Cache Disabled**: Measure cost without cache (worst case)
2. **Model Comparison**: GPT-4 vs O1 vs GPT-3.5-turbo
3. **Temperature Variation**: Test temp=0.7, 1.0, 1.3 on subset

#### Phase 4: Comparative Evaluation

**Compare against:**
1. **Copilot/GitHub Copilot** (if applicable): Manual specification with AI assistance
2. **Manual Human Effort**: Expert verification engineer
3. **Previous Version of VerusAgent** (if available): Track improvements

---

## 4. Data Collection Infrastructure

### 4.1 Automated Metrics Collection

**Extend existing logging:**
```python
# In run_agent.py or create experiment_runner.py

class ExperimentMetricsCollector:
    def __init__(self, experiment_name):
        self.experiment_name = experiment_name
        self.results = []

    def collect_run_metrics(self, benchmark_name, context, start_time, end_time):
        """Collect all metrics for a single run"""
        return {
            "benchmark": benchmark_name,
            "timestamp": datetime.now().isoformat(),

            # Robustness metrics
            "success": context.get_best_score().verified > 0,
            "modules_completed": self._count_completed_modules(context),
            "errors_encountered": len(context.trials[-1].eval.errors),
            "errors_repaired": self._count_repaired_errors(context),

            # Cost metrics
            "total_tokens": self._sum_tokens(context.llm_usage_log),
            "api_calls": len(context.llm_usage_log),
            "cache_hit_rate": self._calc_cache_hit_rate(context),
            "time_seconds": (end_time - start_time).total_seconds(),
            "estimated_cost_usd": self._calc_cost(context.llm_usage_log),

            # Effectiveness metrics
            "final_verified_count": context.get_best_score().verified,
            "final_error_count": context.get_best_score().errors,
            "veval_score": context.get_best_score(),
            "initial_error_count": context.trials[0].eval.errors,
            "improvement_rate": self._calc_improvement(context),

            # Per-module breakdown
            "module_breakdown": self._collect_module_metrics(context)
        }

    def save_results(self, output_path):
        """Save to JSON for analysis"""
        with open(output_path, 'w') as f:
            json.dump(self.results, f, indent=2)
```

### 4.2 Statistical Analysis Scripts

**Create analysis pipeline:**
```python
# experiments/analyze_results.py

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from scipy import stats

def load_experiment_data(metrics_file):
    """Load collected metrics"""
    with open(metrics_file) as f:
        return pd.DataFrame(json.load(f))

def analyze_robustness(df):
    """Statistical analysis of robustness"""
    return {
        "success_rate": df['success'].mean(),
        "success_rate_ci": stats.binom.interval(0.95, len(df), df['success'].mean()),
        "module_completion_avg": df['modules_completed'].mean(),
        "error_recovery_rate": (df['errors_repaired'] / df['errors_encountered']).mean(),
        "stability_score": df.groupby('benchmark')['success'].std().mean()
    }

def analyze_cost(df):
    """Cost analysis"""
    return {
        "avg_tokens": df['total_tokens'].mean(),
        "median_tokens": df['total_tokens'].median(),
        "avg_time_min": df['time_seconds'].mean() / 60,
        "avg_cost_usd": df['estimated_cost_usd'].mean(),
        "cache_hit_rate": df['cache_hit_rate'].mean(),
        "total_cost_usd": df['estimated_cost_usd'].sum()
    }

def analyze_effectiveness(df):
    """Effectiveness analysis"""
    return {
        "verification_success_rate": (df['final_error_count'] == 0).mean(),
        "avg_improvement": df['improvement_rate'].mean(),
        "median_errors_reduced": (df['initial_error_count'] - df['final_error_count']).median()
    }

def compare_configurations(df, group_by='config'):
    """Compare different experimental configurations"""
    grouped = df.groupby(group_by)
    comparison = grouped.agg({
        'success': 'mean',
        'total_tokens': ['mean', 'std'],
        'time_seconds': ['mean', 'std'],
        'final_error_count': ['mean', 'std'],
        'estimated_cost_usd': 'sum'
    })
    return comparison

def generate_report(df, output_dir):
    """Generate comprehensive report with visualizations"""
    # Success rate by category
    plt.figure(figsize=(10, 6))
    category_success = df.groupby('category')['success'].mean()
    category_success.plot(kind='bar')
    plt.title('Success Rate by Benchmark Category')
    plt.ylabel('Success Rate')
    plt.savefig(f'{output_dir}/success_by_category.png')

    # Cost distribution
    plt.figure(figsize=(10, 6))
    df['estimated_cost_usd'].hist(bins=30)
    plt.title('Cost Distribution per Benchmark')
    plt.xlabel('Cost (USD)')
    plt.ylabel('Frequency')
    plt.savefig(f'{output_dir}/cost_distribution.png')

    # Time vs Tokens scatter
    plt.figure(figsize=(10, 6))
    plt.scatter(df['total_tokens'], df['time_seconds'] / 60)
    plt.xlabel('Total Tokens')
    plt.ylabel('Time (minutes)')
    plt.title('Token Usage vs Execution Time')
    plt.savefig(f'{output_dir}/tokens_vs_time.png')

    # Module-wise contribution
    module_breakdown = pd.DataFrame([r['module_breakdown'] for r in df.to_dict('records')])
    module_tokens = module_breakdown.filter(like='_tokens').mean()
    plt.figure(figsize=(12, 6))
    module_tokens.plot(kind='bar')
    plt.title('Average Token Usage by Module')
    plt.ylabel('Tokens')
    plt.xticks(rotation=45, ha='right')
    plt.tight_layout()
    plt.savefig(f'{output_dir}/module_token_usage.png')
```

---

## 5. Execution Timeline

### Week 1: Preparation
- [ ] Organize and categorize benchmark corpus (50 benchmarks)
- [ ] Implement ExperimentMetricsCollector
- [ ] Set up automated test harness
- [ ] Run baseline measurements

### Week 2: Standard Workflow Testing
- [ ] Run Phase 1: All 50 benchmarks with standard config
- [ ] Collect metrics and preliminary analysis
- [ ] Identify any infrastructure issues

### Week 3: Ablation Studies
- [ ] Run Phase 2A: Module ablation (20 benchmarks × 6 configs = 120 runs)
- [ ] Run Phase 2B: Repair strategy ablation (20 benchmarks × 5 configs = 100 runs)
- [ ] Run Phase 2C: Example selection ablation (20 benchmarks × 4 configs = 80 runs)

### Week 4: Stress Testing & Comparative Evaluation
- [ ] Run Phase 3: Stress tests
- [ ] Run Phase 4: Comparative evaluation (manual baseline on 10 benchmarks)
- [ ] Compile all data

### Week 5: Analysis & Reporting
- [ ] Run statistical analysis scripts
- [ ] Generate visualizations and reports
- [ ] Write findings document
- [ ] Prepare presentation

---

## 6. Analysis Methodology

### 6.1 Statistical Tests

**Hypothesis Testing:**
```
H0: VerusAgent success rate ≤ 50% (baseline/random)
H1: VerusAgent success rate > 50%

Test: One-sample proportion test (z-test)
Significance level: α = 0.05
```

**Comparison Tests:**
```
- Mann-Whitney U test: Compare cost distributions between configs
- Kruskal-Wallis H test: Compare effectiveness across >2 groups
- Paired t-test: Compare before/after for same benchmarks
```

### 6.2 Qualitative Analysis

**Error Pattern Analysis:**
1. Extract and classify all VerusError types
2. Map errors to repair success/failure
3. Identify systematic weaknesses (e.g., "always fails on bit-vector proofs")

**Case Study Selection:**
- Best case: Fully successful verification
- Worst case: Complete failure
- Interesting case: Partial success with insights

**Code Quality Review:**
- Manual review of 20 generated specifications
- Check for semantic correctness (not just syntactic)
- Identify "hallucinations" or incorrect specs

---

## 7. Expected Outputs

### 7.1 Quantitative Report

**Template:**
```markdown
# VerusAgent Experimental Evaluation Results

## Summary Statistics

### Robustness
- Overall Success Rate: XX.X% (CI: [X.X%, X.X%])
- Module Completion Rate: XX.X%
- Error Recovery Rate: XX.X%
- Stability Score: X.XX

### Cost
- Average Total Tokens: XXX,XXX
- Average Time: XX.X minutes
- Average Cost: $X.XX per benchmark
- Cache Hit Rate: XX.X%
- Total Experiment Cost: $XXX.XX

### Effectiveness
- Verification Success Rate: XX.X%
- Average Error Reduction: XX.X%
- Compared to Manual Baseline: XXX% faster, XX% accuracy

## Detailed Analysis by Category
[Tables and charts]

## Ablation Study Results
[Comparison tables]

## Key Findings
1. ...
2. ...
```

### 7.2 Visualizations

1. **Dashboard-style summary** (single page with key metrics)
2. **Success rate heatmap** (categories × error types)
3. **Cost-effectiveness frontier** (Pareto chart: cost vs effectiveness)
4. **Module contribution analysis** (stacked bar: tokens per module)
5. **Error flow diagram** (Sankey: error types → repair → outcomes)

### 7.3 Recommendations Document

Based on findings, provide:
- Configuration recommendations (optimal repair rounds, examples, etc.)
- Benchmark categorization for triage (easy/medium/hard)
- Workflow improvements (e.g., "skip proof_generation for simple cases")
- Cost optimization strategies

---

## 8. Risk Mitigation

### Potential Issues & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| LLM API rate limits | High | Medium | Implement exponential backoff, use multiple API keys |
| Budget overrun | High | Medium | Set hard cost limit ($500?), stop if exceeded |
| Benchmark diversity insufficient | Medium | Low | Conduct pilot with 10 benchmarks first |
| Verus version changes | Medium | Low | Lock Verus version, document exact commit |
| Non-deterministic LLM outputs | Medium | High | Run 3 trials per config, use temperature=0 for determinism test |
| Time constraints | High | Medium | Parallelize runs, use preemptible instances |

---

## 9. Success Validation Criteria

### Tier 1: Minimum Viable Results
- [ ] Collected data from ≥40/50 benchmarks
- [ ] All metrics defined in Section 2.2 computed
- [ ] At least one ablation study completed

### Tier 2: Comprehensive Results
- [ ] All 50 benchmarks tested
- [ ] All ablation studies completed
- [ ] Statistical significance demonstrated for key findings
- [ ] Comparison with manual baseline

### Tier 3: Publication-Ready
- [ ] All of Tier 2
- [ ] Case studies documented
- [ ] Visualizations polished
- [ ] Reproducibility package prepared (scripts, data, configs)

---

## 10. Reproducibility Package

### Contents
```
experiments/
├── README.md                     # Reproduction instructions
├── configs/
│   ├── standard.yaml
│   ├── ablation_*.yaml
│   └── stress_test.yaml
├── benchmarks/
│   ├── categorized_list.json     # Benchmark metadata
│   └── [50 .rs files]
├── scripts/
│   ├── run_experiment.sh         # Master execution script
│   ├── collect_metrics.py
│   ├── analyze_results.py
│   └── generate_report.py
├── results/
│   ├── raw_metrics.json          # All collected data
│   ├── analysis_output/
│   └── visualizations/
└── docs/
    ├── EXPERIMENT_PLAN.md        # This document
    └── FINDINGS.md               # Results writeup
```

### Execution Instructions
```bash
# 1. Setup environment
pip install -r requirements.txt
export VERUS_PATH=/path/to/verus
export AZURE_OPENAI_KEY=your_key

# 2. Run experiments
cd experiments
./run_experiment.sh --phase all --config standard.yaml

# 3. Analyze results
python analyze_results.py --input results/raw_metrics.json --output results/analysis_output/

# 4. Generate report
python generate_report.py --data results/analysis_output/ --output results/FINDINGS.md
```

---

## Appendix A: Benchmark Selection Criteria

Each benchmark should:
1. Have clear TODO markers for specifications
2. Be representative of real-world Verus usage
3. Have known verification outcome (if from existing corpus)
4. Cover diverse Verus features (traits, generics, spec functions, etc.)
5. Range in complexity: 50-1500 LOC

## Appendix B: Example Metrics Log Schema

```json
{
  "experiment_id": "standard_run_20251105",
  "benchmark": "bitmap_2_todo.rs",
  "timestamp": "2025-11-05T16:35:51",
  "robustness": {
    "success": true,
    "modules_completed": 5,
    "errors_encountered": 8,
    "errors_repaired": 4,
    "safety_checks_passed": 12,
    "safety_checks_failed": 1
  },
  "cost": {
    "total_tokens": 125840,
    "input_tokens": 87230,
    "output_tokens": 38610,
    "api_calls": 18,
    "cache_hits": 5,
    "cache_misses": 13,
    "time_seconds": 423.7,
    "estimated_cost_usd": 4.85
  },
  "effectiveness": {
    "initial_errors": 8,
    "final_errors": 0,
    "verification_success": true,
    "verified_functions": 7,
    "improvement_rate": 1.0,
    "veval_score": {
      "compilation_error": false,
      "verified": 7,
      "errors": 0,
      "verus_errors": 0
    }
  },
  "module_breakdown": {
    "view_inference": {"tokens": 12400, "time": 45.2, "success": true},
    "spec_inference": {"tokens": 45200, "time": 185.3, "success": true},
    "proof_generation": {"tokens": 38100, "time": 142.1, "success": true},
    "repair_precond": {"tokens": 15200, "time": 28.4, "success": true},
    "repair_invariant": {"tokens": 14940, "time": 22.7, "success": true}
  }
}
```

## Appendix C: Analysis Script Templates

See Section 4.2 for Python analysis scripts.

---

**Document Version**: 1.0
**Created**: November 5, 2025
**Owner**: VerusAgent Research Team
