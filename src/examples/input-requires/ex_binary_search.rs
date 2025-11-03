fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    // TODO: add requires and ensures
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        // TODO: add invariant
    {
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
    }
    i1
}
