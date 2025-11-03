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

            if key == boxed_node.key {
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));

                if boxed_node.left.is_none() {
                    *node = boxed_node.right;
                } else {
                    if boxed_node.right.is_none() {
                        *node = boxed_node.left;
                    } else {
                        proof {
                            assert(boxed_node.left.is_some());
                        }
                        let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.left);
                        boxed_node.key = popped_key;
                        boxed_node.value = popped_value;
                        *node = Some(boxed_node);
                    }
                }
            } else if key < boxed_node.key {
                assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));
                Node::<V>::delete_from_optional(&mut boxed_node.left, key);
                *node = Some(boxed_node);
            } else {
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                Node::<V>::delete_from_optional(&mut boxed_node.right, key);
                *node = Some(boxed_node);
            }
        }
    }

    pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
    {
        proof {
            assert(node.is_some());
        } // Added by AI
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        proof {
            assert(tmp.is_some());
        } // Added by AI
        let mut boxed_node = tmp.unwrap();

        if boxed_node.right.is_none() {
            *node = boxed_node.left;
            assert(Node::<V>::optional_as_map(boxed_node.right) =~= Map::empty());
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(boxed_node.key));
            return (boxed_node.key, boxed_node.value);
        } else {
            proof {
                assert(boxed_node.right.is_some());
            }
            let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.right);
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(popped_key));
            *node = Some(boxed_node);
            return (popped_key, popped_value);
        }
    }

    pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
    {
        match node {
            None => None,
            Some(node) => {
                node.get(key)
            }
        }
    }

    pub fn get(&self, key: u64) -> (ret: Option<&V>)
    {
        if key == self.key {
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

fn test_node(v: u64)
requires
    v < u64::MAX - 10,
{
    let mut root: Option<Box<Node<bool>>>= None;

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

// Final VEval Score: Compilation Error: False, Verified: 7, Errors: 4, Verus Errors: 13
// Verified: 7, Errors: 4, Verus Errors: 13
