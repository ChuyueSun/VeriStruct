fn reverse(v: &mut Vec<u64>)
    // TODO: add requires and ensures
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        // TODO: add invariants
    {
        let x = v[n];
        let y = v[length - 1 - n];
        v.set(n, y);
        v.set(length - 1 - n, x);
    }
}
