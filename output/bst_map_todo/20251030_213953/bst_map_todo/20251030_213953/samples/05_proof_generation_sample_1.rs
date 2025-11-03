#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{

/// A node in the binary search tree containing a key-value pair and optional left/right children.
/// The node maintains BST property: all keys in left subtree < node.key < all keys in right subtree.
struct Node<V> {
    key: u64,                          // The key used for ordering in the BST
    value: V,                          // The value associated with this key
    left: Option<Box<Node<V>>>,        // Optional left child (contains keys smaller than this node's key)
    right: Option<Box<Node<V>>>,       // Optional right child (contains keys larger than this node's key)
}

/// A binary search tree map data structure that maintains key-value pairs in sorted order.
/// Provides efficient insertion, deletion, and lookup operations with O(log n) average complexity.
pub struct TreeMap<V> {
    root: Option<Box<Node<V>>>,        // The root node of the BST, or None if the tree is empty
}

impl<V> Node<V> {
    /// Converts an optional node reference to a map representation.
    /// Returns the mapping from keys to values contained in the node and its subtrees.
    /// For None, returns an empty map; for Some(node), returns the node's map representation.
    spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
        decreases node_opt,
    {
        match node_opt {
            None => Map::empty(),
            Some(node) => node.as_map(),
        }
    }

    /// Converts this node and its entire subtree to a map representation.
    /// Returns a map containing all key-value pairs from this node and its left/right subtrees.
    /// The map is formed by taking the union of left subtree, right subtree, and this node's key-value pair.
    spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
         Node::<V>::optional_as_map(self.left)
          .union_prefer_right(Node::<V>::optional_as_map(self.right))
          .insert(self.key, self.value)
    }
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
        match self.root {
            None => Map::empty(),
            Some(ref node) => (*node).as_map(),
        }
    }
}

impl<V> Node<V> {
    /// Checks if this node and its subtrees satisfy the binary search tree property.
    /// Returns true if all keys in left subtree are less than this node's key,
    /// all keys in right subtree are greater than this node's key, and both subtrees are well-formed.
    spec fn well_formed(self) -> bool
        decreases self
    {
        &&& (forall |elem| Node::<V>::optional_as_map(self.left).dom().contains(elem) ==> elem < self.key)
        &&& (forall |elem| Node::<V>::optional_as_map(self.right).dom().contains(elem) ==> elem > self.key)
        &&& (match self.left {
            Some(left_node) => left_node.well_formed(),
            None => true,
        })
        &&& (match self.right {
            Some(right_node) => right_node.well_formed(),
            None => true,
        })
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
    /// Requires: Nothing.
    /// Ensures: The returned TreeMap represents an empty map with no key-value pairs and is well-formed.
    pub fn new() -> (s: Self)
        requires
            true
        ensures
            s@ == Map::empty(),
            s.well_formed()
    {
        TreeMap::<V> { root: None }
    }
}

impl<V> Node<V> {
    /// Inserts a key-value pair into an optional node, creating a new node if None.
    ///
    /// Requires: If the node exists, it must be well-formed.
    /// Ensures: The map representation of the node equals the original map with the key-value pair inserted.
    fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).insert(key, value)
    {
        if node.is_none() {
            // Create a new leaf node if the current position is empty
            *node = Some(Box::new(Node::<V> {
                key: key,
                value: value,
                left: None,
                right: None,
            }));
        } else {
            // Extract the existing node, insert into it, then put it back
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            (&mut *boxed_node).insert(key, value);

            *node = Some(boxed_node);
        }
    }

    /// Inserts a key-value pair into this node's subtree, maintaining BST properties.
    ///
    /// Requires: This node must be well-formed.
    /// Ensures: The node remains well-formed after insertion, and its map representation equals the original map with the key-value pair inserted.
    fn insert(&mut self, key: u64, value: V)
        requires
            old(self).well_formed()
        ensures
            self.well_formed(),
            self.as_map() =~= old(self).as_map().insert(key, value)
    {
        if key == self.key {
            // Update the value for an existing key
            self.value = value;

            // Proof assertions to help the verifier understand BST invariants
            assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
            assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
        } else if key < self.key {
            // Insert into left subtree for smaller keys
            Self::insert_into_optional(&mut self.left, key, value);

            // Proof assertion: key cannot be in right subtree due to BST property
            assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
        } else {
            // Insert into right subtree for larger keys
            Self::insert_into_optional(&mut self.right, key, value);

            // Proof assertion: key cannot be in left subtree due to BST property
            assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
        }
    }
}

impl<V> TreeMap<V> {
    /// Inserts a key-value pair into the TreeMap.
    ///
    /// Requires: The tree is initially well-formed.
    /// Ensures: The tree remains well-formed and its view equals the original view with the key-value pair inserted.
    pub fn insert(&mut self, key: u64, value: V)
        requires
            old(self).well_formed()
        ensures
            self.well_formed(),
            self@ =~= old(self)@.insert(key, value)
    {
        // Use the type invariant to establish that the tree is well-formed
        proof {
            use_type_invariant(&*self);
            // By the ensures clauses of insert_into_optional and insert,
            // the map representation of the tree after insertion satisfies:
            //     self@ == old(self)@.insert(key, value)
            // This follows from the definition of as_map and the correctness of the helper functions.
            assert(self@ =~= old(self)@.insert(key, value));
        }

        // Extract the root, perform insertion, then restore it
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }
}

impl<V> Node<V> {
    /// Deletes a key from an optional node, handling the case where the node might not exist.
    ///
    /// Requires: If the node exists, it must be well-formed.
    /// Ensures: The map representation of the node equals the original map with the key removed.
    fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(key)
    {
        if node.is_some() {
            // Extract the node to work with it
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            if key == boxed_node.key {
                // Found the key to delete - need to handle node removal
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));

                if boxed_node.left.is_none() {
                    // No left child, replace with right child
                    *node = boxed_node.right;
                } else {
                    if boxed_node.right.is_none() {
                        // No right child, replace with left child
                        *node = boxed_node.left;
                    } else {
                        // Both children exist, replace with rightmost key from left subtree
                        let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.left);
                        boxed_node.key = popped_key;
                        boxed_node.value = popped_value;
                        *node = Some(boxed_node);
                    }
                }
            } else if key < boxed_node.key {
                // Key is in left subtree
                assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));
                Node::<V>::delete_from_optional(&mut boxed_node.left, key);
                *node = Some(boxed_node);
            } else {
                // Key is in right subtree
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                Node::<V>::delete_from_optional(&mut boxed_node.right, key);
                *node = Some(boxed_node);
            }
        }
    }

    /// Deletes and returns the rightmost (largest) key-value pair from a subtree.
    /// Used as a helper for deletion when a node has both left and right children.
    ///
    /// Requires: The node exists and is well-formed.
    /// Ensures: The map representation of the node equals the original map with the returned key removed,
    ///          the returned key was in the original map, it is the maximum key, and its associated value is returned.
    fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
        requires
            old(node).is_some() && old(node).unwrap().well_formed()
        ensures
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(result.0),
            Node::<V>::optional_as_map(*old(node)).dom().contains(result.0),
            Node::<V>::optional_as_map(*old(node))[result.0] == result.1,
            forall |elem: u64| Node::<V>::optional_as_map(*old(node)).dom().contains(elem) ==> elem <= result.0
    {
        // Extract the node to work with it
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        let mut boxed_node = tmp.unwrap();

        if boxed_node.right.is_none() {
            // This is the rightmost node, return its key-value and replace with left subtree
            *node = boxed_node.left;
            assert(Node::<V>::optional_as_map(boxed_node.right) =~= Map::empty());
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(boxed_node.key));
            return (boxed_node.key, boxed_node.value);
        } else {
            // Continue searching in the right subtree for the rightmost node
            let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.right);
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(popped_key));
            *node = Some(boxed_node);
            return (popped_key, popped_value);
        }
    }
}

impl<V> TreeMap<V> {
    /// Deletes a key from the TreeMap.
    ///
    /// Requires: The tree is initially well-formed.
    /// Ensures: The tree remains well-formed and its view equals the original view with the key removed.
    pub fn delete(&mut self, key: u64)
        requires
            old(self).well_formed()
        ensures
            self.well_formed(),
            self@ =~= old(self)@.remove(key)
    {
        // Use the type invariant to establish that the tree is well-formed
        proof { use_type_invariant(&*self); }

        // Extract the root, perform deletion, then restore it
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::delete_from_optional(&mut root, key);
        self.root = root;
    }
}

impl<V> Node<V> {
    /// Looks up a key in an optional node, handling the case where the node might not exist.
    ///
    /// Requires: If the node exists, it must be well-formed.
    /// Ensures: Returns Some(reference to value) if the key exists in the original map, None otherwise.
    fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
        requires
            (node.is_some() ==> node.as_ref().unwrap().well_formed())
        ensures
            ret == (if Node::<V>::optional_as_map(*old(node)).dom().contains(key) {
                        Some(Node::<V>::optional_as_map(*old(node))[key])
                    } else {
                        None
                    })
    {
        match node {
            None => None,
            Some(node) => {
                node.get(key)
            }
        }
    }

    /// Looks up a key in this node's subtree using BST search.
    ///
    /// Requires: This node must be well-formed.
    /// Ensures: Returns Some(reference to value) if the key exists in the subtree, None otherwise.
    fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed()
        ensures
            ret == (if self.as_map().dom().contains(key) { Some(self.as_map()[key]) } else { None })
    {
        if key == self.key {
            // Found the key at this node
            Some(&self.value)
        } else if key < self.key {
            // Search in left subtree for smaller keys
            proof { assert(!Node::<V>::optional_as_map(self.right).dom().contains(key)); }
            Self::get_from_optional(&self.left, key)
        } else {
            // Search in right subtree for larger keys
            proof { assert(!Node::<V>::optional_as_map(self.left).dom().contains(key)); }
            Self::get_from_optional(&self.right, key)
        }
    }
}

impl<V> TreeMap<V> {
    /// Looks up a key in the TreeMap.
    ///
    /// Requires: The tree is well-formed.
    /// Ensures: Returns Some(reference to value) if the key exists in the tree, None otherwise.
    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed()
        ensures
            ret == (if self@.dom().contains(key) { Some(self@[key]) } else { None })
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
/// Requires: The input value v must be less than u64::MAX - 10 to avoid overflow.
/// Ensures: All operations complete successfully and assertions hold.
fn test(v: u64)
requires
    v < u64::MAX - 10
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
/// Requires: Nothing specific (no preconditions needed).
/// Ensures: Two new key-value pairs are inserted into the tree map.
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 5
