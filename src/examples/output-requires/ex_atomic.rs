pub open spec fn well_formed(&self) -> bool {
    self.atomic_inv@.constant().1 == self.patomic.id()
}

#[inline(always)]
pub fn load(&self) -> $value_ty
    requires self.well_formed(),
{
    atomic_with_ghost!(self => load(); g => { })
}