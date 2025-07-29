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
        &&& !(self is Invalid)
    }

    open spec fn op(self, other: Self) -> Self {
        match (self, other) {
            (
                Self::PrefixKnowledge { prefix: prefix1 },
                Self::PrefixKnowledge { prefix: prefix2 },
            ) => if is_prefix(prefix1, prefix2) {
                other
            } else {
                if is_prefix(prefix2, prefix1) {
                    self
                } else {
                    Self::Invalid
                }
            },
            (Self::PrefixKnowledge { prefix }, Self::HalfAuthority { log }) => if is_prefix(
                prefix,
                log,
            ) {
                other
            } else {
                Self::Invalid
            },
            (Self::HalfAuthority { log }, Self::PrefixKnowledge { prefix }) => if is_prefix(
                prefix,
                log,
            ) {
                self
            } else {
                Self::Invalid
            },
            (Self::PrefixKnowledge { prefix }, Self::FullAuthority { log }) => if is_prefix(
                prefix,
                log,
            ) {
                other
            } else {
                Self::Invalid
            },
            (Self::FullAuthority { log }, Self::PrefixKnowledge { prefix }) => if is_prefix(
                prefix,
                log,
            ) {
                self
            } else {
                Self::Invalid
            },
            (Self::HalfAuthority { log: log1 }, Self::HalfAuthority { log: log2 }) => if log1
                == log2 {
                Self::FullAuthority { log: log1 }
            } else {
                Self::Invalid
            },
            (_, _) => Self::Invalid,
        }
    }

    open spec fn unit() -> Self {
        Self::PrefixKnowledge { prefix: Seq::<T>::empty() }
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
        match self {
            LogResourceValue::PrefixKnowledge { prefix } => prefix,
            LogResourceValue::HalfAuthority { log } => log,
            LogResourceValue::FullAuthority { log } => log,
            LogResourceValue::Invalid => Seq::<T>::empty(),
        }
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
        self.r.loc()
    }

    pub closed spec fn view(self) -> LogResourceValue<T> {
        self.r.value()
    }

    pub proof fn alloc() -> (tracked result: LogResource<T>)
        ensures
            result@ is FullAuthority,
            result@.log() == Seq::<T>::empty(),
    {
        let v = LogResourceValue::<T>::FullAuthority { log: Seq::<T>::empty() };
        let tracked r = Resource::<LogResourceValue::<T>>::alloc(v);
        Self { r }
    }

    pub proof fn split(tracked self) -> (tracked halves: (Self, Self))
        requires
            self@ is FullAuthority,
        ensures
            ({
                let (half1, half2) = halves;
                &&& half1@ is HalfAuthority
                &&& half2@ is HalfAuthority
                &&& half1.id() == half2.id() == self.id()
                &&& half1@.log() == self@.log()
                &&& half2@ == half1@
            }),
    {
        let half_value = LogResourceValue::<T>::HalfAuthority { log: self@.log() };
        let tracked (half1, half2) = self.r.split(half_value, half_value);
        (Self { r: half1 }, Self { r: half2 })
    }

    pub proof fn append(tracked &mut self, v: T)
        requires
            old(self)@ is FullAuthority,
        ensures
            self@ is FullAuthority,
            self.id() == old(self).id(),
            self@.log() == old(self)@.log() + seq![v],
    {
        let value = LogResourceValue::<T>::FullAuthority { log: self@.log() + seq![v] };
        update_mut(&mut self.r, value);
    }

    pub proof fn append_using_two_halves(tracked &mut self, tracked other: &mut Self, v: T)
        requires
            old(self)@ is HalfAuthority,
            old(other)@ is HalfAuthority,
            old(self).id() == old(other).id(),
        ensures
            self@ is HalfAuthority,
            self.id() == other.id() == old(self).id(),
            self@.log() == old(self)@.log() + seq![v],
            other@ == self@,
    {
        self.r.validate_2(&other.r);
        let new_log = self@.log() + seq![v];
        let new_value = LogResourceValue::<T>::HalfAuthority { log: new_log };
        update_and_redistribute(&mut self.r, &mut other.r, new_value, new_value);
    }

    pub proof fn extract_prefix_knowledge(tracked &self) -> (tracked out: Self)
        ensures
            out@ is PrefixKnowledge,
            out.id() == self.id(),
            out@.log() == self@.log(),
    {
        let v = LogResourceValue::<T>::PrefixKnowledge { prefix: self@.log() };
        let tracked r = copy_duplicable_part(&self.r, v);
        Self { r }
    }

    pub proof fn deduce_prefix_relation(tracked &mut self, tracked other: &Self)
        requires
            old(self).id() == other.id(),
        ensures
            self@ == old(self)@,
            is_prefix(self@.log(), other@.log()) || is_prefix(other@.log(), self@.log()),
            self@ is HalfAuthority ==> is_prefix(other@.log(), self@.log()),
            self@ is FullAuthority ==> is_prefix(other@.log(), self@.log()),
            other@ is HalfAuthority ==> is_prefix(self@.log(), other@.log()),
            other@ is FullAuthority ==> is_prefix(self@.log(), other@.log()),
    {
        self.r.validate_2(&other.r)
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
