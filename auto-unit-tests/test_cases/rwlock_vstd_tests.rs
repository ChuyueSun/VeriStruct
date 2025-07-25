pub fn main() {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_runs_without_panicking() {
        // Calling main should not cause any panic.
        main();
    }
}