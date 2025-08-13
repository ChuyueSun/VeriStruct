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

impl<V> View for DoublyLinkedList<V> {
    // TODO: implement this
}