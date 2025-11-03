// Minimal example focusing ONLY on Option<Box<T>> specification patterns
// Shows CORRECT patterns to avoid generating tautologies

use vstd::prelude::*;
verus! {

pub struct TreeNode<T> { pub id: u64, pub data: T, pub left: Option<Box<TreeNode<T>>>, pub right: Option<Box<TreeNode<T>>>, }
impl<T> TreeNode<T> {
    pub closed spec fn opt_to_map(t: Option<Box<TreeNode<T>>>) -> Map<u64, T> { Map::empty() /* simplified */ }
    pub closed spec fn is_valid(self) -> bool { true /* simplified */ }

    // ✅ CORRECT PATTERN for Option<Box<T>> parameter
    // Establishes meaningful invariants, NOT tautologies
    pub fn add_to_optional(ptr: &mut Option<Box<TreeNode<T>>>, id: u64, data: T)
        // ========== INFERRED SPECIFICATIONS ==========
        requires
            old(ptr).is_some() ==> old(ptr).unwrap().is_valid(),  // ✅ Meaningful!
            // ❌ AVOID: old(ptr).is_some() ==> true  (tautology, makes verification worse!)
        ensures
            ptr.is_some() ==> ptr.unwrap().is_valid(),            // ✅ Meaningful!
            // ❌ AVOID: ptr.is_none() || true  (always true, useless!)
            TreeNode::<T>::opt_to_map(*ptr) =~= TreeNode::<T>::opt_to_map(*old(ptr)).insert(id, data)
        // =============================================
    {
        if ptr.is_none() {
            *ptr = Some(Box::new(TreeNode::<T> { id, data, left: None, right: None }));
        } else {
            let mut tmp = None;
            std::mem::swap(&mut tmp, ptr);
            (&mut *tmp.unwrap()).add(id, data);
            *ptr = tmp;
        }
    }

    // ✅ CORRECT PATTERN with multiple preconditions
    // When function requires Option to be Some, state both conditions clearly
    pub fn remove_max(ptr: &mut Option<Box<TreeNode<T>>>) -> (result: (u64, T))
        // ========== INFERRED SPECIFICATIONS ==========
        requires
            old(ptr).is_some(),                  // Must have a value
            old(ptr).unwrap().is_valid(),        // That value must be well-formed
            // ❌ AVOID: old(ptr).is_some() ==> true  (would be meaningless!)
        ensures
            ptr.is_some() ==> ptr.unwrap().is_valid(),
            TreeNode::<T>::opt_to_map(*ptr) =~= TreeNode::<T>::opt_to_map(*old(ptr)).remove(result.0),
            TreeNode::<T>::opt_to_map(*old(ptr)).dom().contains(result.0),
            TreeNode::<T>::opt_to_map(*old(ptr))[result.0] == result.1,
        // =============================================
    {
        let mut tmp = None;
        std::mem::swap(&mut tmp, ptr);
        let boxed = tmp.unwrap();
        if boxed.right.is_none() {
            *ptr = boxed.left;
            (boxed.id, boxed.data)
        } else {
            let (max_id, max_data) = TreeNode::<T>::remove_max(&mut boxed.right);
            *ptr = Some(boxed);
            (max_id, max_data)
        }
    }

    pub fn add(&mut self, id: u64, data: T) { /* stub */ }
}

} // verus
fn main() {}
