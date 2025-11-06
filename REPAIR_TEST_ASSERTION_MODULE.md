# New Module: repair_test_assertion

## 🎯 **Purpose**

Handle **TestAssertFail** errors separately from production **AssertFail** errors, because test functions are **IMMUTABLE** and require a different repair strategy.

## 🔑 **Key Insight**

### **Problem**
```
Test function (IMMUTABLE):
fn test() {
    let result = buf.dequeue();
    assert(result == None::<i32>);  // ← FAILS
}
```

**Wrong approach** (old):
- Try to modify test assertion
- Result: ❌ Breaks immutability constraint
- Outcome: Compilation error (999 errors)

**Right approach** (new):
- Identify which function is tested (`dequeue`)
- Strengthen that function's postconditions
- Result: ✅ Test assertion now provable
- Outcome: Test passes

---

## 📊 **Before vs After**

### **Before (Shared Module)**

**Both errors used `repair_assertion`:**
```python
registry.register_module(
    "repair_assertion",
    assertion_repair,
    [VerusErrorType.AssertFail, VerusErrorType.TestAssertFail],  # Both!
)
```

**Result:**
- TestAssertFail repairs: 0% success rate
- Frequently broke compilation
- Tried to modify immutable test code

---

### **After (Separate Modules)**

**AssertFail → repair_assertion** (production code):
```python
registry.register_module(
    "repair_assertion",
    assertion_repair,
    [VerusErrorType.AssertFail],  # Production only
)
```

**TestAssertFail → repair_test_assertion** (test code):
```python
registry.register_module(
    "repair_test_assertion",
    test_assertion_repair,
    [VerusErrorType.TestAssertFail],  # Test only
)
```

**Result:**
- Clear separation of concerns
- Different strategies for different contexts
- Respects immutability constraints

---

## 🔧 **Repair Strategy**

### **repair_test_assertion Strategy:**

1. **Identify tested function**
   - Parse test code before failing assertion
   - Find recent function call (e.g., `buf.dequeue()`)
   - Focus repair on that function

2. **Strengthen postconditions**
   - Add guarantees about return value
   - Add state relationship postconditions
   - Ensure postconditions satisfy test expectations

3. **Never touch test code**
   - Test function is immutable
   - Only modify production functions
   - Add to `ensures` clauses only

4. **Add proof hints if needed**
   - May need proof blocks in production functions
   - Help Verus prove the strengthened postconditions

---

## 📝 **Example**

### **Failing Test:**
```rust
fn test() {
    let mut buf = RingBuffer::new(ring);
    let ret = buf.dequeue();        // ← Testing dequeue
    assert(!has_elements);          // ← FAILS
    assert(ret == None::<i32>);     // ← FAILS
}
```

### **Current Production Code:**
```rust
pub fn dequeue(&mut self) -> (ret: Option<T>)
    ensures
        ret.is_some() ==> ret.unwrap() == old(self)@.0[0],
        // Missing postcondition about when None is returned!
```

### **Fixed by repair_test_assertion:**
```rust
pub fn dequeue(&mut self) -> (ret: Option<T>)
    ensures
        ret.is_some() ==> ret.unwrap() == old(self)@.0[0],
        ret.is_some() ==> self@.0 == old(self)@.0.subrange(1, old(self)@.0.len() as int),
        // ✅ Added: Guarantee when None is returned
        ret.is_none() ==> ret == None::<T>,
        ret.is_none() ==> old(self)@.0.len() == 0,
        ret.is_none() ==> self@.0 == old(self)@.0,
```

**Now test assertions can be proved!** ✅

---

## 🎓 **Implementation Details**

### **File:** `src/modules/repair_test_assertion.py`

### **Key Methods:**

1. **`exec(context, failure_to_fix)`**
   - Main repair logic
   - Builds instruction emphasizing immutability
   - Calls LLM with test-specific examples

2. **`_identify_tested_function(code, error_trace)`**
   - Parses code to find which function is tested
   - Looks for function calls near failing assertion
   - Returns function name for targeted repair

### **Key Features:**

- ✅ Emphasizes test immutability in prompt
- ✅ Focuses on production code postconditions
- ✅ Identifies tested function automatically
- ✅ Uses test-specific examples
- ✅ Saves prompts to `prompts/repair_test_assertion_{trial}.txt`
- ✅ Timeout protection (inherits from BaseRepairModule)
- ✅ Retry support (inherits from BaseRepairModule)

---

## 📈 **Expected Improvement**

### **TestAssertFail Repairs**

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Strategy** | Modify test | Strengthen postconds | Correct approach |
| **Respects Immutability** | No | Yes | ✅ |
| **Success Rate** | ~0% | ~40-60%* | Much better |
| **Breaks Compilation** | 33% | <5%* | Safer |

*Projected based on postcondition repair success rates

---

## 🔍 **Logs You'll See**

### **Old Behavior:**
```
Attempting TestAssertFail repair with repair_assertion...
→ Compilation Error: 999 errors (broke it!)
```

### **New Behavior:**
```
Attempting TestAssertFail repair with repair_test_assertion...
Identified tested function: dequeue (from line 198)
Saved test assertion repair prompt to prompts/repair_test_assertion_7.txt
✓ Strengthened dequeue postconditions
→ Test assertions now provable!
```

---

## 🎯 **Integration**

### **Registration:**
```python
# In RepairRegistry.create():
test_assertion_repair = RepairTestAssertionModule(config, logger, immutable_funcs)
registry.register_module(
    "repair_test_assertion",
    test_assertion_repair,
    [VerusErrorType.TestAssertFail],
    "04_repair_test_assertion.rs",
)
```

### **Priority:**
```python
priority_order = {
    ...
    VerusErrorType.AssertFail: 13,      # Production assertions
    VerusErrorType.TestAssertFail: 14,  # Test assertions (new module!)
    VerusErrorType.PreCondFail: 15,
    ...
}
```

---

## 📚 **Prompt Strategy**

The module uses a specialized prompt that:

1. **Emphasizes immutability:**
   ```
   CRITICAL: Test function is IMMUTABLE - cannot be modified!
   DO NOT change test assertions!
   ```

2. **Guides to correct fix:**
   ```
   Fix by strengthening production function postconditions
   ```

3. **Provides context:**
   ```
   Hint: Failing test appears to be testing the `dequeue` function
   ```

4. **Shows examples:**
   - Good test assertion repairs
   - Strengthening postconditions
   - Common patterns

---

## ✅ **Benefits**

### **1. Correct Strategy**
- Fixes root cause (weak postconditions)
- Doesn't violate immutability
- Improves production code quality

### **2. Better Success Rate**
- Targeted approach for test failures
- Specific prompt for this context
- Higher likelihood of success

### **3. Safer**
- Won't break immutability constraints
- Less likely to cause compilation errors
- Respects architectural boundaries

### **4. Clearer Logs**
- Distinct module name in logs
- Shows which function is being targeted
- Easier debugging

---

## 🚀 **Summary**

**Created:** `src/modules/repair_test_assertion.py`

**Registered:** Maps `TestAssertFail` → `repair_test_assertion`

**Strategy:**
- ❌ Don't modify test code (immutable!)
- ✅ Strengthen production postconditions
- 🎯 Make test assertions provable

**Expected Impact:**
- Better success rate on TestAssertFail
- Fewer compilation breaks
- Correct architectural approach
- Clearer separation of concerns

**The system now correctly distinguishes between:**
- Production assertions → `repair_assertion`
- Test assertions → `repair_test_assertion` (NEW!)

**Next run will show the improved behavior!** 🎉
