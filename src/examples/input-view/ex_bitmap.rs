macro_rules! get_bit64_macro {
    ($a:expr, $b:expr) => {{
        (0x1u64 & ($a >> $b)) == 1
    }};
}

// since this wraps with `verus_proof_macro_exprs`, should use the above `get_bit64_macro` if it is going to be executable.
#[allow(unused_macros)]
macro_rules! get_bit64 {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(get_bit64_macro!($($a)*))
    }
}


verus! {

spec fn u64_view(u: u64) -> Seq<bool> {
    Seq::new(64, |i: int| get_bit64!(u, i as u64))
}


/*
u64 bit vector library ends
bitmap impl begins
*/

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        // TODO: implement View
    }
}
}
