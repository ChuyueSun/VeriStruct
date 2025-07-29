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
    // TODO: add specification
}

impl<T> PCM for LogResourceValue<T> {
    open spec fn valid(self) -> bool {
    // TODO: add specification
    }

    open spec fn op(self, other: Self) -> Self {
    // TODO: add specification
    }

    open spec fn unit() -> Self {
    // TODO: add specification
    }

    proof fn closed_under_incl(a: Self, b: Self) {
    }

    proof fn commutative(a: Self, b: Self) {
    // TODO: add proof
    }

    proof fn associative(a: Self, b: Self, c: Self) {
    // TODO: add proof
    }

    proof fn op_unit(a: Self) {
    // TODO: add proof
    }

    proof fn unit_valid() {
    }
}

impl<T> LogResourceValue<T> {
    pub open spec fn log(self) -> Seq<T> {
    // TODO: add specification
    }

    proof fn op_unit(a: Self) {
    // TODO: add proof
    }

    proof fn unit_valid() {
    }
}

pub struct LogResource<T> {
    r: Resource<LogResourceValue<T>>,
}

impl<T> LogResource<T> {
    pub closed spec fn id(self) -> Loc {
    // TODO: add specification
    }

    pub closed spec fn view(self) -> LogResourceValue<T> {
    // TODO: add specification
    }

    pub proof fn alloc() -> (tracked result: LogResource<T>)
    // TODO: add requires and ensures
    {
        // TODO: add proof
    }

    pub proof fn split(tracked self) -> (tracked halves: (Self, Self))
    // TODO: add requires and ensures
    {
        // TODO: add proof
    }

    pub proof fn append(tracked &mut self, v: T)
    // TODO: add requires and ensures
    {
        // TODO: add proof
    }

    pub proof fn append_using_two_halves(tracked &mut self, tracked other: &mut Self, v: T)
    // TODO: add requires and ensures
    {
        // TODO: add proof
    }

    pub proof fn extract_prefix_knowledge(tracked &self) -> (tracked out: Self)
    // TODO: add requires and ensures
    {
        // TODO: add proof
    }

    pub proof fn deduce_prefix_relation(tracked &mut self, tracked other: &Self)
    // TODO: add requires and ensures
    {
        // TODO: add proof
    }
}

/* TEST CODE BELOW */

pub fn test(t: Vec<int>, v: int) {
    let tracked full_auth = LogResource::<int>::alloc();
    assert(full_auth@ is FullAuthority);
    assert(full_auth@.log().len() == 0);

    for i in 0..t.len() 
    invariant
        full_auth@ is FullAuthority,
        full_auth@.log().len() == i,
    {
        proof {
            full_auth.append(t[i as int]);
        }
        assert(full_auth@.log()[i as int] == t[i as int]);
    }
    let tracked (half_auth1, half_auth2) = full_auth.split();
    assert(half_auth1@ == half_auth2@);
    assert(half_auth1@ is HalfAuthority);
    let tracked pk = half_auth1.extract_prefix_knowledge();
    assert(is_prefix(pk@.log(), half_auth1@.log()) && is_prefix(half_auth1@.log(), pk@.log()));
    proof {
        half_auth1.append_using_two_halves(&mut half_auth2, v);
    }
    assert(is_prefix(pk@.log(), half_auth1@.log()));
    assert(!is_prefix(half_auth1@.log(), pk@.log()));
    assert(half_auth1.id() == half_auth2.id());
    assert(half_auth1@.log().len() == t.len() + 1);
    assert(half_auth1@.log()[t.len() as int] == v);
    assert(half_auth2@ == half_auth1@);

    // Additional tests for the PCM op() function
    proof {
        // prefix vs larger prefix: smaller prefix is absorbed
        let pk1 = LogResourceValue::<int>::PrefixKnowledge { prefix: seq![1, 2] };
        let pk2 = LogResourceValue::<int>::PrefixKnowledge { prefix: seq![1, 2, 3] };
        assert(pk1.op(pk2) == pk2);
        assert(pk2.op(pk1) == pk2);

        // conflicting prefixes produce Invalid
        let pk3 = LogResourceValue::<int>::PrefixKnowledge { prefix: seq![4] };
        assert(pk1.op(pk3) is Invalid);
        assert(pk3.op(pk1) is Invalid);

        // two equal half authorities combine into a full authority
        let ha1 = LogResourceValue::<int>::HalfAuthority { log: seq![5, 6] };
        let ha2 = LogResourceValue::<int>::HalfAuthority { log: seq![5, 6] };
        let combined = ha1.op(ha2);
        assert(combined == LogResourceValue::<int>::FullAuthority { log: seq![5, 6] });

        // prefix vs half authority
        let pk4 = LogResourceValue::<int>::PrefixKnowledge { prefix: seq![5] };
        let ha3 = LogResourceValue::<int>::HalfAuthority { log: seq![5, 6] };
        assert(pk4.op(ha3) == ha3);
        assert(ha3.op(pk4) == ha3);

        // prefix vs full authority
        let fa = LogResourceValue::<int>::FullAuthority { log: seq![7, 8] };
        let pk_empty = LogResourceValue::<int>::PrefixKnowledge { prefix: seq![] };
        assert(pk_empty.op(fa) == fa);
        assert(fa.op(pk_empty) == fa);

        // any combination with Invalid yields Invalid
        let inv = LogResourceValue::<int>::Invalid;
        assert(inv.op(pk1) is Invalid);
        assert(pk1.op(inv) is Invalid);
        assert(inv.op(ha1) is Invalid);
        assert(ha1.op(inv) is Invalid);
        assert(inv.op(fa) is Invalid);
        assert(fa.op(inv) is Invalid);

        // Test commutativity of op()
        LogResourceValue::<int>::commutative(pk1, pk2);
        LogResourceValue::<int>::commutative(pk1, pk1);
        LogResourceValue::<int>::commutative(ha1, ha2);
        LogResourceValue::<int>::commutative(pk4, fa);
        LogResourceValue::<int>::commutative(fa, pk_empty);
        LogResourceValue::<int>::commutative(inv, pk1);
        LogResourceValue::<int>::associative(pk1, pk2, pk3);
        LogResourceValue::<int>::associative(ha1, ha2, ha3);
        LogResourceValue::<int>::associative(pk4, ha1, fa);
        LogResourceValue::<int>::associative(pk_empty, pk1, fa);

    }

    proof {
        // Test deduce_prefix_relation on a half authority
        half_auth1.deduce_prefix_relation(&pk);
    }
}

pub fn main() {
}

} // verus!
