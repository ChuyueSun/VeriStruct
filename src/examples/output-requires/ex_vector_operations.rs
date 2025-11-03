// Function signature given, infer requires/ensures below:
fn find_max(v: &Vec<u64>) -> (max_idx: usize)
    // ========== INFERRED SPECIFICATIONS ==========
    requires
        v@.len() > 0,
    ensures
        max_idx < v@.len(),
        forall|i: int| 0 <= i < v@.len() ==> v@[max_idx as int] >= v@[i]
    // =============================================
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
    // ========== INFERRED SPECIFICATIONS ==========
    requires
        v@.len() <= usize::MAX,
    ensures
        count as int == count_occurrences(v@, target)
    // =============================================
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

spec fn count_occurrences(s: Seq<u64>, target: u64) -> int
    decreases s.len()
{
    if s.len() == 0 {
        0
    } else {
        (if s[0] == target { 1 } else { 0 }) + count_occurrences(s.skip(1), target)
    }
}

fn copy_range(src: &Vec<u64>, start: usize, end: usize, dst: &mut Vec<u64>)
    // ========== INFERRED SPECIFICATIONS ==========
    requires
        start <= end,
        end <= src@.len(),
        old(dst)@.len() + (end - start) <= usize::MAX,
    ensures
        dst@.len() == old(dst)@.len() + (end - start),
        forall|i: int| 0 <= i < old(dst)@.len() ==>
            dst@[i] == old(dst)@[i],
        forall|i: int| 0 <= i < (end - start) ==>
            dst@[old(dst)@.len() + i] == src@[start + i]
    // =============================================
{
    for i in start..end
        // TODO: add invariants
    {
        dst.push(src[i]);
    }
}
