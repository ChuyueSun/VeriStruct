pub enum MyOption<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    // TODO: add specification
}
