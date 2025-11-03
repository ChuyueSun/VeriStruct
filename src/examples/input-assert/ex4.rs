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
