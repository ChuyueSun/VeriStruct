#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;
fn main() {}

verus! {

/// Example demonstrating CORRECT patterns for Option<Box<T>> requires/ensures specifications
///
/// CRITICAL LESSONS from debugging (to avoid generating weak/tautological specs):
///
/// ✅ CORRECT PATTERNS:
/// - Use: old(ptr).is_some() ==> old(ptr).unwrap().well_formed()
/// - Use: ptr.is_some() ==> ptr.unwrap().well_formed()
/// - Use: old(ptr).is_some() as a standalone precondition (when appropriate)
///
/// ❌ WRONG PATTERNS (avoid these tautologies):
/// - DON'T use: old(ptr).is_some() ==> true  (always true, meaningless!)
/// - DON'T use: ptr.is_none() || true  (always true, meaningless!)
/// - DON'T add comments like "due to restrictions, we omit..." - there are NO such restrictions!
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

    /// CORRECT PATTERN: Option methods and ensures clauses
    pub fn add_to_optional(ptr: &mut Option<Box<TreeNode<T>>>, id: u64, data: T)
        requires
            // ✅ CORRECT: Use is_some() and unwrap() (lowercase, standard Rust)
            old(ptr).is_some() ==> old(ptr).unwrap().is_valid(),
        ensures
            // ✅ CORRECT: Use is_some() and unwrap()
            ptr.is_some() ==> ptr.unwrap().is_valid(),
            // ✅ CORRECT: *old(ptr) not old(*ptr)
            // ✅ CORRECT: Inline expression, no let...in
            TreeNode::<T>::opt_to_map(*ptr) =~=
                TreeNode::<T>::opt_to_map(*old(ptr)).insert(id, data)
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
        requires
            old(self).is_valid(),
        ensures
            self.is_valid(),
            // ✅ CORRECT: Use .to_map() explicitly (no View trait, so no @)
            self.to_map() =~= old(self).to_map().insert(id, data),
    {
        if id == self.id {
            self.data = data;
            // TODO: add proof
        } else if id < self.id {
            Self::add_to_optional(&mut self.left, id, data);
            // TODO: add proof
        } else {
            Self::add_to_optional(&mut self.right, id, data);
            // TODO: add proof
        }
    }

    /// CORRECT PATTERN: Complex ensures without let...in
    pub fn remove_max(ptr: &mut Option<Box<TreeNode<T>>>) -> (result: (u64, T))
        requires
            // ✅ CORRECT: is_some() and unwrap()
            old(ptr).is_some(),
            old(ptr).unwrap().is_valid(),
        ensures
            ptr.is_some() ==> ptr.unwrap().is_valid(),
            // ✅ CORRECT: *old(ptr) not old(*ptr)
            // ✅ CORRECT: Inline all expressions, NO let...in syntax
            TreeNode::<T>::opt_to_map(*ptr) =~=
                TreeNode::<T>::opt_to_map(*old(ptr)).remove(result.0),
            TreeNode::<T>::opt_to_map(*old(ptr)).dom().contains(result.0),
            TreeNode::<T>::opt_to_map(*old(ptr))[result.0] == result.1,
            forall |elem| TreeNode::<T>::opt_to_map(*old(ptr)).dom().contains(elem) ==>
                result.0 >= elem,
    {
        let mut tmp = None;
        std::mem::swap(&mut tmp, ptr);
        let mut boxed = tmp.unwrap();

        // ✅ CORRECT: is_none() method
        if boxed.right.is_none() {
            *ptr = boxed.left;
            // TODO: add proof
            return (boxed.id, boxed.data);
        } else {
            let (max_id, max_data) = TreeNode::<T>::remove_max(&mut boxed.right);
            // TODO: add proof
            *ptr = Some(boxed);
            return (max_id, max_data);
        }
    }
}

}
