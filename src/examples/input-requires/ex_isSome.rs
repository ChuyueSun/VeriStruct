pub fn as_ref(&self) -> (a: MyOption<&A>)
// TODO: add requires and ensures
{
match self {
    MyOption::Some(x) => MyOption::Some(x),
    MyOption::None => MyOption::None,
}
}
