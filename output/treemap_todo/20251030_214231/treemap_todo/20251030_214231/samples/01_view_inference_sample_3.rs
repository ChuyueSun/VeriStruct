#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus! {

    ////////////////////////////////////////////////////////////////////////////////
    // Node: Represents a node of a Binary Search Tree storing a key-value pair.
    //         Its spec functions convert the tree structure (including subtrees)
    //         into an abstract map.
    ////////////////////////////////////////////////////////////////////////////////

    pub struct Node<V> {
        pub key: u64,                          // The key used for ordering in the BST
        pub value: V,                          // The value associated with this key
        pub left: Option<Box<Node<V>>>,        // Left child: keys smaller than this node's key
        pub right: Option<Box<Node<V>>>,       // Right child: keys larger than this node's key
    }

    impl<V> Node<V> {
        /// Converts an optional node reference to a map representation.
        /// For None, returns an empty map; for Some(node), returns the node's map representation.
        pub open spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
            decreases node_opt
        {
            match node_opt {
                None => Map::empty(),
                Some(node) => node.as_map(),
            }
        }

        /// Converts this node and its entire subtree to a map representation.
        /// The map is formed by taking the union of the left subtree, right subtree, and
        /// this node's key-value pair.
        pub open spec fn as_map(&self) -> Map<u64, V>
            decreases (self.left.clone(), self.right.clone())
        {
            Node::<V>::optional_as_map(self.left.clone())
              .union_prefer_right(Node::<V>::optional_as_map(self.right.clone()))
              .insert(self.key, self.value)
        }

        /// Checks whether this node and its subtrees satisfy the binary search tree property.
        /// All keys in the left subtree are less than this node's key, all keys in the right
        /// subtree are greater than this node's key, and both subtrees are well-formed.
        pub open spec fn well_formed(&self) -> bool
            decreases (self.left.clone(), self.right.clone())
        {
            (forall |elem: u64| Node::<V>::optional_as_map(self.left.clone()).dom().contains(elem) ==> elem < self.key)
            && (forall |elem: u64| Node::<V>::optional_as_map(self.right.clone()).dom().contains(elem) ==> elem > self.key)
            && (match &self.left {
                    Some(left_node) => left_node.well_formed(),
                    None => true,
                })
            && (match &self.right {
                    Some(right_node) => right_node.well_formed(),
                    None => true,
                })
        }

        /// Inserts a key-value pair into an optional node.
        /// If the node is None, a new leaf is created.
        /// Ensures that the resulting node (if any) is well-formed and that its map representation
        /// equals the original map with the key-value pair inserted.
        pub fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
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

        /// Inserts a key-value pair into this node's subtree, maintaining BST properties.
        /// If the key already exists, its value is updated.
        pub fn insert(&mut self, key: u64, value: V)
            requires
                old(self).well_formed(),
            ensures
                self.well_formed(),
                self.as_map() =~= old(self).as_map().insert(key, value)
        {
            if key == self.key {
                self.value = value;
                // In a BST, if key equals this.key then it must not appear in either subtree.
                assert(!Node::<V>::optional_as_map(self.left.clone()).dom().contains(key));
                assert(!Node::<V>::optional_as_map(self.right.clone()).dom().contains(key));
            } else if key < self.key {
                Self::insert_into_optional(&mut self.left, key, value);
                assert(!Node::<V>::optional_as_map(self.right.clone()).dom().contains(key));
            } else {
                Self::insert_into_optional(&mut self.right, key, value);
                assert(!Node::<V>::optional_as_map(self.left.clone()).dom().contains(key));
            }
        }

        /// Deletes a key from an optional node.
        /// Ensures the resulting node (if any) is well-formed and its map equals the original map with the key removed.
        pub fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
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
                    assert(!Node::<V>::optional_as_map(boxed_node.left.clone()).dom().contains(key));
                    assert(!Node::<V>::optional_as_map(boxed_node.right.clone()).dom().contains(key));

                    if boxed_node.left.is_none() {
                        *node = boxed_node.right;
                    } else if boxed_node.right.is_none() {
                        *node = boxed_node.left;
                    } else {
                        let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.left);
                        boxed_node.key = popped_key;
                        boxed_node.value = popped_value;
                        *node = Some(boxed_node);
                    }
                } else if key < boxed_node.key {
                    assert(!Node::<V>::optional_as_map(boxed_node.right.clone()).dom().contains(key));
                    Node::<V>::delete_from_optional(&mut boxed_node.left, key);
                    *node = Some(boxed_node);
                } else {
                    assert(!Node::<V>::optional_as_map(boxed_node.left.clone()).dom().contains(key));
                    Node::<V>::delete_from_optional(&mut boxed_node.right, key);
                    *node = Some(boxed_node);
                }
            }
        }

        /// Deletes and returns the rightmost (largest) key-value pair from a subtree.
        /// Ensures that the returned pair was in the original subtree (and is the largest key),
        /// and the updated subtree's map equals the original with that key removed.
        pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
            requires
                old(node).is_some(),
                old(node).unwrap().well_formed(),
            ensures
                node.is_some() ==> node.unwrap().well_formed(),
                Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
                Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
                Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
                forall |elem: u64| Node::<V>::optional_as_map(*old(node)).dom().contains(elem ==> popped.0 >= elem),
        {
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            if boxed_node.right.is_none() {
                *node = boxed_node.left;
                assert(Node::<V>::optional_as_map(boxed_node.right.clone()) =~= Map::empty());
                assert(!Node::<V>::optional_as_map(boxed_node.left.clone()).dom().contains(boxed_node.key));
                return (boxed_node.key, boxed_node.value);
            } else {
                let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.right);
                assert(!Node::<V>::optional_as_map(boxed_node.left.clone()).dom().contains(popped_key));
                *node = Some(boxed_node);
                return (popped_key, popped_value);
            }
        }

        /// Looks up a key in an optional node.
        /// Returns Some(reference to value) if the key exists, or None otherwise.
        pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
            requires node.is_some() ==> node.unwrap().well_formed(),
            ensures ret == (match node {
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

        /// Looks up a key in this node's subtree using BST search.
        /// Returns Some(reference to value) if the key exists, None otherwise.
        pub fn get(&self, key: u64) -> (ret: Option<&V>)
            requires self.well_formed(),
            ensures ret == (if self.as_map().dom().contains(key) { Some(&self.as_map()[key]) } else { None })
        {
            if key == self.key {
                Some(&self.value)
            } else if key < self.key {
                proof { assert(!Node::<V>::optional_as_map(self.right.clone()).dom().contains(key)); }
                Self::get_from_optional(&self.left, key)
            } else {
                proof { assert(!Node::<V>::optional_as_map(self.left.clone()).dom().contains(key)); }
                Self::get_from_optional(&self.right, key)
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // TreeMap: A binary search tree based map.
    //          Its view is the abstract map of key-value pairs stored in the BST.
    ////////////////////////////////////////////////////////////////////////////////

    pub struct TreeMap<V> {
        root: Option<Box<Node<V>>>,        // The root node of the BST, or None if empty.
    }

    impl<V> TreeMap<V> {
        /// Returns the map representation of the entire tree.
        pub closed spec fn as_map(&self) -> Map<u64, V> {
            Node::<V>::optional_as_map(self.root.clone())
        }
    }

    /// Implementation of the View trait for TreeMap.
    /// The view abstracts the essential logical state as a Map<u64, V>.
    impl<V> View for TreeMap<V> {
        type V = Map<u64, V>;

        open spec fn view(&self) -> Map<u64, V> {
            self.as_map()
        }
    }

    impl<V> TreeMap<V> {
        /// Type invariant for TreeMap: the BST (if nonempty) must be well-formed.
        #[verifier::type_invariant]
        spec fn well_formed(&self) -> bool {
            match &self.root {
                Some(node) => node.well_formed(),
                None => true,
            }
        }
    }

    impl<V> TreeMap<V> {
        /// Creates a new empty TreeMap.
        pub fn new() -> (s: Self)
        // TODO: Add appropriate requires/ensures if needed.
        {
            TreeMap::<V> { root: None }
        }

        /// Inserts a key-value pair into the TreeMap.
        pub fn insert(&mut self, key: u64, value: V)
        // TODO: Add requires and ensures clauses.
        {
            proof {
                // Use the type invariant to establish well-formedness.
                // TODO: add detailed proof if necessary.
            }

            let mut root = None;
            std::mem::swap(&mut root, &mut self.root);
            Node::<V>::insert_into_optional(&mut root, key, value);
            self.root = root;
        }

        /// Deletes a key from the TreeMap.
        pub fn delete(&mut self, key: u64)
        // TODO: Add requires and ensures clauses.
        {
            proof { use_type_invariant(&*self); }

            let mut root = None;
            std::mem::swap(&mut root, &mut self.root);
            Node::<V>::delete_from_optional(&mut root, key);
            self.root = root;
        }

        /// Looks up a key in the TreeMap.
        pub fn get(&self, key: u64) -> (ret: Option<&V>)
        // TODO: Add requires and ensures clauses.
        {
            proof { use_type_invariant(&*self); }
            Node::<V>::get_from_optional(&self.root, key)
        }
    }

    /*
    TEST CODE BELOW
    */

    /// Test function demonstrating basic TreeMap operations.
    /// Requires: v < u64::MAX - 10 to avoid overflow.
    /// Ensures: Operations complete successfully and assertions hold.
    fn test(v: u64)
        requires v < u64::MAX - 10,
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

    /// Additional test function that takes a TreeMap and inserts two more key-value pairs.
    fn test2(tree_map: TreeMap<bool>, key1: u64, key2: u64) {
        let mut tree_map = tree_map;
        tree_map.insert(key1, true);
        tree_map.insert(key2, true);
    }

    /// Main function (entry point).
    fn main() { }

} // verus!

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
