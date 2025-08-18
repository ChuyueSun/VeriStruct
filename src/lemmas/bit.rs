
/// Macro for extracting a single bit from a u64 value
/// 
/// # Arguments
/// * `$a` - The u64 value to extract the bit from
/// * `$b` - The bit position (0-63) to extract
/// 
/// # Returns
/// * `true` if the bit at position `$b` is 1
/// * `false` if the bit at position `$b` is 0
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

/// Macro for setting a single bit in a u64 value
/// 
/// # Arguments
/// * `$a` - The u64 value to modify
/// * `$b` - The bit position (0-63) to set
/// * `$c` - The boolean value to set the bit to (true = 1, false = 0)
/// 
/// # Returns
/// A new u64 with the specified bit modified and all other bits preserved
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

spec fn u64_view(u: u64) -> Seq<bool> {
    Seq::new(64, |i: int| get_bit64!(u, i as u64))
}

#[verifier::bit_vector]
proof fn set_bit64_proof(bv_new: u64, bv_old: u64, index: u64, bit: bool)
    requires
        bv_new == set_bit64!(bv_old, index, bit),
        index < 64,
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64|
            (loc2 < 64 && loc2 != index) ==> (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2)),
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2,
    ensures
        forall|i: u64|
            (i < 64) ==> get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i)),
{
}

proof fn bit_or_64_view_proof(u1: u64, u2: u64, bv_new: u64)
    requires
        bv_new == u1 | u2,
    ensures
        u64_view(bv_new) =~= Seq::new(64, |i: int| u64_view(u1).index(i) || u64_view(u2).index(i)),
{
    bit_or_64_proof(u1, u2, bv_new);
}

spec fn or_u64_relation(u1: u64, u2: u64, or_int: u64) -> bool {
    u64_view(or_int) =~= Seq::new(64, |i: int| u64_view(u1).index(i) || u64_view(u2).index(i))
}