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
    spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
        decreases node_opt,
    {
        match node_opt {
            None => Map::empty(),
            Some(node) => node.as_map(),
        }
    }

    spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
         Node::<V>::optional_as_map(self.left)
          .union_prefer_right(Node::<V>::optional_as_map(self.right))
          .insert(self.key, self.value)
    }
}

impl<V> TreeMap<V> {
    pub closed spec fn as_map(self) -> Map<u64, V> {
        Node::<V>::optional_as_map(self.root)
    }
}

/// Implementation of the View trait for TreeMap to provide a view of the tree as a map.
/// This allows the TreeMap to be treated as a Map<u64, V> in specifications.
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    open spec fn view(&self) -> Map<u64, V> {
        Node::<V>::optional_as_map(self.root)
    }
}

impl<V> Node<V> {
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
    #[verifier::type_invariant]
    spec fn well_formed(self) -> bool {
        match self.root {
            Some(node) => node.well_formed(),
            None => true,
        }
    }
}

impl<V> TreeMap<V> {
    pub fn new() -> (s: Self)
        ensures
            s.as_map() == Map::empty(),
    {
        TreeMap::<V> { root: None }
    }
}

impl<V> Node<V> {
    fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
        requires
            old(node).map_or(true, |n| n.well_formed()),
        ensures
            node.map_or(true, |n| n.well_formed()),
            Node::<V>::optional_as_map(node) == Node::<V>::optional_as_map(old(node)).insert(key, value),
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

    fn insert(&mut self, key: u64, value: V)
        requires
            old(self).well_formed(),
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
}

impl<V> TreeMap<V> {
    pub fn insert(&mut self, key: u64, value: V)
        requires
            self.well_formed(),
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
}

impl<V> Node<V> {
    fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
        requires
            old(node).map_or(true, |n| n.well_formed()),
        ensures
            node.map_or(true, |n| n.well_formed()),
            Node::<V>::optional_as_map(node) == Node::<V>::optional_as_map(old(node)).remove(key),
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

    fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
        requires
            node.is_some(),
            node.as_ref().map_or(false, |n| n.well_formed()),
        ensures
            node.map_or(true, |n| n.well_formed()),
            Node::<V>::optional_as_map(node) == Node::<V>::optional_as_map(old(node)).remove(popped.0),
            Node::<V>::optional_as_map(old(node)).dom().contains(popped.0),
            forall |k| Node::<V>::optional_as_map(old(node)).dom().contains(k) ==> k <= popped.0,
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
}

impl<V> TreeMap<V> {
    pub fn delete(&mut self, key: u64)
        requires
            self.well_formed(),
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
}

impl<V> Node<V> {
    fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
        requires
            node.map_or(true, |n| n.well_formed()),
        ensures
            match ret {
                Some(r) => Node::<V>::optional_as_map(*node).index(key) == *r,
                None => !Node::<V>::optional_as_map(*node).dom().contains(key),
            },
    {
        match node {
            None => None,
            Some(node) => {
                node.get(key)
            }
        }
    }

    fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed(),
        ensures
            match ret {
                Some(r) => self.as_map().index(key) == *r,
                None => !self.as_map().dom().contains(key),
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

impl<V> TreeMap<V> {
    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed(),
        ensures
            match ret {
                Some(r) => self.as_map().index(key) == *r,
                None => !self.as_map().dom().contains(key),
            },
    {
        proof { use_type_invariant(&*self); }
        Node::<V>::get_from_optional(&self.root, key)
    }
}

/*
TEST CODE BELOW
*/

fn test(v: u64)
requires
    v < u64::MAX - 10,
{
    let mut tree_map = TreeMap::<bool>::new();
    tree_map.insert(v, false);
    tree_map.insert(v + 1, false);
    tree_map.insert(v, true);
    tree_map.delete(v);

    let elem17 = tree_map.get(v);
    let elem18 = tree_map.get(v + 1);

    assert(elem17.is_none());
    assert(elem18 == Some(&false));

    test2(tree_map, v + 2, v + 3);
}

fn test2(tree_map: TreeMap<bool>, key1: u64, key2: u64) {
    let mut tree_map = tree_map;
    tree_map.insert(key1, true);
    tree_map.insert(key2, true);
}

fn main() { }
}
