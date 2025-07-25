#![allow(unused_imports)]
use std::result::*;

pub enum AgreementResourceValue<T> {
    Empty,
    Chosen { c: T },
    Invalid,
}

impl<T> AgreementResourceValue<T> {
    pub fn new(c: T) -> Self {
        AgreementResourceValue::Chosen { c }
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T: Clone> AgreementResource<T> {
    pub fn inv(&self) -> bool {
        match self.r.value() {
            AgreementResourceValue::Chosen { .. } => true,
            _ => false,
        }
    }

    pub fn id(&self) -> usize {
        self.r.loc()
    }

    pub fn view(&self) -> T {
        match self.r.value() {
            AgreementResourceValue::Chosen { ref c } => c.clone(),
            _ => panic!("Invalid state"),
        }
    }

    pub fn alloc(c: T) -> AgreementResource<T> {
        let r_value = AgreementResourceValue::new(c);
        let r = Resource::alloc(r_value);
        AgreementResource { r }
    }

    pub fn duplicate(&self) -> AgreementResource<T> {
        let r = duplicate(&self.r);
        AgreementResource { r }
    }

    pub fn lemma_agreement(&mut self, other: &AgreementResource<T>) {
        self.r.validate_2(&other.r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test checks that `alloc` initializes the resource correctly and that
    // `view` returns the value and `inv` returns true.
    #[test]
    fn test_alloc_and_view() {
        let value = 42;
        let resource = AgreementResource::alloc(value);
        assert_eq!(resource.view(), 42);
        assert!(resource.inv());
    }

    // This test checks that `duplicate` creates a new AgreementResource whose view
    // value is the same as the original.
    #[test]
    fn test_duplicate() {
        let value = String::from("test");
        let resource = AgreementResource::alloc(value.clone());
        let dup = resource.duplicate();
        assert_eq!(dup.view(), resource.view());
        assert!(dup.inv());
    }

    // This test verifies that two separately allocated resources have different IDs.
    #[test]
    fn test_distinct_ids() {
        let resource1 = AgreementResource::alloc(10);
        let resource2 = AgreementResource::alloc(20);
        assert_ne!(resource1.id(), resource2.id(), "IDs should be distinct for separate allocations");
    }

    // This test exercises the lemma_agreement function.
    // It validates that the call succeeds and does not change the value of the resource.
    #[test]
    fn test_lemma_agreement() {
        let mut res1 = AgreementResource::alloc(100);
        let res2 = AgreementResource::alloc(200);
        res1.lemma_agreement(&res2);
        assert_eq!(res1.view(), 100);
    }
}