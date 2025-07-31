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

pub fn test() {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_found() {
        let v = vec![1, 3, 5, 7, 9];
        // search for an existing element
        let index = binary_search(&v, 5);
        assert_eq!(index, 2);
        // also test for first and last element
        let index_first = binary_search(&v, 1);
        assert_eq!(index_first, 0);
        let index_last = binary_search(&v, 9);
        // Note: the algorithm returns the index of the first element >= k.
        // For 9 in a sorted array ending in 9, that index is the index of 9.
        assert_eq!(index_last, 4);
    }

    #[test]
    fn test_binary_search_not_found_lower() {
        let v = vec![5, 7, 9];
        // search for a value smaller than the first element; algorithm returns index 0.
        let index = binary_search(&v, 3);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_binary_search_not_found_higher() {
        let v = vec![1, 3, 5];
        // search for a value higher than all elements; algorithm returns the last index.
        let index = binary_search(&v, 10);
        assert_eq!(index, v.len() - 1);
    }

    #[test]
    fn test_binary_search_one_element() {
        let v = vec![10];
        // When the element is equal to the only element.
        let index_equal = binary_search(&v, 10);
        assert_eq!(index_equal, 0);
        // When the target is less than the only element.
        let index_less = binary_search(&v, 5);
        assert_eq!(index_less, 0);
        // When the target is greater than the only element.
        let index_greater = binary_search(&v, 15);
        assert_eq!(index_greater, 0);
    }

    #[test]
    #[should_panic]
    fn test_binary_search_empty() {
        // Function is not designed for empty vectors.
        let v: Vec<u64> = vec![];
        // This should panic because of v.len()-1.
        let _ = binary_search(&v, 1);
    }

    #[test]
    fn test_binary_search_no_spinoff_found() {
        let v = vec![2, 4, 6, 8, 10];
        let index = binary_search_no_spinoff(&v, 6);
        assert_eq!(index, 2);
        let index_first = binary_search_no_spinoff(&v, 2);
        assert_eq!(index_first, 0);
        let index_last = binary_search_no_spinoff(&v, 10);
        assert_eq!(index_last, 4);
    }

    #[test]
    fn test_binary_search_no_spinoff_not_found() {
        let v = vec![2, 4, 6, 8, 10];
        // Search for a value lower than the first element.
        let index_lower = binary_search_no_spinoff(&v, 1);
        assert_eq!(index_lower, 0);
        // Search for a value greater than the last element.
        let index_higher = binary_search_no_spinoff(&v, 12);
        // According to the algorithm, returns the last index.
        assert_eq!(index_higher, v.len() - 1);
    }

    #[test]
    #[should_panic]
    fn test_binary_search_no_spinoff_empty() {
        let v: Vec<u64> = vec![];
        let _ = binary_search_no_spinoff(&v, 1);
    }

    #[test]
    fn test_binary_search_functions_agree() {
        // For a variety of keys, both function variants should return the same index.
        let v = vec![1, 3, 5, 7, 9, 11];
        for &key in &[0, 1, 4, 5, 8, 9, 10, 11, 15] {
            let idx1 = binary_search(&v, key);
            let idx2 = binary_search_no_spinoff(&v, key);
            assert_eq!(idx1, idx2, "Failed for key {}: {} != {}", key, idx1, idx2);
        }
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
    fn test_reverse_single() {
        let mut v = vec![42];
        reverse(&mut v);
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn test_reverse_empty() {
        let mut v: Vec<u64> = vec![];
        // Reversing an empty vector should work without changes.
        reverse(&mut v);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_binary_search_duplicates() {
        let v = vec![1, 2, 2, 2, 3];
        // For duplicate elements, binary_search should return the index of the first element >= k.
        let index = binary_search(&v, 2);
        assert_eq!(index, 1);
        let index_no_spinoff = binary_search_no_spinoff(&v, 2);
        assert_eq!(index_no_spinoff, 1);
    }

    #[test]
    fn test_binary_search_two_elements() {
        let v = vec![3, 5];
        // When searching for a value lower than the first element, expect index 0.
        assert_eq!(binary_search(&v, 2), 0);
        assert_eq!(binary_search_no_spinoff(&v, 2), 0);
        // When searching for the value equal to the first element, expect index 0.
        assert_eq!(binary_search(&v, 3), 0);
        assert_eq!(binary_search_no_spinoff(&v, 3), 0);
        // When searching for a value between the two elements, expect index 1.
        assert_eq!(binary_search(&v, 4), 1);
        assert_eq!(binary_search_no_spinoff(&v, 4), 1);
        // When searching for the value equal to the second element, expect index 1.
        assert_eq!(binary_search(&v, 5), 1);
        assert_eq!(binary_search_no_spinoff(&v, 5), 1);
        // When searching for a value greater than the second element, expect last index.
        assert_eq!(binary_search(&v, 6), 1);
        assert_eq!(binary_search_no_spinoff(&v, 6), 1);
    }

    #[test]
    fn test_reverse_two_elements() {
        let mut v = vec![10, 20];
        reverse(&mut v);
        assert_eq!(v, vec![20, 10]);
    }
}