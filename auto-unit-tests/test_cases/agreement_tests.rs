use std::result::*;

pub struct AgreementResource<T> {
    id: usize,
    value: T,
}

impl<T: Copy + PartialEq> AgreementResource<T> {
    pub fn alloc(c: T) -> Self {
        AgreementResource { id: 0, value: c }
    }

    pub fn duplicate(&self) -> Self {
        AgreementResource { id: self.id, value: self.value }
    }

    pub fn lemma_agreement(&self, other: &Self) {
        assert_eq!(self.id, other.id);
        assert_eq!(self.value, other.value);
    }

    pub fn view(&self) -> T {
        self.value
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

fn main() {
    let r1 = AgreementResource::<i32>::alloc(72);
    assert_eq!(r1.view(), 72);
    let r2 = r1.duplicate();
    assert_eq!(r2.view(), r1.view());
    r1.lemma_agreement(&r2);

    let mut rb1 = AgreementResource::<bool>::alloc(true);
    let rb2 = rb1.duplicate();
    rb1.lemma_agreement(&rb2);
    assert_eq!(rb1.view(), rb2.view());
    assert!(rb1.view());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_i32() {
        let resource = AgreementResource::<i32>::alloc(72);
        assert_eq!(resource.view(), 72);
        assert_eq!(resource.id(), 0);
    }

    #[test]
    fn test_duplicate_i32() {
        let resource = AgreementResource::<i32>::alloc(100);
        let duplicate = resource.duplicate();
        // Verify that the duplicated resource matches the original.
        resource.lemma_agreement(&duplicate);
        assert_eq!(duplicate.view(), 100);
        assert_eq!(duplicate.id(), resource.id());
    }

    #[test]
    fn test_alloc_bool() {
        let resource = AgreementResource::<bool>::alloc(true);
        assert_eq!(resource.view(), true);
        assert_eq!(resource.id(), 0);
    }

    #[test]
    fn test_duplicate_bool() {
        let resource = AgreementResource::<bool>::alloc(false);
        let duplicate = resource.duplicate();
        // Verify lemma_agreement on boolean resources.
        resource.lemma_agreement(&duplicate);
        assert_eq!(resource.view(), duplicate.view());
    }

    #[test]
    fn test_main_integration() {
        // Calling main() to exercise the integrated behavior.
        main();
    }
}