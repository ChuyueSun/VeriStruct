#![allow(unused_attributes)]
#![allow(unused_imports)]
// --------------------------------------------
// vstd::seq_lib
// --------------------------------------------

/// The Verus sequence library, supporting specifications,
/// proofs, and ghost code for sequence operations.

verus! {

// -- Everything below is unchanged --------------------

use crate::builtin_macros::*;

macro_rules! get_bit_macro {
    ($a:expr, $b:expr) => {{
        (0x1u32 & ($a >> $b)) == 1
    }};
}

macro_rules! get_bit {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(get_bit_macro!($($a)*))
    }
}


verus_proof_macro_exprs!() { /* proc-macro */ }
verus_proof_macro_exprs!(f!(exprs)) applies verus syntax to transform exprs into exprs’, then returns f!(exprs’), where exprs is a sequence of expressions separated by “,”, “;”, and/or “=>”.

#[allow(unused_imports)]
use super::multiset::Multiset;
#[allow(unused_imports)]
use super::pervasive::*;
#[allow(unused_imports)]
use super::prelude::*;
#[allow(unused_imports)]
use super::relations::*;
#[allow(unused_imports)]
use super::seq::*;
#[allow(unused_imports)]
use super::set::Set;

broadcast use group_seq_axioms;

impl<A> Seq<A> {
    /// Applies the function `f` to each element of the sequence, and returns
    /// the resulting sequence.
    /// The `int` parameter of `f` is the index of the element being mapped.
    // TODO(verus): rename to map_entries, for consistency with Map::map
    pub open spec fn map<B>(self, f: spec_fn(int, A) -> B) -> Seq<B> {
        Seq::new(self.len(), |i: int| f(i, self[i]))
    }

    /// Applies the function `f` to each element of the sequence, and returns
    /// the resulting sequence.
    // TODO(verus): rename to map, because this is what everybody wants.
    pub open spec fn map_values<B>(self, f: spec_fn(A) -> B) -> Seq<B> {
        Seq::new(self.len(), |i: int| f(self[i]))
    }

    /// Is true if the calling sequence is a prefix of the given sequence 'other'.
    ///
    /// ## Example
    ///
    /// ```rust
    /// proof fn prefix_test() {
    ///     let pre: Seq<int> = seq![1, 2, 3];
    ///     let whole: Seq<int> = seq![1, 2, 3, 4, 5];
    ///     assert(pre.is_prefix_of(whole));
    /// }
    /// ```
    pub open spec fn is_prefix_of(self, other: Self) -> bool {
        self.len() <= other.len() && self =~= other.subrange(0, self.len() as int)
    }

    /// Is true if the calling sequence is a suffix of the given sequence 'other'.
    ///
    /// ## Example
    ///
    /// ```rust
    /// proof fn suffix_test() {
    ///     let end: Seq<int> = seq![3, 4, 5];
    ///     let whole: Seq<int> = seq![1, 2, 3, 4, 5];
    ///     assert(end.is_suffix_of(whole));
    /// }
    /// ```
    pub open spec fn is_suffix_of(self, other: Self) -> bool {
        self.len() <= other.len() && self =~= other.subrange(
            (other.len() - self.len()) as int,
            other.len() as int,
        )
    }

    /// Sorts the sequence according to the given leq function
    ///
    /// ## Example
    ///
    /// ```rust
    /// {{#include ../../../rust_verify/example/multiset.rs:sorted_by_leq}}
    /// ```
    pub closed spec fn sort_by(self, leq: spec_fn(A, A) -> bool) -> Seq<A>
        recommends
            total_ordering(leq),
        decreases self.len(),
    {
        if self.len() <= 1 {
            self
        } else {
            let split_index = self.len() / 2;
            let left = self.subrange(0, split_index as int);
            let right = self.subrange(split_index as int, self.len() as int);
            let left_sorted = left.sort_by(leq);
            let right_sorted = right.sort_by(leq);
            merge_sorted_with(left_sorted, right_sorted, leq)
        }
    }

    pub proof fn lemma_sort_by_ensures(self, leq: spec_fn(A, A) -> bool)
        requires
            total_ordering(leq),
        ensures
            self.to_multiset() =~= self.sort_by(leq).to_multiset(),
            sorted_by(self.sort_by(leq), leq),
            forall|x: A| !self.contains(x) ==> !(#[trigger] self.sort_by(leq).contains(x)),
        decreases self.len(),
    {
        if self.len() <= 1 {
        } else {
            let split_index = self.len() / 2;
            let left = self.subrange(0, split_index as int);
            let right = self.subrange(split_index as int, self.len() as int);
            assert(self =~= left + right);
            let left_sorted = left.sort_by(leq);
            left.lemma_sort_by_ensures(leq);
            let right_sorted = right.sort_by(leq);
            right.lemma_sort_by_ensures(leq);
            lemma_merge_sorted_with_ensures(left_sorted, right_sorted, leq);
            lemma_multiset_commutative(left, right);
            lemma_multiset_commutative(left_sorted, right_sorted);
            assert forall|x: A| !self.contains(x) implies !(#[trigger] self.sort_by(leq).contains(
                x,
            )) by {
                broadcast use group_to_multiset_ensures;

                assert(!self.contains(x) ==> self.to_multiset().count(x) == 0);
            }
        }
    }

    /// Returns the sequence containing only the elements of the original sequence
    /// such that pred(element) is true.
    ///
    /// ## Example
    ///
    /// ```rust
    /// proof fn filter_test() {
    ///    let seq: Seq<int> = seq![1, 2, 3, 4, 5];
    ///    let even: Seq<int> = seq.filter(|x| x % 2 == 0);
    ///    reveal_with_fuel(Seq::<int>::filter, 6); //Needed for Verus to unfold the recursive definition of filter
    ///    assert(even =~= seq![2, 4]);
    /// }
    /// ```
    #[verifier::opaque]
    pub open spec fn filter(self, pred: spec_fn(A) -> bool) -> Self
        decreases self.len(),
    {
        if self.len() == 0 {
            self
        } else {
            let subseq = self.drop_last().filter(pred);
            if pred(self.last()) {
                subseq.push(self.last())
            } else {
                subseq
            }
        }
    }

    pub broadcast proof fn lemma_filter_len(self, pred: spec_fn(A) -> bool)
        ensures
    // the filtered list can't grow

            #[trigger] self.filter(pred).len() <= self.len(),
        decreases self.len(),
    {
        reveal(Seq::filter);
        let out = self.filter(pred);
        if 0 < self.len() {
            self.drop_last().lemma_filter_len(pred);
        }
    }

    pub broadcast proof fn lemma_filter_pred(self, pred: spec_fn(A) -> bool, i: int)
        requires
            0 <= i < self.filter(pred).len(),
        ensures
            pred(#[trigger] self.filter(pred)[i]),
    {
        // TODO: remove this after proved filter_lemma is proved
        #[allow(deprecated)]
        self.filter_lemma(pred);
    }

    pub broadcast proof fn lemma_filter_contains(self, pred: spec_fn(A) -> bool, i: int)
        requires
            0 <= i < self.len() && pred(self[i]),
        ensures
            #[trigger] self.filter(pred).contains(self[i]),
    {
        // TODO: remove this after proved filter_lemma is proved
        #[allow(deprecated)]
        self.filter_lemma(pred);
    }

    // deprecated since the triggers inside of 2 of the conjuncts are blocked
    #[deprecated = "Use `broadcast use group_filter_ensures` instead" ]
    pub proof fn filter_lemma(self, pred: spec_fn(A) -> bool)
        ensures
    // we don't keep anything bad
    // TODO(andrea): recommends didn't catch this error, where i isn't known to be in
    // self.filter(pred).len()
    //forall |i: int| 0 <= i < self.len() ==> pred(#[trigger] self.filter(pred)[i]),

            forall|i: int|
                0 <= i < self.filter(pred).len() ==> pred(#[trigger] self.filter(pred)[i]),
            // we keep everything we should
            forall|i: int|
                0 <= i < self.len() && pred(self[i]) ==> #[trigger] self.filter(pred).contains(
                    self[i],
                ),
            // the filtered list can't grow
            #[trigger] self.filter(pred).len() <= self.len(),
        decreases self.len(),
    {
        reveal(Seq::filter);
        let out = self.filter(pred);
        if 0 < self.len() {
            self.drop_last().filter_lemma(pred);
            assert forall|i: int| 0 <= i < out.len() implies pred(out[i]) by {
                if i < out.len() - 1 {
                    assert(self.drop_last().filter(pred)[i] == out.drop_last()[i]);  // trigger drop_last
                    assert(pred(out[i]));  // TODO(andrea): why is this line required? It's the conclusion of the assert-forall.
                }
            }
            assert forall|i: int|
                0 <= i < self.len() && pred(self[i]) implies #[trigger] out.contains(self[i]) by {
                if i == self.len() - 1 {
                    assert(self[i] == out[out.len() - 1]);  // witness to contains
                } else {
                    let subseq = self.drop_last().filter(pred);
                    assert(subseq.contains(self.drop_last()[i]));  // trigger recursive invocation
                    let j = choose|j| 0 <= j < subseq.len() && subseq[j] == self[i];
                    assert(out[j] == self[i]);  // TODO(andrea): same, seems needless
                }
            }
        }
    }

    pub broadcast proof fn filter_distributes_over_add(a: Self, b: Self, pred: spec_fn(A) -> bool)
        ensures
            #[trigger] (a + b).filter(pred) == a.filter(pred) + b.filter(pred),
        decreases b.len(),
    {
        reveal(Seq::filter);
        if 0 < b.len() {
            Self::drop_last_distributes_over_add(a, b);
            Self::filter_distributes_over_add(a, b.drop_last(), pred);
            if pred(b.last()) {
                Self::push_distributes_over_add(
                    a.filter(pred),
                    b.drop_last().filter(pred),
                    b.last(),
                );
            }
        } else {
            Self::add_empty_right(a, b);
            Self::add_empty_right(a.filter(pred), b.filter(pred));
        }
    }

    pub broadcast proof fn add_empty_left(a: Self, b: Self)
        requires
            a.len() == 0,
        ensures
            #[trigger] (a + b) == b,
    {
        assert(a + b =~= b);
    }

    pub broadcast proof fn add_empty_right(a: Self, b: Self)
        requires
            b.len() == 0,
        ensures
            #[trigger] (a + b) == a,
    {
        assert(a + b =~= a);
    }

    pub broadcast proof fn push_distributes_over_add(a: Self, b: Self, elt: A)
        ensures
            #[trigger] (a + b).push(elt) == a + b.push(elt),
    {
        assert((a + b).push(elt) =~= a + b.push(elt));
    }

    /// Returns the sequence containing only the elements of the original sequence
    /// such that pred(element) is true.
    pub open spec fn max_via(self, leq: spec_fn(A, A) -> bool) -> A
        recommends
            self.len() > 0,
        decreases self.len(),
    {
        if self.len() > 1 {
            if leq(self[0], self.subrange(1, self.len() as int).max_via(leq)) {
                self.subrange(1, self.len() as int).max_via(leq)
            } else {
                self[0]
            }
        } else {
            self[0]
        }
    }

    /// Returns the sequence containing only the elements of the original sequence
    /// such that pred(element) is true.
    pub open spec fn min_via(self, leq: spec_fn(A, A) -> bool) -> A
        recommends
            self.len() > 0,
        decreases self.len(),
    {
        if self.len() > 1 {
            let subseq = self.subrange(1, self.len() as int);
            let elt = subseq.min_via(leq);
            if leq(elt, self[0]) {
                elt
            } else {
                self[0]
            }
        } else {
            self[0]
        }
    }

    // TODO is_sorted -- extract from summer_school e22
    pub open spec fn contains(self, needle: A) -> bool {
        exists|i: int| 0 <= i < self.len() && self[i] == needle
    }

    /// Returns an index where `needle` appears in the sequence.
    /// Returns an arbitrary value if the sequence does not contain the `needle`.
    pub open spec fn index_of(self, needle: A) -> int {
        choose|i: int| 0 <= i < self.len() && self[i] == needle
    }

    /// For an element that occurs at least once in a sequence, if its first occurence
    /// is at index i, Some(i) is returned. Otherwise, None is returned
    pub closed spec fn index_of_first(self, needle: A) -> (result: Option<int>) {
        if self.contains(needle) {
            Some(self.first_index_helper(needle))
        } else {
            None
        }
    }

    // Recursive helper function for index_of_first
    spec fn first_index_helper(self, needle: A) -> int
        recommends
            self.contains(needle),
        decreases self.len(),
    {
        if self.len() <= 0 {
            -1  //arbitrary, will never get to this case

        } else if self[0] == needle {
            0
        } else {
            1 + self.subrange(1, self.len() as int).first_index_helper(needle)
        }
    }

    pub proof fn index_of_first_ensures(self, needle: A)
        ensures
            match self.index_of_first(needle) {
                Some(index) => {
                    &&& self.contains(needle)
                    &&& 0 <= index < self.len()
                    &&& self[index] == needle
                    &&& forall|j: int| 0 <= j < index < self.len() ==> self[j] != needle
                },
                None => { !self.contains(needle) },
            },
        decreases self.len(),
    {
        if self.contains(needle) {
            let index = self.index_of_first(needle).unwrap();
            if self.len() <= 0 {
            } else if self[0] == needle {
            } else {
                assert(Seq::empty().push(self.first()).add(self.drop_first()) =~= self);
                self.drop_first().index_of_first_ensures(needle);
            }
        }
    }

    /// For an element that occurs at least once in a sequence, if its last occurence
    /// is at index i, Some(i) is returned. Otherwise, None is returned
    pub closed spec fn index_of_last(self, needle: A) -> Option<int> {
        if self.contains(needle) {
            Some(self.last_index_helper(needle))
        } else {
            None
        }
    }

    // Recursive helper function for last_index_of
    spec fn last_index_helper(self, needle: A) -> int
        recommends
            self.contains(needle),
        decreases self.len(),
    {
        if self.len() <= 0 {
            -1  //arbitrary, will never get to this case

        } else if self.last() == needle {
            self.len() - 1
        } else {
            self.drop_last().last_index_helper(needle)
        }
    }

    pub proof fn index_of_last_ensures(self, needle: A)
        ensures
            match self.index_of_last(needle) {
                Some(index) => {
                    &&& self.contains(needle)
                    &&& 0 <= index < self.len()
                    &&& self[index] == needle
                    &&& forall|j: int| 0 <= index < j < self.len() ==> self[j] != needle
                },
                None => { !self.contains(needle) },
            },
        decreases self.len(),
    {
        if self.contains(needle) {
            let index = self.index_of_last(needle).unwrap();
            if self.len() <= 0 {
            } else if self.last() == needle {
            } else {
                assert(self.drop_last().push(self.last()) =~= self);
                self.drop_last().index_of_last_ensures(needle);
            }
        }
    }

    /// Drops the last element of a sequence and returns a sequence whose length is
    /// thereby 1 smaller.
    ///
    /// If the input sequence is empty, the result is meaningless and arbitrary.
    pub open spec fn drop_last(self) -> Seq<A>
        recommends
            self.len() >= 1,
    {
        self.subrange(0, self.len() as int - 1)
    }

    /// Dropping the last element of a concatenation of `a` and `b` is equivalent
    /// to skipping the last element of `b` and then concatenating `a` and `b`
    pub proof fn drop_last_distributes_over_add(a: Self, b: Self)
        requires
            0 < b.len(),
        ensures
            (a + b).drop_last() == a + b.drop_last(),
    {
        assert_seqs_equal!((a+b).drop_last(), a+b.drop_last());
    }

    pub open spec fn drop_first(self) -> Seq<A>
        recommends
            self.len() >= 1,
    {
        self.subrange(1, self.len() as int)
    }

    /// returns `true` if the sequence has no duplicate elements
    pub open spec fn no_duplicates(self) -> bool {
        forall|i, j| (0 <= i < self.len() && 0 <= j < self.len() && i != j) ==> self[i] != self[j]
    }

    /// Returns `true` if two sequences are disjoint
    pub open spec fn disjoint(self, other: Self) -> bool {
        forall|i: int, j: int| 0 <= i < self.len() && 0 <= j < other.len() ==> self[i] != other[j]
    }

    /// Converts a sequence into a set
    pub open spec fn to_set(self) -> Set<A> {
        Set::new(|a: A| self.contains(a))
    }

    /// Converts a sequence into a multiset
    pub closed spec fn to_multiset(self) -> Multiset<A>
        decreases self.len(),
    {
        if self.len() == 0 {
            Multiset::<A>::empty()
        } else {
            Multiset::<A>::empty().insert(self.first()).add(self.drop_first().to_multiset())
        }
    }

    // Parts of verified lemma used to be an axiom in the Dafny prelude
    // Note: the inner triggers in this lemma are blocked by `to_multiset_len`
    /// Proof of function to_multiset() correctness
    pub broadcast proof fn to_multiset_ensures(self)
        ensures
            forall|a: A| #[trigger] (self.push(a).to_multiset()) =~= self.to_multiset().insert(a),  // to_multiset_build
            forall|i: int|
                0 <= i < self.len() ==> #[trigger] (self.remove(i).to_multiset())
                    =~= self.to_multiset().remove(self[i]),  // to_multiset_remove
            self.len() == #[trigger] self.to_multiset().len(),  // to_multiset_len
            forall|a: A|
                self.contains(a) <==> #[trigger] self.to_multiset().count(a)
                    > 0,  // to_multiset_contains
    {
        broadcast use group_seq_properties;

    }

    /// Insert item a at index i, shifting remaining elements (if any) to the right
    pub open spec fn insert(self, i: int, a: A) -> Seq<A>
        recommends
            0 <= i <= self.len(),
    {
        self.subrange(0, i).push(a) + self.subrange(i, self.len() as int)
    }

    /// Proof of correctness and expected properties of insert function
    pub proof fn insert_ensures(self, pos: int, elt: A)
        requires
            0 <= pos <= self.len(),
        ensures
            self.insert(pos, elt).len() == self.len() + 1,
            forall|i: int| 0 <= i < pos ==> #[trigger] self.insert(pos, elt)[i] == self[i],
            forall|i: int| pos <= i < self.len() ==> self.insert(pos, elt)[i + 1] == self[i],
            self.insert(pos, elt)[pos] == elt,
    {
    }

    /// Remove item at index i, shifting remaining elements to the left
    pub open spec fn remove(self, i: int) -> Seq<A>
        recommends
            0 <= i < self.len(),
    {
        self.subrange(0, i) + self.subrange(i + 1, self.len() as int)
    }

    /// Proof of function remove() correctness
    pub proof fn remove_ensures(self, i: int)
        requires
            0 <= i < self.len(),
        ensures
            self.remove(i).len() == self.len() - 1,
            forall|index: int| 0 <= index < i ==> #[trigger] self.remove(i)[index] == self[index],
            forall|index: int|
                i <= index < self.len() - 1 ==> #[trigger] self.remove(i)[index] == self[index + 1],
    {
    }

    /// If a given element occurs at least once in a sequence, the sequence without
    /// its first occurrence is returned. Otherwise the same sequence is returned.
    pub open spec fn remove_value(self, val: A) -> Seq<A> {
        let index = self.index_of_first(val);
        match index {
            Some(i) => self.remove(i),
            None => self,
        }
    }

    /// Returns the sequence that is in reverse order to a given sequence.
    pub open spec fn reverse(self) -> Seq<A>
        decreases self.len(),
    {
        if self.len() == 0 {
            Seq::empty()
        } else {
            Seq::new(self.len(), |i: int| self[self.len() - 1 - i])
        }
    }

    /// Zips two sequences of equal length into one sequence that consists of pairs.
    /// If the two sequences are different lengths, returns an empty sequence
    pub open spec fn zip_with<B>(self, other: Seq<B>) -> Seq<(A, B)>
        recommends
            self.len() == other.len(),
        decreases self.len(),
    {
        if self.len() != other.len() {
            Seq::empty()
        } else if self.len() == 0 {
            Seq::empty()
        } else {
            Seq::new(self.len(), |i: int| (self[i], other[i]))
        }
    }

    /// Folds the sequence to the left, applying `f` to perform the fold.
    ///
    /// Equivalent to `Iterator::fold` in Rust.
    ///
    /// Given a sequence `s = [x0, x1, x2, ..., xn]`, applying this function `s.fold_left(b, f)`
    /// returns `f(...f(f(b, x0), x1), ..., xn)`.
    pub open spec fn fold_left<B>(self, b: B, f: spec_fn(B, A) -> B) -> (res: B)
        decreases self.len(),
    {
        if self.len() == 0 {
            b
        } else {
            f(self.drop_last().fold_left(b, f), self.last())
        }
    }

    /// Equivalent to [`Self::fold_left`] but defined by breaking off the leftmost element when
    /// recursing, rather than the rightmost. See [`Self::lemma_fold_left_alt`] that proves
    /// equivalence.
    pub open spec fn fold_left_alt<B>(self, b: B, f: spec_fn(B, A) -> B) -> (res: B)
        decreases self.len(),
    {
        if self.len() == 0 {
            b
        } else {
            self.subrange(1, self.len() as int).fold_left_alt(f(b, self[0]), f)
        }
    }

    /// An auxiliary lemma for proving [`Self::lemma_fold_left_alt`].
    proof fn aux_lemma_fold_left_alt<B>(self, b: B, f: spec_fn(B, A) -> B, k: int)
        requires
            0 < k <= self.len(),
        ensures
            self.subrange(k, self.len() as int).fold_left_alt(
                self.subrange(0, k).fold_left_alt(b, f),
                f,
            ) == self.fold_left_alt(b, f),
        decreases k,
    {
        reveal_with_fuel(Seq::fold_left_alt, 2);
        if k == 1 {
            // trivial base case
        } else {
            self.subrange(1, self.len() as int).aux_lemma_fold_left_alt(f(b, self[0]), f, k - 1);
            assert_seqs_equal!(
                self.subrange(1, self.len() as int)
                    .subrange(k - 1, self.subrange(1, self.len() as int).len() as int) ==
                self.subrange(k, self.len() as int)
            );
            assert_seqs_equal!(
                self.subrange(1, self.len() as int).subrange(0, k - 1) ==
                self.subrange(1, k)
            );
            assert_seqs_equal!(
                self.subrange(0, k).subrange(1, self.subrange(0, k).len() as int) ==
                self.subrange(1, k)
            );
        }
    }

    /// [`Self::fold_left`] and [`Self::fold_left_alt`] are equivalent.
    pub proof fn lemma_fold_left_alt<B>(self, b: B, f: spec_fn(B, A) -> B)
        ensures
            self.fold_left(b, f) == self.fold_left_alt(b, f),
        decreases self.len(),
    {
        reveal_with_fuel(Seq::fold_left, 2);
        reveal_with_fuel(Seq::fold_left_alt, 2);
        if self.len() <= 1 {
            // trivial base cases
        } else {
            self.aux_lemma_fold_left_alt(b, f, self.len() - 1);
            self.subrange(self.len() - 1, self.len() as int).lemma_fold_left_alt(
                self.drop_last().fold_left_alt(b, f),
                f,
            );
            self.subrange(0, self.len() - 1).lemma_fold_left_alt(b, f);
        }
    }

    /// Folds the sequence to the right, applying `f` to perform the fold.
    ///
    /// Equivalent to `DoubleEndedIterator::rfold` in Rust.
    ///
    /// Given a sequence `s = [x0, x1, x2, ..., xn]`, applying this function `s.fold_right(f, b)`
    /// returns `f(x0, f(x1, f(x2, ..., f(xn, b)...)))`.
    pub open spec fn fold_right<B>(self, f: spec_fn(A, B) -> B, b: B) -> (res: B)
        decreases self.len(),
    {
        if self.len() == 0 {
            b
        } else {
            self.drop_last().fold_right(f, f(self.last(), b))
        }
    }

    /// Equivalent to [`Self::fold_right`] but defined by breaking off the leftmost element when
    /// recursing, rather than the rightmost. See [`Self::lemma_fold_right_alt`] that proves
    /// equivalence.
    pub open spec fn fold_right_alt<B>(self, f: spec_fn(A, B) -> B, b: B) -> (res: B)
        decreases self.len(),
    {
        if self.len() == 0 {
            b
        } else {
            f(self[0], self.subrange(1, self.len() as int).fold_right_alt(f, b))
        }
    }

    /// A lemma that proves how [`Self::fold_right`] distributes over splitting a sequence.
    pub proof fn lemma_fold_right_split<B>(self, f: spec_fn(A, B) -> B, b: B, k: int)
        requires
            0 <= k <= self.len(),
        ensures
            self.subrange(0, k).fold_right(f, self.subrange(k, self.len() as int).fold_right(f, b))
                == self.fold_right(f, b),
        decreases self.len(),
    {
        reveal_with_fuel(Seq::fold_right, 2);
        if k == self.len() {
            assert(self.subrange(0, k) == self);
        } else if k == self.len() - 1 {
            // trivial base case
        } else {
            self.subrange(0, self.len() - 1).lemma_fold_right_split(f, f(self.last(), b), k);
            assert_seqs_equal!(
                self.subrange(0, self.len() - 1).subrange(0, k) ==
                self.subrange(0, k)
            );
            assert_seqs_equal!(
                self.subrange(0, self.len() - 1).subrange(k, self.subrange(0, self.len() - 1).len() as int) ==
                self.subrange(k, self.len() - 1)
            );
            assert_seqs_equal!(
                self.subrange(k, self.len() as int).drop_last() ==
                self.subrange(k, self.len() - 1)
            );
        }
    }

    // Lemma that proves it's possible to commute a commutative operator across fold_right.
    pub proof fn lemma_fold_right_commute_one<B>(self, a:

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 10
