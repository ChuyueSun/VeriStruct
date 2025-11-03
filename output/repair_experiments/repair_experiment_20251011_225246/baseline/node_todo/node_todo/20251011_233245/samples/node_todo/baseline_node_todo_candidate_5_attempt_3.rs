#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{

pub open spec fn well_formed_opt<V>(node_opt: Option<Box<Node<V>>>) -> bool {
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
    pub open spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
         Self::optional_as_map(self.left)
          .union_prefer_right(Self::optional_as_map(self.right))
          .insert(self.key, self.value)
    }

    /// Checks if this node and its subtrees satisfy the binary search tree property.
    pub open spec fn well_formed(self) -> bool
        decreases self
    {
        (forall |k| Self::optional_as_map(self.left).dom().contains(k) ==> k < self.key)
        && (forall |k| Self::optional_as_map(self.right).dom().contains(k) ==> k > self.key)
        && match self.left {
            None => true,
            Some(l) => l.well_formed(),
        }
        && match self.right {
            None => true,
            Some(r) => r.well_formed(),
        }
    }

    /// Inserts a key-value pair into an optional node, creating a new node if None.
    #[verifier::spec]  // Not strictly necessary; used to allow old(...) in ensures
    pub fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
        requires
            well_formed_opt(*node),
        ensures
            well_formed_opt(*node),
            Self::optional_as_map(*node) == old(Self::optional_as_map(*node)).insert(key, value),
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
    pub fn insert(&mut self, key: u64, value: V)
        requires
            self.well_formed(),
        ensures
            self.well_formed(),
            self.as_map() == old(self.as_map()).insert(key, value),
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
    #[verifier::spec]
    pub fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
        requires
            well_formed_opt(*node),
        ensures
            well_formed_opt(*node),
            Self::optional_as_map(*node) == old(Self::optional_as_map(*node)).remove(key),
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
    #[verifier::spec]
    pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
        requires
            node.is_some(),
            well_formed_opt(*node),
        ensures
            well_formed_opt(*node),
            Self::optional_as_map(*node) == old(Self::optional_as_map(*node)).remove(popped.0),
            old(Self::optional_as_map(*node)).dom().contains(popped.0),
            forall |k| old(Self::optional_as_map(*node)).dom().contains(k) ==> k <= popped.0,
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
    #[verifier::spec]
    pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
        requires
            well_formed_opt(*node),
        ensures
            old(Self::optional_as_map(*node)).dom().contains(key) == ret.is_Some(),
            ret.is_Some() ==> *ret.get_Some_0() == old(Self::optional_as_map(*node))[key],
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
    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed(),
        ensures
            old(self.as_map()).dom().contains(key) == ret.is_Some(),
            ret.is_Some() ==> *ret.get_Some_0() == old(self.as_map())[key],
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
