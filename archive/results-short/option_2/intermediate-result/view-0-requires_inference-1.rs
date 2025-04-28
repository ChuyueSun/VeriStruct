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

impl<A: Copy> Copy for MyOption<A> { }

impl<A> MyOption<A> {
    pub open spec fn Or(self, optb: MyOption<A>) -> MyOption<A> {
        // TODO: add specification
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        requires
            true,
        ensures
            if old(self).is_some() { res == old(self) } else { res == optb },
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
            res == is_Some(self@),
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
            res == is_None(self@),
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
            if is_Some(old(self)@) {
                is_Some(a@)
            } else {
                is_None(a@)
            },
    {
        match self {
            MyOption::Some(x) => MyOption::Some(x),
            MyOption::None => MyOption::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        requires
            is_Some(self@),
        ensures
            a == get_Some_0(old(self)@),
    {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
        requires
            is_Some(self@),
        ensures
            a == get_Some_0(old(self)@),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 5
// VerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scope
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":951,"byte_end":954,"line_start":45,"line_end":45,"column_start":16,"column_end":19,"is_primary":true,"text":[{"text":"            if old(self).is_some() { res == old(self) } else { res == optb },","highlight_start":16,"highlight_end":19}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":951,"byte_end":960,"line_start":45,"line_end":45,"column_start":16,"column_end":25,"is_primary":true,"text":[{"text":"            if old(self).is_some() { res == old(self) } else { res == optb },","highlight_start":16,"highlight_end":25}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng:45:16\n   |\n45 | ...   if old(self).is_some() { res == old(self) } else { res == optb },\n   |          ^^^\n   |\nhelp: try calling `old` as a method\n   |\n45 |             if self.old().is_some() { res == old(self) } else { res == optb },\n   |                ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":980,"byte_end":983,"line_start":45,"line_end":45,"column_start":45,"column_end":48,"is_primary":true,"text":[{"text":"            if old(self).is_some() { res == old(self) } else { res == optb },","highlight_start":45,"highlight_end":48}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":980,"byte_end":989,"line_start":45,"line_end":45,"column_start":45,"column_end":54,"is_primary":true,"text":[{"text":"            if old(self).is_some() { res == old(self) } else { res == optb },","highlight_start":45,"highlight_end":54}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng:45:45\n   |\n45 | ...   if old(self).is_some() { res == old(self) } else { res == optb },\n   |                                       ^^^\n   |\nhelp: try calling `old` as a method\n   |\n45 |             if old(self).is_some() { res == self.old() } else { res == optb },\n   |                                             ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":1803,"byte_end":1806,"line_start":83,"line_end":83,"column_start":24,"column_end":27,"is_primary":true,"text":[{"text":"            if is_Some(old(self)@) {","highlight_start":24,"highlight_end":27}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":1803,"byte_end":1812,"line_start":83,"line_end":83,"column_start":24,"column_end":33,"is_primary":true,"text":[{"text":"            if is_Some(old(self)@) {","highlight_start":24,"highlight_end":33}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng:83:24\n   |\n83 |             if is_Some(old(self)@) {\n   |                        ^^^\n   |\nhelp: try calling `old` as a method\n   |\n83 |             if is_Some(self.old()@) {\n   |                        ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":2174,"byte_end":2177,"line_start":99,"line_end":99,"column_start":29,"column_end":32,"is_primary":true,"text":[{"text":"            a == get_Some_0(old(self)@),","highlight_start":29,"highlight_end":32}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":2174,"byte_end":2183,"line_start":99,"line_end":99,"column_start":29,"column_end":38,"is_primary":true,"text":[{"text":"            a == get_Some_0(old(self)@),","highlight_start":29,"highlight_end":38}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng:99:29\n   |\n99 |             a == get_Some_0(old(self)@),\n   |                             ^^^\n   |\nhelp: try calling `old` as a method\n   |\n99 |             a == get_Some_0(self.old()@),\n   |                             ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":2463,"byte_end":2466,"line_start":111,"line_end":111,"column_start":29,"column_end":32,"is_primary":true,"text":[{"text":"            a == get_Some_0(old(self)@),","highlight_start":29,"highlight_end":32}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":2463,"byte_end":2472,"line_start":111,"line_end":111,"column_start":29,"column_end":38,"is_primary":true,"text":[{"text":"            a == get_Some_0(old(self)@),","highlight_start":29,"highlight_end":38}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkf9qn9ng:111:29\n    |\n111 |             a == get_Some_0(old(self)@),\n    |                             ^^^\n    |\nhelp: try calling `old` as a method\n    |\n111 |             a == get_Some_0(self.old()@),\n    |                             ~~~~~~~~~~\nhelp: consider importing one of these functions\n    |\n1   + use builtin::old;\n    |\n1   + use vstd::prelude::old;\n    |\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 5 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 5 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0425`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0425`.\n"}
//
//
