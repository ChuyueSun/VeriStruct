pub fn as_ref(&self) -> (a: MyOption<&A>)
ensures
    is_Some(a) <==> is_Some(*self),
    is_Some(a) ==> get_Some_0(*self) == get_Some_0(a),
{
match self {
    MyOption::Some(x) => MyOption::Some(x),
    MyOption::None => MyOption::None,
}
}