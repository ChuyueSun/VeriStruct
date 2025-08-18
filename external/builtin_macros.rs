The bit_vector solver doesn’t allow arbitrary functions. However, you can use macros. This is useful when certain operations need a common shorthand, like “get the ith bit of an integer”.


macro_rules! get_bit_macro {
    ($a:expr, $b:expr) => {{
        (0x1u32 & ($a >> $b)) == 1
    }};
}

macro_rules! get_bit {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(get_bit_macro!($($a)*))
    }
}


verus_proof_macro_exprs!() { /* proc-macro */ }
verus_proof_macro_exprs!(f!(exprs)) applies verus syntax to transform exprs into exprs’, then returns f!(exprs’), where exprs is a sequence of expressions separated by “,”, “;”, and/or “=>”.