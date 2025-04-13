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
    match opt {
        MyOption::Some(_) => true,
        MyOption::None => false,
    }
}

pub open spec fn is_None<A>(opt: MyOption<A>) -> bool {
    match opt {
        MyOption::Some(_) => false,
        MyOption::None => true,
    }
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A {
    match opt {
        MyOption::Some(a) => a,
        MyOption::None => arbitrary(),
    }
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

pub open spec fn Or<A>(opt1: MyOption<A>, opt2: MyOption<A>) -> MyOption<A> {
    match opt1 {
        MyOption::Some(_) => opt1,
        MyOption::None => opt2,
    }
}

impl<A> MyOption<A> {
    pub open spec fn Or(self, optb: MyOption<A>) -> MyOption<A> {
        Or(self, optb)
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        ensures
            if is_Some::<A>(old(self)) {
                res == old(self)
            } else {
                res == optb
            },
    {
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
        ensures
            res == is_Some::<A>(*self),
    {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        ensures
            res == is_None::<A>(*self),
    {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: MyOption<&A>)
        ensures
            is_Some::<A>(*old(self)) ==> (
                is_Some::<&A>(a)
                && get_Some_0::<&A>(a) == &get_Some_0::<A>(*old(self))
            ),
            is_None::<A>(*old(self)) ==> is_None::<&A>(a),
    {
        match self {
            MyOption::Some(x) => MyOption::Some(x),
            MyOption::None => MyOption::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        requires
            is_Some::<A>(old(self)),
        ensures
            a == get_Some_0::<A>(old(self)),
    {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
        requires
            is_Some::<A>(old(self)),
        ensures
            a == get_Some_0::<A>(old(self)),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 9
// VerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scopeVerusErrorType.Other: cannot find function `old` in this scope
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":1268,"byte_end":1271,"line_start":60,"line_end":60,"column_start":29,"column_end":32,"is_primary":true,"text":[{"text":"            if is_Some::<A>(old(self)) {","highlight_start":29,"highlight_end":32}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":1268,"byte_end":1277,"line_start":60,"line_end":60,"column_start":29,"column_end":38,"is_primary":true,"text":[{"text":"            if is_Some::<A>(old(self)) {","highlight_start":29,"highlight_end":38}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:60:29\n   |\n60 |             if is_Some::<A>(old(self)) {\n   |                             ^^^\n   |\nhelp: try calling `old` as a method\n   |\n60 |             if is_Some::<A>(self.old()) {\n   |                             ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":1304,"byte_end":1307,"line_start":61,"line_end":61,"column_start":24,"column_end":27,"is_primary":true,"text":[{"text":"                res == old(self)","highlight_start":24,"highlight_end":27}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":1304,"byte_end":1313,"line_start":61,"line_end":61,"column_start":24,"column_end":33,"is_primary":true,"text":[{"text":"                res == old(self)","highlight_start":24,"highlight_end":33}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:61:24\n   |\n61 |                 res == old(self)\n   |                        ^^^\n   |\nhelp: try calling `old` as a method\n   |\n61 |                 res == self.old()\n   |                        ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2075,"byte_end":2078,"line_start":96,"line_end":96,"column_start":27,"column_end":30,"is_primary":true,"text":[{"text":"            is_Some::<A>(*old(self)) ==> (","highlight_start":27,"highlight_end":30}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2075,"byte_end":2084,"line_start":96,"line_end":96,"column_start":27,"column_end":36,"is_primary":true,"text":[{"text":"            is_Some::<A>(*old(self)) ==> (","highlight_start":27,"highlight_end":36}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:96:27\n   |\n96 |             is_Some::<A>(*old(self)) ==> (\n   |                           ^^^\n   |\nhelp: try calling `old` as a method\n   |\n96 |             is_Some::<A>(*self.old()) ==> (\n   |                           ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2185,"byte_end":2188,"line_start":98,"line_end":98,"column_start":61,"column_end":64,"is_primary":true,"text":[{"text":"                && get_Some_0::<&A>(a) == &get_Some_0::<A>(*old(self))","highlight_start":61,"highlight_end":64}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2185,"byte_end":2194,"line_start":98,"line_end":98,"column_start":61,"column_end":70,"is_primary":true,"text":[{"text":"                && get_Some_0::<&A>(a) == &get_Some_0::<A>(*old(self))","highlight_start":61,"highlight_end":70}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:98:61\n   |\n98 |                 && get_Some_0::<&A>(a) == &get_Some_0::<A>(*old(self))\n   |                                                             ^^^\n   |\nhelp: try calling `old` as a method\n   |\n98 |                 && get_Some_0::<&A>(a) == &get_Some_0::<A>(*self.old())\n   |                                                             ~~~~~~~~~~\nhelp: consider importing one of these functions\n   |\n1  + use builtin::old;\n   |\n1  + use vstd::prelude::old;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2237,"byte_end":2240,"line_start":100,"line_end":100,"column_start":27,"column_end":30,"is_primary":true,"text":[{"text":"            is_None::<A>(*old(self)) ==> is_None::<&A>(a),","highlight_start":27,"highlight_end":30}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2237,"byte_end":2246,"line_start":100,"line_end":100,"column_start":27,"column_end":36,"is_primary":true,"text":[{"text":"            is_None::<A>(*old(self)) ==> is_None::<&A>(a),","highlight_start":27,"highlight_end":36}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:100:27\n    |\n100 |             is_None::<A>(*old(self)) ==> is_None::<&A>(a),\n    |                           ^^^\n    |\nhelp: try calling `old` as a method\n    |\n100 |             is_None::<A>(*self.old()) ==> is_None::<&A>(a),\n    |                           ~~~~~~~~~~\nhelp: consider importing one of these functions\n    |\n1   + use builtin::old;\n    |\n1   + use vstd::prelude::old;\n    |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2488,"byte_end":2491,"line_start":110,"line_end":110,"column_start":26,"column_end":29,"is_primary":true,"text":[{"text":"            is_Some::<A>(old(self)),","highlight_start":26,"highlight_end":29}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2488,"byte_end":2497,"line_start":110,"line_end":110,"column_start":26,"column_end":35,"is_primary":true,"text":[{"text":"            is_Some::<A>(old(self)),","highlight_start":26,"highlight_end":35}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:110:26\n    |\n110 |             is_Some::<A>(old(self)),\n    |                          ^^^\n    |\nhelp: try calling `old` as a method\n    |\n110 |             is_Some::<A>(self.old()),\n    |                          ~~~~~~~~~~\nhelp: consider importing one of these functions\n    |\n1   + use builtin::old;\n    |\n1   + use vstd::prelude::old;\n    |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2549,"byte_end":2552,"line_start":112,"line_end":112,"column_start":34,"column_end":37,"is_primary":true,"text":[{"text":"            a == get_Some_0::<A>(old(self)),","highlight_start":34,"highlight_end":37}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2549,"byte_end":2558,"line_start":112,"line_end":112,"column_start":34,"column_end":43,"is_primary":true,"text":[{"text":"            a == get_Some_0::<A>(old(self)),","highlight_start":34,"highlight_end":43}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:112:34\n    |\n112 |             a == get_Some_0::<A>(old(self)),\n    |                                  ^^^\n    |\nhelp: try calling `old` as a method\n    |\n112 |             a == get_Some_0::<A>(self.old()),\n    |                                  ~~~~~~~~~~\nhelp: consider importing one of these functions\n    |\n1   + use builtin::old;\n    |\n1   + use vstd::prelude::old;\n    |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2790,"byte_end":2793,"line_start":122,"line_end":122,"column_start":26,"column_end":29,"is_primary":true,"text":[{"text":"            is_Some::<A>(old(self)),","highlight_start":26,"highlight_end":29}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2790,"byte_end":2799,"line_start":122,"line_end":122,"column_start":26,"column_end":35,"is_primary":true,"text":[{"text":"            is_Some::<A>(old(self)),","highlight_start":26,"highlight_end":35}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:122:26\n    |\n122 |             is_Some::<A>(old(self)),\n    |                          ^^^\n    |\nhelp: try calling `old` as a method\n    |\n122 |             is_Some::<A>(self.old()),\n    |                          ~~~~~~~~~~\nhelp: consider importing one of these functions\n    |\n1   + use builtin::old;\n    |\n1   + use vstd::prelude::old;\n    |\n\n"}
// {"$message_type":"diagnostic","message":"cannot find function `old` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2851,"byte_end":2854,"line_start":124,"line_end":124,"column_start":34,"column_end":37,"is_primary":true,"text":[{"text":"            a == get_Some_0::<A>(old(self)),","highlight_start":34,"highlight_end":37}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"try calling `old` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":2851,"byte_end":2860,"line_start":124,"line_end":124,"column_start":34,"column_end":43,"is_primary":true,"text":[{"text":"            a == get_Some_0::<A>(old(self)),","highlight_start":34,"highlight_end":43}],"label":null,"suggested_replacement":"self.old()","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null},{"message":"consider importing one of these functions","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use builtin::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::old;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function `old` in this scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfete4gj:124:34\n    |\n124 |             a == get_Some_0::<A>(old(self)),\n    |                                  ^^^\n    |\nhelp: try calling `old` as a method\n    |\n124 |             a == get_Some_0::<A>(self.old()),\n    |                                  ~~~~~~~~~~\nhelp: consider importing one of these functions\n    |\n1   + use builtin::old;\n    |\n1   + use vstd::prelude::old;\n    |\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 9 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 9 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0425`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0425`.\n"}
// 
// 