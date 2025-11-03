// Function signature given, infer requires/ensures below:
fn swap_elements(v: &mut Vec<u64>, i: usize, j: usize)
    // ========== INFERRED SPECIFICATIONS ==========
    requires
        i < old(v)@.len(),
        j < old(v)@.len(),
    ensures
        v@.len() == old(v)@.len(),
        v@[i as int] == old(v)@[j as int],
        v@[j as int] == old(v)@[i as int],
        forall|k: int| 0 <= k < v@.len() && k != i && k != j ==>
            v@[k] == old(v)@[k]
    // =============================================
{
    let temp = v[i];
    v.set(i, v[j]);
    v.set(j, temp);
}
