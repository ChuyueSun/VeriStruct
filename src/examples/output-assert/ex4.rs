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
