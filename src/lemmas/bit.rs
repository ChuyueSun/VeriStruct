
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