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


}
```


}
```


}
```


}
```


}
```


}
```
