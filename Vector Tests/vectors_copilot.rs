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
    fn test_binary_search_basic() {
        let v = vec![1, 3, 5, 7, 9];
        assert_eq!(binary_search(&v, 1), 0);
        assert_eq!(binary_search(&v, 5), 2);
        assert_eq!(binary_search(&v, 9), 4);
        assert_eq!(binary_search(&v, 6), 3); // insertion point
    }

    #[test]
    fn test_binary_search_empty() {
        let v: Vec<u64> = vec![];
        // Should panic or error, but let's check for panic safety
        let result = std::panic::catch_unwind(|| binary_search(&v, 1));
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_search_single() {
        let v = vec![10];
        assert_eq!(binary_search(&v, 10), 0);
        assert_eq!(binary_search(&v, 5), 0);
        assert_eq!(binary_search(&v, 15), 0);
    }

    #[test]
    fn test_binary_search_duplicates() {
        let v = vec![2, 2, 2, 2, 2];
        assert_eq!(binary_search(&v, 2), 0);
        assert_eq!(binary_search(&v, 1), 0);
        assert_eq!(binary_search(&v, 3), 4);
    }

    #[test]
    fn test_reverse_even() {
        let mut v = vec![1, 2, 3, 4];
        reverse(&mut v);
        assert_eq!(v, vec![4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_odd() {
        let mut v = vec![1, 2, 3, 4, 5];
        reverse(&mut v);
        assert_eq!(v, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_empty() {
        let mut v: Vec<u64> = vec![];
        reverse(&mut v);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_reverse_single() {
        let mut v = vec![42];
        reverse(&mut v);
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn test_binary_search_no_spinoff_basic() {
        let v = vec![1, 3, 5, 7, 9];
        assert_eq!(binary_search_no_spinoff(&v, 1), 0);
        assert_eq!(binary_search_no_spinoff(&v, 5), 2);
        assert_eq!(binary_search_no_spinoff(&v, 9), 4);
        assert_eq!(binary_search_no_spinoff(&v, 6), 3);
    }

    #[test]
    fn test_binary_search_no_spinoff_empty() {
        let v: Vec<u64> = vec![];
        let result = std::panic::catch_unwind(|| binary_search_no_spinoff(&v, 1));
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_search_no_spinoff_single() {
        let v = vec![10];
        assert_eq!(binary_search_no_spinoff(&v, 10), 0);
        assert_eq!(binary_search_no_spinoff(&v, 5), 0);
        assert_eq!(binary_search_no_spinoff(&v, 15), 0);
    }

    #[test]
    fn test_binary_search_no_spinoff_duplicates() {
        let v = vec![2, 2, 2, 2, 2];
        assert_eq!(binary_search_no_spinoff(&v, 2), 0);
        assert_eq!(binary_search_no_spinoff(&v, 1), 0);
        assert_eq!(binary_search_no_spinoff(&v, 3), 4);
    }
}

