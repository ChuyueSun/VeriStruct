use std::vec::Vec;

fn binary_search(v: &Vec<u64>, k: u64) -> usize {
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2 {
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
    }
    i1
}

fn reverse(v: &mut Vec<u64>) {
    let length = v.len();
    for n in 0..(length / 2) {
        let x = v[n];
        let y = v[length - 1 - n];
        v[n] = y;
        v[length - 1 - n] = x;
    }
}

fn binary_search_no_spinoff(v: &Vec<u64>, k: u64) -> usize {
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2 {
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
    fn test_binary_search_found() {
        let v = vec![1, 3, 5, 7, 9];
        // Test for each element that exists in the vector
        for (i, &val) in v.iter().enumerate() {
            assert_eq!(binary_search(&v, val), i);
        }
    }

    #[test]
    fn test_binary_search_insertion_point() {
        let v = vec![1, 3, 5, 7, 9];
        // For key between existing values, should return the index of the smallest element that is >= key.
        // 2 should give index 1 because 3 is the first element >=2.
        assert_eq!(binary_search(&v, 2), 1);
        // 4 should give index 2 because 5 is the first element >=4.
        assert_eq!(binary_search(&v, 4), 2);
        // 6 should give index 3 because 7 is the first element >=6.
        assert_eq!(binary_search(&v, 6), 3);
        // 8 should give index 4 because 9 is the first element >=8.
        assert_eq!(binary_search(&v, 8), 4);
    }

    #[test]
    #[should_panic]
    fn test_binary_search_empty() {
        // This should panic because v.len() - 1 underflows when v is empty.
        let v: Vec<u64> = Vec::new();
        let _ = binary_search(&v, 5);
    }

    #[test]
    fn test_binary_search_edge_greater_than_all() {
        // When the key is greater than the greatest element, the algorithm returns last index.
        let v = vec![1, 3, 5, 7, 9];
        // Although 10 is greater than all, our algorithm is not designed to return the length,
        // so we expect it to return the last index.
        assert_eq!(binary_search(&v, 10), v.len() - 1);
    }

    #[test]
    fn test_binary_search_no_spinoff_found() {
        let v = vec![2, 4, 6, 8, 10];
        for (i, &val) in v.iter().enumerate() {
            assert_eq!(binary_search_no_spinoff(&v, val), i);
        }
    }

    #[test]
    fn test_binary_search_no_spinoff_insertion_point() {
        let v = vec![2, 4, 6, 8, 10];
        // Key values that are not in the vector.
        assert_eq!(binary_search_no_spinoff(&v, 3), 1);
        assert_eq!(binary_search_no_spinoff(&v, 5), 2);
        assert_eq!(binary_search_no_spinoff(&v, 7), 3);
        assert_eq!(binary_search_no_spinoff(&v, 9), 4);
    }

    #[test]
    #[should_panic]
    fn test_binary_search_no_spinoff_empty() {
        let v: Vec<u64> = Vec::new();
        let _ = binary_search_no_spinoff(&v, 100);
    }

    #[test]
    fn test_binary_search_no_spinoff_edge_greater_than_all() {
        let v = vec![2, 4, 6, 8, 10];
        // For a key greater than the last element, expect the last index.
        assert_eq!(binary_search_no_spinoff(&v, 12), v.len() - 1);
    }

    #[test]
    fn test_reverse_empty() {
        let mut v: Vec<u64> = Vec::new();
        reverse(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_reverse_single() {
        let mut v = vec![42];
        reverse(&mut v);
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn test_reverse_even() {
        let mut v = vec![1, 2, 3, 4];
        reverse(&mut v);
        assert_eq!(v, vec![4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_odd() {
        let mut v = vec![10, 20, 30, 40, 50];
        reverse(&mut v);
        assert_eq!(v, vec![50, 40, 30, 20, 10]);
    }
}