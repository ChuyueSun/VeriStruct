# Prompt

## Instruction
Your mission is to fix the assertion error for the following code. Basically, you should either introduce the necessary proof blocks before the location where the assertion fails, or, if the assertion is within a loop or after a loop, you may need to add appropriate loop invariants to ensure the assertion holds true.

Response with the Rust code only, do not include any explanation.

IMPORTANT:
1. Don't change the anything in immutable function(s): . Instead, consider adjusting the preconditions or postconditions of other functions or methods.
2. Don't delete existing non-buggy `#[trigger]`, `use` statements, main function.


## Exemplars

### Example 1

## Query
Failed assertion
```
Line 48-48:
                    assert(j < n);
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {
    spec fn sorted_between(a: Seq<u32>, from: int, to: int) -> bool {
        forall |i: int, j:int|  from <= i < j < to ==> a[i] <= a[j]
    }


    spec fn is_reorder_of<T>(r: Seq<int>, p: Seq<T>, s: Seq<T>) -> bool {
    &&& r.len() == s.len()
    &&& forall|i: int| 0 <= i < r.len() ==> 0 <= #[trigger] r[i] < r.len()
    &&& forall|i: int, j: int| 0 <= i < j < r.len() ==> r[i] != r[j]
    &&& p =~= r.map_values(|i: int| s[i])
    }


    fn test1(nums: &mut Vec<u32>)
        ensures
            sorted_between(nums@, 0, nums@.len() as int),
            exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
    {
        proof {
            let r = Seq::new(nums@.len(), |i: int| i); // Added by AI, for assertion fail
            assert(is_reorder_of(r, nums@, nums@)); // Added by AI, for assertion fail
            assert(exists|r: Seq<int>| is_reorder_of(r, nums@, nums@));
        }
        let n = nums.len();
        if n == 0 {
            return;
        }
        for i in 1..n
            invariant
                n == nums.len(),
                sorted_between(nums@, 0, i as int),
                exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
        {
            let mut j = i;
            while j != 0
                invariant
                    0 <= j <= i,
                    n == nums.len(),
                    forall|x: int, y: int| 0 <= x <= y <= i ==> x != j && y != j ==> nums[x] <= nums[y],
                    sorted_between(nums@, j as int, i + 1),
                    exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
            {
                proof {
                    assert(j < n);
                }
                if nums[j - 1] > nums[j] {
                    proof {
                        let r1 = choose|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@);
                        let r2 = r1.update(j-1, r1[j as int]).update(j as int, r1[j-1]);
                        assert(is_reorder_of(r2, nums@.update(j-1, nums@[j as int]).update(j as int, nums@[j-1]), old(nums)@));
                    }
                    let temp = nums[j - 1];
                    nums.set(j - 1, nums[j]);
                    nums.set(j, temp);
                }
                j -= 1;
                proof{
                    assert(exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@));
                }
            }
        }
    }
}
```


## Answer
use vstd::prelude::*;
fn main() {}

verus! {
    spec fn sorted_between(a: Seq<u32>, from: int, to: int) -> bool {
        forall |i: int, j:int|  from <= i < j < to ==> a[i] <= a[j]
    }


    spec fn is_reorder_of<T>(r: Seq<int>, p: Seq<T>, s: Seq<T>) -> bool {
    &&& r.len() == s.len()
    &&& forall|i: int| 0 <= i < r.len() ==> 0 <= #[trigger] r[i] < r.len()
    &&& forall|i: int, j: int| 0 <= i < j < r.len() ==> r[i] != r[j]
    &&& p =~= r.map_values(|i: int| s[i])
    }


    fn test1(nums: &mut Vec<u32>)
        ensures
            sorted_between(nums@, 0, nums@.len() as int),
            exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
    {
        proof {
            let r = Seq::new(nums@.len(), |i: int| i); // Added by AI, for assertion fail
            assert(is_reorder_of(r, nums@, nums@)); // Added by AI, for assertion fail
            assert(exists|r: Seq<int>| is_reorder_of(r, nums@, nums@));
        }
        let n = nums.len();
        if n == 0 {
            return;
        }
        for i in 1..n
            invariant
                n == nums.len(),
                sorted_between(nums@, 0, i as int),
                exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
        {
            let mut j = i;
            while j != 0
                invariant
                    0 <= j <= i,
                    j < n, // Added by AI, for assertion fail
                    i < n, // Added by AI, for assertion fail
                    n == nums.len(),
                    forall|x: int, y: int| 0 <= x <= y <= i ==> x != j && y != j ==> nums[x] <= nums[y],
                    sorted_between(nums@, j as int, i + 1),
                    exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
            {
                proof {
                    assert(j < n);
                }
                if nums[j - 1] > nums[j] {
                    proof {
                        let r1 = choose|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@);
                        let r2 = r1.update(j-1, r1[j as int]).update(j as int, r1[j-1]);
                        assert(is_reorder_of(r2, nums@.update(j-1, nums@[j as int]).update(j as int, nums@[j-1]), old(nums)@));
                    }
                    let temp = nums[j - 1];
                    nums.set(j - 1, nums[j]);
                    nums.set(j, temp);
                }
                j -= 1;
                proof{
                    assert(exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@));
                }
            }
        }
    }
}


### Example 2

## Query
Failed assertion
```
Line 24-24:
            assert(exists|r: Seq<int>| is_reorder_of(r, nums@, nums@));
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {
    spec fn sorted_between(a: Seq<u32>, from: int, to: int) -> bool {
        forall |i: int, j:int|  from <= i < j < to ==> a[i] <= a[j]
    }


    spec fn is_reorder_of<T>(r: Seq<int>, p: Seq<T>, s: Seq<T>) -> bool {
    &&& r.len() == s.len()
    &&& forall|i: int| 0 <= i < r.len() ==> 0 <= #[trigger] r[i] < r.len()
    &&& forall|i: int, j: int| 0 <= i < j < r.len() ==> r[i] != r[j]
    &&& p =~= r.map_values(|i: int| s[i])
    }


    fn test1(nums: &mut Vec<u32>)
        ensures
            sorted_between(nums@, 0, nums@.len() as int),
            exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
    {
        proof {
            assert(exists|r: Seq<int>| is_reorder_of(r, nums@, nums@));
        }
        let n = nums.len();
        if n == 0 {
            return;
        }
        for i in 1..n
            invariant
                n == nums.len(),
                sorted_between(nums@, 0, i as int),
                exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
        {
            let mut j = i;
            while j != 0
                invariant
                    0 <= j <= i < n == nums.len(),
                    forall|x: int, y: int| 0 <= x <= y <= i ==> x != j && y != j ==> nums[x] <= nums[y],
                    sorted_between(nums@, j as int, i + 1),
                    exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
            {
                if nums[j - 1] > nums[j] {
                    proof {
                        let r1 = choose|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@);
                        let r2 = r1.update(j-1, r1[j as int]).update(j as int, r1[j-1]);
                        assert(is_reorder_of(r2, nums@.update(j-1, nums@[j as int]).update(j as int, nums@[j-1]), old(nums)@));
                    }
                    let temp = nums[j - 1];
                    nums.set(j - 1, nums[j]);
                    nums.set(j, temp);
                }
                j -= 1;
                proof{
                    assert(exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@));
                }
            }
        }
    }
}
```


## Answer
use vstd::prelude::*;
fn main() {}

verus! {
    spec fn sorted_between(a: Seq<u32>, from: int, to: int) -> bool {
        forall |i: int, j:int|  from <= i < j < to ==> a[i] <= a[j]
    }


    spec fn is_reorder_of<T>(r: Seq<int>, p: Seq<T>, s: Seq<T>) -> bool {
    &&& r.len() == s.len()
    &&& forall|i: int| 0 <= i < r.len() ==> 0 <= #[trigger] r[i] < r.len()
    &&& forall|i: int, j: int| 0 <= i < j < r.len() ==> r[i] != r[j]
    &&& p =~= r.map_values(|i: int| s[i])
    }


    fn test1(nums: &mut Vec<u32>)
        ensures
            sorted_between(nums@, 0, nums@.len() as int),
            exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
    {
        proof {
            let r = Seq::new(nums@.len(), |i: int| i); // Added by AI, for assertion fail
            assert(is_reorder_of(r, nums@, nums@)); // Added by AI, for assertion fail
            assert(exists|r: Seq<int>| is_reorder_of(r, nums@, nums@));
        }
        let n = nums.len();
        if n == 0 {
            return;
        }
        for i in 1..n
            invariant
                n == nums.len(),
                sorted_between(nums@, 0, i as int),
                exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
        {
            let mut j = i;
            while j != 0
                invariant
                    0 <= j <= i < n == nums.len(),
                    forall|x: int, y: int| 0 <= x <= y <= i ==> x != j && y != j ==> nums[x] <= nums[y],
                    sorted_between(nums@, j as int, i + 1),
                    exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
            {
                if nums[j - 1] > nums[j] {
                    proof {
                        let r1 = choose|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@);
                        let r2 = r1.update(j-1, r1[j as int]).update(j as int, r1[j-1]);
                        assert(is_reorder_of(r2, nums@.update(j-1, nums@[j as int]).update(j as int, nums@[j-1]), old(nums)@));
                    }
                    let temp = nums[j - 1];
                    nums.set(j - 1, nums[j]);
                    nums.set(j, temp);
                }
                j -= 1;
                proof{
                    assert(exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@));
                }
            }
        }
    }
}


### Example 3

## Query
Failed assertion
```
Line 87-87:
            assert(exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@));
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

proof fn lemma_sorted_between_single_element(a: Seq<u32>, i: int)
    requires
        0 <= i < a.len() as int,
    ensures
        sorted_between(a, i, i + 1),
{
    assert(sorted_between(a, i, i + 1));
    assert(forall |x: int, y:int| i <= x < y < (i + 1) ==> a[x] <= a[y]);
}

proof fn lemma_sorted_between_transitive(a: Seq<u32>, i: int, j: int, k: int)
    requires
        sorted_between(a, i, k),
        i <= j,
        j <= k,
    ensures
        sorted_between(a, i, j),
        sorted_between(a, j, k),
{
    assert(sorted_between(a, i, k));
    assert(forall |x: int, y:int| i <= x < y < j ==> a[x] <= a[y]);
    assert(forall |x: int, y:int| j <= x < y < k ==> a[x] <= a[y]);
}

spec fn sorted_between(a: Seq<u32>, from: int, to: int) -> bool {
    forall |i: int, j:int| from <= i < j < to ==> a[i] <= a[j]
}

spec fn is_reorder_of<T>(r: Seq<int>, p: Seq<T>, s: Seq<T>) -> bool {
    &&& r.len() == s.len()
    &&& forall|i: int| 0 <= i < r.len() ==> 0 <= #[trigger] r[i] < r.len()
    &&& forall|i: int, j: int| 0 <= i < j < r.len() ==> r[i] != r[j]
    &&& p =~= r.map_values(|i: int| s[i])
}

fn test1(nums: &mut Vec<u32>)
    ensures
        sorted_between(nums@, 0, nums@.len() as int),
        exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
{
    let n = nums.len();
    if n == 0 {
        proof {
            let r = Seq::new(0, |i: int| i);
            assert(is_reorder_of(r, nums@, nums@));
        }
        return;
    }
    let ghost mut r = Seq::new(nums@.len(), |i: int| i);
    proof {
        assert(is_reorder_of(r, nums@, nums@));
    }
    for i in 1..n
    invariant
        sorted_between(nums@, 0, i as int),
        is_reorder_of(r, nums@, old(nums)@),
        exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
    {
        let mut j = i;
        while j > 0
        invariant
            0 <= j <= i < n,
            n == nums.len(),
            sorted_between(nums@, 0, j as int),
            sorted_between(nums@, j as int, i as int + 1),
            is_reorder_of(r, nums@, old(nums)@),
            exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
        {
            if nums[j - 1] > nums[j] {
                let temp = nums[j - 1];
                nums.set(j - 1, nums[j]);
                nums.set(j, temp);
                proof {
                    lemma_sorted_between_single_element(nums@, j as int - 1);
                }
            }
            proof {
                lemma_sorted_between_transitive(nums@, 0, j as int, i as int + 1);
            }
            j -= 1;
            proof {
                assert(exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@));
            }
        }
    }
}
}
```


## Answer
use vstd::prelude::*;
fn main() {}

verus! {

proof fn lemma_sorted_between_single_element(a: Seq<u32>, i: int)
    requires
        0 <= i < a.len() as int,
    ensures
        sorted_between(a, i, i + 1),
{
    assert(sorted_between(a, i, i + 1));
    assert(forall |x: int, y:int| i <= x < y < (i + 1) ==> a[x] <= a[y]);
}

proof fn lemma_sorted_between_transitive(a: Seq<u32>, i: int, j: int, k: int)
    requires
        sorted_between(a, i, k),
        i <= j,
        j <= k,
    ensures
        sorted_between(a, i, j),
        sorted_between(a, j, k),
{
    assert(sorted_between(a, i, k));
    assert(forall |x: int, y:int| i <= x < y < j ==> a[x] <= a[y]);
    assert(forall |x: int, y:int| j <= x < y < k ==> a[x] <= a[y]);
}

spec fn sorted_between(a: Seq<u32>, from: int, to: int) -> bool {
    forall |i: int, j:int| from <= i < j < to ==> a[i] <= a[j]
}

spec fn is_reorder_of<T>(r: Seq<int>, p: Seq<T>, s: Seq<T>) -> bool {
    &&& r.len() == s.len()
    &&& forall|i: int| 0 <= i < r.len() ==> 0 <= #[trigger] r[i] < r.len()
    &&& forall|i: int, j: int| 0 <= i < j < r.len() ==> r[i] != r[j]
    &&& p =~= r.map_values(|i: int| s[i])
}

fn test1(nums: &mut Vec<u32>)
    ensures
        sorted_between(nums@, 0, nums@.len() as int),
        exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
{
    let n = nums.len();
    if n == 0 {
        proof {
            let r = Seq::new(0, |i: int| i);
            assert(is_reorder_of(r, nums@, nums@));
        }
        return;
    }
    let ghost mut r = Seq::new(nums@.len(), |i: int| i);
    proof {
        assert(is_reorder_of(r, nums@, nums@));
    }
    for i in 1..n
    invariant
        sorted_between(nums@, 0, i as int),
        is_reorder_of(r, nums@, old(nums)@),
        exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
    {
        let mut j = i;
        while j > 0
        invariant
            0 <= j <= i < n,
            n == nums.len(),
            sorted_between(nums@, 0, j as int),
            sorted_between(nums@, j as int, i as int + 1),
            is_reorder_of(r, nums@, old(nums)@),
            exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@),
        {
            if nums[j - 1] > nums[j] {
                let temp = nums[j - 1];
                nums.set(j - 1, nums[j]);
                nums.set(j, temp);
                proof {
                    lemma_sorted_between_single_element(nums@, j as int - 1);
                    r = r.update(j - 1, r[j as int]).update(j as int, r[j - 1]); // Added by AI, for assertion fail
                }
            }
            proof {
                lemma_sorted_between_transitive(nums@, 0, j as int, i as int + 1);
            }
            j -= 1;
            proof {
                assert(is_reorder_of(r, nums@, old(nums)@)); // Added by AI, for assertion fail
                assert(exists|r: Seq<int>| is_reorder_of(r, nums@, old(nums)@));
            }
        }
    }
}
}


### Example 4

## Query
Failed assertion
```
Line 20-20:
        assert(forall|i: int| 0 <= i < result.len() ==> result[i] == source[i]);
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Universal property needs loop invariant to accumulate
fn copy_vec(source: &Vec<i32>) -> (result: Vec<i32>)
    ensures
        result@ == source@,
{
    let mut res = Vec::new();
    for i in 0..source.len()
    {
        res.push(source[i]);
        proof {
            // Assertion fails: universal property not carried through loop
            assert(forall|i: int| 0 <= i < res.len() ==> res[i] == source[i]);
        }
    }
    res
}

}
```

            assert(forall|i: int| 0 <= i < res.len() ==> res[i] == source[i]);
        }
    }
    res
}

}
```


## Answer
use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Add forall property to loop invariant to maintain it
fn copy_vec(source: &Vec<i32>) -> (result: Vec<i32>)
    ensures
        result@ == source@,
{
    let mut res = Vec::new();
    for i in 0..source.len()
        invariant
            res.len() == i, // Added by AI: track length
            forall|j: int| 0 <= j < i ==> res[j] == source[j], // Added by AI: universal property
    {
        res.push(source[i]);
        proof {
            assert(forall|j: int| 0 <= j < res.len() ==> res[j] == source[j]);
        }
    }
    res
}

}

        res.push(source[i]);
        proof {
            assert(forall|j: int| 0 <= j < res.len() ==> res[j] == source[j]);
        }
    }
    res
}

}


### Example 5

## Query
Failed assertion
```
Line 21-21:
        assert(count <= i);
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Conditional increment needs invariant relating accumulator to loop variable
fn count_zeros(nums: &Vec<u32>) -> (count: usize)
    ensures
        count <= nums.len(),
{
    let mut c = 0;
    for i in 0..nums.len()
    {
        if nums[i] == 0 {
            c += 1;
        }
        proof {
            // Assertion fails: relationship between c and i not maintained
            assert(c <= i + 1);
        }
    }
    c
}

}
```


}
```


## Answer
use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Add invariant relating counter to loop progress
fn count_zeros(nums: &Vec<u32>) -> (count: usize)
    ensures
        count <= nums.len(),
{
    let mut c = 0;
    for i in 0..nums.len()
        invariant
            c <= i, // Added by AI: maintain relationship between counter and index
    {
        if nums[i] == 0 {
            c += 1;
        }
        proof {
            assert(c <= i + 1);
        }
    }
    c
}

}

}

}


### Example 6

## Query


## Answer


### Example 7

## Query


## Answer


### Example 8

## Query


## Answer


### Example 9

## Query


## Answer


## Query
Failed assertion
```
                assert(node.unwrap().well_formed());
```

Code
```
#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;

verus!{

/// A node in the binary search tree containing a key-value pair and optional left/right children.
/// The node maintains BST property: all keys in left subtree < node.key < all keys in right subtree.
pub struct Node<V> {
    pub key: u64,                          // The key used for ordering in the BST
    pub value: V,                          // The value associated with this key
    pub left: Option<Box<Node<V>>>,        // Optional left child (contains keys smaller than this node's key)
    pub right: Option<Box<Node<V>>>,       // Optional right child (contains keys larger than this node's key)
}

impl<V> Node<V> {
    pub open spec fn optional_as_map(node_opt: Option<Box<Node<V>>>) -> Map<u64, V>
        decreases node_opt,
    {
        match node_opt {
            None => Map::empty(),
            Some(node) => node.as_map(),
        }
    }

    pub open spec fn as_map(self) -> Map<u64, V>
        decreases self,
    {
         Node::<V>::optional_as_map(self.left)
          .union_prefer_right(Node::<V>::optional_as_map(self.right))
          .insert(self.key, self.value)
    }

    pub open spec fn well_formed(self) -> bool
        decreases self
    {
        (forall |k: u64| Node::<V>::optional_as_map(self.left).dom().contains(k) === (k < self.key))
     && (forall |k: u64| Node::<V>::optional_as_map(self.right).dom().contains(k) === (k > self.key))
     && match self.left {
            None => true,
            Some(l) => l.well_formed()
        }
     && match self.right {
            None => true,
            Some(r) => r.well_formed()
        }
    }

    pub fn insert_into_optional(node: &mut Option<Box<Node<V>>>, key: u64, value: V)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).insert(key, value)
    {
        if node.is_none() {
            *node = Some(Box::new(Node::<V> {
                key: key,
                value: value,
                left: None,
                right: None,
            }));
        } else {
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            (&mut *boxed_node).insert(key, value);

            *node = Some(boxed_node);
        }
        proof {
            if node.is_some() {
                assert(node.unwrap().well_formed());
            }
        } // Added by AI
    }

    pub fn insert(&mut self, key: u64, value: V)
        requires
            old(self).well_formed()
        ensures
            self.well_formed(),
            self.as_map() =~= old(self).as_map().insert(key, value)
    {
        if key == self.key {
            self.value = value;
            assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
            assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
        } else if key < self.key {
            Self::insert_into_optional(&mut self.left, key, value);
            assert(!Node::<V>::optional_as_map(self.right).dom().contains(key));
        } else {
            Self::insert_into_optional(&mut self.right, key, value);
            assert(!Node::<V>::optional_as_map(self.left).dom().contains(key));
        }
    }

    pub fn delete_from_optional(node: &mut Option<Box<Node<V>>>, key: u64)
        requires
            old(node).is_some() ==> old(node).unwrap().well_formed()
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(key)
    {
        if node.is_some() {
            let mut tmp = None;
            std::mem::swap(&mut tmp, node);
            let mut boxed_node = tmp.unwrap();

            if key == boxed_node.key {
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));

                if boxed_node.left.is_none() {
                    *node = boxed_node.right;
                } else {
                    if boxed_node.right.is_none() {
                        *node = boxed_node.left;
                    } else {
                        let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.left);
                        boxed_node.key = popped_key;
                        boxed_node.value = popped_value;
                        *node = Some(boxed_node);
                    }
                }
            } else if key < boxed_node.key {
                assert(!Node::<V>::optional_as_map(boxed_node.right).dom().contains(key));
                Node::<V>::delete_from_optional(&mut boxed_node.left, key);
                *node = Some(boxed_node);
            } else {
                assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(key));
                Node::<V>::delete_from_optional(&mut boxed_node.right, key);
                *node = Some(boxed_node);
            }
        }
    }

    pub fn delete_rightmost(node: &mut Option<Box<Node<V>>>) -> (popped: (u64, V))
        requires
            old(node).is_some(),
            old(node).unwrap().well_formed()
        ensures
            node.is_some() ==> node.unwrap().well_formed(),
            Node::<V>::optional_as_map(*node) =~= Node::<V>::optional_as_map(*old(node)).remove(popped.0),
            Node::<V>::optional_as_map(*old(node)).dom().contains(popped.0),
            Node::<V>::optional_as_map(*old(node))[popped.0] == popped.1,
            forall|k: u64| Node::<V>::optional_as_map(*old(node)).dom().contains(k) ==> popped.0 >= k
    {
        let mut tmp = None;
        std::mem::swap(&mut tmp, node);
        let mut boxed_node = tmp.unwrap();

        if boxed_node.right.is_none() {
            *node = boxed_node.left;
            assert(Node::<V>::optional_as_map(boxed_node.right) =~= Map::empty());
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(boxed_node.key));
            return (boxed_node.key, boxed_node.value);
        } else {
            let (popped_key, popped_value) = Node::<V>::delete_rightmost(&mut boxed_node.right);
            assert(!Node::<V>::optional_as_map(boxed_node.left).dom().contains(popped_key));
            *node = Some(boxed_node);
            return (popped_key, popped_value);
        }
    }

    pub fn get_from_optional(node: &Option<Box<Node<V>>>, key: u64) -> (ret: Option<&V>)
        requires
            node.is_some() ==> node.unwrap().well_formed()
        ensures
            ret == (if Node::<V>::optional_as_map(*node).dom().contains(key) {
                Some(&Node::<V>::optional_as_map(*node)[key])
            } else {
                None::<&V>
            })
    {
        match node {
            None => None,
            Some(node) => {
                node.get(key)
            }
        }
    }

    pub fn get(&self, key: u64) -> (ret: Option<&V>)
        requires
            self.well_formed()
        ensures
            ret == (if self.as_map().dom().contains(key) {
                Some(&self.as_map()[key])
            } else {
                None::<&V>
            })
    {
        if key == self.key {
            Some(&self.value)
        } else if key < self.key {
            proof { assert(!Node::<V>::optional_as_map(self.right).dom().contains(key)); }
            Self::get_from_optional(&self.left, key)
        } else {
            proof { assert(!Node::<V>::optional_as_map(self.left).dom().contains(key)); }
            Self::get_from_optional(&self.right, key)
        }
    }
}

fn test_node(v: u64)
requires
    v < u64::MAX - 10
{
    let mut root: Option<Box<Node<bool>>> = None;

    Node::insert_into_optional(&mut root, v, false);
    Node::insert_into_optional(&mut root, v + 1, false);
    Node::insert_into_optional(&mut root, v, true);

    let val1 = Node::get_from_optional(&root, v);
    let val2 = Node::get_from_optional(&root, v + 1);

    Node::delete_from_optional(&mut root, v);

    let val3 = Node::get_from_optional(&root, v);
    let val4 = Node::get_from_optional(&root, v + 1);
}

fn main() { }
}```
