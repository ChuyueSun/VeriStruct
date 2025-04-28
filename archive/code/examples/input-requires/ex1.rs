use vstd::prelude::*;

verus! {

mod doubly_linked_list {
    use vstd::prelude::*;
    use vstd::simple_pptr::*;
    use vstd::raw_ptr::MemContents;
    use vstd::assert_by_contradiction;

    // Single node in the list
    struct Node<V> {
        prev: Option<PPtr<Node<V>>>,
        next: Option<PPtr<Node<V>>>,
        payload: V,
    }

    // Doubly-linked list
    // Contains head pointer, tail pointer
    // and in ghost code, tracks all the pointers and all the PointsTo permissions
    // to access the nodes
    pub struct DoublyLinkedList<V> {
        // physical data:
        head: Option<PPtr<Node<V>>>,
        tail: Option<PPtr<Node<V>>>,

        // ghost and tracked data:
        ghost_state: Tracked<GhostState<V>>,
    }

    pub tracked struct GhostState<V> {
        ghost ptrs: Seq<PPtr<Node<V>>>,
        tracked points_to_map: Map<nat, PointsTo<Node<V>>>,
    }

    impl<V> DoublyLinkedList<V> {
        /// Pointer to the node of index (i-1), or None if i is 0.
        spec fn prev_of(&self, i: nat) -> Option<PPtr<Node<V>>> {
            if i == 0 {
                None
            } else {
                Some(self.ghost_state@.ptrs[i as int - 1])
            }
        }

        /// Pointer to the node of index (i+1), or None if i is the last index.
        spec fn next_of(&self, i: nat) -> Option<PPtr<Node<V>>> {
            if i + 1 == self.ghost_state@.ptrs.len() {
                None
            } else {
                Some(self.ghost_state@.ptrs[i as int + 1])
            }
        }

        /// Node at index `i` is well-formed
        spec fn well_formed_node(&self, i: nat) -> bool {
            &&& self.ghost_state@.points_to_map.dom().contains(i)
            &&& self.ghost_state@.points_to_map[i].pptr() == self.ghost_state@.ptrs[i as int]
            &&& self.ghost_state@.points_to_map[i].mem_contents() matches MemContents::Init(node)
                  && node.prev == self.prev_of(i) && node.next == self.next_of(i)
        }

        /// Linked list is well-formed
        #[verifier::type_invariant]
        closed spec fn inv(&self) -> bool {
            // Every node from 0 .. len - 1 is well-formed
            &&& forall|i: nat| 0 <= i && i < self.ghost_state@.ptrs.len() ==> self.well_formed_node(i)
            &&& if self.ghost_state@.ptrs.len() == 0 {
                // If the list is empty, then the `head` and `tail` pointers are both None
                self.head.is_none() && self.tail.is_none()
            } else {
                // If the list is non-empty, then `head` and `tail` pointers point to the
                // the first and last nodes.
                &&& self.head == Some(self.ghost_state@.ptrs[0])
                &&& self.tail == Some(self.ghost_state@.ptrs[self.ghost_state@.ptrs.len() as int - 1])
            }
        }

        /// Representation of this list as a sequence
        pub closed spec fn view(&self) -> Seq<V> {
            Seq::<V>::new(
                self.ghost_state@.ptrs.len(),
                |i: int| { self.ghost_state@.points_to_map[i as nat].value().payload },
            )
        }

        //// Interface of executable functions

        /// Construct a new, empty, doubly-linked list.
        pub fn new() -> (s: Self)
        // TODO: implement this.
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

        /// Insert one node, assuming the linked list is empty.
        fn push_empty_case(&mut self, v: V)
        // TODO: implement this.
        {
            // Allocate a node to contain the payload
            let (ptr, Tracked(points_to)) = PPtr::<Node<V>>::new(
                Node::<V> { prev: None, next: None, payload: v },
            );

            // Update head and tail pointers
            self.tail = Some(ptr);
            self.head = Some(ptr);
        }

        /// Insert a value to the end of the list
        pub fn push_back(&mut self, v: V)
        // TODO: implement this.
        {
            match self.tail {
                None => {
                    // Special case: list is empty
                    self.push_empty_case(v);
                }
                Some(old_tail_ptr) => {
                    // Allocate a new node to go on the end. It's 'prev' field points
                    // to the old tail pointer.
                    let (new_tail_ptr, Tracked(new_tail_pointsto)) = PPtr::<Node<V>>::new(
                        Node::<V> { prev: Some(old_tail_ptr), next: None, payload: v },
                    );

                    // Update the 'next' pointer of the previous tail node
                    // This is all equivalent to `(*old_tail_ptr).next = new_tail_ptr;`
                    let tracked mut old_tail_pointsto: PointsTo<Node<V>> =
                        self.ghost_state.borrow_mut().points_to_map.tracked_remove((self.ghost_state@.ptrs.len() - 1) as nat);
                    let mut old_tail_node = old_tail_ptr.take(Tracked(&mut old_tail_pointsto));
                    old_tail_node.next = Some(new_tail_ptr);
                    old_tail_ptr.put(Tracked(&mut old_tail_pointsto), old_tail_node);
                    // Update `self.tail`
                    self.tail = Some(new_tail_ptr);
                }
            }
        }
    }
}
} // verus!
