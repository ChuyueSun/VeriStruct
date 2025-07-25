fn binary_search(v: &Vec<u64>, k: u64) -> usize
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
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

fn reverse(v: &mut Vec<u64>)
{
    let length = v.len();
    for n in 0..(length / 2)
    {
        let x = v[n];
        let y = v[length - 1 - n];
        v[n] = y;
        v[length - 1 - n] = x;
    }
}

fn binary_search_no_spinoff(v: &Vec<u64>, k: u64) -> usize
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
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

/*
TEST CODE BELOW
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop() {
        let mut t = vec![1, 2, 3];
        pop_test(t);
    }

    #[test]
    fn test_push() {
        let t = vec![1, 2, 3];
        push_test(t, 4);
    }

    #[test]
    fn test_binary_search() {
        let t = vec![1, 2, 3, 4, 5];
        binary_search_test(t);
    }

    #[test]
    fn test_reverse() {
        let mut t = vec![1, 2, 3, 4, 5];
        reverse_test(&mut t);
    }

    fn pop_test(t: Vec<u64>)
    {
        let mut t = t;
        let _x = t.pop().unwrap();
    }

    fn push_test(t: Vec<u64>, y: u64)
    {
        let mut t = t;
        t.push(y);
    }

    fn binary_search_test(t: Vec<u64>)
    {
        for i in 0 .. t.len()
        {
            let k = t[i];
            let r = binary_search(&t, k);
            assert!(r < t.len());
            assert!(t[r] == k);
            let r = binary_search_no_spinoff(&t, k);
            assert!(r < t.len());
            assert!(t[r] == k);
        }
    }

    fn reverse_test(t: &mut Vec<u64>)
    {
        reverse(t);
    }
}

fn main() {}