#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{
    pub struct Node<V> {
        pub key: u64,
        pub value: V,
        pub left: Option<Box<Node<V>>>,
        pub right: Option<Box<Node<V>>>,
    }

    impl<V> Node<V> {
        pub open spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
            decreases node_opt,
        {
            match node_opt {
                None => Map::empty(),
                Some(node) => node.as_map(),
            }
        }

        pub open spec fn as_map(self) -> Map<u64, V>
            decreases self,
        {
            Node::<V>::optional_as_map(self.left)
              .union_prefer_right(Node::<V>::optional_as_map(self.right))
              .insert(self.key, self.value)
        }

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

        pub fn insert(&mut self, key: u64, value: V)
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

        pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
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

        pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
            requires node.is_some() ==> node.unwrap().well_formed(),
            ensures ret==(match node {
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

        pub fn get(&self, key: u64) -> (ret: Option<&V>)
            requires self.well_formed(),
            ensures ret==(if self.as_map().dom().contains(key) { Some(&self.as_map()[key]) } else { None })
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

    pub struct TreeMap<V> {
        root: Option<Box<Node<V>>>,
    }

    impl<V> TreeMap<V> {
        pub closed spec fn as_map(self) -> Map<u64, V>
            ensures
                result == Node::<V>::optional_as_map(self.root)
        {
            Node::<V>::optional_as_map(self.root)
        }
    }

    impl<V> View for TreeMap<V> {
        type V = Map<u64, V>;

        open spec fn view(&self) -> Map<u64, V>
            ensures
                result == self.as_map()
        {
            self.as_map()
        }
    }

    impl<V> TreeMap<V> {
        #[verifier::type_invariant]
        spec fn well_formed(self) -> bool {
            match self.root {
                Some(node) => node.well_formed(),
                None => true,
            }
        }
    }

    impl<V> TreeMap<V> {
        pub fn new() -> (s: Self)
            requires
                true,
            ensures
                s.as_map() == Map::empty(),
                s.well_formed()
        {
            TreeMap::<V> { root: None }
        }

        pub fn insert(&mut self, key: u64, value: V)
            ensures
                self.well_formed(),
                self.as_map() =~= old(self).as_map().insert(key, value)
        {
            proof {
            }
            let mut root = None;
            std::mem::swap(&mut root, &mut self.root);
            Node::<V>::insert_into_optional(&mut root, key, value);
            self.root = root;
        }

        pub fn delete(&mut self, key: u64)
            ensures
                self.well_formed(),
                self.as_map() =~= old(self).as_map().remove(key)
        {
            proof { use_type_invariant(&*self); }
            let mut root = None;
            std::mem::swap(&mut root, &mut self.root);
            Node::<V>::delete_from_optional(&mut root, key);
            self.root = root;
        }

        pub fn get(&self, key: u64) -> (ret: Option<&V>)
            ensures
                ret == (if self.as_map().dom().contains(key) { Some(&self.as_map()[key]) } else { None })
        {
            proof { use_type_invariant(&*self); }
            Node::<V>::get_from_optional(&self.root, key)
        }
    }

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

    fn main() { }
}
