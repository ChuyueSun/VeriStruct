#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{

    /// A node in the BST, containing a key-value pair, and optional references to left and right children
    pub struct Node<V> {
        pub key: u64,                          // The key used for ordering in the BST
        pub value: V,                          // The value associated with this key
        pub left: Option<Box<Node<V>>>,        // Optional left child (keys < this node's key)
        pub right: Option<Box<Node<V>>>,       // Optional right child (keys > this node's key)
    }

    impl<V> Node<V> {
        /// Converts an optional node reference to a map representation.
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
        /// The map is formed by taking the union of left subtree, right subtree, and this node's key-value pair.
        pub open spec fn as_map(self) -> Map<u64, V>
            decreases self,
        {
            Node::<V>::optional_as_map(self.left)
              .union_prefer_right(Node::<V>::optional_as_map(self.right))
              .insert(self.key, self.value)
        }

        /// Checks if this node and its subtrees satisfy the BST property.
        /// - all keys in left subtree < this node's key
        /// - all keys in right subtree > this node's key
        /// - both subtrees are well_formed
        pub open spec fn well_formed(self) -> bool
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

        /// Inserts a key-value pair into an optional node, creating a new node if None.
        /// The resulting subtree is a BST with the updated mapping.
        pub fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
            ensures
                Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).insert(key, value)
        {
            if node.is_none() {
                // Create a new leaf node if the current position is empty
                *node = Some(Box::new(Node::<V> {
                    key: key,
                    value: value,
                    left: None,
                    right: None,
                }));
            } else {
                // Extract the existing node, insert into it, then put it back
                let mut tmp = None;
                std::mem::swap(&mut tmp, node);
                let mut boxed_node = tmp.unwrap();

                (&mut *boxed_node).insert(key, value);

                *node = Some(boxed_node);
            }
        }

        /// Inserts a key-value pair into this node's subtree, maintaining BST properties.
        /// The resulting node is a BST with the updated mapping.
        pub fn insert(&mut self, key: u64, value: V)
            ensures
                self.as_map() =~= old(self).as_map().insert(key, value)
        {
            if key == self.key {
                // Update the value for an existing key
                self.value = value;

                // Proof assertions to help the verifier understand BST invariants
                assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
                assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
            } else if key < self.key {
                // Insert into left subtree
                Self::insert_into_optional(&mut self.left, key, value);

                // Proof assertion: key not in right subtree due to BST property
                assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
            } else {
                // Insert into right subtree
                Self::insert_into_optional(&mut self.right, key, value);

                // Proof assertion: key not in left subtree due to BST property
                assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
            }
        }

        /// Deletes a key from an optional node, handling the case where the node might not exist.
        /// The resulting subtree is a BST with the key removed (if it was present).
        pub fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
            ensures
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

        /// Deletes and returns the rightmost (largest) key-value pair from a subtree.
        /// Used as a helper for deletion when a node has both left and right children.
        /// The resulting subtree no longer contains that key-value pair.
        pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
            requires old(node).is_some(),
            ensures
                Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
                Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
                Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
                forall |elem| Node::<V>::optional_as_map(*old(node)).dom().contains(elem) ==> popped.0 >= elem
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

        /// Looks up a key in an optional node.
        /// Returns Some(&value) if key is in the subtree, None otherwise.
        pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
            ensures
                ret==(match node {
                    Some(node) => (if node.as_map().dom().contains(key) { Some(&node.as_map()[key]) } else { None }),
                    None => None,
                })
        {
            match node {
                None => None,
                Some(node) => {
                    node.get(key)
                }
            }
        }

        /// Looks up a key in this node's subtree using BST search.
        /// Returns Some(&value) if the key exists in the subtree, None otherwise.
        pub fn get(&self, key: u64) -> (ret: Option<&V>)
            ensures
                ret==(if self.as_map().dom().contains(key) { Some(&self.as_map()[key]) } else { None })
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

    /// Represents the entire BST-based data structure holding a root node
    /// and providing a high-level map interface.
    pub struct TreeMap<V> {
        root: Option<Box<Node<V>>>,
    }

    impl<V> TreeMap<V> {
        /// Returns the map representation of the entire tree.
        /// Delegates to the optional_as_map function to convert the root to a map.
        pub closed spec fn as_map(self) -> Map<u64, V> {
            Node::<V>::optional_as_map(self.root)
        }
    }

    /// Allows TreeMap to be treated as a Map<u64, V> (via @ syntax)
    impl<V> View for TreeMap<V> {
        type V = Map<u64, V>;

        open spec fn view(&self) -> Map<u64, V> {
            self.as_map()
        }
    }

    impl<V> TreeMap<V> {
        /// A BST is well-formed if its optional root node is well_formed.
        #[verifier::type_invariant]
        spec fn well_formed(self) -> bool {
            match self.root {
                Some(node) => node.well_formed(),
                None => true,
            }
        }
    }

    impl<V> TreeMap<V> {
        /// Creates a new, empty TreeMap.
        pub fn new() -> (s: Self)
            ensures
                s@ =~= Map::empty(),
                s.well_formed()
        {
            TreeMap::<V> { root: None }
        }

        /// Inserts a key-value pair into the TreeMap.
        /// The updated tree's map representation is the old map plus the inserted pair.
        pub fn insert(&mut self, key: u64, value: V)
            ensures
                self@ =~= old(self)@.insert(key, value)
        {
            proof {
                use_type_invariant(&*self);
            }

            // Swap out the root; insert; swap back
            let mut root = None;
            std::mem::swap(&mut root, &mut self.root);
            Node::<V>::insert_into_optional(&mut root, key, value);
            self.root = root;
        }

        /// Deletes a key from the TreeMap.
        /// The updated tree's map representation is the old map with the key removed.
        pub fn delete(&mut self, key: u64)
            ensures
                self@ =~= old(self)@.remove(key)
        {
            proof {
                use_type_invariant(&*self);
            }

            let mut root = None;
            std::mem::swap(&mut root, &mut self.root);
            Node::<V>::delete_from_optional(&mut root, key);
            self.root = root;
        }

        /// Looks up a key in the TreeMap.
        /// Returns Some(&value) if present, or None otherwise.
        pub fn get(&self, key: u64) -> (ret: Option<&V>)
            ensures
                ret == (if self@.dom().contains(key) { Some(&self@[key]) } else { None::<&V> })
        {
            proof {
                use_type_invariant(&*self);
            }
            Node::<V>::get_from_optional(&self.root, key)
        }
    }

    /*
    TEST CODE BELOW
    (Immutable functions must remain unchanged)
    */

    /// Test function demonstrating basic TreeMap operations.
    /// Requires: The input value v must be less than u64::MAX - 10 to avoid overflow
    fn test(v: u64)
    requires
        v < u64::MAX - 10
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

    /// Another test function that inserts two more key-value pairs.
    fn test2(tree_map: TreeMap<bool>, key1: u64, key2: u64) {
        let mut tree_map = tree_map;
        tree_map.insert(key1, true);
        tree_map.insert(key2, true);
    }

    /// Main function - entry point (kept for completeness).
    fn main() { }
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
