struct Node<V> {
    key: u64,
    value: V,
    left: Option<Box<Node<V>>>,
    right: Option<Box<Node<V>>>,
}

pub struct TreeMap<V> {
    root: Option<Box<Node<V>>>,
}

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
    pub fn new() -> Self
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

    /// Inserts a key-value pair into this BST node.
    ///
    /// If the key is less than the node's key, it is inserted into the left subtree.
    /// If the key is greater than the node's key, it is inserted into the right subtree.
    /// If the key is equal to the node's key, the value is updated.
    fn insert(&mut self, key: u64, value: V) {
        if key < self.key {
            Node::insert_into_optional(&mut self.left, key, value);
        } else if key > self.key {
            Node::insert_into_optional(&mut self.right, key, value);
        } else {
            self.value = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_treemap_is_empty() {
        let tree: TreeMap<i32> = TreeMap::new();
        assert!(tree.root.is_none(), "New TreeMap should have no root");
    }

    #[test]
    fn test_insert_into_empty_node() {
        let mut node: Option<Box<Node<&str>>> = None;
        Node::insert_into_optional(&mut node, 10, "hello");
        let node = node.expect("Node should be created");
        assert_eq!(node.key, 10, "Inserted key should match");
        assert_eq!(node.value, "hello", "Inserted value should match");
        assert!(node.left.is_none(), "Newly created node should have no left child");
        assert!(node.right.is_none(), "Newly created node should have no right child");
    }

    // The following tests assume that the Node::insert method (called inside insert_into_optional)
    // implements BST insertion logic:
    // - If the new key is less than the current node's key, it is inserted into the left subtree.
    // - If the key is greater than the current node's key, it is inserted into the right subtree.
    // - If the key is equal to the current node's key, the value is updated.

    #[test]
    fn test_insert_into_nonempty_left() {
        let mut node: Option<Box<Node<&str>>> = Some(Box::new(Node {
            key: 50,
            value: "root",
            left: None,
            right: None,
        }));
        // Insert a key less than 50. It should go to the left subtree.
        Node::insert_into_optional(&mut node, 25, "left");
        let root = node.as_ref().expect("Root should exist");
        // Root remains unchanged.
        assert_eq!(root.key, 50, "Root key should remain unchanged");
        assert_eq!(root.value, "root", "Root value should remain unchanged");
        // Check left child.
        let left = root.left.as_ref().expect("Left child should be created");
        assert_eq!(left.key, 25, "Left child's key should be 25");
        assert_eq!(left.value, "left", "Left child's value should be 'left'");
        // Right child should still be empty.
        assert!(root.right.is_none(), "Right child should remain None");
    }

    #[test]
    fn test_insert_into_nonempty_right() {
        let mut node: Option<Box<Node<&str>>> = Some(Box::new(Node {
            key: 50,
            value: "root",
            left: None,
            right: None,
        }));
        // Insert a key greater than 50. It should go to the right subtree.
        Node::insert_into_optional(&mut node, 75, "right");
        let root = node.as_ref().expect("Root should exist");
        // Root remains unchanged.
        assert_eq!(root.key, 50, "Root key should remain unchanged");
        assert_eq!(root.value, "root", "Root value should remain unchanged");
        // Check right child.
        let right = root.right.as_ref().expect("Right child should be created");
        assert_eq!(right.key, 75, "Right child's key should be 75");
        assert_eq!(right.value, "right", "Right child's value should be 'right'");
        // Left child should still be empty.
        assert!(root.left.is_none(), "Left child should remain None");
    }

    #[test]
    fn test_insert_duplicate_key_updates_value() {
        let mut node: Option<Box<Node<&str>>> = Some(Box::new(Node {
            key: 50,
            value: "root",
            left: None,
            right: None,
        }));
        // Insert a duplicate key. According to BST semantics,
        // this should update the node's value.
        Node::insert_into_optional(&mut node, 50, "updated_root");
        let root = node.as_ref().expect("Root should exist");
        assert_eq!(root.key, 50, "Key should remain unchanged for duplicate insertion");
        assert_eq!(root.value, "updated_root", "Value should be updated for duplicate key");
    }
}