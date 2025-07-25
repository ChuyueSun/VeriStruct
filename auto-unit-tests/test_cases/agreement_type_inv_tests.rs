use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct AgreementResource<T> {
    id: usize,
    value: T,
}

impl<T: Clone> AgreementResource<T> {
    pub fn alloc(c: T) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        AgreementResource { id, value: c }
    }

    pub fn duplicate(&self) -> Self {
        AgreementResource { id: self.id, value: self.value.clone() }
    }

    pub fn lemma_agreement(&self, other: &Self) {
        assert_eq!(self.id, other.id);
    }

    pub fn view(&self) -> &T {
        &self.value
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_view() {
        let number = 42;
        let resource = AgreementResource::alloc(number);
        assert_eq!(*resource.view(), number);
    }

    #[test]
    fn test_duplicate() {
        let text = String::from("Hello, Rust!");
        let resource = AgreementResource::alloc(text.clone());
        let duplicate_resource = resource.duplicate();
        // Check that duplicate has the same id and the same value.
        assert_eq!(resource.id(), duplicate_resource.id());
        assert_eq!(resource.view(), duplicate_resource.view());
    }

    #[test]
    fn test_lemma_agreement_success() {
        let value = 100;
        let resource = AgreementResource::alloc(value);
        let duplicate_resource = resource.duplicate();
        // Both resources have the same id, lemma_agreement should not panic.
        resource.lemma_agreement(&duplicate_resource);
    }

    #[test]
    #[should_panic]
    fn test_lemma_agreement_failure() {
        let resource1 = AgreementResource::alloc(10);
        let resource2 = AgreementResource::alloc(20);
        // The ids of resource1 and resource2 are different, so lemma_agreement should panic.
        resource1.lemma_agreement(&resource2);
    }

    #[test]
    fn test_id_uniqueness() {
        let resource_a = AgreementResource::alloc(1);
        let resource_b = AgreementResource::alloc(2);
        // Independently allocated resources should have distinct ids.
        assert_ne!(resource_a.id(), resource_b.id());
    }

    #[test]
    fn test_multiple_allocs_increment_counter() {
        let res1 = AgreementResource::alloc(1);
        let res2 = AgreementResource::alloc(2);
        let res3 = AgreementResource::alloc(3);
        // Check that allocated ids are increasing.
        assert!(res2.id() > res1.id());
        assert!(res3.id() > res2.id());
    }
}