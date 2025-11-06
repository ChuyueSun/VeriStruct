/*
u64 bit vector library begins
*/

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

macro_rules! set_bit64_macro {
    ($a:expr,$b:expr, $c:expr) => {{
        if $c {
            $a | 1u64 << $b
        } else {
            $a & (!(1u64 << $b))
        }
    }};
}

// since this wraps with `verus_proof_macro_exprs`, should use the above `set_bit64_macro` if it is going to be executable.
#[allow(unused_macros)]
macro_rules! set_bit64 {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(set_bit64_macro!($($a)*))
    }
}

#[verifier::bit_vector]
proof fn set_bit64_proof(bv_new: u64, bv_old: u64, index: u64, bit: bool)
    requires
        bv_new == set_bit64!(bv_old, index, bit),
        index < 64,
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64| #![auto]
            (loc2 < 64 && loc2 != index) ==> (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2)),
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2,
    ensures
        forall|i: u64| #![auto]
            (i < 64) ==> get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i)),
{
}
