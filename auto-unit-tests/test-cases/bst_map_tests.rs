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
    pub fn new() -> Self {
        TreeMap { root: None }
    }
}

impl<V> Node<V> {
    fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V) {
        if node.is_none() {
            *node = Some(Box::new(Node {
                key: key,
                value: value,
                left: None,
                right: None,
            }));
        } else {
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();
            boxed_node.insert(key, value);
            *node = Some(boxed_node);
        }
    }

    fn insert(&mut self, key: u64, value: V) {
        if key == self.key {
            self.value = value;
        } else if key < self.key {
            Self::insert_into_optional(&mut self.left, key, value);
        } else {
            Self::insert_into_optional(&mut self.right, key, value);
        }
    }

    fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64) {
        if node.is_some() {
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();
            if key == boxed_node.key {
                if boxed_node.left.is_none() {
                    *node = boxed_node.right;
                } else if boxed_node.right.is_none() {
                    *node = boxed_node.left;
                } else {
                    let (popped_key, popped_value) = Node::delete_rightmost(&mut boxed_node.left);
                    boxed_node.key = popped_key;
                    boxed_node.value = popped_value;
                    *node = Some(boxed_node);
                }
            } else if key < boxed_node.key {
                Node::delete_from_optional(&mut boxed_node.left, key);
                *node = Some(boxed_node);
            } else {
                Node::delete_from_optional(&mut boxed_node.right, key);
                *node = Some(boxed_node);
            }
        }
    }

    fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (u64, V) {
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        let mut boxed_node = tmp.unwrap();
        if boxed_node.right.is_none() {
            *node = boxed_node.left;
            (boxed_node.key, boxed_node.value)
        } else {
            let (popped_key, popped_value) = Node::delete_rightmost(&mut boxed_node.right);
            *node = Some(boxed_node);
            (popped_key, popped_value)
        }
    }

    fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> Option<&V> {
        match node {
            None => None,
            Some(node) => node.get(key),
        }
    }

    fn get(&self, key: u64) -> Option<&V> {
        if key == self.key {
            Some(&self.value)
        } else if key < self.key {
            Self::get_from_optional(&self.left, key)
        } else {
            Self::get_from_optional(&self.right, key)
        }
    }
}

impl<V> TreeMap<V> {
    /// Inserts a key-value pair into the TreeMap.
    pub fn insert(&mut self, key: u64, value: V) {
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::insert_into_optional(&mut root, key, value);
        self.root = root;
    }

    /// Deletes a key-value pair from the TreeMap.
    pub fn delete(&mut self, key: u64) {
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::delete_from_optional(&mut root, key);
        self.root = root;
    }

    /// Retrieves a reference to a value by key from the TreeMap.
    pub fn get(&self, key: u64) -> Option<&V> {
        Node::get_from_optional(&self.root, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a TreeMap with some key-value pairs.
    fn populate_tree() -> TreeMap<&'static str> {
        let mut map = TreeMap::new();
        map.insert(10, "ten");
        map.insert(5, "five");
        map.insert(15, "fifteen");
        map
    }

    #[test]
    fn test_new_empty() {
        let map: TreeMap<&str> = TreeMap::new();
        // Because tree is empty, any lookup should return None.
        assert!(map.get(1).is_none());
        assert!(map.get(100).is_none());
    }

    #[test]
    fn test_insert_and_get() {
        let mut map = TreeMap::new();
        map.insert(10, "ten");
        map.insert(5, "five");
        map.insert(15, "fifteen");

        // Verify that all inserted keys can be retrieved.
        assert_eq!(map.get(10), Some(&"ten"));
        assert_eq!(map.get(5), Some(&"five"));
        assert_eq!(map.get(15), Some(&"fifteen"));

        // Key not present.
        assert!(map.get(100).is_none());
    }

    #[test]
    fn test_update_existing_key() {
        let mut map = TreeMap::new();
        map.insert(20, "twenty");
        // Update the same key.
        map.insert(20, "TWENTY");
        assert_eq!(map.get(20), Some(&"TWENTY"));
    }

    #[test]
    fn test_delete_leaf_node() {
        // In this test, we delete a node with no children (leaf).
        let mut map = populate_tree();
        // '15' is a leaf in the populated tree.
        map.delete(15);
        assert!(map.get(15).is_none());
        // Other keys should remain.
        assert_eq!(map.get(10), Some(&"ten"));
        assert_eq!(map.get(5), Some(&"five"));
    }

    #[test]
    fn test_delete_node_with_one_child() {
        // Create a tree where a node has exactly one child.
        let mut map = TreeMap::new();
        map.insert(10, "ten");
        map.insert(5, "five");
        // Insert a value that becomes left child of 5.
        map.insert(3, "three");
        // At this point, node with key 5 has only a left child (3).
        map.delete(5);
        // Key 5 should be deleted.
        assert!(map.get(5).is_none());
        // The child (3) should still be accessible.
        assert_eq!(map.get(3), Some(&"three"));
        // Root should remain.
        assert_eq!(map.get(10), Some(&"ten"));
    }

    #[test]
    fn test_delete_node_with_two_children() {
        // Create a more complex tree to test deletion of a node with two children.
        // We want a node with both left and right children.
        // Build tree:
        //         10 ("ten")
        //         /
        //      5 ("five")
        //      /   \
        //  3 ("three")  7 ("seven")
        //             /      \
        //     6 ("six")      8 ("eight")
        let mut map = TreeMap::new();
        map.insert(10, "ten");
        map.insert(5, "five");
        map.insert(15, "fifteen"); // To ensure tree branching, though not used in deletion here.
        map.insert(3, "three");
        map.insert(7, "seven");
        map.insert(6, "six");
        map.insert(8, "eight");

        // Delete node with key 7, which has two children.
        map.delete(7);
        // The deletion strategy replaces node 7 with its left subtree's rightmost element.
        // For node with key 7, left subtree is node with key 6 (which has no right child).
        // So, the node 7 should be replaced with key 6 and value "six".
        assert!(map.get(7).is_none());
        // Verify that key 6 exists and its value is what was originally at key 6.
        assert_eq!(map.get(6), Some(&"six"));
        // Right child of the deleted node (8) should remain reachable.
        assert_eq!(map.get(8), Some(&"eight"));
        // Also verify other parts of the tree remain intact.
        assert_eq!(map.get(5), Some(&"five"));
        assert_eq!(map.get(10), Some(&"ten"));
        assert_eq!(map.get(15), Some(&"fifteen"));
    }

    #[test]
    fn test_delete_nonexistent_key() {
        // Deleting a key that does not exist should not affect the tree.
        let mut map = populate_tree();
        map.delete(100); // 100 is not present.
        // All original keys should still be accessible.
        assert_eq!(map.get(10), Some(&"ten"));
        assert_eq!(map.get(5), Some(&"five"));
        assert_eq!(map.get(15), Some(&"fifteen"));
    }

    #[test]
    fn test_delete_on_empty_tree() {
        // Deleting on an empty TreeMap should be a no-op.
        let mut map: TreeMap<&str> = TreeMap::new();
        // Should not panic.
        map.delete(42);
        assert!(map.get(42).is_none());
    }

    // New tests to further exercise edge cases

    #[test]
    fn test_delete_root_only() {
        // Test deleting the only node in the tree, i.e., the root.
        let mut map = TreeMap::new();
        map.insert(100, "hundred");
        assert_eq!(map.get(100), Some(&"hundred"));
        map.delete(100);
        // Now the tree should be empty.
        assert!(map.get(100).is_none());
    }

    #[test]
    fn test_delete_root_with_two_children() {
        // Test deleting the root when it has two children.
        // Build tree:
        //         20 ("twenty")
        //         /      \
        //   10 ("ten")  30 ("thirty")
        //                /
        //         25 ("twenty-five")
        let mut map = TreeMap::new();
        map.insert(20, "twenty");
        map.insert(10, "ten");
        map.insert(30, "thirty");
        map.insert(25, "twenty-five");

        // Delete the root.
        map.delete(20);
        // The new root should be one of the candidates from the subtrees.
        // According to our deletion logic, since 20 has two children, it will replace its value
        // with the rightmost node of its left subtree. In this case the left subtree of 20 is just 10.
        // So the new key should be 10.
        // However, if the deletion instead came from the right subtree (if left were empty) it would
        // have taken from there. In our implementation, for a node with two children we take the rightmost
        // of the left subtree. Since left of 20 is just 10 (with no right child), the new key becomes 10.
        assert!(map.get(20).is_none());
        assert_eq!(map.get(10), Some(&"ten"));
        // The right subtree should remain intact.
        assert_eq!(map.get(30), Some(&"thirty"));
        assert_eq!(map.get(25), Some(&"twenty-five"));
    }

    #[test]
    fn test_multiple_operations() {
        // Test a sequence of mixed operations.
        let mut map = TreeMap::new();
        // Insert several keys.
        for i in [50, 30, 70, 20, 40, 60, 80].iter() {
            map.insert(*i, *i);
        }
        // Verify that all keys can be retrieved.
        for i in [50, 30, 70, 20, 40, 60, 80].iter() {
            assert_eq!(map.get(*i), Some(i));
        }
        // Update some keys.
        map.insert(70, 700);
        assert_eq!(map.get(70), Some(&700));
        // Delete a leaf.
        map.delete(20);
        assert!(map.get(20).is_none());
        // Delete a node with one child.
        map.delete(30);
        assert!(map.get(30).is_none());
        // Delete a node with two children.
        map.delete(70);
        assert!(map.get(70).is_none());
        // Final check for remaining keys.
        for key in [50, 40, 60, 80].iter() {
            assert!(map.get(*key).is_some());
        }
    }
}