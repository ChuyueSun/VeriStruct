use vstd::prelude::*;
use vstd::seq::*;

verus! {

pub struct AgreementResource<T> {
    id: usize,
    value: T,
}

impl<T: Copy + PartialEq> AgreementResource<T> {
    pub fn alloc(c: T) -> Self
        ensures
            result.view() == c,
            result.id() == 0,
    {
        AgreementResource { id: 0, value: c }
    }

    pub fn duplicate(&self) -> Self
        ensures
            result.view() == self.view(),
            result.id() == self.id(),
    {
        AgreementResource { id: self.id, value: self.value }
    }

    pub fn lemma_agreement(&self, other: &Self)
        ensures
            self.view() == other.view(),
            self.id() == other.id(),
    {
        assert(self.id() == other.id());
        assert(self.value == other.value);
    }

    pub fn view(&self) -> T
        ensures
            result == self.value,
    {
        self.value
    }

    pub fn id(&self) -> usize
        ensures
            result == self.id,
    {
        self.id
    }
}

fn main() {
    let r1 = AgreementResource::<i32>::alloc(72);
    assert(r1.view() == 72);
    let r2 = r1.duplicate();
    assert(r2.view() == r1.view());
    r1.lemma_agreement(&r2);

    let rb1 = AgreementResource::<bool>::alloc(true);
    let rb2 = rb1.duplicate();
    rb1.lemma_agreement(&rb2);
    assert(rb1.view() == rb2.view());
    assert(rb1.view());
}

/* TEST CODE BELOW */

pub fn test_alloc_i32() {
    let resource = AgreementResource::<i32>::alloc(72);
    assert(resource.view() == 72);
    assert(resource.id() == 0);
}

pub fn test_duplicate_i32() {
    let resource = AgreementResource::<i32>::alloc(100);
    let duplicate = resource.duplicate();
    resource.lemma_agreement(&duplicate);
    assert(duplicate.view() == 100);
    assert(duplicate.id() == resource.id());
}

pub fn test_alloc_bool() {
    let resource = AgreementResource::<bool>::alloc(true);
    assert(resource.view() == true);
    assert(resource.id() == 0);
}

pub fn test_duplicate_bool() {
    let resource = AgreementResource::<bool>::alloc(false);
    let duplicate = resource.duplicate();
    resource.lemma_agreement(&duplicate);
    assert(resource.view() == duplicate.view());
}

pub fn test_main_integration() {
    main();
}

pub fn main_test() {
    test_alloc_i32();
    test_duplicate_i32();
    test_alloc_bool();
    test_duplicate_bool();
    test_main_integration();
}

}