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

}

}

}

}

}

}

}

}

}

}

}

}
