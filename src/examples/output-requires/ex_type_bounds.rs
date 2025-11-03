// Function signature given, infer requires/ensures below:
fn process_values(a: u64, b: u32, c: usize, d: i32) -> (result: bool)
    // ========== INFERRED SPECIFICATIONS ==========
    requires
        a <= u64::MAX,
        b <= u32::MAX,
        c <= usize::MAX,
        d >= i32::MIN,
        d <= i32::MAX,
    ensures
        result == (a < b as u64 && c < d as usize)
    // =============================================
{
    a < b as u64 && c < d as usize
}

fn vector_length_check(v: &Vec<u64>) -> (ok: bool)
    // ========== INFERRED SPECIFICATIONS ==========
    requires
        v@.len() <= usize::MAX as nat,
    ensures
        ok == (v@.len() < 1000)
    // =============================================
{
    v.len() < 1000
}
