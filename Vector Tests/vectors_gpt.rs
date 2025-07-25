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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_found_exact() {
        let v = vec![1, 3, 5, 7, 9];
        assert_eq!(binary_search(&v, 5), 2);
        assert_eq!(binary_search(&v, 1), 0);
        assert_eq!(binary_search(&v, 9), 4);
    }

    #[test]
    fn test_binary_search_found_not_exact() {
        let v = vec![1, 3, 5, 7, 9];
        // For values not present, it returns the smallest index where v[ix] >= k
        assert_eq!(binary_search(&v, 4), 2); // 5 is first >= 4
        assert_eq!(binary_search(&v, 0), 0); // 1 is first >= 0
        assert_eq!(binary_search(&v, 10), 4); // returns last index since 10 > all
    }

    #[test]
    fn test_binary_search_single_element() {
        let v = vec![5];
        assert_eq!(binary_search(&v, 5), 0);
        assert_eq!(binary_search(&v, 4), 0);
        assert_eq!(binary_search(&v, 6), 0);
    }

    #[test]
    #[should_panic]
    fn test_binary_search_empty_vector() {
        let v: Vec<u64> = vec![];
        // This will panic due to usize - 1 underflow when v.len() == 0
        binary_search(&v, 1);
    }

    #[test]
    fn test_reverse_even_length() {
        let mut v = vec![1, 2, 3, 4];
        reverse(&mut v);
        assert_eq!(v, vec![4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_odd_length() {
        let mut v = vec![1, 2, 3, 4, 5];
        reverse(&mut v);
        assert_eq!(v, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_single_element() {
        let mut v = vec![1];
        reverse(&mut v);
        assert_eq!(v, vec![1]);
    }

    #[test]
    fn test_reverse_empty_vector() {
        let mut v: Vec<u64> = vec![];
        reverse(&mut v);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_binary_search_no_spinoff_equivalence() {
        let v = vec![2, 4, 6, 8, 10];
        for &k in &[1, 2, 3, 5, 10, 11] {
            assert_eq!(binary_search(&v, k), binary_search_no_spinoff(&v, k));
        }
    }
}