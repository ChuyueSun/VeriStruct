# Verus Map Knowledge

## CRITICAL: Extensional Equality (=~=) for Maps ⚠️

**ALWAYS use `=~=` for comparing Map types in specifications:**

- ✅ **CORRECT**: `map1 =~= map2`
- ❌ **WRONG**: `map1 == map2`

**Why**: `=~=` is extensional equality that Verus reasons about effectively. Using `==` for maps will often fail verification even when the maps are logically equal.

### Common Mistakes to Avoid

```rust
// ❌ WRONG - will fail verification:
ensures self.as_map() == old(self).as_map().insert(key, value)
ensures result_map == original_map.remove(key)
ensures my_map_fn(*data) == my_map_fn(*old(data)).update(...)

// ✅ CORRECT - will verify:
ensures self.as_map() =~= old(self).as_map().insert(key, value)
ensures result_map =~= original_map.remove(key)
ensures my_map_fn(*data) =~= my_map_fn(*old(data)).update(...)
```

### Examples in Different Contexts

```rust
// In struct methods with View
impl<K, V> MyMap<K, V> {
    pub fn insert(&mut self, key: K, value: V)
        ensures
            self@ =~= old(self)@.insert(key, value)  // ✅ Use =~=
}

// In helper functions
fn update_mapping<K, V>(map: &mut Map<K, V>, key: K, value: V)
    ensures
        *map =~= old(map).insert(key, value)  // ✅ Use =~=

// In functions with custom map conversions
fn modify_structure<T>(data: &mut SomeType<T>, key: u64, value: T)
    ensures
        data.to_map() =~= old(data).to_map().insert(key, value)  // ✅
```

---

## Map<K, V> - Mathematical Map Type

Map<K, V> is a mathematical map type used in specifications:

### Construction
- `Map::empty()` - Create empty map
- `Map::new(...)` - Create map (if supported)

### Operations (Return New Map)
- `map.insert(key, value)` - Returns new map with key→value added/updated
- `map.remove(key)` - Returns new map with key removed (if it existed)
- `map.union_prefer_right(other)` - Union of two maps, preferring values from right on conflicts

### Queries
- `map[key]` - Get value for key (requires key exists in domain)
- `map.dom()` - Returns `Set<K>` of all keys in the map
- `map.dom().contains(key)` - Check if key exists in map

### Common Patterns

#### Checking Key Existence
```rust
// Check if key exists
if map.dom().contains(key) {
    let value = map[key];  // Safe - key is in domain
}

// In specifications
requires map.dom().contains(key)
ensures result == map[key]
```

#### Map Updates in Postconditions
```rust
// Insertion
ensures self@ =~= old(self)@.insert(key, value)

// Deletion
ensures self@ =~= old(self)@.remove(key)

// Conditional update
ensures
    if condition {
        self@ =~= old(self)@.insert(key, new_value)
    } else {
        self@ =~= old(self)@
    }
```

#### Map Equality Assertions
```rust
// In proof blocks
assert(map1 =~= map2);  // ✅ Correct

// In ensures
ensures
    map1 =~= map2,
    map1.dom() == map2.dom(),  // Set equality uses ==
```

### Key-Value Relationships
```rust
// Accessing values
ensures
    result_map.dom().contains(key) ==> result_map[key] == value

// Comparing with original
ensures
    forall |k| result_map.dom().contains(k) ==>
        result_map[k] == original_map[k]
```

---

## Important Notes

### Equality Operators Summary

| Type | Equality Operator | Example |
|------|------------------|---------|
| **Map<K, V>** | `=~=` | `map1 =~= map2` |
| **Seq<T>** | `=~=` | `seq1 =~= seq2` |
| **Set<T>** | `==` or `=~=` | Both work for sets |
| **Primitive types** | `==` | `x == y` |
| **Struct fields** | `==` | `self.field == value` |

### When to Use Each

- **`=~=`**: Use for Map, Seq, and other collection types in specifications
- **`==`**: Use for primitive types, booleans, integers, and struct field comparisons

### Common Verification Failures

If you see "postcondition not satisfied" with map comparisons:
1. Check if you used `==` instead of `=~=`
2. Verify the map operations (insert/remove) are correct
3. Ensure all required keys are in the domain

---

## Complete Example: Data Structure with Map Representation

```rust
// Generic example (not specific to any benchmark)
pub struct MyDataStructure<K, V> {
    // ... internal fields ...
}

impl<K, V> MyDataStructure<K, V> {
    // Specification function that converts structure to map
    pub spec fn to_map(self) -> Map<K, V> {
        // ... conversion logic ...
    }

    pub fn insert(&mut self, key: K, value: V)
        requires
            old(self).well_formed(),
        ensures
            self.well_formed(),
            self.to_map() =~= old(self).to_map().insert(key, value)  // ✅ Use =~=
    {
        // ... implementation ...
    }

    pub fn remove(&mut self, key: K)
        requires
            old(self).well_formed(),
        ensures
            self.well_formed(),
            self.to_map() =~= old(self).to_map().remove(key)  // ✅ Use =~=
    {
        // ... implementation ...
    }

    pub fn get(&self, key: K) -> (result: Option<&V>)
        requires
            self.well_formed(),
        ensures
            result == (if self.to_map().dom().contains(key) {
                Some(&self.to_map()[key])
            } else {
                None
            })
    {
        // ... implementation ...
    }
}
```

**Key Point**: All insert/remove operations use `=~=` to compare map states before and after!
