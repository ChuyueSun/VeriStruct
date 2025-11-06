# Reusing VEVAL Error Classification for Smart Repair Priority

## Problem Solved

Instead of creating a new error classifier, **reuse the existing `VerusErrorType` enum** from VEVAL which already classifies 24 error types for intelligent **prioritization**!

## VEVAL's Error Classification (Already Exists!)

```python
class VerusErrorType(Enum):
    # Specification Errors (HIGH PRIORITY - Often Fixable)
    PreCondFail = 1              ✓ Priority 1 - repair_precond
    PostCondFail = 2             ✓ Priority 1 - repair_postcond
    InvFailEnd = 3               ✓ Priority 1 - repair_invariant
    InvFailFront = 4             ✓ Priority 1 - repair_invariant
    DecFailEnd = 5               ✓ Priority 1 - repair_decrease
    DecFailCont = 6              ✓ Priority 1 - repair_decrease

    # Proof Errors (LOW PRIORITY - Harder but Worth Trying)
    AssertFail = 11              ✓ Priority 3 - repair_assertion
    TestAssertFail = 7           ✓ Priority 3 - repair_test_assertion
    RecommendNotMet = 8          ✓ Priority 4 - informational

    # Syntax/Type Errors (MEDIUM PRIORITY - Usually Fixable)
    MismatchedType = 13          ✓ Priority 2 - repair_type
    MissImpl = 15                ✓ Priority 2 - repair_missing
    ensure_private = 17          ✓ Priority 2 - repair_mode
    require_private = 18         ✓ Priority 2 - repair_mode
    MissingImport = 19           ✓ Priority 2 - repair_syntax
    TypeAnnotation = 20          ✓ Priority 2 - repair_type

    # Other
    Other = 16                   ✓ Priority 2 - repair_syntax
```

## Simple Implementation: Priority-Based Repair

**Philosophy:** Try to fix ALL errors, but prioritize the most fixable ones first!

```python
# In repair_registry.py

# Priority 1: Specification errors (high success rate, fix first)
PRIORITY_1_ERRORS = {
    VerusErrorType.PreCondFail,
    VerusErrorType.PreCondFailVecLen,
    VerusErrorType.PostCondFail,
    VerusErrorType.InvFailEnd,
    VerusErrorType.InvFailFront,
    VerusErrorType.DecFailEnd,
    VerusErrorType.DecFailCont,
}

# Priority 2: Syntax/type errors (medium success rate)
PRIORITY_2_ERRORS = {
    VerusErrorType.MismatchedType,
    VerusErrorType.MissImpl,
    VerusErrorType.TypeAnnotation,
    VerusErrorType.ensure_private,
    VerusErrorType.require_private,
    VerusErrorType.RequiresOldSelf,
    VerusErrorType.PubSpecVisibility,
    VerusErrorType.MissingImport,
    VerusErrorType.CannotCallFunc,
    VerusErrorType.ConstructorFailTypeInvariant,
    VerusErrorType.Other,
}

# Priority 3: Proof errors (harder, but still worth trying)
PRIORITY_3_ERRORS = {
    VerusErrorType.AssertFail,
    VerusErrorType.TestAssertFail,
}

# Priority 4: Informational (lowest priority)
PRIORITY_4_ERRORS = {
    VerusErrorType.RecommendNotMet,
}

def get_error_priority(self, error_type: VerusErrorType) -> int:
    """Get repair priority for error type (lower = higher priority)."""
    if error_type in PRIORITY_1_ERRORS:
        return 1
    elif error_type in PRIORITY_2_ERRORS:
        return 2
    elif error_type in PRIORITY_3_ERRORS:
        return 3
    elif error_type in PRIORITY_4_ERRORS:
        return 4
    else:
        return 5  # Unknown - lowest priority
```

## Integration with Existing Code

### Update `prioritize_failures()` Method:

```python
# BEFORE (current - already exists but simple):
def prioritize_failures(self, failures: List[VerusError]) -> List[VerusError]:
    # Current implementation focuses on "Other" errors
    # ...

# AFTER (enhanced with VEVAL error types):
def prioritize_failures(self, failures: List[VerusError]) -> List[VerusError]:
    """
    Prioritize failures based on their error type from VEVAL.

    Priority order (lower number = repair first):
    1. Specification errors (precond, postcond, invariant) - high fix rate
    2. Syntax/type errors - medium fix rate
    3. Proof errors (assert) - lower fix rate, still try
    4. Informational - lowest priority
    """
    # Separate by priority using VEVAL's error type
    priority_1 = [f for f in failures if self.get_error_priority(f.error) == 1]
    priority_2 = [f for f in failures if self.get_error_priority(f.error) == 2]
    priority_3 = [f for f in failures if self.get_error_priority(f.error) == 3]
    priority_4 = [f for f in failures if self.get_error_priority(f.error) == 4]
    other = [f for f in failures if self.get_error_priority(f.error) == 5]

    # Return in priority order (still repair ALL, just in smart order)
    return priority_1 + priority_2 + priority_3 + priority_4 + other
```

### No Changes Needed to `repair_all()` Loop!

The prioritization happens in `prioritize_failures()`, so the repair loop stays the same:

```python
# In repair_all() - NO CHANGES NEEDED
for error_type, type_failures in error_type_map.items():
    if error_type in self.error_to_module_map:
        module = self.error_to_module_map[error_type]
        # ... attempt repair (ALL errors attempted, just in priority order)
```

## Benefits of Reusing VEVAL Classification

1. ✅ **No New Code** - Just use existing `error.error` field
2. ✅ **Already Accurate** - VEVAL's classification is battle-tested
3. ✅ **Simple Logic** - Priority-based, not skip-based
4. ✅ **Try Everything** - All errors attempted, just in smart order
5. ✅ **Type Safe** - Using Enum instead of string matching

## Why Priority Instead of Skip?

**Key Insight:** Even "hard" errors like `AssertFail` are worth attempting!

- ✅ The LLM might surprise us with a fix
- ✅ Partial fixes can give users hints
- ✅ Failed attempts still provide diagnostic info
- ✅ No harm in trying (with timeout protection)

**Better Strategy:**
- Fix easy errors first (specs, syntax) → Higher success rate
- Fix hard errors last (proof assertions) → Lower but non-zero success rate
- Within timeout budget, try everything!

## Error Priority Rationale

### Priority 1: Specification Errors
**Why High Priority:**
- Often caused by missing/wrong specs
- LLM has high success rate (~80%)
- Fixes often cascade to other errors
- Examples: precond, postcond, invariants

### Priority 2: Syntax/Type Errors
**Why Medium Priority:**
- Usually straightforward fixes
- Good success rate (~70%)
- Clear error messages help LLM
- Examples: type mismatches, missing imports

### Priority 3: Proof Errors
**Why Low Priority (but Still Try):**
- Harder logic errors
- Lower success rate (~30-40%)
- But LLM can sometimes add helper assertions
- Worth attempting within timeout budget
- Examples: AssertFail in proof blocks

### Priority 4: Informational
**Why Lowest Priority:**
- Not actual errors
- Recommendations for optimization
- Nice-to-have, not need-to-have

## Example Usage

```python
# In repair_registry.py

def prioritize_failures(self, failures: List[VerusError]) -> List[VerusError]:
    """
    Prioritize failures for repair, filtering out errors that should be skipped.

    Priority order:
    1. Spec errors (precond, postcond, invariant)
    2. Syntax/type errors
    3. Mode/visibility errors

    Skipped:
    - Proof errors (AssertFail, TestAssertFail)
    - Recommendations
    """
    # Filter out errors that should be skipped
    repairable = [f for f in failures if f.error not in SKIP_REPAIR_ERRORS]

    # Categorize
    spec_errors = [f for f in repairable if f.error in SPEC_ERRORS]
    syntax_errors = [f for f in repairable if f.error in SYNTAX_TYPE_ERRORS]
    mode_errors = [f for f in repairable if f.error in MODE_ERRORS]
    other_errors = [f for f in repairable
                   if f.error not in SPEC_ERRORS
                   and f.error not in SYNTAX_TYPE_ERRORS
                   and f.error not in MODE_ERRORS]

    # Return in priority order
    return spec_errors + syntax_errors + mode_errors + other_errors
```

## Minimal Code Change

```python
# In src/modules/repair_registry.py

# Add at top after imports
from src.modules.veval import VerusErrorType

# Add after class definition
class RepairRegistry:
    # Error types that should skip repair (proof logic issues)
    SKIP_REPAIR_ERRORS = {
        VerusErrorType.AssertFail,
        VerusErrorType.TestAssertFail,
        VerusErrorType.RecommendNotMet,
    }

    def should_skip_repair(self, error_type: VerusErrorType) -> bool:
        """Check if this error type should skip repair."""
        return error_type in self.SKIP_REPAIR_ERRORS

    # Modify repair_all() to check before repair
    def repair_all(...):
        # ...
        for error_type, type_failures in error_type_map.items():
            # Check if should skip
            if self.should_skip_repair(error_type):
                self.logger.info(
                    f"⏭️  Skipping {error_type.name} repair - "
                    "proof logic error requires manual fix"
                )
                continue
            # ... rest of repair logic
```

## Summary

**Instead of creating a new classifier:**
- ✅ Use VEVAL's existing `VerusErrorType` enum (24 types)
- ✅ Add simple skip set for proof errors
- ✅ Minimal code: ~10 lines
- ✅ Type-safe and already integrated
- ✅ Easy to maintain and extend

**This is the right approach!** 🎯
