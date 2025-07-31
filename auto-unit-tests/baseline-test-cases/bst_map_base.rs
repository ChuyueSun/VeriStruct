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
                    let (popped_key, popped_value) =
                        Node::<V>::delete_rightmost(&mut boxed_node.left);
                    boxed_node.key = popped_key;
                    boxed_node.value = popped_value;
                    *node = Some(boxed_node);
                }
            } else if key < boxed_node.key {
                Node::<V>::delete_from_optional(&mut boxed_node.left, key);
                *node = Some(boxed_node);
            } else {
                Node::<V>::delete_from_optional(&mut boxed_node.right, key);
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
            return (boxed_node.key, boxed_node.value);
        } else {
            let (popped_key, popped_value) =
                Node::<V>::delete_rightmost(&mut boxed_node.right);
            *node = Some(boxed_node);
            return (popped_key, popped_value);
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
    /// Creates a new empty TreeMap.
    pub fn new() -> Self {
        TreeMap::<V> { root: None }
    }

    pub fn insert(&mut self, key: u64, value: V) {
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }

    pub fn delete(&mut self, key: u64) {
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::delete_from_optional(&mut root, key);
        self.root = root;
    }

    pub fn get(&self, key: u64) -> Option<&V> {
        Node::<V>::get_from_optional(&self.root, key)
    }
}

fn test(v: u64) {
    let mut tree_map = TreeMap::<bool>::new();
    tree_map.insert(v, false);
    tree_map.insert(v + 1, false);
    tree_map.insert(v, true);
    tree_map.delete(v);
    let elem17 = tree_map.get(v);
    let elem18 = tree_map.get(v + 1);
    assert!(elem17.is_none());
    assert!(elem18 == Some(&false));
    test2(tree_map, v + 2, v + 3);
}

fn test2(tree_map: TreeMap<bool>, key1: u64, key2: u64) {
    let mut tree_map = tree_map;
    tree_map.insert(key1, true);
    tree_map.insert(key2, true);
}

fn main() { }