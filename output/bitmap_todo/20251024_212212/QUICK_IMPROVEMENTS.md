# Quick Improvement Checklist

## 🔥 Critical Fixes (Implement First)

### 1. Fix `assert forall` Syntax in proof_generation
```python
# Add to proof generation validation:
def validate_generated_proof(code: str) -> bool:
    if 'assert forall' in code and 'by {' not in code:
        raise SyntaxError("assert forall must have 'by {}' clause")
    return True

# Template:
ASSERT_FORALL_TEMPLATE = """
assert forall|var: Type| condition implies conclusion by {
    // proof body (can be empty)
}
"""
```

### 2. Add Minimalist Proof Strategy
```python
# Try proofs in order of simplicity:
PROOF_STRATEGIES = [
    "empty",           # proof { }
    "call_lemma",      # proof { lemma(args); }
    "simple_assert",   # proof { assert(fact); }
    "assert_forall",   # last resort
]
```

### 3. Enhance Repair Module
```python
# Add pattern-based fixes:
SYNTAX_FIXES = {
    "expected `by`": {
        "pattern": r"(assert forall[^;]+);",
        "fix": r"\1 by {\n    \n}",
    }
}
```

---

## 📋 Implementation Checklist

### Phase 1: Quick Wins (1-2 weeks)

- [ ] **proof_generation.py**
  - [ ] Add `assert forall` syntax validation
  - [ ] Implement minimalist-first strategy
  - [ ] Add proof pattern templates

- [ ] **repair_syntax.py**
  - [ ] Add Verus syntax pattern library
  - [ ] Implement pattern-based auto-fixes
  - [ ] Add escalating repair strategies

- [ ] **spec_inference.py**
  - [ ] Prefer `@` over `.view()`
  - [ ] Simplify type casts
  - [ ] Clean up postconditions

- [ ] **Testing**
  - [ ] Add unit tests for `assert forall` generation
  - [ ] Add integration test with bitmap_todo
  - [ ] Verify fixes on other failed benchmarks

### Phase 2: Proof Intelligence (1 month)

- [ ] **proof_pattern_library.py**
  - [ ] Extract patterns from working benchmarks
  - [ ] Implement pattern matching
  - [ ] Add pattern scoring/selection

- [ ] **loop_invariant_generator.py**
  - [ ] Improve invariant templates
  - [ ] Use high-level @ notation
  - [ ] Reduce verbosity

### Phase 3: Learning System (2-3 months)

- [ ] **verification_memory.py**
  - [ ] Track successful/failed patterns
  - [ ] Implement cross-workflow learning
  - [ ] Add pattern recommendation

---

## 🎯 Expected Impact

| Improvement | Before | After | Time Saved |
|-------------|--------|-------|------------|
| Assert forall fix | ❌ Syntax error | ✅ Compiles | 38 min |
| Minimalist proofs | 🔧 Complex, fragile | ✅ Simple, robust | 20 min |
| Smart repairs | ❌ 7% success | ✅ 80% success | 30 min |
| **Total per benchmark** | **~90 min** | **~30 min** | **~60 min** |

---

## 🔧 Code Snippets to Add

### 1. In `proof_generation.py`

```python
def generate_loop_proof(loop_info, invariant):
    """Generate minimalist proof for loop body."""

    # Try empty proof first
    if has_simple_invariant(invariant):
        return "proof { }"

    # Find relevant lemma
    lemma = find_applicable_lemma(loop_info)
    if lemma:
        return f"proof {{\n    {lemma.name}({lemma.args});\n}}"

    # Only add assertions if necessary
    return generate_complex_proof(loop_info)

def validate_proof_syntax(code: str):
    """Validate Verus proof syntax before returning."""
    errors = []

    # Check assert forall
    if 'assert forall' in code:
        if not re.search(r'assert forall.*by\s*\{', code, re.DOTALL):
            errors.append("assert forall missing 'by' clause")

    # Check proof block closure
    if code.count('proof {') != code.count('}'):
        errors.append("Unmatched braces in proof block")

    if errors:
        raise ProofSyntaxError(errors)
```

### 2. In `repair_syntax.py`

```python
VERUS_SYNTAX_PATTERNS = {
    "assert_forall_fix": {
        "regex": r'(assert forall\|[^|]+\|[^;]+);',
        "replacement": r'\1 by {\n    \n}',
        "error_match": "expected `by`",
    },

    "implies_keyword": {
        "regex": r'(assert forall[^=]+)==>',
        "replacement": r'\1implies',
        "error_match": "expected `by`",
    }
}

def repair_with_patterns(code: str, error: str) -> str:
    """Apply pattern-based fixes."""
    for pattern_name, pattern in VERUS_SYNTAX_PATTERNS.items():
        if pattern["error_match"] in error:
            fixed = re.sub(pattern["regex"],
                          pattern["replacement"],
                          code)
            if fixed != code:
                return fixed

    # Fallback to LLM
    return llm_repair(code, error)
```

### 3. In `spec_inference.py`

```python
def simplify_spec(spec: str) -> str:
    """Simplify verbose specifications."""

    # Use @ shorthand
    spec = spec.replace('.view().len()', '@.len()')
    spec = spec.replace('.view()[', '@[')
    spec = spec.replace('old(self).view()', 'old(self)@')

    # Remove redundant casts in comparisons
    spec = re.sub(r'\((\w+) as int\) < self@\.len\(\)',
                  r'\1 < self@.len()', spec)

    return spec
```

---

## 📊 Testing Strategy

### Before deploying:

```bash
# 1. Unit tests
pytest tests/test_proof_generation.py::test_assert_forall_syntax
pytest tests/test_repair_syntax.py::test_pattern_fixes
pytest tests/test_spec_inference.py::test_simplification

# 2. Integration test with failed benchmark
python run_workflow.py --input benchmarks-complete/bitmap_todo.rs \
                       --output output/bitmap_todo_fixed/

# 3. Regression tests on working benchmarks
pytest tests/integration/test_regression.py

# 4. Full test suite
pytest tests/ -v --benchmark
```

### Success criteria:
- ✅ bitmap_todo verifies successfully
- ✅ No regressions on working benchmarks
- ✅ Repair success rate > 80%
- ✅ Time to verify < 10 minutes

---

## 🚀 Quick Start Guide

### Step 1: Add Syntax Validation (5 minutes)

Add this to `modules/proof_generation.py`:

```python
# At the top
from typing import List
import re

# Before returning generated code
def finalize_proof_code(code: str) -> str:
    validate_proof_syntax(code)  # Add this line
    return code
```

### Step 2: Add Pattern Fixes (10 minutes)

Add this to `modules/repair_syntax.py`:

```python
# Import at top
from .syntax_patterns import VERUS_SYNTAX_PATTERNS

# In repair function
def repair_syntax(code: str, error: str) -> str:
    # Try pattern-based fix first (NEW)
    fixed = apply_syntax_patterns(code, error)
    if fixed != code:
        return fixed

    # Fallback to existing LLM repair
    return llm_repair(code, error)
```

### Step 3: Test on bitmap_todo (2 minutes)

```bash
python run_workflow.py benchmarks-complete/bitmap_todo.rs
# Should now succeed!
```

---

## 📝 Verification Checklist

After implementing improvements, verify:

- [ ] `assert forall` statements have `by` clause
- [ ] Proof blocks are minimal (< 5 lines)
- [ ] Specs use `@` notation consistently
- [ ] No syntax errors in generated code
- [ ] Repair succeeds in < 3 rounds
- [ ] bitmap_todo verifies successfully
- [ ] No regressions on existing benchmarks

---

## 💡 Pro Tips

1. **Start small**: Fix assert forall syntax first - biggest bang for buck
2. **Test incrementally**: Verify each change doesn't break existing tests
3. **Learn from successes**: Extract patterns from bitmap_2_expanded.rs
4. **Keep it simple**: Prefer empty proof blocks when possible
5. **Validate early**: Catch syntax errors before running Verus

---

## 📚 References

- Full recommendations: `IMPROVEMENT_RECOMMENDATIONS.md`
- Detailed analysis: `DEBUG_ANALYSIS.md`
- Side-by-side comparison: `SIDE_BY_SIDE_COMPARISON.md`
- Quick fix guide: `QUICK_FIX_SUMMARY.md`
- Working example: `../../../benchmarks-complete/bitmap_2_expanded.rs`
- Fixed version: `fixed_bitmap.rs`
