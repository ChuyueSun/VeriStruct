#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{

/// A node in the binary search tree containing a key-value pair and optional left/right children.
/// The node maintains BST property: all keys in left subtree < node.key < all keys in right subtree.
pub struct Node<V> {
    pub key: u64,                          // The key used for ordering in the BST
    pub value: V,                          // The value associated with this key
    pub left: Option<Box<Node<V>>>,        // Optional left child (contains keys smaller than this node's key)
    pub right: Option<Box<Node<V>>>,       // Optional right child (contains keys larger than this node's key)
}

impl<V> Node<V> {
    /// Converts an optional node reference to a map representation.
    /// Returns the mapping from keys to values contained in the node and its subtrees.
    /// For None, returns an empty map; for Some(node), returns the node's map representation.
    pub open spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
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
    pub open spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
         Node::<V>::optional_as_map(self.left)
          .union_prefer_right(Node::<V>::optional_as_map(self.right))
          .insert(self.key, self.value)
    }

    /// Checks if this node and its subtrees satisfy the binary search tree property.
    /// Returns true if all keys in left subtree are less than this node's key,
    /// all keys in right subtree are greater than this node's key, and both subtrees are well-formed.
    pub open spec fn well_formed(self) -> bool
        decreases self
    {
        &&& (forall |k: u64| Node::<V>::optional_as_map(self.left).dom().contains(k) ==> k < self.key)
        &&& (forall |k: u64| Node::<V>::optional_as_map(self.right).dom().contains(k) ==> k > self.key)
        &&& (match self.left {
            None => true,
            Some(left_box) => left_box.well_formed(),
        })
        &&& (match self.right {
            None => true,
            Some(right_box) => right_box.well_formed(),
        })
    }

    /// Inserts a key-value pair into an optional node, creating a new node if None.
    ///
    /// Requires: If the node exists, it must be well-formed (satisfy BST properties)
    /// Ensures: The resulting node (if exists) is well-formed, and the map representation
    ///          equals the original map with the key-value pair inserted
    pub fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
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
    /// Requires: This node must be well-formed (satisfy BST properties)
    /// Ensures: The node remains well-formed after insertion, and the map representation
    ///          equals the original map with the key-value pair inserted
    pub fn insert(&mut self, key: u64, value: V)
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

    /// Deletes a key from an optional node, handling the case where the node might not exist.
    ///
    /// Requires: If the node exists, it must be well-formed (satisfy BST properties)
    /// Ensures: The resulting node (if exists) is well-formed, and the map representation
    ///          equals the original map with the key removed
    pub fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            node.is_none() || node.unwrap().well_formed(),
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
    /// Requires: The node must exist and be well-formed
    /// Ensures: The resulting node (if exists) is well-formed, the returned key-value pair was
    ///          in the original tree, the key was the largest in the tree, and the map representation
    ///          equals the original map with that key removed
    pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
        requires
            old(node).is_some(),
            old(node).unwrap().well_formed()
        ensures
            node.is_none() || node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
            Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
            Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
            forall |k: u64| Node::<V>::optional_as_map(*old(node)).dom().contains(k) ==> (popped.0 >= k)
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

    /// Looks up a key in an optional node, handling the case where the node might not exist.
    ///
    /// Requires: If the node exists, it must be well-formed
    /// Ensures: Returns Some(reference to value) if the key exists in the subtree, None otherwise
    pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
        requires
            node.is_some() ==> node.unwrap().well_formed()
        ensures
            (ret.is_some() <==> Node::<V>::optional_as_map(*node).dom().contains(key)),
            ret.is_some() ==> Node::<V>::optional_as_map(*node)[key] == *ret.unwrap()
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
    /// Requires: This node must be well-formed
    /// Ensures: Returns Some(reference to value) if the key exists in the subtree, None otherwise
    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed()
        ensures
            (ret.is_some() <==> self.as_map().dom().contains(key)),
            ret.is_some() ==> self.as_map()[key] == *ret.unwrap()
    {
        if key == self.key {
            // Found the key at this node
            Some(&self.value)
        } else if key < self.key {
            proof { assert(!Node::<V>::optional_as_map(self.right).dom().contains(key)); }
            Self::get_from_optional(&self.left, key)
        } else {
            proof { assert(!Node::<V>::optional_as_map(self.left).dom().contains(key)); }
            Self::get_from_optional(&self.right, key)
        }
    }
}

/*
TEST CODE FOR NODE
*/

/// Simple test for Node operations without TreeMap wrapper.
/// Tests basic insert and get operations on an optional node.
fn test_node(v: u64)
requires
    v < u64::MAX - 10
{
    // Start with an empty tree (None)
    let mut root: Option<Box<Node<bool>>>= None;

    // Insert first key
    Node::insert_into_optional(&mut root, v, false);

    // Insert second key
    Node::insert_into_optional(&mut root, v + 1, false);

    // Update first key
    Node::insert_into_optional(&mut root, v, true);

    // Get values
    let val1 = Node::get_from_optional(&root, v);
    let val2 = Node::get_from_optional(&root, v + 1);

    // Delete first key
    Node::delete_from_optional(&mut root, v);

    // Get after delete
    let val3 = Node::get_from_optional(&root, v);
    let val4 = Node::get_from_optional(&root, v + 1);
}

/// Main function - entry point for the program.
fn main() { }
}
