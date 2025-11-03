#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

// Import the Node module (ground truth version with all specs)
use node::Node;

verus!{

/// A binary search tree map data structure that maintains key-value pairs in sorted order.
/// Provides efficient insertion, deletion, and lookup operations with O(log n) average complexity.
pub struct TreeMap<V> {
    root: Option<Box<Node<V>>>,        // The root node of the BST, or None if the tree is empty
}

impl<V> TreeMap<V> {
    /// Returns the map representation of the entire tree.
    /// Delegates to the optional_as_map function to convert the root node to a map.
    pub closed spec fn as_map(self) -> Map<u64, V> {
        Node::<V>::optional_as_map(self.root)
    }
}

/// Implementation of the View trait for TreeMap to provide a view of the tree as a map.
/// This allows the TreeMap to be treated as a Map<u64, V> in specifications.
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    /// Returns the view of this TreeMap as a Map, enabling the use of @ syntax.
    open spec fn view(&self) -> Map<u64, V> {
        self.as_map()
    }
}

impl<V> TreeMap<V> {
    /// Type invariant for TreeMap that ensures the entire tree maintains BST properties.
    /// Returns true if the root node (if exists) and all its descendants are well-formed according to BST rules.
    #[verifier::type_invariant]
    spec fn well_formed(self) -> bool {
        match self.root {
            Some(node) => node.well_formed(),
            None => true,
        }
    }
}

impl<V> TreeMap<V> {
    /// Creates a new empty TreeMap.
    ///
    /// Requires: Nothing (unconditional)
    /// Ensures: The returned TreeMap represents an empty map with no key-value pairs
    ///          and is well_formed()
    pub fn new() -> (s: Self)
        ensures
            s.well_formed(),
            s.as_map().dom().len() == 0,
    {
        TreeMap::<V> { root: None }
    }

    /// Inserts a key-value pair into the TreeMap.
    ///
    /// Requires: Nothing (the tree maintains its invariants automatically)
    /// Ensures:
    ///   • The tree remains well_formed() after insertion
    ///   • self.as_map() =~= old(self).as_map().insert(key, value)
    pub fn insert(&mut self, key: u64, value: V)
        ensures
            self.well_formed(),
            self.as_map() =~= old(self).as_map().insert(key, value),
    {
        // Use the type invariant to establish that the tree is well-formed
        proof {
            use_type_invariant(&*self);
        }

        // Extract the root, perform insertion, then restore it
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }

    /// Deletes a key from the TreeMap.
    ///
    /// Requires: Nothing (the tree maintains its invariants automatically)
    /// Ensures:
    ///   • The tree remains well_formed() after deletion
    ///   • self.as_map() =~= old(self).as_map().remove(key)
    pub fn delete(&mut self, key: u64)
        ensures
            self.well_formed(),
            self.as_map() =~= old(self).as_map().remove(key),
    {
        // Use the type invariant to establish that the tree is well-formed
        proof { use_type_invariant(&*self); }

        // Extract the root, perform deletion, then restore it
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::delete_from_optional(&mut root, key);
        self.root = root;
    }

    /// Looks up a key in the TreeMap.
    ///
    /// Requires: Nothing (the tree maintains its invariants automatically)
    /// Ensures:
    ///   • The tree remains well_formed()
    ///   • If the key is in the map, returns Some(&value) where value = old(self).as_map()[key]
    ///   • Otherwise, returns None
    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        ensures
            self.well_formed(),
            match ret {
                Some(r) => old(self).as_map().dom().contains(key)
                           && *r == old(self).as_map()[key],
                None => !old(self).as_map().dom().contains(key)
            },
    {
        // Use the type invariant to establish that the tree is well-formed
        proof { use_type_invariant(&*self); }
        Node::<V>::get_from_optional(&self.root, key)
    }
}

/*
TEST CODE BELOW
*/

/// Test function demonstrating basic TreeMap operations.
///
/// Requires: The input value v must be less than u64::MAX - 10 to avoid overflow
/// Ensures: The assertions about lookups hold (no panic or contradiction)
fn test(v: u64)
    requires
        v < u64::MAX - 10,
    ensures
        true,
{
    // Create an empty tree map
    let mut tree_map = TreeMap::<bool>::new();

    // Insert key v with value false
    tree_map.insert(v, false);

    // Insert key v+1 with value false
    tree_map.insert(v + 1, false);

    // Update key v with value true (overwrites previous false)
    tree_map.insert(v, true);

    // Delete key v from the map
    tree_map.delete(v);

    // Look up deleted key v - should return None
    let elem17 = tree_map.get(v);

    // Look up existing key v+1 - should return Some(&false)
    let elem18 = tree_map.get(v + 1);

    // Verify the results
    assert(elem17.is_none());
    assert(elem18 == Some(&false));

    // Continue testing with additional operations
    test2(tree_map, v + 2, v + 3);
}

/// Additional test function that takes a TreeMap and inserts two more key-value pairs.
/// Demonstrates that TreeMap can be passed by value and modified.
///
/// Requires: Nothing specific (no preconditions needed)
/// Ensures: Does not panic; two new keys are inserted
fn test2(tree_map: TreeMap<bool>, key1: u64, key2: u64)
    ensures
        true,
{
    // Take ownership of the tree map and make it mutable
    let mut tree_map = tree_map;

    // Insert two new keys with true values
    tree_map.insert(key1, true);
    tree_map.insert(key2, true);
}


/// Main function - entry point for the program.
/// Currently empty as this is a library implementation for testing purposes.
fn main() { }
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
