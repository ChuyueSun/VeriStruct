// Example showing common vector operations with proper specifications

fn find_max(v: &Vec<u64>) -> (max_idx: usize)
    // TODO: add requires and ensures
{
    let mut max_idx: usize = 0;
    for i in 1..v.len()
        // TODO: add invariants
    {
        if v[i] > v[max_idx] {
            max_idx = i;
        }
    }
    max_idx
}

fn count_value(v: &Vec<u64>, target: u64) -> (count: usize)
    // TODO: add requires and ensures
{
    let mut count: usize = 0;
    for i in 0..v.len()
        // TODO: add invariants
    {
        if v[i] == target {
            count = count + 1;
        }
    }
    count
}

fn copy_range(src: &Vec<u64>, start: usize, end: usize, dst: &mut Vec<u64>)
    // TODO: add requires and ensures
{
    for i in start..end
        // TODO: add invariants
    {
        dst.push(src[i]);
    }
}
