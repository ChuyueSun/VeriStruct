use std::result::*;

pub struct LogResource<T> {
    log: Vec<T>,
}

impl<T> LogResource<T> {
    pub fn alloc() -> Self {
        LogResource { log: Vec::new() }
    }

    pub fn split(&self) -> (Self, Self)
    where
        T: Clone,
    {
        (LogResource { log: self.log.clone() }, LogResource { log: self.log.clone() })
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

    pub fn extract_prefix_knowledge(&self) -> Self
    where
        T: Clone,
    {
        LogResource { log: self.log.clone() }
    }

    pub fn deduce_prefix_relation(&mut self, _other: &Self) {
    }

    pub fn view(&self) -> &Vec<T> {
        &self.log
    }

    pub fn id(&self) -> usize {
        self as *const _ as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_creates_empty_log() {
        let resource: LogResource<i32> = LogResource::alloc();
        assert_eq!(resource.view().len(), 0);
    }

    #[test]
    fn test_append_adds_element() {
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(42);
        assert_eq!(resource.view(), &vec![42]);
    }

    #[test]
    fn test_split_on_empty_log() {
        let resource: LogResource<i32> = LogResource::alloc();
        let (first, second) = resource.split();
        assert_eq!(first.view().len(), 0);
        assert_eq!(second.view().len(), 0);
    }

    #[test]
    fn test_split_on_non_empty_log() {
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(1);
        resource.append(2);
        resource.append(3);
        let (first, second) = resource.split();
        assert_eq!(first.view(), &vec![1, 2, 3]);
        assert_eq!(second.view(), &vec![1, 2, 3]);
        // Ensure that the two split parts are distinct instances by checking their ids.
        assert_ne!(first.id(), second.id());
    }

    #[test]
    fn test_append_using_two_halves_updates_both() {
        let mut first: LogResource<i32> = LogResource::alloc();
        let mut second: LogResource<i32> = LogResource::alloc();
        first.append(10);
        // Initially, second is empty.
        assert_eq!(second.view().len(), 0);
        first.append_using_two_halves(&mut second, 20);
        // Now first should have two elements.
        assert_eq!(first.view(), &vec![10, 20]);
        // second should mirror first's log.
        assert_eq!(second.view(), &vec![10, 20]);
    }

    #[test]
    fn test_extract_prefix_knowledge_clones_log() {
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(5);
        resource.append(15);
        let prefix = resource.extract_prefix_knowledge();
        assert_eq!(prefix.view(), resource.view());
        // Modify original and check that prefix remains unchanged.
        resource.append(25);
        assert_eq!(prefix.view(), &vec![5, 15]);
        assert_eq!(resource.view(), &vec![5, 15, 25]);
    }

    #[test]
    fn test_deduce_prefix_relation_no_effect() {
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(7);
        let other: LogResource<i32> = LogResource::alloc();
        // Calling deduce_prefix_relation should not change either resource.
        resource.deduce_prefix_relation(&other);
        assert_eq!(resource.view(), &vec![7]);
    }

    #[test]
    fn test_multiple_operations() {
        // Test a sequence of operations.
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(100);
        resource.append(200);

        // Split the resource.
        let (mut first, mut second) = resource.split();
        // Append to first and update second using append_using_two_halves.
        first.append_using_two_halves(&mut second, 300);
        // Check both first and second now reflect the new state.
        assert_eq!(first.view(), &vec![100, 200, 300]);
        assert_eq!(second.view(), &vec![100, 200, 300]);

        // Extract prefix knowledge from first.
        let prefix = first.extract_prefix_knowledge();
        assert_eq!(prefix.view(), &vec![100, 200, 300]);
    }

    #[test]
    fn test_id_returns_unique_identifier() {
        let resource1: LogResource<i32> = LogResource::alloc();
        let resource2: LogResource<i32> = LogResource::alloc();
        // Even though the resources are allocated similarly, their ids (addresses) should differ.
        assert_ne!(resource1.id(), resource2.id());

        // Duplicate by splitting and check that the new instance is distinct.
        let (dup, _) = resource1.split();
        assert_ne!(resource1.id(), dup.id());
    }
}