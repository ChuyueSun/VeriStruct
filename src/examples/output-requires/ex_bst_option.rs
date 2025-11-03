#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;
fn main() {}

verus! {

/// Example demonstrating CORRECT patterns for Option<Box<T>> requires/ensures specifications
///
/// CRITICAL LESSONS from debugging (to avoid generating weak/tautological specs):
///
/// ✅ CORRECT PATTERNS (use these!):
/// - Use: old(ptr).is_some() ==> old(ptr).unwrap().well_formed()  ← MEANINGFUL!
/// - Use: ptr.is_some() ==> ptr.unwrap().well_formed()           ← MEANINGFUL!
/// - Use: old(ptr).is_some() as standalone precondition
/// - These establish necessary invariants for verification
///
/// ❌ WRONG PATTERNS (never use these tautologies):
/// - DON'T: old(ptr).is_some() ==> true  ← ALWAYS TRUE, USELESS!
/// - DON'T: ptr.is_none() || true        ← ALWAYS TRUE, USELESS!
/// - DON'T: old(ptr).is_none() || (old(ptr).is_some() ==> true)  ← TAUTOLOGY!
///
/// Why tautologies are harmful:
/// - Don't establish needed invariants
/// - Make verification WORSE (add noise without benefit)
/// - Cause verifier to reject specifications
///
/// Additional patterns:
/// - Correct Option methods: is_none(), is_some(), unwrap() (NOT is_None, get_Some_0)
/// - Correct old() placement: *old(ptr) (NOT old(*ptr))
/// - Ensures clauses: inline expressions (NO let...in syntax)
/// - When no View trait: use explicit .to_map() calls (NOT self@)

pub struct TreeNode<T> {
    pub id: u64,
    pub data: T,
    pub left: Option<Box<TreeNode<T>>>,
    pub right: Option<Box<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    /// Spec function to convert tree to map (NO View trait, so use explicit calls)
    pub closed spec fn to_map(self) -> Map<u64, T>
        decreases self,
    {
        TreeNode::<T>::opt_to_map(self.left)
            .union_prefer_right(TreeNode::<T>::opt_to_map(self.right))
            .insert(self.id, self.data)
    }

    pub closed spec fn opt_to_map(tree_opt: Option<Box<TreeNode<T>>>) -> Map<u64, T>
        decreases tree_opt,
    {
        match tree_opt {
            None => Map::empty(),
            Some(tree) => tree.to_map(),
        }
    }

    pub closed spec fn is_valid(self) -> bool
        decreases self
    {
        &&& (forall |elem| TreeNode::<T>::opt_to_map(self.left).dom().contains(elem) ==> elem < self.id)
        &&& (forall |elem| TreeNode::<T>::opt_to_map(self.right).dom().contains(elem) ==> elem > self.id)
        &&& (match self.left {
            Some(left_child) => left_child.is_valid(),
            None => true,
        })
        &&& (match self.right {
            Some(right_child) => right_child.is_valid(),
            None => true,
        })
    }

    /// CORRECT PATTERN: Meaningful specifications for Option parameters
    ///
    /// ✅ This establishes the invariant needed for verification
    /// ❌ DON'T write: old(ptr).is_some() ==> true (meaningless tautology!)
    pub fn add_to_optional(ptr: &mut Option<Box<TreeNode<T>>>, id: u64, data: T)
        // ========== INFERRED SPECIFICATIONS ==========
        requires
            // ✅ MEANINGFUL: Establishes well-formedness when ptr has a value
            old(ptr).is_some() ==> old(ptr).unwrap().is_valid(),
            // ❌ WRONG: old(ptr).is_some() ==> true (would be useless!)
        ensures
            // ✅ MEANINGFUL: Guarantees well-formedness after operation
            ptr.is_some() ==> ptr.unwrap().is_valid(),
            // ❌ WRONG: ptr.is_none() || true (would be useless!)
            // ✅ CORRECT: *old(ptr) not old(*ptr), inline expression not let...in
            TreeNode::<T>::opt_to_map(*ptr) =~=
                TreeNode::<T>::opt_to_map(*old(ptr)).insert(id, data)
        // =============================================
    {
        // ✅ CORRECT: is_none() method (lowercase)
        if ptr.is_none() {
            *ptr = Some(Box::new(TreeNode::<T> {
                id: id,
                data: data,
                left: None,
                right: None,
            }));
        } else {
            let mut tmp = None;
            std::mem::swap(&mut tmp, ptr);
            let mut boxed = tmp.unwrap();
            (&mut *boxed).add(id, data);
            *ptr = Some(boxed);
        }
    }

    /// CORRECT PATTERN: Ensures with explicit method calls (no View trait)
    pub fn add(&mut self, id: u64, data: T)
        // ========== INFERRED SPECIFICATIONS ==========
        requires
            old(self).is_valid(),
        ensures
            self.is_valid(),
            // ✅ CORRECT: Use .to_map() explicitly (no View trait, so no @)
            self.to_map() =~= old(self).to_map().insert(id, data),
        // =============================================
    {
        if id == self.id {
            self.data = data;

            proof {
                assert(!TreeNode::<T>::opt_to_map(self.left).dom().contains(id));
                assert(!TreeNode::<T>::opt_to_map(self.right).dom().contains(id));
            }
        } else if id < self.id {
            Self::add_to_optional(&mut self.left, id, data);

            proof {
                assert(!TreeNode::<T>::opt_to_map(self.right).dom().contains(id));
            }
        } else {
            Self::add_to_optional(&mut self.right, id, data);

            proof {
                assert(!TreeNode::<T>::opt_to_map(self.left).dom().contains(id));
            }
        }
    }

    /// CORRECT PATTERN: Complex ensures with multiple postconditions
    ///
    /// ✅ This shows how to write comprehensive postconditions
    /// ❌ DON'T simplify to just: old(ptr).is_some() ==> true (loses critical information!)
    pub fn remove_max(ptr: &mut Option<Box<TreeNode<T>>>) -> (result: (u64, T))
        // ========== INFERRED SPECIFICATIONS ==========
        requires
            // ✅ TWO preconditions establish what we need:
            old(ptr).is_some(),                    // ptr must have a value
            old(ptr).unwrap().is_valid(),          // that value must be well-formed
            // ❌ WRONG: old(ptr).is_some() ==> true (would be useless!)
        ensures
            // ✅ MULTIPLE postconditions are often needed:
            ptr.is_some() ==> ptr.unwrap().is_valid(),  // Maintains well-formedness
            // ✅ CORRECT: Inline expressions, NO let...in syntax
            TreeNode::<T>::opt_to_map(*ptr) =~=
                TreeNode::<T>::opt_to_map(*old(ptr)).remove(result.0),
            TreeNode::<T>::opt_to_map(*old(ptr)).dom().contains(result.0),
            TreeNode::<T>::opt_to_map(*old(ptr))[result.0] == result.1,
            forall |elem| TreeNode::<T>::opt_to_map(*old(ptr)).dom().contains(elem) ==>
                result.0 >= elem,  // Result is maximum element
        // =============================================
    {
        let mut tmp = None;
        std::mem::swap(&mut tmp, ptr);
        let mut boxed = tmp.unwrap();

        // ✅ CORRECT: is_none() method
        if boxed.right.is_none() {
            *ptr = boxed.left;

            proof {
                assert(TreeNode::<T>::opt_to_map(boxed.right) =~= Map::empty());
                assert(!TreeNode::<T>::opt_to_map(boxed.left).dom().contains(boxed.id));
            }

            return (boxed.id, boxed.data);
        } else {
            let (max_id, max_data) = TreeNode::<T>::remove_max(&mut boxed.right);

            proof {
                assert(!TreeNode::<T>::opt_to_map(boxed.left).dom().contains(max_id));
            }

            *ptr = Some(boxed);
            return (max_id, max_data);
        }
    }
}

}
