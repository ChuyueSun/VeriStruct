#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]

// ANCHOR: all
use vstd::prelude::*;

verus!{

struct Node<V> {
    key: u64,
    value: V,
    left: Option<Box<Node<V>>>,
    right: Option<Box<Node<V>>>,
}

pub struct TreeMap<V> {
    root: Option<Box<Node<V>>>,
}

impl<V> Node<V> {
    spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
    {
        // TODO: add specification function
    }

    spec fn as_map(self) -> Map<u64, V>
    {
        // TODO: add specification function
    }
}

impl<V> TreeMap<V> {
    pub closed spec fn as_map(self) -> Map<u64, V> {
        // TODO: add specification function
    }
}

impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    open spec fn view(&self) -> Map<u64, V> {
        // TODO: add specification function
    }
}

impl<V> Node<V> {
    spec fn well_formed(self) -> bool
    {
        // TODO: add specification function
    }
}

// ANCHOR: well_formed_with_attr
impl<V> TreeMap<V> {
    #[verifier::type_invariant]
    spec fn well_formed(self) -> bool {
        // TODO: add specification function
    }
}
// ANCHOR_END: well_formed_with_attr

// ANCHOR: new
impl<V> TreeMap<V> {
// ANCHOR: new_signature
    pub fn new() -> (s: Self)
    // TODO: add requires and ensures
// ANCHOR_END: new_signature
    {
        TreeMap::<V> { root: None }
    }
}
// ANCHOR_END: new

impl<V> Node<V> {
    fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
    // TODO: add requires and ensures
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
    // TODO: add requires and ensures
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

// ANCHOR: insert
impl<V> TreeMap<V> {
// ANCHOR: insert_signature
    pub fn insert(&mut self, key: u64, value: V)
    // TODO: add requires and ensures
// ANCHOR_END: insert_signature
    {
        // TODO: add proof
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }
}
// ANCHOR_END: insert

impl<V> Node<V> {
    fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
    // TODO: add requires and ensures
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
    // TODO: add requires and ensures
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

// ANCHOR: delete
impl<V> TreeMap<V> {
// ANCHOR: delete_signature
    pub fn delete(&mut self, key: u64)
    // TODO: add requires and ensures
// ANCHOR_END: delete_signature
    {
        // TODO: add proof
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::delete_from_optional(&mut root, key);
        self.root = root;
    }
}
// ANCHOR_END: delete

impl<V> Node<V> {
    fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> Option<&V>
    // TODO: add requires and ensures
    {
        match node {
            None => None,
            Some(node) => {
                node.get(key)
            }
        }
    }

    fn get(&self, key: u64) -> Option<&V>
    // TODO: add requires and ensures
    {
        if key == self.key {
            Some(&self.value)
        } else if key < self.key {
            // TODO: add proof
            Self::get_from_optional(&self.left, key)
        } else {
            // TODO: add proof
            Self::get_from_optional(&self.right, key)
        }
    }
}

// ANCHOR: get
impl<V> TreeMap<V> {
// ANCHOR: get_signature
    pub fn get(&self, key: u64) -> Option<&V>
    // TODO: add requires and ensures
// ANCHOR_END: get_signature
    {
        // TODO: add proof
        Node::<V>::get_from_optional(&self.root, key)
    }
}
// ANCHOR_END: get

// ANCHOR: example_use
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
// ANCHOR_END: example_use


}
// ANCHOR_END: all

fn main() { }