fn swap_elements(v: &mut Vec<u64>, i: usize, j: usize)
    // TODO: add requires and ensures
{
    let temp = v[i];
    v.set(i, v[j]);
    v.set(j, temp);
}
