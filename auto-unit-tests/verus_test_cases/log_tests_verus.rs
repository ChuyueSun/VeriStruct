use vstd::prelude::*;
use vstd::seq::*;
use std::result::*;

verus! {

pub struct LogResource<T> {
    log: Vec<T>,
}

impl<T> LogResource<T> {
    // Allocates a new LogResource with an empty log.
    pub fn alloc() -> Self
        ensures
            result.log@ == Seq::empty(),
    {
        LogResource { log: Vec::new() }
    }

    // Returns a view of the internal log.
    pub fn view(&self) -> &Vec<T>
        ensures
            result == &self.log,
    {
        &self.log
    }

    // Returns a unique identifier for the resource.
    pub fn id(&self) -> usize
        ensures
            result == (self as *const _ as usize),
    {
        self as *const _ as usize
    }

    // deduce_prefix_relation does not change the log.
    pub fn deduce_prefix_relation(&mut self, _other: &Self)
        ensures
            self.log@ == old(self.log@),
    {
        // no effect
    }
}

impl<T: Clone> LogResource<T> {
    // Splits self into two resources, both with a clone of the original log.
    pub fn split(&self) -> (Self, Self)
        ensures
            (let (first, second) = result;
             first.log@ == self.log@ &&
             second.log@ == self.log@ &&
             first.id() != second.id()),
    {
        (LogResource { log: self.log.clone() }, LogResource { log: self.log.clone() })
    }

    // Appends an element to self and then updates 'other' to be a clone of self.
    pub fn append_using_two_halves(&mut self, other: &mut Self, v: T)
        ensures
            self.log@ == old(self.log@).concat(Seq::singleton(v)),
            other.log@ == self.log@,
    {
        self.log.push(v);
        other.log = self.log.clone();
    }

    // Returns a clone of self, representing prefix knowledge.
    pub fn extract_prefix_knowledge(&self) -> Self
        ensures
            result.log@ == self.log@,
    {
        LogResource { log: self.log.clone() }
    }
}

impl<T> LogResource<T> {
    // Appends an element to the log.
    pub fn append(&mut self, v: T)
        ensures
            self.log@ == old(self.log@).concat(Seq::singleton(v)),
    {
        self.log.push(v);
    }
}

/* TEST CODE BELOW */

pub fn main() {
    {
        // test_alloc_creates_empty_log
        let resource: LogResource<i32> = LogResource::alloc();
        assert(resource.view().len() == 0);
    }
    {
        // test_append_adds_element
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(42);
        assert(resource.view() == &vec![42]);
    }
    {
        // test_split_on_empty_log
        let resource: LogResource<i32> = LogResource::alloc();
        let (first, second) = resource.split();
        assert(first.view().len() == 0);
        assert(second.view().len() == 0);
    }
    {
        // test_split_on_non_empty_log
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(1);
        resource.append(2);
        resource.append(3);
        let (first, second) = resource.split();
        assert(first.view() == &vec![1, 2, 3]);
        assert(second.view() == &vec![1, 2, 3]);
        // Ensure that the two split parts are distinct instances by checking their ids.
        assert(first.id() != second.id());
    }
    {
        // test_append_using_two_halves_updates_both
        let mut first: LogResource<i32> = LogResource::alloc();
        let mut second: LogResource<i32> = LogResource::alloc();
        first.append(10);
        // Initially, second is empty.
        assert(second.view().len() == 0);
        first.append_using_two_halves(&mut second, 20);
        // Now first should have two elements.
        assert(first.view() == &vec![10, 20]);
        // second should mirror first's log.
        assert(second.view() == &vec![10, 20]);
    }
    {
        // test_extract_prefix_knowledge_clones_log
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(5);
        resource.append(15);
        let prefix = resource.extract_prefix_knowledge();
        assert(prefix.view() == resource.view());
        // Modify original and check that prefix remains unchanged.
        resource.append(25);
        assert(prefix.view() == &vec![5, 15]);
        assert(resource.view() == &vec![5, 15, 25]);
    }
    {
        // test_deduce_prefix_relation_no_effect
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(7);
        let other: LogResource<i32> = LogResource::alloc();
        // Calling deduce_prefix_relation should not change either resource.
        resource.deduce_prefix_relation(&other);
        assert(resource.view() == &vec![7]);
    }
    {
        // test_multiple_operations
        // Test a sequence of operations.
        let mut resource: LogResource<i32> = LogResource::alloc();
        resource.append(100);
        resource.append(200);

        // Split the resource.
        let (mut first, mut second) = resource.split();
        // Append to first and update second using append_using_two_halves.
        first.append_using_two_halves(&mut second, 300);
        // Check both first and second now reflect the new state.
        assert(first.view() == &vec![100, 200, 300]);
        assert(second.view() == &vec![100, 200, 300]);

        // Extract prefix knowledge from first.
        let prefix = first.extract_prefix_knowledge();
        assert(prefix.view() == &vec![100, 200, 300]);
    }
    {
        // test_id_returns_unique_identifier
        let resource1: LogResource<i32> = LogResource::alloc();
        let resource2: LogResource<i32> = LogResource::alloc();
        // Even though the resources are allocated similarly, their ids (addresses) should differ.
        assert(resource1.id() != resource2.id());

        // Duplicate by splitting and check that the new instance is distinct.
        let (dup, _) = resource1.split();
        assert(resource1.id() != dup.id());
    }
}

}