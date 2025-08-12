/// This module provides basic vector algorithms.
///
/// - `binary_search`: Performs a binary search on a sorted vector to find the index of a given key.
/// - `reverse`: Reverses the elements of a vector in place.
/// - `binary_search_no_spinoff`: Variant of binary search.
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

    // Tests for binary_search

    #[test]
    fn test_binary_search_single_element_found() {
        let v = vec![5];
        // When the only element is equal to the key.
        assert_eq!(binary_search(&v, 5), 0);
    }

    #[test]
    fn test_binary_search_single_element_not_found() {
        let v = vec![5];
        // For key less than the only element, should return index 0.
        assert_eq!(binary_search(&v, 3), 0);
        // For key greater than the only element, also returns index 0 because that's the only candidate.
        assert_eq!(binary_search(&v, 7), 0);
    }

    #[test]
    fn test_binary_search_exact_match() {
        let v = vec![10, 20, 30, 40, 50];
        // Exact matches should return their index.
        assert_eq!(binary_search(&v, 10), 0);
        assert_eq!(binary_search(&v, 20), 1);
        assert_eq!(binary_search(&v, 30), 2);
        assert_eq!(binary_search(&v, 40), 3);
        assert_eq!(binary_search(&v, 50), 4);
    }

    #[test]
    fn test_binary_search_lower_bound() {
        let v = vec![10, 20, 30, 40, 50];
        // For value not present, binary_search returns the index of the first element not less than the key.
        // e.g., searching for 25 should return index 2 since 30 is the first element >= 25.
        assert_eq!(binary_search(&v, 25), 2);
        // Searching for a number smaller than all elements returns index 0.
        assert_eq!(binary_search(&v, 5), 0);
    }

    #[test]
    fn test_binary_search_key_bigger_than_all() {
        let v = vec![1, 2, 3, 4];
        // When the key is greater than all elements, returns the last index.
        assert_eq!(binary_search(&v, 10), 3);
    }

    #[test]
    #[should_panic]
    fn test_binary_search_empty_vector() {
        // This test expects a panic because the vector is empty
        let v: Vec<u64> = vec![];
        let _ = binary_search(&v, 10);
    }

    // Tests for binary_search_no_spinoff

    #[test]
    fn test_binary_search_no_spinoff_single_element_found() {
        let v = vec![5];
        assert_eq!(binary_search_no_spinoff(&v, 5), 0);
    }

    #[test]
    fn test_binary_search_no_spinoff_single_element_not_found() {
        let v = vec![5];
        // For key less than the element.
        assert_eq!(binary_search_no_spinoff(&v, 3), 0);
        // For key greater than the element.
        assert_eq!(binary_search_no_spinoff(&v, 7), 0);
    }

    #[test]
    fn test_binary_search_no_spinoff_exact_match() {
        let v = vec![10, 20, 30, 40, 50];
        assert_eq!(binary_search_no_spinoff(&v, 10), 0);
        assert_eq!(binary_search_no_spinoff(&v, 20), 1);
        assert_eq!(binary_search_no_spinoff(&v, 30), 2);
        assert_eq!(binary_search_no_spinoff(&v, 40), 3);
        assert_eq!(binary_search_no_spinoff(&v, 50), 4);
    }

    #[test]
    fn test_binary_search_no_spinoff_lower_bound() {
        let v = vec![10, 20, 30, 40, 50];
        // For a value between two numbers.
        assert_eq!(binary_search_no_spinoff(&v, 25), 2);
        // For a value less than the smallest element.
        assert_eq!(binary_search_no_spinoff(&v, 5), 0);
    }

    #[test]
    fn test_binary_search_no_spinoff_key_bigger_than_all() {
        let v = vec![1, 2, 3, 4];
        assert_eq!(binary_search_no_spinoff(&v, 10), 3);
    }

    #[test]
    #[should_panic]
    fn test_binary_search_no_spinoff_empty_vector() {
        let v: Vec<u64> = vec![];
        let _ = binary_search_no_spinoff(&v, 10);
    }

    #[test]
    fn test_binary_search_with_duplicates() {
        // Both binary_search and binary_search_no_spinoff should return the index of the first element
        // that is not less than the key.
        let v = vec![1, 3, 3, 3, 5, 7];
        // Searching for 3 should return index 1.
        assert_eq!(binary_search(&v, 3), 1);
        assert_eq!(binary_search_no_spinoff(&v, 3), 1);
    }

    // Tests for reverse

    #[test]
    fn test_reverse_empty_vector() {
        let mut v: Vec<u64> = vec![];
        reverse(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_reverse_single_element() {
        let mut v = vec![42];
        reverse(&mut v);
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn test_reverse_even_length_vector() {
        let mut v = vec![1, 2, 3, 4];
        reverse(&mut v);
        assert_eq!(v, vec![4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_odd_length_vector() {
        let mut v = vec![1, 2, 3, 4, 5];
        reverse(&mut v);
        assert_eq!(v, vec![5, 4, 3, 2, 1]);
    }
}