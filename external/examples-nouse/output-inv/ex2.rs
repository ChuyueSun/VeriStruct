use vstd::prelude::*;

pub fn main() {}

verus! {
    struct Vector<V> {
    pub ptr: PPtr<V>,
    pub len: usize,
    pub capacity: usize,
    pub elems: Tracked<Map<nat, PointsTo<V>>>,
    pub rest: Tracked<PointsToRaw>,
    pub dealloc: Tracked<DeallocRaw>,
    }

    impl<V> Vector<V> {

    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.len <= self.capacity
        &&& (forall|i: nat| 0 <= i < self.len ==> self.elems@.dom().contains(i))
        &&& (forall|i: nat|
            0 <= i < self.len ==> (#[trigger] self.elems@.index(i))@.pptr == self.ptr.id()
                + i as int * size_of::<V>())
        &&& (forall|i: nat|
            0 <= i < self.len ==> (#[trigger] self.elems@.index(i))@.value.is_Some())
        &&& self.rest@.is_range(
            self.ptr.id() + self.len * size_of::<V>(),
            (self.capacity - self.len) * size_of::<V>(),
        )
        &&& self.dealloc@@.pptr == self.ptr.id()
        &&& self.dealloc@@.size == self.capacity * size_of::<V>()
        &&& self.dealloc@@.align == align_of::<V>()
        &&& is_sized::<V>()
    }

    pub closed spec fn view(&self) -> Seq<V> {
        Seq::new(self.len as nat, |i: int| self.elems@.index(i as nat)@.value.get_Some_0())
    }
    }
}