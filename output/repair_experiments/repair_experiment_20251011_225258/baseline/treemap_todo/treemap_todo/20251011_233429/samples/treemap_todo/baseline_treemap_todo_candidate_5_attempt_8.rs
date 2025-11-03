#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

// Import the Node module (ground truth version with all specs)
mod node;
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
    pub closed spec fn as_map(self) -> Map<u64, V>
        ensures
            result == Node::<V>::optional_as_map(self.root)
    {
        Node::<V>::optional_as_map(self.root)
    }
}

/// Implementation of the View trait for TreeMap to provide a view of the tree as a map.
/// This allows the TreeMap to be treated as a Map<u64, V> in specifications.
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    /// Returns the view of this TreeMap as a Map, enabling the use of @ syntax.
    open spec fn view(&self) -> Map<u64, V>
        ensures
            result == self.as_map()
    {
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
    pub fn new() -> (s: Self)
        requires
            true,
        ensures
            s.as_map() == Map::empty(),
            s.well_formed(),
    {
        TreeMap::<V> { root: None }
    }

    /// Inserts a key-value pair into the TreeMap.
    ///
    /// Requires: Nothing (the tree maintains its invariants automatically)
    /// Ensures: The tree's map representation equals the original map with the key-value pair inserted
    pub fn insert(&mut self, key: u64, value: V)
        requires
            true,
        ensures
            self.well_formed(),
            self.as_map() == old(self).as_map().insert(key, value),
    {
        proof {
            use_type_invariant(&*self);
        }

        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }

    /// Deletes a key from the TreeMap.
    ///
    /// Requires: Nothing (the tree maintains its invariants automatically)
    /// Ensures: The tree's map representation equals the original map with the key removed
    pub fn delete(&mut self, key: u64)
        requires
            true,
        ensures
            self.well_formed(),
            self.as_map() == old(self).as_map().remove(key),
    {
        proof { use_type_invariant(&*self); }

        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::delete_from_optional(&mut root, key);
        self.root = root;
    }

    /// Looks up a key in the TreeMap.
    ///
    /// Requires: Nothing (the tree maintains its invariants automatically)
    /// Ensures: Returns Some(reference to value) if the key exists in the tree, None otherwise
    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            true,
        ensures
            self.well_formed(),
            self.as_map() == old(self).as_map(),
            (ret.is_Some() ==> old(self).as_map().contains_key(key)
                && *ret.get_Some_0() == old(self).as_map()[key]),
            (ret.is_None() ==> !old(self).as_map().contains_key(key)),
    {
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
/// Ensures: All operations complete successfully and assertions hold
fn test(v: u64)
requires
    v < u64::MAX - 10,
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
/// Ensures: Two new key-value pairs are inserted into the tree map
fn test2(tree_map: TreeMap<bool>, key1: u64, key2: u64) {
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
