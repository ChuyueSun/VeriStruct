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
    pub open spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
        decreases node_opt,
    {
        match node_opt {
            None => Map::empty(),
            Some(node) => node.as_map(),
        }
    }

    pub open spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
         Node::<V>::optional_as_map(self.left)
          .union_prefer_right(Node::<V>::optional_as_map(self.right))
          .insert(self.key, self.value)
    }

    pub open spec fn well_formed(self) -> bool
        decreases self
    {
        (forall |k: u64| Node::<V>::optional_as_map(self.left).dom().contains(k) ==> k < self.key)
        && (forall |k: u64| Node::<V>::optional_as_map(self.right).dom().contains(k) ==> k > self.key)
        && match self.left {
            None => true,
            Some(l) => l.well_formed(),
        }
        && match self.right {
            None => true,
            Some(r) => r.well_formed(),
        }
    }

    pub fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
    {
        if node.is_none() {
            *node = Some(Box::new(Node::<V> {
                key: key,
                value: value,
                left: None,
                right: None,
            }));
        } else {
            proof {
                assert(node.is_some());
            } // Added by AI
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            (&mut *boxed_node).insert(key, value);

            *node = Some(boxed_node);
        }
    }

    pub fn insert(&mut self, key: u64, value: V)
    {
        if key == self.key {
            self.value = value;
            assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
            assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
        } else if key < self.key {
            Self::insert_into_optional(&mut self.left, key, value);
            assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
        } else {
            Self::insert_into_optional(&mut self.right, key, value);
            assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
        }
    }

    pub fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
    {
        if node.is_some() {
            proof {
                assert(node.is_some());
            }
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            if

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
