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
    pub closed spec fn inv(&self) -> bool {
      // TODO: implement this.
    }

    pub closed spec fn view(&self) -> Seq<V> {
        Seq::new(self.len as nat, |i: int| self.elems@.index(i as nat)@.value.get_Some_0())
    }
  }
}
