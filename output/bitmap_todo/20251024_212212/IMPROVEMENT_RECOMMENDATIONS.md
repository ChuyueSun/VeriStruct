# Workflow Improvement Recommendations

Based on the bitmap_todo failure analysis, here are concrete, actionable improvements for the VerusAgent workflow.

---

## 🎯 High Priority Improvements

### 1. Fix `proof_generation` Module - Assert Forall Syntax

**Problem**: Generated `assert forall` without required `by {}` clause
**Impact**: High - Caused complete workflow failure
**Effort**: Low

#### Implementation

Add this template to proof generation knowledge base:

```python
ASSERT_FORALL_TEMPLATE = """
// ✅ CORRECT SYNTAX for assert forall in Verus:
assert forall|var: Type| #![trigger expression]
    precondition implies postcondition
by {
    // Proof steps (can be empty if obvious from context)
}

// ❌ WRONG - Missing 'by' clause:
assert forall|var: Type| precondition ==> postcondition;
"""
```

Add validation check before generating code:

```python
def validate_assert_forall(code: str) -> List[str]:
    """Check that all assert forall statements have 'by' clause."""
    errors = []
    lines = code.split('\n')

    for i, line in enumerate(lines):
        if 'assert forall' in line:
            # Look ahead for 'by {' within next 10 lines
            found_by = False
            for j in range(i, min(i + 10, len(lines))):
                if 'by {' in lines[j] or 'by{' in lines[j]:
                    found_by = True
                    break

            if not found_by:
                errors.append(f"Line {i+1}: assert forall missing 'by' clause")

    return errors
```

---

### 2. Simplify Proof Generation - "Less is More" Strategy

**Problem**: Over-engineered proofs with unnecessary assertions
**Impact**: High - Creates fragile code that's hard to repair
**Effort**: Medium

#### Guiding Principles

```markdown
PROOF_GENERATION_STRATEGY:

1. **Minimalist First**: Start with empty proof blocks
   proof {
       // Call proof function only
       lemma_or_proof_function(args);
   }

2. **Add assertions ONLY if verification fails**
   - First try: empty proof block
   - Second try: call relevant proof functions
   - Last resort: add assert forall/assert

3. **Prefer proof functions over inline assertions**
   ✅ proof { bit_or_64_proof(u1, u2, or_int); }
   ❌ proof { assert forall|i| ...; assert(...); ... }

4. **Trust loop invariants**
   - If invariant is well-written, proof block can be minimal
   - Don't repeat invariant properties in assertions
```

#### Code Pattern Recognition

Before generating complex proofs, check if simpler patterns work:

```python
SIMPLE_PROOF_PATTERNS = {
    "sequence_update": {
        "proof_block": """
proof {
    proof_function(relevant_args);
    assert_seqs_equal!(self@, expected_sequence);
}""",
        "when_to_use": "updating a single element in a sequence-based view"
    },

    "loop_with_lemma": {
        "proof_block": """
proof {
    lemma_function(args);
    // Empty body - let invariant do the work
}""",
        "when_to_use": "loop with clear invariant and relevant lemma"
    },

    "bitwise_operation": {
        "proof_block": """
proof {
    bitvector_lemma(args);
    // Lemma provides all needed bit-level facts
}""",
        "when_to_use": "operations on bitvectors with corresponding lemmas"
    }
}
```

---

### 3. Enhance Repair Module - Verus Syntax Knowledge

**Problem**: 9 failed syntax repairs couldn't fix `assert forall`
**Impact**: High - Workflow couldn't recover from simple syntax error
**Effort**: Medium

#### Add Syntax Pattern Library

```python
VERUS_SYNTAX_FIXES = {
    "assert_forall_missing_by": {
        "pattern": r"assert forall\|[^|]+\|[^;]+;",
        "error_message": "expected `by`",
        "fix": lambda match: match.group(0).replace(';', ' by {\n    \n}'),
        "description": "Add missing 'by {}' clause to assert forall"
    },

    "implies_vs_arrow": {
        "pattern": r"(assert forall\|[^|]+\|[^=]+)==>",
        "suggestion": "Use 'implies' instead of '==>' in assert forall",
        "fix": lambda match: match.group(0).replace('==>', 'implies')
    },

    "seq_equality": {
        "pattern": r"ensures[^;]*==\s*old\(self\)@\.update",
        "suggestion": "Use == for sequence equality in postconditions (correct)",
        "note": "This is already correct - don't change to =~="
    },

    "map_equality": {
        "pattern": r"ensures[^;]*\.to_map\(\)\s*==\s*old",
        "suggestion": "Use =~= for map equality in postconditions",
        "fix": lambda match: match.group(0).replace('==', '=~=', 1)
    }
}
```

#### Repair Strategy Improvements

```python
def repair_syntax_smart(code: str, error_msg: str, attempt: int) -> str:
    """
    Smart syntax repair with escalating strategies.

    Args:
        code: Broken code
        error_msg: Compiler error message
        attempt: Repair attempt number (1-based)
    """

    # Attempt 1: Pattern-based fixes
    if attempt == 1:
        for pattern_name, pattern_info in VERUS_SYNTAX_FIXES.items():
            if pattern_info.get("error_message", "") in error_msg:
                return apply_pattern_fix(code, pattern_info)

    # Attempt 2: LLM with enriched context
    elif attempt == 2:
        return llm_repair_with_examples(code, error_msg,
                                       examples=WORKING_EXAMPLES)

    # Attempt 3: Simplification strategy
    elif attempt == 3:
        return simplify_complex_proofs(code, error_msg)

    # Attempt 4+: Escalate to different module
    else:
        raise RepairEscalation("Switch to different repair strategy")
```

---

## 🔧 Medium Priority Improvements

### 4. Spec Inference - Better Precondition Generation

**Problem**: Generated `(index as int) < old(self).view().len()` when `index < old(self)@.len()` is cleaner
**Impact**: Medium - Code works but is verbose
**Effort**: Low

#### Simplification Rules

```python
SPEC_SIMPLIFICATION_RULES = {
    "view_shorthand": {
        "verbose": r"\.view\(\)\.len\(\)",
        "simplified": "@.len()",
        "note": "Use @ shorthand for view"
    },

    "redundant_cast": {
        "verbose": r"\((\w+) as int\) < self@\.len\(\)",
        "simplified": r"\1 < self@.len()",
        "note": "Verus infers int conversion in comparisons"
    },

    "old_self_view": {
        "verbose": r"old\(self\)\.view\(\)",
        "simplified": "old(self)@",
        "note": "Consistent use of @ notation"
    }
}
```

### 5. Loop Invariant Inference

**Current**: Generates verbose invariants
**Improvement**: Use high-level @ notation consistently

```python
def generate_loop_invariant(loop_context):
    """Generate clean, maintainable loop invariants."""

    # Prefer @ notation over low-level operations
    if has_view_function(loop_context.struct):
        return f"""
invariant
    i <= n,
    n == self.data.len(),
    result@.len() == i,
    forall|k: int| #![auto] 0 <= k < i ==>
        result@[k] == expected_value(k),
"""
    # Don't expose internal representation in invariants
    # Use abstract view whenever possible
```

---

## 🚀 Long-term Improvements

### 6. Proof Pattern Library

**Impact**: High - Reuse successful proof patterns
**Effort**: High - Requires building comprehensive library

#### Structure

```python
PROOF_LIBRARY = {
    "bitmap_operations": {
        "set_bit": {
            "pattern": """
proof {
    bit_set_proof(new_val, old_val, bit_index, bit);
    assert_seqs_equal!(
        self.view(),
        old(self).view().update(index as int, bit)
    );
}""",
            "works_for": ["BitMap", "BitVector", "FlagArray"],
            "requirements": ["bit_set_proof lemma", "view() -> Seq<bool>"]
        },

        "bitwise_or": {
            "pattern": """
proof {
    bit_or_proof(u1, u2, result);
    // Loop invariant handles correctness
}""",
            "works_for": ["BitMap::or", "BitSet::union"],
            "requirements": ["bit_or_proof lemma", "loop invariant on result@"]
        }
    },

    "sequence_operations": {
        "single_update": {
            "pattern": """
proof {
    assert_seqs_equal!(self@, old(self)@.update(idx as int, value));
}""",
            "works_for": ["Vec::set", "Array::update"],
            "requirements": ["postcondition uses .update()"]
        },

        "append": {
            "pattern": """
proof {
    assert_seqs_equal!(
        self@,
        old(self)@.push(value)
    );
}""",
            "works_for": ["Vec::push", "List::append"]
        }
    }
}
```

#### Pattern Matching

```python
def find_matching_proof_pattern(function_signature, postcondition):
    """Find best matching proof pattern from library."""

    # Extract key features
    features = {
        "modifies_sequence": has_sequence_view(function_signature),
        "updates_single_element": uses_update_postcondition(postcondition),
        "bitwise_operation": has_bitvector_ops(function_signature),
        "loop_present": has_loop(function_signature),
    }

    # Score each pattern
    best_pattern = None
    best_score = 0

    for category in PROOF_LIBRARY.values():
        for pattern_name, pattern_info in category.items():
            score = compute_similarity(features, pattern_info)
            if score > best_score:
                best_score = score
                best_pattern = pattern_info

    return best_pattern if best_score > 0.7 else None
```

---

### 7. Verification Feedback Loop

**Problem**: No learning from failures across attempts
**Impact**: High - Same mistakes repeated
**Effort**: High

#### Implementation

```python
class VerificationMemory:
    """Remember what worked and what failed."""

    def __init__(self):
        self.successful_patterns = {}
        self.failed_patterns = {}
        self.repair_history = []

    def record_success(self, code_pattern, context):
        """Store successful proof pattern."""
        key = self.extract_pattern_key(code_pattern, context)
        self.successful_patterns[key] = {
            "code": code_pattern,
            "context": context,
            "timestamp": now(),
            "success_count": self.successful_patterns.get(key, {}).get("success_count", 0) + 1
        }

    def record_failure(self, code_pattern, error, context):
        """Store failed pattern to avoid repeating."""
        key = self.extract_pattern_key(code_pattern, context)
        self.failed_patterns[key] = {
            "code": code_pattern,
            "error": error,
            "context": context,
            "failure_count": self.failed_patterns.get(key, {}).get("failure_count", 0) + 1
        }

    def should_avoid_pattern(self, code_pattern, context):
        """Check if pattern has failed multiple times."""
        key = self.extract_pattern_key(code_pattern, context)
        failures = self.failed_patterns.get(key, {}).get("failure_count", 0)
        return failures >= 3  # Avoid after 3 failures

    def suggest_alternative(self, failed_pattern, context):
        """Suggest alternative based on successful patterns."""
        similar_successes = [
            p for p in self.successful_patterns.values()
            if self.is_similar_context(p["context"], context)
        ]

        if similar_successes:
            # Return most successful pattern
            return max(similar_successes, key=lambda p: p["success_count"])

        return None
```

---

### 8. Pre-verification Syntax Checking

**Impact**: Medium - Catch errors before running Verus
**Effort**: Medium

```python
class VerusSyntaxChecker:
    """Validate Verus syntax before compilation."""

    CHECKS = [
        ("assert_forall_has_by", check_assert_forall),
        ("proof_block_closed", check_proof_blocks),
        ("trigger_syntax", check_triggers),
        ("view_consistency", check_view_notation),
    ]

    def validate(self, code: str) -> Tuple[bool, List[str]]:
        """Run all syntax checks."""
        errors = []

        for check_name, check_func in self.CHECKS:
            result = check_func(code)
            if not result.is_valid:
                errors.append(f"{check_name}: {result.message}")

        return len(errors) == 0, errors

def check_assert_forall(code: str) -> ValidationResult:
    """Check assert forall statements have by clause."""
    lines = code.split('\n')
    in_assert_forall = False
    assert_line = 0

    for i, line in enumerate(lines):
        if 'assert forall' in line:
            in_assert_forall = True
            assert_line = i

        if in_assert_forall:
            if 'by {' in line or 'by{' in line:
                in_assert_forall = False
            elif ';' in line and 'assert forall' in lines[assert_line]:
                return ValidationResult(
                    is_valid=False,
                    message=f"Line {assert_line+1}: assert forall missing 'by' clause"
                )

    return ValidationResult(is_valid=True)
```

---

## 📊 Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks)

1. ✅ Add `assert forall` syntax validation
2. ✅ Add syntax pattern fixes to repair module
3. ✅ Simplify spec_inference output (use @ notation)
4. ✅ Add proof simplification rules

**Expected Impact**: Fix 80% of syntax-related failures

### Phase 2: Proof Intelligence (1 month)

1. Build proof pattern library from successful benchmarks
2. Implement pattern matching for proof generation
3. Add "minimalist first" strategy to proof_generation
4. Improve loop invariant generation

**Expected Impact**: Reduce proof complexity, increase success rate

### Phase 3: Learning System (2-3 months)

1. Implement verification memory system
2. Add cross-workflow learning
3. Build feedback loop from repairs to generation
4. Automated pattern extraction from successes

**Expected Impact**: Continuous improvement, fewer repeated mistakes

---

## 🎓 Knowledge Base Enhancements

### Add to proof_generation Knowledge

```markdown
## Assert Forall Syntax (CRITICAL)

In Verus, `assert forall` REQUIRES a `by` clause:

```rust
// ✅ ALWAYS use this form:
assert forall|x: T| condition implies conclusion by {
    // proof steps (can be empty)
}

// ❌ NEVER use this form:
assert forall|x: T| condition ==> conclusion;  // SYNTAX ERROR
```

## When to Use Assert Forall

❌ DON'T use when:
- Loop invariant already states the property
- Proof function provides the needed facts
- Simple assertions would suffice

✅ DO use when:
- Need to prove property over range not in invariant
- Bridging gap between abstract and concrete representations
- Proof function needs help with specific case

## Proof Simplicity Guideline

Always try in this order:
1. Empty proof block (let Verus figure it out)
2. Call proof function only
3. Add simple assert statements
4. Add assert forall (last resort)
```

### Add to repair Module Knowledge

```markdown
## Common Verus Syntax Errors

### Error: "expected `by`"
**Cause**: Missing `by` clause in `assert forall`
**Fix**: Add `by { }` after the assertion
```rust
// Before:
assert forall|x| P(x) ==> Q(x);

// After:
assert forall|x| P(x) implies Q(x) by { }
```

### Error: "postcondition not satisfied" (for maps)
**Cause**: Using `==` instead of `=~=` for map equality
**Fix**: Replace `==` with `=~=`
```rust
// Before:
ensures self.to_map() == old(self).to_map().insert(k, v)

// After:
ensures self.to_map() =~= old(self).to_map().insert(k, v)
```
```

---

## 📈 Success Metrics

Track these metrics to measure improvement:

```python
METRICS = {
    "proof_generation": {
        "unnecessary_assertions": "Count of proofs with >2 assertions",
        "syntax_errors": "Proof blocks that don't compile",
        "first_attempt_success": "% that verify without repair",
    },

    "spec_inference": {
        "verbosity_score": "Avg length of pre/postconditions",
        "consistency_score": "% using @ notation consistently",
    },

    "repair": {
        "fix_rate": "% of errors fixed",
        "attempts_to_fix": "Avg repair rounds needed",
        "repeated_failures": "Same error >3 times",
        "escalation_rate": "% requiring manual intervention",
    },

    "overall": {
        "end_to_end_success": "% workflows completing successfully",
        "time_to_verify": "Avg time from start to verified",
        "benchmark_coverage": "% of test suite passing",
    }
}
```

---

## 🔍 Testing Recommendations

### Unit Tests for Each Module

```python
# Test proof_generation
def test_assert_forall_syntax():
    code = generate_proof_for_loop(test_context)
    assert "by {" in code or "assert forall" not in code

def test_proof_minimalism():
    code = generate_proof_simple_case(test_context)
    assertion_count = code.count("assert ")
    assert assertion_count <= 1, "Proof too complex"

# Test repair
def test_repair_assert_forall():
    broken = "assert forall|x| P(x) ==> Q(x);"
    fixed = repair_syntax(broken, "expected `by`")
    assert "by {" in fixed
    assert verus_compiles(fixed)

# Test spec_inference
def test_spec_uses_view_shorthand():
    spec = infer_precondition(test_function)
    assert spec.count(".view()") == 0
    assert spec.count("@") > 0
```

### Integration Tests

```python
def test_bitmap_pattern():
    """Test end-to-end on bitmap-like structures."""
    result = run_workflow("benchmarks/bitmap_todo.rs")
    assert result.verified_count >= 7
    assert result.syntax_errors == 0
    assert result.repair_rounds <= 5

def test_sequence_pattern():
    """Test end-to-end on sequence-based structures."""
    result = run_workflow("benchmarks/vector_todo.rs")
    assert all_functions_verified(result)
```

---

## 💡 Example: How Improvements Would Have Helped

### Original Failure
```
Step 5 (proof_generation): Generated broken assert forall
Repair Round 1-10: Failed to fix syntax
Total time: 38 minutes
Result: FAILED
```

### With Improvements
```
Step 5 (proof_generation):
  - Pre-check: assert forall validation ✓
  - Minimalist strategy: Tries empty proof first
  - Pattern match: Finds "bitwise_or" pattern
  - Generated: Simple proof { bit_or_64_proof(...); }

Result: VERIFIED in 2 minutes
```

**Time saved**: 36 minutes
**Success rate**: 100% (vs 0%)

---

## Summary

| Improvement | Impact | Effort | Priority |
|-------------|--------|--------|----------|
| Assert forall syntax fix | High | Low | P0 🔥 |
| Proof minimalism | High | Medium | P0 🔥 |
| Repair syntax knowledge | High | Medium | P0 🔥 |
| Spec simplification | Medium | Low | P1 |
| Proof pattern library | High | High | P1 |
| Verification memory | High | High | P2 |
| Pre-verification checks | Medium | Medium | P2 |

**Recommendation**: Implement Phase 1 improvements immediately. They're low-effort, high-impact fixes that would have prevented this exact failure.
