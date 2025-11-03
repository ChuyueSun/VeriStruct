#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{
/*
 This lemma is often useful before a vector-remove call, and it can be useful to prove what element is contained in a vector.
 The parameters to this lemma function should match the executable code after it.
 Do NOT pass `old(..)' to this lemma as parameter.
 Example usage:
    proof{
	lemma_vec_remove(vec@, index);
    }
    vec.remove(index);
 */
proof fn lemma_vec_remove<T>(vec: Seq<T>, i: int)
    requires
        0 <= i < vec.len(),
    ensures
        forall |k: int| 0 <= k < i ==> #[trigger] vec[k] == vec.remove(i)[k],
        forall |k: int| i < k  < vec.len() ==> #[trigger] vec[k] ==  vec.remove(i)[k-1],
{

}

/*
 This lemma is often useful before a vector-push call, and it can be useful to prove what element is contained in a vector.
 Example usage:
    proof{
	lemma_vec_push(vec@, value, vec.len());
    }
    vec.push(value);
 */
proof fn lemma_vec_push<T>(vec: Seq<T>, i: T, l: usize)
    requires
        l == vec.len(),
    ensures
        forall |k: int| 0 <= k < vec.len() ==> #[trigger] vec[k] == vec.push(i)[k],
        vec.push(i).index(l as int) == i,
{
}

/*
 This lemma is often useful before a vector-remove call, and it can be useful to prove what element is contained in a vector.
 The parameters to this lemma function should match the executable code after it.
 Do NOT pass `old(..)' to this lemma as parameter.
 Example usage:
    proof{
	lemma_vec_remove(vec@, index);
    }
    vec.remove(index);
 */
proof fn lemma_vec_remove<T>(vec: Seq<T>, i: int)
    requires
        0 <= i < vec.len(),
    ensures
        forall |k: int| 0 <= k < i ==> #[trigger] vec[k] == vec.remove(i)[k],
        forall |k: int| i < k  < vec.len() ==> #[trigger] vec[k] ==  vec.remove(i)[k-1],
{

}

/*
 This lemma is often useful before a vector-push call, and it can be useful to prove what element is contained in a vector.
 Example usage:
    proof{
	lemma_vec_push(vec@, value, vec.len());
    }
    vec.push(value);
 */
proof fn lemma_vec_push<T>(vec: Seq<T>, i: T, l: usize)
    requires
        l == vec.len(),
    ensures
        forall |k: int| 0 <= k < vec.len() ==> #[trigger] vec[k] == vec.push(i)[k],
        vec.push(i).index(l as int) == i,
{
}


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
    /// Converts an optional node reference to a map representation.
    /// Returns the mapping from keys to values contained in the node and its subtrees.
    /// For None, returns an empty map; for Some(node), returns the node's map representation.
    spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
        decreases node_opt,
    {
        match node_opt {
            None => Map::empty(),
            Some(node) => node.as_map(),
        }
    }

    /// Converts this node and its entire subtree to a map representation.
    /// Returns a map containing all key-value pairs from this node and its left/right subtrees.
    /// The map is formed by taking the union of left subtree, right subtree, and this node's key-value pair.
    spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
         Node::<V>::optional_as_map(self.left)
          .union_prefer_right(Node::<V>::optional_as_map(self.right))
          .insert(self.key, self.value)
    }
}

impl<V> TreeMap<V> {
    /// Returns the map representation of the entire tree.
    /// Delegates to the optional_as_map function to convert the root node to a map.
    pub closed spec fn as_map(self) -> Map<u64, V> {
        Node::<V>::optional_as_map(self.root)
    }
}

/// Implementation of the View trait for TreeMap to provide a view of the tree as a map.
/// This allows the TreeMap to be treated as a Map<u64, V> in specifications.
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    /// Returns the view of this TreeMap as a Map, enabling the use of @ syntax.
    open spec fn view(&self) -> Map<u64, V> {
        self.as_map()
    }
}

impl<V> Node<V> {
    /// Checks if this node and its subtrees satisfy the binary search tree property.
    /// Returns true if all keys in left subtree are less than this node's key,
    /// all keys in right subtree are greater than this node's key, and both subtrees are well-formed.
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
    /// Type invariant for TreeMap that ensures the entire tree maintains BST properties.
    /// Returns true if the root node (if exists) and all its descendants are well-formed according to BST rules.
    #[verifier::type_invariant]
    spec fn well_formed(self) -> bool {
        match self.root {
            Some(node) => node.well_formed(),
            None => true,
        }
    }
}

impl<V> TreeMap<V> {
    /// Creates a new empty TreeMap.
    ///
    /// Requires: No conditions
    /// Ensures: The returned TreeMap represents an empty map
    pub fn new() -> (s: Self)
        requires
            true
        ensures
            s@ =~= Map::empty()
    {
        TreeMap::<V> { root: None }
    }
}

impl<V> Node<V> {
    /// Inserts a key-value pair into an optional node, creating a new node if None.
    ///
    /// Requires: If old(node).is_some(), then that node must be well_formed
    /// Ensures: node.is_some() ==> node.unwrap().well_formed(),
    ///          and the map is updated with key inserted
    fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
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
    ///
    /// Requires: old(self).well_formed()
    /// Ensures: self.well_formed(),
    ///          self.as_map() =~= old(self).as_map().insert(key, value)
    fn insert(&mut self, key: u64, value: V)
        requires
            old(self).well_formed()
        ensures
            self.well_formed(),
            self.as_map() =~= old(self).as_map().insert(key, value)
    {
        if key == self.key {
            self.value = value;
            proof {
                assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
                assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
            }
        } else if key < self.key {
            Self::insert_into_optional(&mut self.left, key, value);
            proof {
                assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
            }
        } else {
            Self::insert_into_optional(&mut self.right, key, value);
            proof {
                assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
            }
        }
    }
}

impl<V> TreeMap<V> {
    /// Inserts a key-value pair into the TreeMap.
    ///
    /// Requires: Nothing
    /// Ensures: self@ =~= old(self)@.insert(key, value)
    pub fn insert(&mut self, key: u64, value: V)
        requires
            true
        ensures
            self@ =~= old(self)@.insert(key, value)
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
    /// Deletes a key from an optional node, handling the case where the node might not exist.
    ///
    /// Requires: old(node).is_some() ==> old(node).unwrap().well_formed()
    /// Ensures: node.is_some() ==> node.unwrap().well_formed(),
    ///          Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(key)
    fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(key)
    {
        if node.is_some() {
            proof {
                assert(node.is_some());
            }
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            proof {
                assert(tmp.is_some());
            }
            let mut boxed_node = tmp.unwrap();

            if key == boxed_node.key {
                proof {
                    assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                    assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));
                }
                if boxed_node.left.is_none() {
                    *node = boxed_node.right;
                } else {
                    if boxed_node.right.is_none() {
                        *node = boxed_node.left;
                    } else {
                        proof {
                            assert(boxed_node.left.is_some());
                        } // Added by AI
                        let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.left);
                        boxed_node.key = popped_key;
                        boxed_node.value = popped_value;
                        *node = Some(boxed_node);
                    }
                }
            } else if key < boxed_node.key {
                proof {
                    assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));
                }
                Node::<V>::delete_from_optional(&mut boxed_node.left, key);
                *node = Some(boxed_node);
            } else {
                proof {
                    assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                }
                Node::<V>::delete_from_optional(&mut boxed_node.right, key);
                *node = Some(boxed_node);
            }
        }
    }

    /// Deletes and returns the rightmost (largest) key-value pair from a subtree.
    ///
    /// Requires: old(node).is_some() ==> old(node).unwrap().well_formed()
    /// Ensures: node.is_some() ==> node.unwrap().well_formed(),
    ///          Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
    ///          Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
    ///          Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
    ///          forall|k| Node::<V>::optional_as_map(*old(node)).dom().contains(k) ==> popped.0 >= k
    fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
            Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
            Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
            forall|k| Node::<V>::optional_as_map(*old(node)).dom().contains(k) ==> popped.0 >= k
    {
        proof {
            assert(node.is_some());
        } // Added by AI

        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        proof {
            assert(tmp.is_some());
        } // Added by AI

        let mut boxed_node = tmp

// Repair Round 4 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// Verified: -1, Errors: 999, Verus Errors: 2
