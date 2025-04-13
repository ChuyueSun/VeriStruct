
use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum Option<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: Option<A>) -> bool {
    matches!(opt, Option::Some(_))
}

pub open spec fn is_None<A>(opt: Option<A>) -> bool {
    matches!(opt, Option::None)
}

pub open spec fn get_Some_0<A>(opt: Option<A>) -> A
{
    match opt {
        Option::Some(a) => a,
        Option::None => arbitrary(),
    }
}


impl<A: Clone> Clone for Option<A> {
    fn clone(&self) -> Self {
        match self {
            Option::None => Option::None,
            Option::Some(a) => Option::Some(a.clone()),
        }
    }
}

impl<A: Copy> Copy for Option<A> {

}

impl<A> Option<A> {
    pub open spec fn or(self, optb: Option<A>) -> Option<A> {
        match self {
            Option::None => optb,
            Option::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
    {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
    {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: Option<&A>)
    {
        match self {
            Option::Some(x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
    {
        match self {
            Option::Some(a) => a,
            Option::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
    {
        match self {
            Option::Some(a) => a,
            Option::None => proof_from_false(),
        }
    }
}

} // verus!

fn main() {
}

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfied
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/pervasive.rs","byte_start":5502,"byte_end":5507,"line_start":179,"line_end":179,"column_start":9,"column_end":14,"is_primary":false,"text":[{"text":"        false,","highlight_start":9,"highlight_end":14}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpn5ge_9t_","byte_start":1602,"byte_end":1613,"line_start":82,"line_end":82,"column_start":29,"column_end":40,"is_primary":true,"text":[{"text":"            Option::None => unreached(),","highlight_start":29,"highlight_end":40}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpn5ge_9t_:82:29\n    |\n82  |             Option::None => unreached(),\n    |                             ^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/pervasive.rs:179:9\n    |\n179 |         false,\n    |         ----- failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/pervasive.rs","byte_start":5362,"byte_end":5367,"line_start":171,"line_end":171,"column_start":14,"column_end":19,"is_primary":false,"text":[{"text":"    requires(false);","highlight_start":14,"highlight_end":19}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpn5ge_9t_","byte_start":1785,"byte_end":1803,"line_start":90,"line_end":90,"column_start":29,"column_end":47,"is_primary":true,"text":[{"text":"            Option::None => proof_from_false(),","highlight_start":29,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpn5ge_9t_:90:29\n    |\n90  |             Option::None => proof_from_false(),\n    |                             ^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/pervasive.rs:171:14\n    |\n171 |     requires(false);\n    |              ----- failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 2 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 2 previous errors\n\n"}
// 
// 