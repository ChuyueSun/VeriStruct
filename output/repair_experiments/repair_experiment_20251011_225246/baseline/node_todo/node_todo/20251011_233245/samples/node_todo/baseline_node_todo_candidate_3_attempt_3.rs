#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus! {

// Spec function to check if an optional node is well-formed
pub open spec fn optional_well_formed<V>(node_opt: Option<Box<Node<V>>>) -> bool {
    match node_opt {
        None => true,
        Some(node) => node.well_formed(),
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
    pub open spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
        decreases node_opt,
    {
        match node_opt {
            None => Map::empty(),
            Some(node) => node.as_map(),
        }
    }

    /// Converts this node and its entire subtree to a map representation.
    pub open spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
        Node::<V>::optional_as_map(self.left)
            .union_prefer_right(Node::<V>::optional_as_map(self.right))
            .insert(self.key, self.value)
    }

    /// Checks if this node and its subtrees satisfy the binary search tree property.
    pub open spec fn well_formed(self) -> bool
        decreases self
    {
        let left_ok = match self.left {
            None => true,
            Some(left_node) => left_node.well_formed()
                && left_node.as_map().dom().forall(|k| k < self.key),
        };
        let right_ok = match self.right {
            None => true,
            Some(right_node) => right_node.well_formed()
                && right_node.as_map().dom().forall(|k| k > self.key),
        };
        left_ok && right_ok
    }

    /// Inserts a key-value pair into an optional node, creating a new node if None.
    #[verifier::spec]
    pub fn insert_into_optional(
        node: &mut Option<Box<Node<V>>>,
        key: u64,
        value: V
    )
    requires
        optional_well_formed(*node),
    ensures
        optional_well_formed(*node),
        Node::<V>::optional_as_map(*node) == Node::<V>::optional_as_map(old(*node)).insert(key, value),
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
    #[verifier::spec]
    pub fn insert(
        &mut self,
        key: u64,
        value: V
    )
    requires
        self.well_formed(),
    ensures
        self.well_formed(),
        self.as_map() == old(self).as_map().insert(key, value),
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

    /// Deletes a key from an optional node, handling the case where the node might not exist.
    #[verifier::spec]
    pub fn delete_from_optional(
        node: &mut Option<Box<Node<V>>>,
        key: u64
    )
    requires
        optional_well_formed(*node),
    ensures
        optional_well_formed(*node),
        Node::<V>::optional_as_map(*node) == Node::<V>::optional_as_map(old(*node)).remove(key),
    {
        if node.is_some() {
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

    /// Deletes and returns the rightmost (largest) key-value pair from a subtree.
    #[verifier::spec]
    pub fn delete_rightmost(
        node: &mut Option<Box<Node<V>>>
    ) -> (popped: (u64, V))
    requires
        node.is_Some(),
        optional_well_formed(*node),
    ensures
        optional_well_formed(*node),
        Node::<V>::optional_as_map(*node) == Node::<V>::optional_as_map(old(*node)).remove(result.0),
        Node::<V>::optional_as_map(old(*node)).dom().contains(result.0),
        forall |k: u64| Node::<V>::optional_as_map(old(*node)).dom().contains(k) && k != result.0 ==> Node::<V>::optional_as_map(*node).dom().contains(k),
    {
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        let mut boxed_node = tmp.unwrap();

        if boxed_node.right.is_none() {
            *node = boxed_node.left;
            assert(Node::<V>::optional_as_map(boxed_node.right) =~= Map::empty());
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(boxed_node.key));
            return (boxed_node.key, boxed_node.value);
        } else {
            let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.right);
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(popped_key));
            *node = Some(boxed_node);
            return (popped_key, popped_value);
        }
    }

    /// Looks up a key in an optional node, handling the case where the node might not exist.
    #[verifier::spec]
    pub fn get_from_optional(
        node: &Option<Box<Node<V>>>,
        key: u64
    ) -> (ret: Option<&V>)
    requires
        optional_well_formed(*node),
    ensures
        match ret {
            None => !Node::<V>::optional_as_map(*node).dom().contains(key),
            Some(r) => Node::<V>::optional_as_map(*node).dom().contains(key)
                && Node::<V>::optional_as_map(*node).index(key) == *r,
        },
    {
        match node {
            None => None,
            Some(node) => {
                node.get(key)
            }
        }
    }

    /// Looks up a key in this node's subtree using BST search.
    #[verifier::spec]
    pub fn get(
        &self,
        key: u64
    ) -> (ret: Option<&V>)
    requires
        self.well_formed(),
    ensures
        match ret {
            None => !self.as_map().dom().contains(key),
            Some(r) => self.as_map().dom().contains(key)
                && self.as_map().index(key) == *r,
        },
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
