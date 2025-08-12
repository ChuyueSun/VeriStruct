#![allow(unused_imports)]

pub fn main() {
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let u: u32 = 5u32;
        let mut i = u;
        if i == 1u32 {
            i = 3u32;
        }
        let mut j = 7u32;
        {
            let tmp = i;
            i = j;
            j = tmp;
        }
        let j = i;
        assert!(j % 2 == 1);
    }
}