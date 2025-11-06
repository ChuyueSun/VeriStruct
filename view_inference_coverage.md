# View Inference Module - Pattern Coverage

## ✅ All Benchmark View Patterns Now Supported

The `view_inference.py` module has been enhanced to handle **all 5 View patterns** found in the benchmarks.

---

## Supported Patterns

### **Pattern 1: `spec fn view`**
**Example:** `bitmap_2_todo.rs`, `bitmap_todo.rs`

```rust
impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        // TODO: Implement the view function
    }
}
```

**Handling:**
- ✅ Detected by: `has_spec_fn_view()`
- ✅ Action: Fill in function body only
- ✅ Preserves: `spec` keyword and function signature

---

### **Pattern 2: `pub closed spec fn view`**
**Example:** `set_from_vec_todo.rs`

```rust
impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // TODO: add requires and ensures
    }
}
```

**Handling:**
- ✅ Detected by: `has_spec_fn_view()` (now supports pub/closed/open modifiers)
- ✅ Action: Fill in function body only
- ✅ Preserves: `pub closed spec` keywords and function signature

---

### **Pattern 3: Empty `impl View for`**
**Example:** `rb_type_invariant_todo.rs`

```rust
impl<T: Copy> View for RingBuffer<T> {
    // TODO: add specification
}
```

**Handling:**
- ✅ Detected by: Neither pattern (empty View trait)
- ✅ Action: Insert complete View trait implementation
- ✅ Generates: `type V = ...` and `closed spec fn view(...)`

---

### **Pattern 4: `impl View for` with TODO in view function**
**Example:** `bst_map_todo.rs`, `treemap_todo.rs`

```rust
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    open spec fn view(&self) -> Map<u64, V> {
        // TODO: add specification
    }
}
```

**Handling:**
- ✅ Detected by: `has_view_trait_with_todo()`
- ✅ Action: Fill in view function body only
- ✅ Preserves: `impl View for`, `type V`, and function signature

---

### **Pattern 5: Complete `impl View for`** (Should NOT modify)
**Example:** Complete benchmarks

```rust
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    open spec fn view(&self) -> Map<u64, V> {
        self.as_map()
    }
}
```

**Handling:**
- ✅ Detected by: NOT detected (complete code, no TODO)
- ✅ Action: Skipped (no modification needed)
- ✅ Correctly ignores complete implementations

---

## Implementation Details

### Detection Methods

1. **`has_spec_fn_view(code)`**
   - Pattern: `[pub] [open|closed] spec fn view(&self) -> Type { ... }`
   - Returns: `(has_spec_fn, struct_name, start_pos, end_pos)`
   - Captures: Function body position for replacement

2. **`has_view_trait_with_todo(code)`**
   - Pattern: `impl View for Struct { type V = ...; [open|closed] spec fn view(...) { TODO } }`
   - Returns: `(has_view_trait, struct_name, start_pos, end_pos)`
   - Detects TODO by: Explicit "TODO" keyword OR empty/minimal body

### Processing Logic

```python
# Detect pattern
has_spec_fn, name1, pos1_s, pos1_e = has_spec_fn_view(code)
has_view_todo, name2, pos2_s, pos2_e = has_view_trait_with_todo(code)

if has_spec_fn:
    # Pattern 1 or 2: Fill in spec fn body
    insert_view_body(code, implementation, pos1_s, pos1_e)

elif has_view_todo:
    # Pattern 4: Fill in View trait's view function body
    insert_view_body(code, implementation, pos2_s, pos2_e)

else:
    # Pattern 3: Insert complete View trait
    insert_view_trait(code, implementation, struct_name)
```

### Surgical Insertion Approach

**Key Innovation:** Ask LLM for implementation only, not full file

**Benefits:**
- ✅ Prevents accidental deletion of `spec` keyword
- ✅ Prevents accidental modification of other code
- ✅ Prevents nested `impl View for` blocks
- ✅ Reduces token usage
- ✅ More reliable and predictable

**LLM Output Formats:**

For Pattern 1-2-4 (fill in body):
```rust
let total_bits = self.bits@.len() * 64;
Seq::new(total_bits, |i: int| {
    get_bit64!(self.bits@[i/64], (i%64) as u64)
})
```

For Pattern 3 (complete trait):
```rust
impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, usize);

    closed spec fn view(&self) -> Self::V {
        (self.ring@, self.ring.len())
    }
}
```

---

## Benchmark Coverage Summary

| Benchmark | Pattern | Status |
|-----------|---------|--------|
| `bitmap_2_todo.rs` | spec fn view | ✅ Supported |
| `bitmap_todo.rs` | spec fn view | ✅ Supported |
| `set_from_vec_todo.rs` | pub closed spec fn view | ✅ Supported |
| `rb_type_invariant_todo.rs` | Empty impl View for | ✅ Supported |
| `bst_map_todo.rs` | impl View for + TODO | ✅ Supported |
| `treemap_todo.rs` | impl View for + TODO | ✅ Supported |

**Total:** 6/6 benchmarks requiring View inference are now supported ✅

---

## Testing

All patterns verified with comprehensive unit tests:
- ✅ Pattern detection
- ✅ Implementation extraction
- ✅ Code insertion
- ✅ Preservation of keywords and structure
- ✅ Rejection of complete (non-TODO) code

---

## Migration Notes

### Before
```python
# Old approach: Return entire file
instruction = "Return the ENTIRE file with View implemented"
response = llm.infer(...)
final_code = parse_llm_response(response)  # Full file, prone to errors
```

### After
```python
# New approach: Return implementation only
instruction = "Return ONLY the view implementation"
response = llm.infer(...)
view_impl = extract_view_implementation(response, is_spec_fn)
final_code = insert_view_body(original_code, view_impl, start, end)  # Surgical
```

---

## Future Enhancements

Potential improvements (not critical):

1. **Auto-detect simple vs complex views** - Skip view_refinement for simple mappings
2. **Better error messages** - If pattern detection fails, suggest which pattern to use
3. **Support custom spec fn names** - Handle `spec fn my_view()` in addition to `spec fn view()`
4. **Validate View type correctness** - Check if `type V` matches function return type

---

## Summary

✅ **All View patterns from benchmarks are now handled correctly**
✅ **Surgical insertion prevents accidental code modifications**
✅ **Comprehensive testing ensures reliability**
✅ **Ready for production use on all benchmark types**
