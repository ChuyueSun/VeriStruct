struct Vector<V> {
    pub ptr: PPtr<V>,
    pub len: usize,
    pub capacity: usize,
    pub elems: Tracked<Map<nat, PointsTo<V>>>,
    pub rest: Tracked<PointsToRaw>,
    pub dealloc: Tracked<DeallocRaw>,
}

impl<V> Vector<V> {

    pub closed spec fn view(&self) -> Seq<V> {
            // TODO: implement this. Use sequence to represent the vector as a sequence.
        }
}
