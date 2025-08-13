/// This module provides basic vector algorithms with specifications suitable for formal verification.
/// 
/// - `binary_search`: Performs a binary search on a sorted vector to find the index of a given key. The vector must be sorted in ascending order and the key must be present in the vector.
/// - `reverse`: Reverses the elements of a vector in place, with postconditions about the resulting order.
/// - `binary_search_no_spinoff`: Variant of binary search with loop isolation disabled for verification purposes.
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
    // Removed the unused clone: let v1 = v.clone();
    for n in 0..(length / 2) {
        v.swap(n, length - 1 - n);
    }
}

fn binary_search_no_spinoff(v: &Vec<u64>, k: u64) -> usize {
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2 {
        let d = i2 - i1;
        let ix = i1 + (i2 - i1) / 2;
        if v[ix] < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
        assert!(i2 - i1 < d);
    }
    i1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_single_element() {
        let v = vec![42];
        let index = binary_search(&v, 42);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_binary_search_first_element() {
        let v = vec![1, 3, 5, 7, 9];
        let index = binary_search(&v, 1);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_binary_search_middle_element() {
        let v = vec![2, 4, 6, 8, 10];
        let index = binary_search(&v, 6);
        assert_eq!(index, 2);
    }

    #[test]
    fn test_binary_search_last_element() {
        let v = vec![10, 20, 30, 40, 50];
        let index = binary_search(&v, 50);
        assert_eq!(index, 4);
    }

    #[test]
    fn test_binary_search_multiple_occurrences() {
        // Though specification assumes key is present and unique, test with duplicate keys.
        // Our binary_search returns the first occurrence.
        let v = vec![5, 7, 7, 7, 9];
        let index = binary_search(&v, 7);
        // The binary search should return the first index where 7 appears.
        assert_eq!(v[index], 7);
        // Ensure that either index==1 or index==0 if the key were placed earlier.
        // In this sorted array the first occurrence is index 1.
        assert_eq!(index, 1);
    }

    #[test]
    fn test_binary_search_no_spinoff_single_element() {
        let v = vec![100];
        let index = binary_search_no_spinoff(&v, 100);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_binary_search_no_spinoff_first_element() {
        let v = vec![3, 6, 9, 12, 15];
        let index = binary_search_no_spinoff(&v, 3);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_binary_search_no_spinoff_middle_element() {
        let v = vec![1, 2, 3, 4, 5, 6, 7];
        let index = binary_search_no_spinoff(&v, 4);
        assert_eq!(index, 3);
    }

    #[test]
    fn test_binary_search_no_spinoff_last_element() {
        let v = vec![11, 22, 33, 44, 55];
        let index = binary_search_no_spinoff(&v, 55);
        assert_eq!(index, 4);
    }

    #[test]
    fn test_reverse_empty() {
        let mut v: Vec<u64> = vec![];
        reverse(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_reverse_single_element() {
        let mut v = vec![999];
        reverse(&mut v);
        assert_eq!(v, vec![999]);
    }

    #[test]
    fn test_reverse_even_number_of_elements() {
        let mut v = vec![1, 2, 3, 4];
        reverse(&mut v);
        assert_eq!(v, vec![4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_odd_number_of_elements() {
        let mut v = vec![10, 20, 30, 40, 50];
        reverse(&mut v);
        assert_eq!(v, vec![50, 40, 30, 20, 10]);
    }

    #[test]
    fn test_reverse_and_binary_search_consistency() {
        // Verify that after reversing a vector, if we re-reverse it,
        // binary_search finds the same element at the original position.
        let mut v = vec![5, 10, 15, 20, 25];
        let key = 15;
        let index_before = binary_search(&v, key);
        reverse(&mut v);
        reverse(&mut v);
        let index_after = binary_search(&v, key);
        assert_eq!(index_before, index_after);
    }
}