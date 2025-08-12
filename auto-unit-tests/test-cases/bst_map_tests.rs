struct Node<V> {
    key: u64,
    value: V,
    left: Option<Box<Node<V>>>,
    right: Option<Box<Node<V>>>,
}

pub struct TreeMap<V> {
    root: Option<Box<Node<V>>>,
}

impl<V> TreeMap<V> {
    /// Creates a new empty TreeMap.
    /// 
    /// # Returns
    /// * A new TreeMap instance with no elements
    ///
    /// # Ensures
    /// * The resulting map is empty
    pub fn new() -> Self {
        TreeMap::<V> { root: None }
    }
}

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
    fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V) {
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

    fn insert(&mut self, key: u64, value: V) {
        if key == self.key {
            self.value = value;
        } else if key < self.key {
            if let Some(ref mut left) = self.left {
                left.insert(key, value);
            } else {
                self.left = Some(Box::new(Node {
                    key,
                    value,
                    left: None,
                    right: None,
                }));
            }
        } else {
            if let Some(ref mut right) = self.right {
                right.insert(key, value);
            } else {
                self.right = Some(Box::new(Node {
                    key,
                    value,
                    left: None,
                    right: None,
                }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that a new TreeMap is indeed empty.
    #[test]
    fn test_tree_map_new_is_empty() {
        let map: TreeMap<i32> = TreeMap::new();
        assert!(map.root.is_none());
    }

    // Test that inserting into an empty Option<Box<Node>> creates a new node.
    #[test]
    fn test_insert_into_optional_creates_node_when_none() {
        let mut node: Option<Box<Node<i32>>> = None;
        Node::insert_into_optional(&mut node, 5, 50);
        let n = node.expect("Node should have been created");
        assert_eq!(n.key, 5);
        assert_eq!(n.value, 50);
        assert!(n.left.is_none());
        assert!(n.right.is_none());
    }

    // Test that inserting a key less than the existing node's key goes to the left.
    #[test]
    fn test_insert_into_optional_inserts_left() {
        let mut node: Option<Box<Node<i32>>> = Some(Box::new(Node {
            key: 10,
            value: 100,
            left: None,
            right: None,
        }));
        // Inserting key 5 (<10) should go to the left branch.
        Node::insert_into_optional(&mut node, 5, 50);
        let n = node.expect("Root node must exist");
        // Depending on the actual BST insert implementation inside Node::insert,
        // the node with key==5 should be in the left child.
        assert!(n.left.is_some());
        let left_child = n.left.expect("Left child must have been inserted");
        assert_eq!(left_child.key, 5);
        assert_eq!(left_child.value, 50);
    }

    // Test that inserting a key greater than the existing node's key goes to the right.
    #[test]
    fn test_insert_into_optional_inserts_right() {
        let mut node: Option<Box<Node<i32>>> = Some(Box::new(Node {
            key: 10,
            value: 100,
            left: None,
            right: None,
        }));
        // Inserting key 15 (>10) should go to the right branch.
        Node::insert_into_optional(&mut node, 15, 150);
        let n = node.expect("Root node must exist");
        assert!(n.right.is_some());
        let right_child = n.right.expect("Right child must have been inserted");
        assert_eq!(right_child.key, 15);
        assert_eq!(right_child.value, 150);
    }

    // Test that inserting a duplicate key updates its associated value.
    #[test]
    fn test_insert_into_optional_updates_value_for_existing_key() {
        let mut node: Option<Box<Node<i32>>> = Some(Box::new(Node {
            key: 10,
            value: 100,
            left: Some(Box::new(Node {
                key: 5,
                value: 50,
                left: None,
                right: None,
            })),
            right: None,
        }));
        // Inserting the same key (5) with a new value should update the node.
        Node::insert_into_optional(&mut node, 5, 55);
        let n = node.expect("Root node must exist");
        assert!(n.left.is_some());
        let left_child = n.left.expect("Left child must exist");
        assert_eq!(left_child.key, 5);
        assert_eq!(left_child.value, 55);
    }

    // Test inserting extreme key values.
    #[test]
    fn test_insert_into_optional_extreme_keys() {
        let mut node: Option<Box<Node<i32>>> = None;
        // Insert the minimum possible key.
        Node::insert_into_optional(&mut node, 0, 0);
        // Insert the maximum possible key.
        Node::insert_into_optional(&mut node, u64::MAX, 1);
        let root = node.expect("Root node must exist");
        // Depending on the order of insertion, one of the extreme keys will be the root.
        // Check that the other extreme key was inserted into the proper branch.
        if root.key == 0 {
            assert!(root.right.is_some());
            let right_child = root.right.expect("Right child must exist");
            assert_eq!(right_child.key, u64::MAX);
            assert_eq!(right_child.value, 1);
        } else if root.key == u64::MAX {
            assert!(root.left.is_some());
            let left_child = root.left.expect("Left child must exist");
            assert_eq!(left_child.key, 0);
            assert_eq!(left_child.value, 0);
        } else {
            panic!("Unexpected root key value");
        }
    }
}