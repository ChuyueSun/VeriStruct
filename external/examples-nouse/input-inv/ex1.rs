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
        pub closed spec fn inv(&self) -> bool {
            // TODO: implement this.
        }
    }
}