pub fn insert_seq(&mut vt, v: u64)
    // ========== INFERRED SPECIFICATIONS ==========
    ensures
        vt@ =~= old(vt)@.insert(v),
    // =============================================
{
    vt.push(v);
    // add proof
}
