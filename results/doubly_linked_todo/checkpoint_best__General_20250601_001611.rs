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
        spec fn prev_of(&self, i: nat) -> ( Option<PPtr<Node<V>>> ) as Option<PPtr<Node<V>>> {
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
            /* TODO: part of view */
            Seq::new(
                self.ghost_state@.ptrs.len(),
                |i: int| self.ghost_state@.points_to_map[i as nat].value().payload
            )
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
                self.ghost_state.borrow_mut().ptrs = self.ghost_state@.ptrs.push(ptr);
                self.ghost_state.borrow_mut().points_to_map.tracked_insert(
                    (self.ghost_state@.ptrs.len() - 1) as nat,
                    points_to,
                );
            }
        }

        pub fn push_back(&mut self, v: V)
            // TODO: add requires and ensures
        {
            match self.tail {
                None => {
                    proof {
                        assert_by_contradiction!(self.ghost_state@.ptrs.len() == 0,
                        {
                            assert(self.well_formed_node((self.ghost_state@.ptrs.len() - 1) as nat));
                        });
                    }
                    self.push_empty_case(v);
                }
                Some(old_tail_ptr) => {
                    proof {
                        assert(self.well_formed_node((self.ghost_state@.ptrs.len() - 1) as nat));
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
                        self.ghost_state.borrow_mut().points_to_map.tracked_insert(
                            (self.ghost_state@.ptrs.len() - 1) as nat,
                            old_tail_pointsto,
                        );
                    }
                    self.tail = Some(new_tail_ptr);

                    proof {
                        self.ghost_state.borrow_mut().points_to_map.tracked_insert(self.ghost_state@.ptrs.len(), new_tail_pointsto);
                        self.ghost_state@.ptrs = self.ghost_state@.ptrs.push(new_tail_ptr);

                        assert(self.well_formed_node((self.ghost_state@.ptrs.len() - 2) as nat));
                        assert(self.well_formed_node((self.ghost_state@.ptrs.len() - 1) as nat));
                        assert(forall|i: nat| i < self.ghost_state@.ptrs.len() && old(self).well_formed_node(i)
                            ==> self.well_formed_node(i));
                        assert forall|i: int| 0 <= i && i < self.ghost_state@.ptrs.len() as int - 1
                            implies old(self)@[i] == self@[i]
                        by {
                            assert(old(self).well_formed_node(i as nat));
                        }
                        assert(self@ =~= old(self)@.push(v));

                        assert(self.well_formed());
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
                        assert_by_contradiction!(self.ghost_state@.ptrs.len() == 1,
                        {
                            assert(old(self).well_formed_node((self.ghost_state@.ptrs.len() - 2) as nat));
                        });
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
                        self.ghost_state.borrow_mut().points_to_map.tracked_insert(
                            (self.ghost_state@.ptrs.len() - 2) as nat,
                            penultimate_pointsto,
                        );
                    }
                },
            }

            proof {
                self.ghost_state@.ptrs = self.ghost_state@.ptrs.drop_last();
                if self.ghost_state@.ptrs.len() > 0 {
                    assert(self.well_formed_node((self.ghost_state@.ptrs.len() - 1) as nat));
                }
                assert(forall|i: nat| i < self@.len() && old(self).well_formed_node(i) ==> self.well_formed_node(i));
                assert forall|i: int| 0 <= i && i < self@.len() implies #[trigger] self@[i] == old(
                    self,
                )@.drop_last()[i] by {
                    assert(old(self).well_formed_node(i as nat));
                }
                assert(self@ =~= old(self)@.drop_last());

                assert(self.well_formed());
            }

            return v;
        }

        pub fn push_front(&mut self, v: V)
            // TODO: add requires and ensures
        {
            match self.head {
                None => {
                    proof {
                        assert_by_contradiction!(self.ghost_state@.ptrs.len() == 0, {
                            assert(self.well_formed_node((self.ghost_state@.ptrs.len() - 1) as nat));
                        });
                    }
                    self.push_empty_case(v);
                    assert(self@ =~= seq![v].add(old(self)@));
                }
                Some(old_head_ptr) => {
                    proof {
                        assert(self.ghost_state@.ptrs.len() > 0);
                        assert(self.well_formed_node(0));
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
                        self.ghost_state.borrow_mut().points_to_map.tracked_insert(0, old_head_pointsto);
                    }
                    self.head = Some(new_head_ptr);

                    proof {
                        assert forall|j: nat|
                            0 <= j && j < old(self)@.len() implies self.ghost_state@.points_to_map.dom().contains(
                            j,
                        ) by {
                            assert(old(self).well_formed_node(j));
                        }
                        self.ghost_state.borrow_mut().points_to_map.tracked_map_keys_in_place(
                            Map::<nat, nat>::new(
                                |j: nat| 1 <= j && j <= old(self).view().len(),
                                |j: nat| (j - 1) as nat,
                            ),
                        );
                        self.ghost_state.borrow_mut().points_to_map.tracked_insert(0, new_head_pointsto);
                        self.ghost_state@.ptrs = seq![new_head_ptr].add(self.ghost_state@.ptrs);

                        assert(self.well_formed_node(0));
                        assert(self.well_formed_node(1));
                        assert(forall|i: nat|
                            1 <= i && i <= old(self).ghost_state@.ptrs.len() && old(self).well_formed_node((i - 1) as nat)
                                ==> #[trigger] self.well_formed_node(i));
                        assert forall|i: int| 1 <= i && i <= self.ghost_state@.ptrs.len() as int - 1
                            implies old(self)@[i - 1] == self@[i]
                        by {
                            assert(old(self).well_formed_node((i - 1) as nat));
                        }
                        assert(self@ =~= seq![v].add(old(self)@));

                        assert(self.well_formed());
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
                        assert_by_contradiction!(self.ghost_state@.ptrs.len() == 1,
                        {
                            assert(old(self).well_formed_node(1));
                        });
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
                        self.ghost_state.borrow_mut().points_to_map.tracked_insert(1, second_pointsto);

                        assert forall|j: nat|
                            1 <= j && j < old(self)@.len() implies self.ghost_state@.points_to_map.dom().contains(
                            j,
                        ) by {
                            assert(old(self).well_formed_node(j));
                        };
                        self.ghost_state.borrow_mut().points_to_map.tracked_map_keys_in_place(
                            Map::<nat, nat>::new(
                                |j: nat| 0 <= j && j < old(self).view().len() - 1,
                                |j: nat| (j + 1) as nat,
                            ),
                        );
                    }
                }
            }

            proof {
                self.ghost_state@.ptrs = self.ghost_state@.ptrs.subrange(1, self.ghost_state@.ptrs.len() as int);
                if self.ghost_state@.ptrs.len() > 0 {
                    assert(self.well_formed_node(0));
                }
                assert(forall|i: nat|
                    i < self.view().len() && old(self).well_formed_node(i + 1) ==> self.well_formed_node(i));
                assert forall|i: int| 0 <= i && i < self@.len() implies #[trigger] self@[i] == old(
                    self,
                )@.subrange(1, old(self)@.len() as int)[i] by {
                    assert(old(self).well_formed_node(i as nat + 1));
                }
                assert(self@ =~= old(self)@.subrange(1, old(self)@.len() as int));

                assert(self.well_formed());
            }

            return v;
        }

        fn get<'a>(&'a self, i: usize) -> (v: &'a V)
            // TODO: add requires and ensures
        {
            let mut j = 0;
            let mut ptr = self.head.unwrap();
            while j < i
                invariant
                    self.well_formed(),
                    0 <= j <= i < self@.len(),
                    ptr == self.ghost_state@.ptrs[j as int],
            {
                proof {
                    assert(self.well_formed_node(j as nat));
                }

                let tracked pointsto_ref: &PointsTo<Node<V>> =
                    self.ghost_state.borrow().points_to_map.tracked_borrow(j as nat);
                let node_ref: &Node<V> = ptr.borrow(Tracked(pointsto_ref));
                let next_ptr = node_ref.next.unwrap();

                j += 1;
                ptr = next_ptr;
            }

            proof {
                assert(self.well_formed_node(j as nat));
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
                self.index@ = self.index@ + 1;
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

mod main {
    use super::doubly_linked_list::{DoublyLinkedList, Iterator};

    pub fn run() {
        let mut t = DoublyLinkedList::<u32>::new();
        t.push_back(2);
        t.push_back(3);
        t.push_front(1);
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
        let x = t.pop_back();
        let y = t.pop_front();
        let z = t.pop_front();
        assert(x == 3);
        assert(y == 1);
        assert(z == 2);
    }

}

fn main() {
    main::run();
}

} // verus!


// Checkpoint Best VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
// Compilation Error: True