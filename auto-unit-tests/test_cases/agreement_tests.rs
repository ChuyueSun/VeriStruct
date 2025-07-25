use std::result::*;

static mut COUNTER: usize = 1;

pub struct AgreementResource<T> {
    id: usize,
    value: T,
}

impl<T: Clone + PartialEq> AgreementResource<T> {
    pub fn alloc(c: T) -> Self {
        let id = unsafe {
            let id = COUNTER;
            COUNTER += 1;
            id
        };
        AgreementResource { id, value: c }
    }

    pub fn duplicate(&self) -> Self {
        AgreementResource {
            id: self.id,
            value: self.value.clone(),
        }
    }

    pub fn lemma_agreement(&self, other: &Self) {
        assert_eq!(self.value, other.value);
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn view(&self) -> &T {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_counter() {
        unsafe {
            COUNTER = 1;
        }
    }

    #[test]
    fn test_alloc_assigns_unique_ids() {
        reset_counter();
        let resource1 = AgreementResource::alloc(10);
        let resource2 = AgreementResource::alloc(20);
        assert_ne!(resource1.id(), resource2.id());
    }

    #[test]
    fn test_duplicate_creates_same_id_and_value() {
        reset_counter();
        let original = AgreementResource::alloc(42);
        let duplicate = original.duplicate();
        assert_eq!(original.id(), duplicate.id());
        assert_eq!(original.view(), duplicate.view());
    }

    #[test]
    fn test_lemma_agreement_pass() {
        reset_counter();
        let resource = AgreementResource::alloc("test");
        let duplicate = resource.duplicate();
        // Should not panic since the values are equal.
        resource.lemma_agreement(&duplicate);
    }

    #[test]
    #[should_panic]
    fn test_lemma_agreement_fail() {
        reset_counter();
        let resource1 = AgreementResource::alloc("first".to_string());
        let resource2 = AgreementResource::alloc("second".to_string());
        // This should panic because the internal values are not equal.
        resource1.lemma_agreement(&resource2);
    }

    #[test]
    fn test_view_returns_correct_value() {
        reset_counter();
        let value = 100;
        let resource = AgreementResource::alloc(value);
        let view_value = *resource.view();
        assert_eq!(value, view_value);
    }

    #[test]
    fn test_id_returns_correct_value() {
        reset_counter();
        let resource = AgreementResource::alloc(0);
        // Since COUNTER is reset before each test, the first allocation should have id 1.
        assert_eq!(resource.id(), 1);
    }
}