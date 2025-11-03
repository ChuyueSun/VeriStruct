use vstd::prelude::*;

pub fn main() {}

verus! {
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
        /// Representation of this list as a sequence
        pub closed spec fn view(&self) -> Seq<V> {
            Seq::<V>::new(
                self.ghost_state@.ptrs.len(),
                |i: int| { self.ghost_state@.points_to_map[i as nat].value().payload },
            )
        }

        /// Node at index `i` is well-formed
        spec fn inv_node(&self, i: nat) -> bool {
            &&& self.ghost_state@.points_to_map.dom().contains(i)
            &&& self.ghost_state@.points_to_map[i].pptr() == self.ghost_state@.ptrs[i as int]
            &&& self.ghost_state@.points_to_map[i].mem_contents() matches MemContents::Init(node)
                    && node.prev == self.prev_of(i) && node.next == self.next_of(i)
        }

        /// Linked list is well-formed
        #[verifier::type_invariant]
        closed spec fn inv(&self) -> bool {
            // Every node from 0 .. len - 1 is well-formed
            &&& forall|i: nat| 0 <= i && i < self.ghost_state@.ptrs.len() ==> self.inv_node(i)
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
    }
}
