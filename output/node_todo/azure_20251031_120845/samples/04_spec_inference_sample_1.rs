use vstd::prelude::*;

verus!{

/// Inserts a key-value pair into an optional node, creating a new node if None.
pub fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
    requires
        // If the node existed before, it must be well_formed
        old(node).is_some() ==> old(node).unwrap().well_formed()
    ensures
        // The node remains well_formed if it exists
        node.is_some() ==> node.unwrap().well_formed(),
        // The map representation is updated with (key, value)
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
pub fn insert(&mut self, key: u64, value: V)
    requires
        // The node must be well_formed before insertion
        old(self).well_formed()
    ensures
        // The node remains well_formed
        self.well_formed(),
        // The map representation is updated with (key, value)
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

/// Deletes a key from an optional node (BST), handling the case where the node might not exist.
pub fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
    requires
        // If the node existed before, it must be well_formed
        old(node).is_some() ==> old(node).unwrap().well_formed()
    ensures
        // The node remains well_formed if it exists
        node.is_some() ==> node.unwrap().well_formed(),
        // The map representation is updated with the key removed
        Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(key)
{
    if node.is_some() {
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
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
pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
    requires
        // The node must exist and be well_formed
        old(node).is_some() ==> old(node).unwrap().well_formed()
    ensures
        // The node remains well_formed if it still exists
        node.is_some() ==> node.unwrap().well_formed(),
        // popped key-value pair was in the original tree
        Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
        Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
        // The resulting map is the original minus the popped key
        Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
        // popped.0 is the largest key in the old map
        forall |k: u64| Node::<V>::optional_as_map(*old(node)).dom().contains(k) ==> k <= popped.0
    {
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        let mut boxed_node = tmp.unwrap();

        if boxed_node.right.is_none() {
            *node = boxed_node.left;
            proof {
                assert(Node::<V>::optional_as_map(boxed_node.right) =~= Map::empty());
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(boxed_node.key));
            }
            return (boxed_node.key, boxed_node.value);
        } else {
            let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.right);
            proof {
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(popped_key));
            }
            *node = Some(boxed_node);
            return (popped_key, popped_value);
        }
    }

/// Looks up a key in an optional node, handling the case where the node might not exist.
pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
    requires
        // If node has a value, it must be well_formed
        node.is_some() ==> node.unwrap().well_formed()
    ensures
        // If ret is Some, then key must have been in the map
        // If ret is None, then key was not in the map
        (ret.is_none() && !Node::<V>::optional_as_map(*node).dom().contains(key))
        ||
        (ret.is_some() && Node::<V>::optional_as_map(*node).dom().contains(key)
         && *ret.unwrap() == Node::<V>::optional_as_map(*node)[key])
{
    match node {
        None => None,
        Some(node) => {
            node.get(key)
        }
    }
}

/// Looks up a key in this node's subtree using BST search.
pub fn get(&self, key: u64) -> (ret: Option<&V>)
    requires
        // The node must be well_formed
        self.well_formed()
    ensures
        // If ret is Some, the key was in the map
        // If ret is None, the key was not in the map
        (ret.is_none() && !self.as_map().dom().contains(key))
        ||
        (ret.is_some() && self.as_map().dom().contains(key) && *ret.unwrap() == self.as_map()[key])
{
    if key == self.key {
        Some(&self.value)
    } else if key < self.key {
        proof {
            assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
        }
        Self::get_from_optional(&self.left, key)
    } else {
        proof {
            assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
        }
        Self::get_from_optional(&self.right, key)
    }
}

}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 79
