#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

// Import the Node module (ground truth version with all specs)
mod node;
use node::Node;

verus!{

/// A binary search tree map data structure that maintains key-value pairs in sorted order.
/// Provides efficient insertion, deletion, and lookup operations with O(log n) average complexity.
pub struct TreeMap<V> {
    root: Option<Box<Node<V>>>,
}

impl<V> TreeMap<V> {
    /// Returns the map representation of the entire tree.
    pub closed spec fn as_map(self) -> Map<u64, V> {
        Node::<V>::optional_as_map(self.root)
    }
}

/// Implementation of the View trait for TreeMap to provide a view of the tree as a map.
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    open spec fn view(&self) -> Map<u64, V> {
        self.as_map()
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
            s.well_formed(),
    {
        TreeMap::<V> { root: None }
    }

    pub fn insert(&mut self, key: u64, value: V)
        ensures
            self.well_formed(),
            self.as_map() == old(self.as_map()).insert(key, value),
    {
        proof {
            use_type_invariant(&*self);
        }
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }

    pub fn delete(&mut self, key: u64)
        ensures
            self.well_formed(),
            self.as_map() == old(self.as_map()).remove(key),
    {
        proof { use_type_invariant(&*self); }
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::delete_from_optional(&mut root, key);
        self.root = root;
    }

    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        ensures
            self.well_formed(),
            match ret {
                Some(r) => self.as_map().contains_key(key) && *r == self.as_map().index(key),
                None => !self.as_map().contains_key(key),
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
