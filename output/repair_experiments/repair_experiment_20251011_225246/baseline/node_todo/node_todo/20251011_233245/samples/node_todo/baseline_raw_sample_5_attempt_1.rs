#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{

pub open spec fn optional_well_formed<V>(node_opt: Option<Box<Node<V>>>) -> bool {
    match node_opt {
        None => true,
        Some(n) => n.well_formed(),
    }
}

/// A node in the binary search tree containing a key-value pair and optional left/right children.
/// The node maintains BST property: all keys in left subtree < node.key < all keys in right subtree.
pub struct Node<V> {
    pub key: u64,
    pub value: V,
    pub left: Option<Box<Node<V>>>,
    pub right: Option<Box<Node<V>>>,
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
         Self::optional_as_map(self.left)
          .union_prefer_right(Self::optional_as_map(self.right))
          .insert(self.key, self.value)
    }

    /// Checks if this node and its subtrees satisfy the binary search tree property.
    /// Returns true if all keys in left subtree are less than this node's key,
    /// all keys in right subtree are greater than this node's key, and both subtrees are well-formed.
    pub open spec fn well_formed(self) -> bool
        decreases self
    {
        optional_well_formed(self.left)
        && optional_well_formed(self.right)
        && (forall |k| Self::optional_as_map(self.left).dom().contains(k) ==> k < self.key)
        && (forall |k| Self::optional_as_map(self.right).dom().contains(k) ==> k > self.key)
    }

    /// Inserts a key-value pair into an optional node, creating a new node if None.
    ///
    /// Requires: If the node exists, it must be well-formed (satisfy BST properties).
    /// Ensures: The resulting node (if exists) is well-formed, and the map representation
    ///          equals the original map with the key-value pair inserted.
    pub fn insert_into_optional(
        node: &mut Option<Box<Node<V>>>,
        key: u64,
        value: V
    )
        requires
            optional_well_formed(*node),
        ensures
            optional_well_formed(*node)
            && Self::optional_as_map(*node) == old(Self::optional_as_map(*node)).insert(key, value)
    {
        if node.is_none() {
            *node = Some(Box::new(Node::<V> {
                key: key,
                value: value,
                left: None,
                right: None,
            }));
        } else {
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
    /// Ensures: The node remains well-formed after insertion, and the map representation
    ///          equals the original map with the key-value pair inserted.
    pub fn insert(
        &mut self,
        key: u64,
        value: V
    )
        requires
            self.well_formed(),
        ensures
            self.well_formed()
            && self.as_map() == old(self.as_map()).insert(key, value)
    {
        if key == self.key {
            self.value = value;
            assert(!Self::optional_as_map(self.left).dom().contains(key));
            assert(!Self::optional_as_map(self.right).dom().contains(key));
        } else if key < self.key {
            Self::insert_into_optional(&mut self.left, key, value);
            assert(!Self::optional_as_map(self.right).dom().contains(key));
        } else {
            Self::insert_into_optional(&mut self.right, key, value);
            assert(!Self::optional_as_map(self.left).dom().contains(key));
        }
    }

    /// Deletes a key from an optional node, handling the case where the node might not exist.
    ///
    /// Requires: If the node exists, it must be well-formed.
    /// Ensures: The resulting node (if exists) is well-formed, and the map representation
    ///          equals the original map with the key removed.
    pub fn delete_from_optional(
        node: &mut Option<Box<Node<V>>>,
        key: u64
    )
        requires
            optional_well_formed(*node),
        ensures
            optional_well_formed(*node)
            && Self::optional_as_map(*node) == old(Self::optional_as_map(*node)).remove(key)
    {
        if node.is_some() {
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            if key == boxed_node.key {
                assert(!Self::optional_as_map(boxed_node.left).dom().contains(key));
                assert(!Self::optional_as_map(boxed_node.right).dom().contains(key));

                if boxed_node.left.is_none() {
                    *node = boxed_node.right;
                } else {
                    if boxed_node.right.is_none() {
                        *node = boxed_node.left;
                    } else {
                        let (popped_key, popped_value) = Self::delete_rightmost(&mut boxed_node.left);
                        boxed_node.key = popped_key;
                        boxed_node.value = popped_value;
                        *node = Some(boxed_node);
                    }
                }
            } else if key < boxed_node.key {
                assert(!Self::optional_as_map(boxed_node.right).dom().contains(key));
                Self::delete_from_optional(&mut boxed_node.left, key);
                *node = Some(boxed_node);
            } else {
                assert(!Self::optional_as_map(boxed_node.left).dom().contains(key));
                Self::delete_from_optional(&mut boxed_node.right, key);
                *node = Some(boxed_node);
            }
        }
    }

    /// Deletes and returns the rightmost (largest) key-value pair from a subtree.
    /// Used as a helper for deletion when a node has both left and right children.
    ///
    /// Requires: The node must exist and be well-formed.
    /// Ensures: The resulting node (if exists) is well-formed, the returned key-value pair was
    ///          in the original tree, the key was the largest in the tree, and the map representation
    ///          equals the original map with that key removed.
    pub fn delete_rightmost(
        node: &mut Option<Box<Node<V>>>,
    ) -> (popped: (u64, V))
        requires
            optional_well_formed(*node),
            node.is_some(),
        ensures
            optional_well_formed(*node)
            && let (retk, retv) = result in
               old(Self::optional_as_map(*node)).dom().contains(retk)
               && forall |k| old(Self::optional_as_map(*node)).dom().contains(k) ==> k <= retk
               && Self::optional_as_map(*node) == old(Self::optional_as_map(*node)).remove(retk)
               && old(Self::optional_as_map(*node))[retk] == retv
    {
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        let mut boxed_node = tmp.unwrap();

        if boxed_node.right.is_none() {
            *node = boxed_node.left;
            assert(Self::optional_as_map(boxed_node.right) =~= Map::empty());
            assert(!Self::optional_as_map(boxed_node.left).dom().contains(boxed_node.key));
            return (boxed_node.key, boxed_node.value);
        } else {
            let (popped_key, popped_value) = Self::delete_rightmost(&mut boxed_node.right);
            assert(!Self::optional_as_map(boxed_node.left).dom().contains(popped_key));
            *node = Some(boxed_node);
            return (popped_key, popped_value);
        }
    }

    /// Looks up a key in an optional node, handling the case where the node might not exist.
    ///
    /// Requires: If the node exists, it must be well-formed.
    /// Ensures: Returns Some(reference to value) if the key exists in the subtree, None otherwise.
    pub fn get_from_optional(
        node: &Option<Box<Node<V>>>,
        key: u64
    ) -> (ret: Option<&V>)
        requires
            optional_well_formed(*node),
        ensures
            match result {
                None => !old(Self::optional_as_map(*node)).dom().contains(key),
                Some(r) => old(Self::optional_as_map(*node)).dom().contains(key)
                           && *r == old(Self::optional_as_map(*node)).index(key)
            }
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
    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed(),
        ensures
            match result {
                None => !old(self.as_map()).dom().contains(key),
                Some(r) => old(self.as_map()).dom().contains(key) && *r == old(self.as_map()).index(key)
            }
    {
        if key == self.key {
            Some(&self.value)
        } else if key < self.key {
            proof { assert(!Self::optional_as_map(self.right).dom().contains(key)); }
            Self::get_from_optional(&self.left, key)
        } else {
            proof { assert(!Self::optional_as_map(self.left).dom().contains(key)); }
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
    v < u64::MAX - 10,
{
    let mut root: Option<Box<Node<bool>>> = None;

    Node::insert_into_optional(&mut root, v, false);
    Node::insert_into_optional(&mut root, v + 1, false);
    Node::insert_into_optional(&mut root, v, true);
    let val1 = Node::get_from_optional(&root, v);
    let val2 = Node::get_from_optional(&root, v + 1);
    Node::delete_from_optional(&mut root, v);
    let val3 = Node::get_from_optional(&root, v);
    let val4 = Node::get_from_optional(&root, v + 1);
}

fn main() { }
}
