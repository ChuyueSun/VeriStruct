use vstd::prelude::*;

verus! {

mod doubly_linked_list {
    use vstd::prelude::*;
    use vstd::simple_pptr::*;
    use vstd::raw_ptr::MemContents;
    use vstd::assert_by_contradiction;

    struct Node<V> {
        prev: Option<PPtr<Node<V>>>,
        next: Option<PPtr<Node<V>>>,
        payload: V,
    }

    pub struct DoublyLinkedList<V> {
        head: Option<PPtr<Node<V>>>,
        tail: Option<PPtr<Node<V>>>,
        ghost_state: Tracked<GhostState<V>>,
    }

    pub tracked struct GhostState<V> {
        ghost ptrs: Seq<PPtr<Node<V>>>,
        tracked points_to_map: Map<nat, PointsTo<Node<V>>>,
    }

    impl<V> DoublyLinkedList<V> {
        spec fn prev_of(&self, i: nat) -> Option<PPtr<Node<V>>> {
            // TODO: add specification
        }

        spec fn next_of(&self, i: nat) -> Option<PPtr<Node<V>>> {
            // TODO: add specification
        }

        spec fn well_formed_node(&self, i: nat) -> bool {
            // TODO: add specification
        }

        pub closed spec fn well_formed(&self) -> bool {
            // TODO: add specification
        }

        pub closed spec fn view(&self) -> Seq<V> {
            // TODO: add specification
        }

        pub fn new() -> (s: Self)
            // TODO: add requires and ensures
        {
            DoublyLinkedList {
                ghost_state: Tracked(GhostState {
                    ptrs: Seq::empty(),
                    points_to_map: Map::tracked_empty(),
                }),
                head: None,
                tail: None,
            }
        }

        fn push_empty_case(&mut self, v: V)
            // TODO: add requires and ensures
        {
            let (ptr, Tracked(points_to)) = PPtr::<Node<V>>::new(
                Node::<V> { prev: None, next: None, payload: v },
            );

            self.tail = Some(ptr);
            self.head = Some(ptr);

            proof {
                // TODO: add proof
            }
        }

        pub fn push_back(&mut self, v: V)
            // TODO: add requires and ensures
        {
            match self.tail {
                None => {
                    proof {
                        // TODO: add proof
                    }
                    self.push_empty_case(v);
                }
                Some(old_tail_ptr) => {
                    proof {
                        // TODO: add proof
                    }
                    let (new_tail_ptr, Tracked(new_tail_pointsto)) = PPtr::<Node<V>>::new(
                        Node::<V> { prev: Some(old_tail_ptr), next: None, payload: v },
                    );

                    let tracked mut old_tail_pointsto: PointsTo<Node<V>> =
                        self.ghost_state.borrow_mut().points_to_map.tracked_remove((self.ghost_state@.ptrs.len() - 1) as nat);
                    let mut old_tail_node = old_tail_ptr.take(Tracked(&mut old_tail_pointsto));
                    old_tail_node.next = Some(new_tail_ptr);
                    old_tail_ptr.put(Tracked(&mut old_tail_pointsto), old_tail_node);
                    proof {
                        // TODO: add proof
                    }
                    self.tail = Some(new_tail_ptr);

                    proof {
                        // TODO: add proof
                    }
                }
            }
        }

        pub fn pop_back(&mut self) -> (v: V)
            // TODO: add requires and ensures
        {
            assert(self.well_formed_node((self.ghost_state@.ptrs.len() - 1) as nat));

            let last_ptr = self.tail.unwrap();
            let tracked last_pointsto = self.ghost_state.borrow_mut().points_to_map.tracked_remove(
                (self.ghost_state@.ptrs.len() - 1) as nat,
            );
            let last_node = last_ptr.into_inner(Tracked(last_pointsto));
            let v = last_node.payload;

            match last_node.prev {
                None => {
                    self.tail = None;
                    self.head = None;
                    proof {
                        // TODO: add proof
                    }
                },
                Some(penultimate_ptr) => {
                    assert(old(self)@.len() >= 2);
                    assert(old(self).well_formed_node((self.ghost_state@.ptrs.len() - 2) as nat));

                    self.tail = Some(penultimate_ptr);

                    let tracked mut penultimate_pointsto =
                        self.ghost_state.borrow_mut().points_to_map.tracked_remove((self.ghost_state@.ptrs.len() - 2) as nat);
                    let mut penultimate_node = penultimate_ptr.take(Tracked(&mut penultimate_pointsto));
                    penultimate_node.next = None;
                    penultimate_ptr.put(Tracked(&mut penultimate_pointsto), penultimate_node);
                    proof {
                        // TODO: add proof
                    }
                },
            }

            proof {
                // TODO: add proof
            }

            return v;
        }

        pub fn push_front(&mut self, v: V)
            // TODO: add requires and ensures
        {
            match self.head {
                None => {
                    proof {
                        // TODO: add proof
                    }
                    self.push_empty_case(v);
                    assert(self@ =~= seq![v].add(old(self)@));
                }
                Some(old_head_ptr) => {
                    proof {
                        // TODO: add proof
                    }

                    let (new_head_ptr, Tracked(new_head_pointsto)) = PPtr::new(
                        Node::<V> { prev: None, next: Some(old_head_ptr), payload: v },
                    );

                    let tracked mut old_head_pointsto =
                        self.ghost_state.borrow_mut().points_to_map.tracked_remove(0);
                    let mut old_head_node = old_head_ptr.take(Tracked(&mut old_head_pointsto));
                    old_head_node.prev = Some(new_head_ptr);
                    old_head_ptr.put(Tracked(&mut old_head_pointsto), old_head_node);
                    proof {
                        // TODO: add proof
                    }
                    self.head = Some(new_head_ptr);

                    proof {
                        // TODO: add proof
                    }
                }
            }
        }

        pub fn pop_front(&mut self) -> (v: V)
            // TODO: add requires and ensures
        {
            assert(self.well_formed_node(0));

            let first_ptr = self.head.unwrap();
            let tracked first_pointsto = self.ghost_state.borrow_mut().points_to_map.tracked_remove(0);
            let first_node = first_ptr.into_inner(Tracked(first_pointsto));
            let v = first_node.payload;

            match first_node.next {
                None => {
                    self.tail = None;
                    self.head = None;
                    proof {
                        // TODO: add proof
                    }
                }
                Some(second_ptr) => {
                    assert(old(self)@.len() >= 2);
                    assert(old(self).well_formed_node(1));

                    self.head = Some(second_ptr);

                    let tracked mut second_pointsto = self.ghost_state.borrow_mut().points_to_map.tracked_remove(1);
                    let mut second_node = second_ptr.take(Tracked(&mut second_pointsto));
                    second_node.prev = None;
                    second_ptr.put(Tracked(&mut second_pointsto), second_node);
                    proof {
                        // TODO: add proof
                    }
                }
            }

            proof {
                // TODO: add proof
            }

            return v;
        }

        pub fn get<'a>(&'a self, i: usize) -> (v: &'a V)
            // TODO: add requires and ensures
        {
            let mut j = 0;
            let mut ptr = self.head.unwrap();
            while j < i
                // TODO: add invariant
            {
                proof {
                    // TODO: add proof
                }

                let tracked pointsto_ref: &PointsTo<Node<V>> =
                    self.ghost_state.borrow().points_to_map.tracked_borrow(j as nat);
                let node_ref: &Node<V> = ptr.borrow(Tracked(pointsto_ref));
                let next_ptr = node_ref.next.unwrap();

                j += 1;
                ptr = next_ptr;
            }

            proof {
                // TODO: add proof
            }

            let tracked pointsto_ref: &PointsTo<Node<V>> =
                self.ghost_state.borrow().points_to_map.tracked_borrow(j as nat);
            let node_ref: &Node<V> = ptr.borrow(Tracked(pointsto_ref));
            return &node_ref.payload;
        }
    }

    pub struct Iterator<'a, V> {
        l: &'a DoublyLinkedList<V>,
        cur: Option<PPtr<Node<V>>>,
        index: Ghost<nat>,
    }

    impl<'a, V> Iterator<'a, V> {
        pub closed spec fn list(&self) -> &'a DoublyLinkedList<V> {
            // TODO: add specification
        }

        pub closed spec fn index(&self) -> nat {
            // TODO: add specification
        }

        pub closed spec fn valid(&self) -> bool {
            // TODO: add specification
        }

        pub fn new(l: &'a DoublyLinkedList<V>) -> (it: Self)
            // TODO: add requires and ensures
        {
            Iterator { l, cur: l.head, index: Ghost(0) }
        }

        pub fn value(&self) -> (v: &V)
            // TODO: add requires and ensures
        {
            let cur = self.cur.unwrap();
            assert(self.l.well_formed_node(self.index()));
            let tracked pointsto = self.l.ghost_state.borrow().points_to_map.tracked_borrow(self.index());
            let node = cur.borrow(Tracked(pointsto));
            &node.payload
        }

        pub fn move_next(&mut self) -> (good: bool)
            // TODO: add requires and ensures
        {
            assert(self.l.well_formed_node(self.index()));
            let cur = self.cur.unwrap();
            let tracked pointsto = self.l.ghost_state.borrow().points_to_map.tracked_borrow(self.index());
            let node = cur.borrow(Tracked(pointsto));
            proof {
                // TODO: add proof
            }
            match node.next {
                None => {
                    self.cur = None;
                    false
                },
                Some(next_ptr) => {
                    self.cur = Some(next_ptr);
                    true
                },
            }
        }
    }

}

/* TEST CODE BELOW */

mod main {
    use super::doubly_linked_list::{DoublyLinkedList, Iterator};

    pub fn run() {
        let mut t = DoublyLinkedList::<u32>::new();
        t.push_back(2);
        t.push_back(3);
        let v = t.get(1);
        assert(*v == 3);
        t.push_front(1);  // 1, 2, 3
        let mut it = Iterator::new(&t);
        let v1 = it.value();
        assert(*v1 == 1);
        let g = it.move_next();
        let v2 = it.value();
        assert(*v2 == 2);
        let _ = it.move_next();
        let v3 = it.value();
        assert(*v3 == 3);
        let g = it.move_next();
        assert(!g);
        let x = t.pop_back();  // 3
        let y = t.pop_front();  // 1
        let z = t.pop_front();  // 2
        assert(x == 3);
        assert(y == 1);
        assert(z == 2);
    }

}


fn main() {
    main::run();
}

} // verus!
