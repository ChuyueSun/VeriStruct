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

    /// Inserts a key-value pair into a BST node.
    /// 
    /// This method inserts or updates a key-value pair in the binary search tree while
    /// maintaining the BST invariants. If the key already exists, its value is updated.
    /// Otherwise, the pair is inserted in the appropriate position:
    /// - If key < node.key: insert into left subtree
    /// - If key > node.key: insert into right subtree
    ///
    /// # Arguments
    /// * `key` - The key to insert
    /// * `value` - The value to associate with the key
    ///
    /// # Requires
    /// * The node must be well-formed (satisfy BST properties)
    ///
    /// # Ensures
    /// * The node remains well-formed after insertion
    /// * The resulting map is equivalent to inserting (key, value) into the original map
    fn insert(&mut self, key: u64, value: V)
        requires
            old(self).well_formed(),
        ensures
            self.well_formed(),
            self.as_map() =~= old(self).as_map().insert(key, value),
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
    /// Inserts a key-value pair into the TreeMap.
    /// 
    /// If the key already exists, its value is updated. Otherwise, a new key-value
    /// pair is inserted while maintaining the BST properties.
    ///
    /// # Arguments
    /// * `key` - The key to insert
    /// * `value` - The value to associate with the key
    ///
    /// # Ensures
    /// * The resulting map is equivalent to inserting (key, value) into the original map
    pub fn insert(&mut self, key: u64, value: V)
        ensures
            self@ == old(self)@.insert(key, value)
// ANCHOR_END: insert_signature
    {
        proof { use_type_invariant(&*self); }
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }
}
// ANCHOR_END: insert

impl<V> Node<V> {
    /// Deletes a key-value pair from an optional BST node.
    /// 
    /// This helper function handles deletion from an optional node pointer:
    /// - Does nothing if the node is None
    /// - If the key is found:
    ///   * For leaf nodes: removes the node
    ///   * For nodes with one child: replaces with the child
    ///   * For nodes with two children: replaces with rightmost node from left subtree
    ///
    /// # Arguments
    /// * `node` - Mutable reference to an optional boxed node
    /// * `key` - The key to delete
    ///
    /// # Requires
    /// * If node is Some, then the node must be well-formed (satisfy BST properties)
    ///
    /// # Ensures
    /// * The resulting node (if Some) is well-formed
    /// * The resulting map is equivalent to removing the key from the original map
    fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed(),
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(key)
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

    /// Removes and returns the rightmost node from a BST.
    /// 
    /// This helper function is used during deletion when removing a node with two children.
    /// It finds and removes the rightmost node (which will have no right child) and returns
    /// its key-value pair.
    ///
    /// # Arguments
    /// * `node` - Mutable reference to an optional boxed node
    ///
    /// # Returns
    /// * A tuple of (key, value) from the rightmost node
    ///
    /// # Requires
    /// * node must be Some
    /// * The node must be well-formed (satisfy BST properties)
    ///
    /// # Ensures
    /// * The resulting node (if Some) is well-formed
    /// * The resulting map is equivalent to removing the rightmost key from the original map
    fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
        requires
            old(node).is_some(),
            old(node).unwrap().well_formed(),
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
            Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
            Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
            forall |elem| Node::<V>::optional_as_map(*old(node)).dom().contains(elem) ==> popped.0 >= elem,
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
    /// Deletes a key-value pair from the TreeMap.
    /// 
    /// If the key exists, removes it and its associated value from the map
    /// while maintaining BST properties.
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Ensures
    /// * The resulting map is equivalent to removing the key from the original map
    pub fn delete(&mut self, key: u64)
        ensures
            self@ == old(self)@.remove(key)
// ANCHOR_END: delete_signature
    {
        proof { use_type_invariant(&*self); }
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::delete_from_optional(&mut root, key);
        self.root = root;
    }
}
// ANCHOR_END: delete

impl<V> Node<V> {
    /// Retrieves a reference to a value from an optional BST node by key.
    /// 
    /// This helper function searches for a value in an optional node pointer:
    /// - Returns None if the node is None
    /// - Recursively searches the node if it exists
    ///
    /// # Arguments
    /// * `node` - Reference to an optional boxed node
    /// * `key` - The key to search for
    ///
    /// # Returns
    /// * Some(&V) if the key exists in the tree
    /// * None if the key is not found
    ///
    /// # Requires
    /// * If node is Some, then the node must be well-formed (satisfy BST properties)
    fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> Option<&V>
        requires node.is_some() ==> node.unwrap().well_formed(),
        returns (match node {
            Some(node) => (if node.as_map().dom().contains(key) { Some(&node.as_map()[key]) } else { None }),
            None => None,
        }),
    {
        match node {
            None => None,
            Some(node) => {
                node.get(key)
            }
        }
    }

    /// Retrieves a reference to a value by key from a BST node.
    /// 
    /// Searches for a value in the binary search tree following BST properties:
    /// - If key == node.key: return the current node's value
    /// - If key < node.key: search left subtree
    /// - If key > node.key: search right subtree
    ///
    /// # Arguments
    /// * `key` - The key to search for
    ///
    /// # Returns
    /// * Some(&V) if the key exists in the tree
    /// * None if the key is not found
    ///
    /// # Requires
    /// * The node must be well-formed (satisfy BST properties)
    fn get(&self, key: u64) -> Option<&V>
        requires self.well_formed(),
        returns (if self.as_map().dom().contains(key) { Some(&self.as_map()[key]) } else { None })
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

// ANCHOR: get
impl<V> TreeMap<V> {
// ANCHOR: get_signature
    /// Retrieves a reference to a value by key from the TreeMap.
    /// 
    /// # Arguments
    /// * `key` - The key to search for
    ///
    /// # Returns
    /// * Some(&V) if the key exists in the map
    /// * None if the key is not found
    ///
    /// # Returns
    /// Returns Some reference to the value if the key exists, None otherwise
    pub fn get(&self, key: u64) -> Option<&V>
        returns (if self@.dom().contains(key) { Some(&self@[key]) } else { None })
// ANCHOR_END: get_signature
    {
        proof { use_type_invariant(&*self); }
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


fn main() { }
}
// ANCHOR_END: all
