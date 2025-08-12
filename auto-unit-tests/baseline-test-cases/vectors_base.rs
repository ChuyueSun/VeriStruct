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
    let v1 = v.clone();
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

fn pusher() -> Vec<u64> {
    let mut v = Vec::new();
    v.push(0);
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);
    assert!(v[2] == 2);
    v.pop();
    v.push(4);
    v
}

fn uninterp_fn(x: u64) -> bool {
    true
}

fn pop_test(t: Vec<u64>) {
    let mut t = t;
    let x = t.pop().unwrap();
    assert!(uninterp_fn(x));
}

fn push_test(mut t: Vec<u64>, y: u64) {
    t.push(y);
}

fn binary_search_test(t: Vec<u64>) {
    for i in 0..t.len() {
        let k = t[i];
        let r = binary_search(&t, k);
        assert!(r < t.len());
        assert!(t[r] == k);
        let r = binary_search_no_spinoff(&t, k);
        assert!(r < t.len());
        assert!(t[r] == k);
    }
}

fn reverse_test(t: &mut Vec<u64>) {
    let t1 = t.clone();
    reverse(t);
    assert!(t.len() == t1.len());
    for i in 0..t1.len() {
        assert!(t[i] == t1[t1.len() - i - 1]);
    }
}

pub fn test() {
}

pub fn main() {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search() {
        let t = vec![0, 1, 2, 3, 4];
        binary_search_test(t);
    }

    #[test]
    fn test_reverse() {
        let mut t = vec![0, 1, 2, 3, 4];
        reverse_test(&mut t);
    }

    #[test]
    fn test_pop() {
        let t = vec![0, 1, 2, 3, 4];
        pop_test(t);
    }

    #[test]
    fn test_push() {
        let t = vec![0, 1, 2, 3, 4];
        push_test(t, 5);
    }

    #[test]
    fn test_pusher() {
        let _ = pusher();
    }
}