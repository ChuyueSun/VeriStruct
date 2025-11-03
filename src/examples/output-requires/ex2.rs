use vstd::prelude::*;

verus! {
    pub fn ex_u8_max(a: u8, b: u8) -> (res: u8)
        // ========== INFERRED SPECIFICATIONS ==========
        ensures res == if a > b { a } else { b },
        // =============================================
    {
        a.max(b)
    }
}
