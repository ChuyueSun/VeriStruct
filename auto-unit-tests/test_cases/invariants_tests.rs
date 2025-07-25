pub fn main() {
    let u: u32 = 5u32;
    let mut i: u32 = u;
    let mut j: u32 = 7u32;
    std::mem::swap(&mut i, &mut j);
    let j = i;
    assert!(j % 2 == 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_main_normal_execution() {
        // Calling main should complete without panicking.
        main();
    }
    
    #[test]
    fn test_main_no_panic() {
        // Use catch_unwind to ensure that main does not panic.
        let result = std::panic::catch_unwind(|| main());
        assert!(result.is_ok(), "main() panicked unexpectedly");
    }
}