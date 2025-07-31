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

impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    open spec fn view(&self) -> Map<u64, V> {
        self.as_map()
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

// ANCHOR: well_formed_with_attr
impl<V> TreeMap<V> {
    #[verifier::type_invariant]
    spec fn well_formed(self) -> bool {
        match self.root {
            Some(node) => node.well_formed(),
            None => true, // empty tree always well-formed
        }
    }
}
// ANCHOR_END: well_formed_with_attr

// ANCHOR: new
impl<V> TreeMap<V> {
// ANCHOR: new_signature
    /// Creates a new empty TreeMap.
    /// 
    /// # Returns
    /// * A new TreeMap instance with no elements
    ///
    /// # Ensures
    /// * The resulting map is empty
    pub fn new() -> (s: Self)
        ensures
            s@ == Map::<u64, V>::empty()
// ANCHOR_END: new_signature
    {
        TreeMap::<V> { root: None }
    }
}
// ANCHOR_END: new

impl<V> Node<V> {
    /// Inserts a key-value pair into an optional BST node.
    /// 
    /// This helper function handles insertion into an optional node pointer, either by:
    /// 1. Creating a new leaf node if the pointer is None
    /// 2. Recursively inserting into an existing node while maintaining BST invariants
    ///
    /// # Arguments
    /// * `node` - Mutable reference to an optional boxed node
    /// * `key` - The key to insert
    /// * `value` - The value to associate with the key
    ///
    /// # Requires
    /// * If node is Some, then the node must be well-formed (satisfy BST properties)
    ///
    /// # Ensures 
    /// * The resulting node (if Some) is well-formed
    /// * The resulting map is equivalent to inserting (key, value) into the original map
    fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed(),
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).insert(key, value)
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

    /* TEST CODE BELOW */