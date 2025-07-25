use std::result::*;

pub struct LogResource<T> {
    log: Vec<T>,
}

impl<T> LogResource<T> {
    pub fn alloc() -> LogResource<T> {
        LogResource { log: Vec::new() }
    }

    pub fn split(&mut self) -> (LogResource<T>, LogResource<T>)
    where
        T: Clone,
    {
        let cloned = self.log.clone();
        (LogResource { log: cloned.clone() }, LogResource { log: cloned })
    }

    pub fn append(&mut self, v: T) {
        self.log.push(v);
    }

    pub fn append_using_two_halves(&mut self, other: &mut Self, v: T)
    where
        T: Clone,
    {
        self.log.push(v);
        other.log = self.log.clone();
    }

    pub fn extract_prefix_knowledge(&self) -> LogResource<T>
    where
        T: Clone,
    {
        LogResource { log: self.log.clone() }
    }

    pub fn deduce_prefix_relation(&mut self, _other: &Self) {}

    pub fn log(&self) -> &Vec<T> {
        &self.log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_returns_empty_log() {
        let log_resource: LogResource<i32> = LogResource::alloc();
        assert_eq!(log_resource.log().len(), 0);
    }

    #[test]
    fn test_append_adds_element() {
        let mut log_resource: LogResource<i32> = LogResource::alloc();
        log_resource.append(10);
        assert_eq!(log_resource.log(), &vec![10]);

        log_resource.append(20);
        assert_eq!(log_resource.log(), &vec![10, 20]);
    }

    #[test]
    fn test_split_on_empty_log() {
        let mut log_resource: LogResource<i32> = LogResource::alloc();
        let (first_half, second_half) = log_resource.split();
        assert_eq!(first_half.log().len(), 0);
        assert_eq!(second_half.log().len(), 0);
    }

    #[test]
    fn test_split_on_non_empty_log() {
        let mut log_resource: LogResource<i32> = LogResource::alloc();
        log_resource.append(1);
        log_resource.append(2);
        log_resource.append(3);

        let (first_half, second_half) = log_resource.split();
        let expected = vec![1, 2, 3];
        assert_eq!(first_half.log(), &expected);
        assert_eq!(second_half.log(), &expected);
    }

    #[test]
    fn test_append_using_two_halves_updates_other_log() {
        let mut first: LogResource<i32> = LogResource::alloc();
        let mut second: LogResource<i32> = LogResource::alloc();
        first.append(100);
        // Before calling append_using_two_halves, second should be empty.
        assert_eq!(second.log().len(), 0);

        first.append_using_two_halves(&mut second, 200);
        // first should have two elements: [100, 200]
        assert_eq!(first.log(), &vec![100, 200]);
        // second should be updated to a clone of first.log
        assert_eq!(second.log(), &vec![100, 200]);

        // Further modify first and ensure that append_using_two_halves reflects changes again.
        first.append_using_two_halves(&mut second, 300);
        assert_eq!(first.log(), &vec![100, 200, 300]);
        assert_eq!(second.log(), &vec![100, 200, 300]);
    }

    #[test]
    fn test_extract_prefix_knowledge_clones_log() {
        let mut log_resource: LogResource<&str> = LogResource::alloc();
        log_resource.append("a");
        log_resource.append("b");

        let prefix = log_resource.extract_prefix_knowledge();
        // The extracted prefix should equal the original log.
        assert_eq!(prefix.log(), log_resource.log());

        // Changing the original log should not affect the extracted prefix.
        log_resource.append("c");
        assert_eq!(prefix.log(), &vec!["a", "b"]);
        assert_eq!(log_resource.log(), &vec!["a", "b", "c"]);
    }

    #[test]
    fn test_deduce_prefix_relation_does_nothing() {
        let mut log_resource1: LogResource<i32> = LogResource::alloc();
        let mut log_resource2: LogResource<i32> = LogResource::alloc();

        log_resource1.append(42);
        log_resource2.append(99);

        // Call deduce_prefix_relation which is a no-op.
        log_resource1.deduce_prefix_relation(&log_resource2);

        // Verify that both logs remain unchanged.
        assert_eq!(log_resource1.log(), &vec![42]);
        assert_eq!(log_resource2.log(), &vec![99]);
    }

    #[test]
    fn test_multiple_operations_in_sequence() {
        let mut log_resource: LogResource<i32> = LogResource::alloc();
        // Append several elements
        for i in 0..5 {
            log_resource.append(i);
        }
        assert_eq!(log_resource.log(), &vec![0, 1, 2, 3, 4]);

        // Extract prefix knowledge and verify
        let prefix = log_resource.extract_prefix_knowledge();
        assert_eq!(prefix.log(), &vec![0, 1, 2, 3, 4]);

        // Split the log resource and verify both halves
        let (first_half, second_half) = log_resource.split();
        assert_eq!(first_half.log(), &vec![0, 1, 2, 3, 4]);
        assert_eq!(second_half.log(), &vec![0, 1, 2, 3, 4]);

        // Use append_using_two_halves to add a new element and propagate change
        let mut other = LogResource::alloc();
        log_resource.append_using_two_halves(&mut other, 5);
        assert_eq!(log_resource.log(), &vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(other.log(), &vec![0, 1, 2, 3, 4, 5]);
    }
}