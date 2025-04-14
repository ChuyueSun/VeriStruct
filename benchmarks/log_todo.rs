//! This file implements logs with entries of type `T` using a custom
//! resource algebra.
//!
//! To use it, use LogResource::<T>::alloc(), which will create a
//! fresh log and return a `LogResource<T>` representing full
//! authority to append to the log. Here's example code for this:
//!
//! ```
//! let tracked full_auth = LogResource::<int>::alloc();
//! assert(full_auth@ is FullAuthority);
//! assert(full_auth@.log().len() == 0);
//! ```
//!
//! You can use the full authority you obtain via `alloc()` to append
//! to the log, as in the following example:
//!
//! ```
//! proof { full_auth.append(42); }
//! proof { full_auth.append(86); }
//! assert(full_auth@.log().len() == 2);
//! assert(full_auth@.log()[0] == 42);
//! assert(full_auth@.log()[1] == 86);
//! ```
//!
//! If desired, you can split a `LogResource` representing full
//! authority into two half authorities using `split`. You may want to
//! do this if you're stashing half the authority in an invariant.
//! Here's an example use of `LogResource::split()`; note that it
//! consumes the resource.
//!
//! ```
//! let tracked (half_auth1, half_auth2) = full_auth.split();
//! assert(half_auth1@ == half_auth2@);
//! assert(half_auth1@ is HalfAuthority);
//! ```
//!
//! You can use two half authorities to append to the log using
//! `append_using_two_halves` as in the following example:
//!
//! ```
//! proof { half_auth1.append_using_two_halves(&mut half_auth2, 17); }
//! assert(half_auth1@.log().len() == 3);
//! assert(half_auth1@.log()[2] == 17);
//! assert(half_auth2@ == half_auth1@);
//! ```
#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

pub enum LogResourceValue<T> {
    PrefixKnowledge { prefix: Seq<T> },
    HalfAuthority { log: Seq<T> },
    FullAuthority { log: Seq<T> },
    Invalid,
}

pub open spec fn is_prefix<T>(s1: Seq<T>, s2: Seq<T>) -> bool {
    &&& s1.len() <= s2.len()
    &&& forall|i| 0 <= i < s1.len() ==> s1[i] == s2[i]
}

impl<T> PCM for LogResourceValue<T> {
    open spec fn valid(self) -> bool {
        // TODO: implement specification.
    }

    open spec fn op(self, other: Self) -> Self {
       // TODO: implement specification.
    }

    open spec fn unit() -> Self {
        // TODO: implement specification.
    }

    proof fn closed_under_incl(a: Self, b: Self) {
    }

    proof fn commutative(a: Self, b: Self) {
        assert(forall|log1: Seq<T>, log2: Seq<T>|
            is_prefix(log1, log2) && is_prefix(log2, log1) ==> log1 =~= log2);
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        assert(forall|log1: Seq<T>, log2: Seq<T>|
            is_prefix(log1, log2) && is_prefix(log2, log1) <==> log1 =~= log2);
        assert(forall|log| is_prefix(log, Seq::<T>::empty()) ==> log =~= Seq::<T>::empty());
    }

    proof fn op_unit(a: Self) {
        assert(forall|log| is_prefix(log, Seq::<T>::empty()) ==> log =~= Seq::<T>::empty());
    }

    proof fn unit_valid() {
    }
}

impl<T> LogResourceValue<T> {
    pub open spec fn log(self) -> Seq<T> {
        // TODO: implement specification.
    }

    proof fn op_unit(a: Self) {
        assert(forall|log| is_prefix(log, Seq::<T>::empty()) ==> log =~= Seq::<T>::empty());
    }

    proof fn unit_valid() {
    }
}

pub struct LogResource<T> {
    r: Resource<LogResourceValue<T>>,
}

impl<T> LogResource<T> {
    pub closed spec fn id(self) -> Loc {
        // TODO: implement specification.
    }

    pub closed spec fn view(self) -> LogResourceValue<T> {
        // TODO: implement specification.
    }

    pub proof fn alloc() -> (tracked result: LogResource<T>)
    // TODO: add requires and ensures
    {
        let v = LogResourceValue::<T>::FullAuthority { log: Seq::<T>::empty() };
        let tracked r = Resource::<LogResourceValue::<T>>::alloc(v);
        Self { r }
    }

    pub proof fn split(tracked self) -> (tracked halves: (Self, Self))
    // TODO: add requires and ensures
    {
        let half_value = LogResourceValue::<T>::HalfAuthority { log: self@.log() };
        let tracked (half1, half2) = self.r.split(half_value, half_value);
        (Self { r: half1 }, Self { r: half2 })
    }

    pub proof fn append(tracked &mut self, v: T)
    // TODO: add requires and ensures

    {
        let value = LogResourceValue::<T>::FullAuthority { log: self@.log() + seq![v] };
        update_mut(&mut self.r, value);
    }

    pub proof fn append_using_two_halves(tracked &mut self, tracked other: &mut Self, v: T)
    // TODO: add requires and ensures
    {
        self.r.validate_2(&other.r);
        let new_log = self@.log() + seq![v];
        let new_value = LogResourceValue::<T>::HalfAuthority { log: new_log };
        update_and_redistribute(&mut self.r, &mut other.r, new_value, new_value);
    }

    pub proof fn extract_prefix_knowledge(tracked &self) -> (tracked out: Self)
    // TODO: add requires and ensures
    {
        let v = LogResourceValue::<T>::PrefixKnowledge { prefix: self@.log() };
        let tracked r = copy_duplicable_part(&self.r, v);
        Self { r }
    }

    pub proof fn deduce_prefix_relation(tracked &mut self, tracked other: &Self)
    // TODO: add requires and ensures
    {
        self.r.validate_2(&other.r)
    }
}

pub fn main() {
    let tracked full_auth = LogResource::<int>::alloc();
    assert(full_auth@ is FullAuthority);
    assert(full_auth@.log().len() == 0);
    proof {
        full_auth.append(42);
    }
    proof {
        full_auth.append(86);
    }
    assert(full_auth@.log().len() == 2);
    assert(full_auth@.log()[0] == 42);
    assert(full_auth@.log()[1] == 86);
    let tracked (half_auth1, half_auth2) = full_auth.split();
    assert(half_auth1@ == half_auth2@);
    assert(half_auth1@ is HalfAuthority);
    proof {
        half_auth1.append_using_two_halves(&mut half_auth2, 17);
    }
    assert(half_auth1@.log().len() == 3);
    assert(half_auth1@.log()[2] == 17);
    assert(half_auth2@ == half_auth1@);
}

} // verus!
