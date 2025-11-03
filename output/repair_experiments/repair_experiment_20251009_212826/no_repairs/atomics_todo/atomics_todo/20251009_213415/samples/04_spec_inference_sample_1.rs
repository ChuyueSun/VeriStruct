#[verifier::exec_allows_no_decreases_clause]
fn take<T>(lock: &Lock<T>) -> (t: Tracked<T>)
    requires
        lock.well_formed(),
    ensures
        lock.well_formed(),
        lock.field@ == None::<T>,
{
    loop
        // TODO: add loop invariant
    {
        let tracked ghost_value: Option<T>;
        let result =
            atomic_with_ghost!(
            &lock.field => compare_exchange(true, false);
            update prev -> next;
            ghost g => {
                if prev == true {
                    ghost_value = g;
                    g = Option::None;
                } else {
                    ghost_value = Option::None;
                }
            }
        );
        if let Result::Ok(_) = result {
            return Tracked(
                match ghost_value {
                    Option::Some(s) => s,
                    _ => { proof_from_false() },
                },
            );
        }
    }
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
