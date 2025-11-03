fn process_values(a: u64, b: u32, c: usize, d: i32) -> (result: bool)
    // TODO: add requires and ensures
{
    a < b as u64 && c < d as usize
}

fn vector_length_check(v: &Vec<u64>) -> (ok: bool)
    // TODO: add requires and ensures
{
    v.len() < 1000
}
