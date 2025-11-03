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

// ========== INFERRED VIEW IMPLEMENTATION ==========
impl<V> View for DoublyLinkedList<V> {
    /// Representation of this list as a sequence
    type V_list = Seq<V>;
    pub closed spec fn view(&self) -> self::V_list {
        Seq::<V>::new(
            self.ghost_state@.ptrs.len(),
            |i: int| { self.ghost_state@.points_to_map[i as nat].value().payload },
        )
    }
}
// ==================================================
