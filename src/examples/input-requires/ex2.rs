use vstd::prelude::*;

verus! {
    pub fn ex_u8_max(a: u8, b: u8) -> (res: u8)
        //TODO: add specification
        {
        a.max(b)
        }
}
