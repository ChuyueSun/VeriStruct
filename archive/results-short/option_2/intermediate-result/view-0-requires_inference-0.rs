use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum MyOption<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    // TODO: add specification
}

pub open spec fn is_None<A>(opt: MyOption<A>) -> bool {
    // TODO: add specification
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A {
    // TODO: add specification
}

impl<A: Clone> Clone for MyOption<A> {
    fn clone(&self) -> Self {
        match self {
            MyOption::None => MyOption::None,
            MyOption::Some(a) => MyOption::Some(a.clone()),
        }
    }
}

impl<A: Copy> Copy for MyOption<A> {
}

impl<A> MyOption<A> {
    pub open spec fn Or(self, optb: MyOption<A>) -> MyOption<A> {
        // TODO: add specification
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        requires
            true,
        ensures
            (old(self) != MyOption::None) ==> (res == old(self)),
            (old(self) == MyOption::None) ==> (res == optb),
    {
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
        requires
            true,
        ensures
            res == (self != MyOption::None),
    {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        requires
            true,
        ensures
            res == (self == MyOption::None),
    {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: MyOption<&A>)
        requires
            true,
        ensures
            (self == MyOption::None) ==> (a == MyOption::None),
            (self != MyOption::None) ==> (a != MyOption::None),
    {
        match self {
            MyOption::Some(x) => MyOption::Some(x),
            MyOption::None => MyOption::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        requires
            self != MyOption::None,
        ensures
            self == MyOption::Some(a),
    {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
        requires
            self != MyOption::None,
        ensures
            self == MyOption::Some(a),
    {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => proof_from_false(),
        }
    }
}

fn test_option_generic() {
    let opt: MyOption<i32> = MyOption::None;
    let is_none = opt.is_none();
    let is_some = opt.is_some();
    assert(is_none);
    assert(!is_some);

    let opt2: MyOption<i32> = MyOption::Some(42);
    let is_none = opt2.is_none();
    let is_some = opt2.is_some();
    assert(!is_none);
    assert(is_some);

    let opt3 = opt.or(opt2);
    let is_some = opt3.is_some();
    let val = opt3.unwrap();
    assert(is_some);
    assert(val == 42);

    let opt4 = opt2.or(opt);
    let is_some = opt4.is_some();
    let val = opt4.unwrap();
    assert(is_some);
    assert(val == 42);

    let opt5 = opt.or(MyOption::None);
    let is_none = opt5.is_none();
    let is_some = opt5.is_some();
    assert(is_none);
    assert(!is_some);
}

} // verus!

fn main() {
}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
// VerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scope
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":949,"byte_end":952,"line_start":46,"line_end":46,"column_start":14,"column_end":17,"is_primary":true,"text":[{"text":"            (old(self) != MyOption::None) ==> (res == old(self)),","highlight_start":14,"highlight_end":17}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":949,"byte_end":958,"line_start":46,"line_end":46,"column_start":14,"column_end":23,"is_primary":true,"text":[{"text":"            (old(self) != MyOption::None) ==> (res == old(self)),","highlight_start":14,"highlight_end":23}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj:46:14\n   |\n46 |             (old(self) != MyOption::None) ==> (res == old(self)),\n   |              ^^^\n   |\nhelp: try calling `old` as a method\n   |\n46 |             (self.old() != MyOption::None) ==> (res == old(self)),\n   |              ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":990,"byte_end":993,"line_start":46,"line_end":46,"column_start":55,"column_end":58,"is_primary":true,"text":[{"text":"            (old(self) != MyOption::None) ==> (res == old(self)),","highlight_start":55,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":990,"byte_end":999,"line_start":46,"line_end":46,"column_start":55,"column_end":64,"is_primary":true,"text":[{"text":"            (old(self) != MyOption::None) ==> (res == old(self)),","highlight_start":55,"highlight_end":64}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj:46:55\n   |\n46 |             (old(self) != MyOption::None) ==> (res == old(self)),\n   |                                                       ^^^\n   |\nhelp: try calling `old` as a method\n   |\n46 |             (old(self) != MyOption::None) ==> (res == self.old()),\n   |                                                       ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":1015,"byte_end":1018,"line_start":47,"line_end":47,"column_start":14,"column_end":17,"is_primary":true,"text":[{"text":"            (old(self) == MyOption::None) ==> (res == optb),","highlight_start":14,"highlight_end":17}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":1015,"byte_end":1024,"line_start":47,"line_end":47,"column_start":14,"column_end":23,"is_primary":true,"text":[{"text":"            (old(self) == MyOption::None) ==> (res == optb),","highlight_start":14,"highlight_end":23}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpikzvqqpj:47:14\n   |\n47 |             (old(self) == MyOption::None) ==> (res == optb),\n   |              ^^^\n   |\nhelp: try calling `old` as a method\n   |\n47 |             (self.old() == MyOption::None) ==> (res == optb),\n   |              ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0425`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0425`.\n"}
// 
// 