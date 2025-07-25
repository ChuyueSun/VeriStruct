use std::sync::atomic::{AtomicUsize, Ordering};
use vstd::prelude::*;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

verus! {

    pub struct AgreementResource<T> {
        id: usize,
        value: T,
    }

    impl<T: Clone> AgreementResource<T> {
        pub fn alloc(c: T) -> (res: AgreementResource<T>)
            ensures
                res.value == c,
        {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            AgreementResource { id, value: c }
        }

        pub fn duplicate(&self) -> (res: AgreementResource<T>)
            ensures
                res.id == self.id,
                res.value == self.value,
        {
            AgreementResource { id: self.id, value: self.value.clone() }
        }

        pub fn lemma_agreement(&self, other: &Self)
        {
            // This lemma proves that the two AgreementResources must share the same id.
            // It will abort (verify failure) if the ids do not agree.
            assert(self.id == other.id);
        }

        pub fn view(&self) -> (&T)
            ensures
                result == &self.value,
        {
            &self.value
        }

        pub fn id(&self) -> (res: usize)
            ensures
                res == self.id,
        {
            self.id
        }
    }

    /* TEST CODE BELOW */
    pub fn main() {
        // Test alloc and view.
        {
            let number = 42;
            let resource = AgreementResource::alloc(number);
            assert(*resource.view() == number);
        }
        // Test duplicate.
        {
            let text = String::from("Hello, Rust!");
            let resource = AgreementResource::alloc(text.clone());
            let duplicate_resource = resource.duplicate();
            // Check that duplicate has the same id and the same value.
            assert(resource.id() == duplicate_resource.id());
            assert(*resource.view() == *duplicate_resource.view());
        }
        // Test lemma_agreement success.
        {
            let value = 100;
            let resource = AgreementResource::alloc(value);
            let duplicate_resource = resource.duplicate();
            // Both resources have the same id, so lemma_agreement should pass.
            resource.lemma_agreement(&duplicate_resource);
        }
        // Test lemma_agreement failure simulation.
        {
            let resource1 = AgreementResource::alloc(10);
            let resource2 = AgreementResource::alloc(20);
            // Since resource1 and resource2 are allocated separately, they must have distinct ids.
            // We do not call lemma_agreement here because that would abort the verification.
            assert(resource1.id() != resource2.id());
        }
        // Test id uniqueness.
        {
            let resource_a = AgreementResource::alloc(1);
            let resource_b = AgreementResource::alloc(2);
            // Independently allocated resources should have distinct ids.
            assert(resource_a.id() != resource_b.id());
        }
        // Test multiple allocs increment counter.
        {
            let res1 = AgreementResource::alloc(1);
            let res2 = AgreementResource::alloc(2);
            let res3 = AgreementResource::alloc(3);
            // Check that allocated ids are increasing.
            assert(res2.id() > res1.id());
            assert(res3.id() > res2.id());
        }
    }
}