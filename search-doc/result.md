vstd::pcm

# Struct Resource

pub struct Resource<P> { /* private fields */ }

Expand description

Interface for ghost state that is consistent with the common presentations of
partially commutative monoids (PCMs) / resource algebras.

For applications, the general advice is to use the `tokenized_state_machine!`
system, which lets you
focus on updates and invariants rather than composition.

However, the PCM interface you’ll find here may be more familiar to people.

## Implementations

### impl<P: PCM>
Resource<P>

#### pub uninterp fn value(self) -> P

#### pub uninterp fn loc(self) -> Loc

#### pub proof fn alloc(value: P) -> tracked out : Self

requires

value.valid(),

ensures

out.value() == value,

#### pub proof fn join(tracked self, tracked other: Self) -> tracked out :
Self

requires

self.loc() == other.loc(),

ensures

out.loc() == self.loc(),

out.value() == P::op(self.value(), other.value()),

#### pub proof fn split(tracked self, left: P, right: P) -> tracked out :
(Self, Self)

requires

self.value() == P::op(left, right),

ensures

out.0.loc() == self.loc(),

out.1.loc() == self.loc(),

out.0.value() == left,

out.1.value() == right,

#### pub proof fn create_unit(loc: Loc)
-> tracked out : Self

ensures

out.value() == P::unit(),

out.loc() == loc,

#### pub proof fn validate(tracked &self)

ensures

self.value().valid(),

#### pub proof fn update(tracked self, new_value: P) -> tracked out : Self

requires

frame_preserving_update(self.value(), new_value),

ensures

out.loc() == self.loc(),

out.value() == new_value,

#### pub proof fn update_nondeterministic(tracked self, new_values:
Set<P>) -> tracked out :
Self

requires

frame_preserving_update_nondeterministic(self.value(), new_values),

ensures

out.loc() == self.loc(),

new_values.contains(out.value()),

#### pub proof fn join_shared<'a>(tracked &'a self, tracked other: &'a Self)
-> tracked out : &'a Self

requires

self.loc() == other.loc(),

ensures

out.loc() == self.loc(),

incl(self.value(), out.value()),

incl(other.value(), out.value()),

#### pub proof fn join_shared_to_target<'a>(tracked  &'a self, tracked  other:
&'a Self, target: P, ) -> tracked out : &'a Self

requires

self.loc() == other.loc(),

conjunct_shared(self.value(), other.value(), target),

ensures

out.loc() == self.loc(),

out.value() == target,

#### pub proof fn weaken<'a>(tracked &'a self, target: P) -> tracked out : &'a
Self

requires

incl(target, self.value()),

ensures

out.loc() == self.loc(),

out.value() == target,

#### pub proof fn validate_2(tracked &mut self, tracked other: &Self)

requires

old(self).loc() == other.loc(),

ensures

*self == *old(self),

P::op(self.value(), other.value()).valid(),

#### pub proof fn update_with_shared(tracked self, tracked other: &Self,
new_value: P) -> tracked out : Self

requires

self.loc() == other.loc(),

frame_preserving_update(
P::op(self.value(), other.value()),
P::op(new_value, other.value()),
),

ensures

out.loc() == self.loc(),

out.value() == new_value,

#### pub proof fn update_nondeterministic_with_shared(tracked  self, tracked
other: &Self, new_values: Set<P>, ) -> tracked out : Self

requires

self.loc() == other.loc(),

frame_preserving_update_nondeterministic(
P::op(self.value(), other.value()),
set_op(new_values, other.value()),
),

ensures

out.loc() == self.loc(),

new_values.contains(out.value()),

## Auto Trait Implementations

### impl<P> Freeze
for Resource<P>

### impl<P> RefUnwindSafe for Resource<P>

where P: RefUnwindSafe,

### impl<P> Send for
Resource<P>

where P: Send,

### impl<P> Sync for
Resource<P>

where P: Sync,

### impl<P> Unpin for
Resource<P>

where P: Unpin,

### impl<P> UnwindSafe for Resource<P>

where P: UnwindSafe,

## Blanket Implementations

### impl<T> Any for T

where T: 'static + ?Sized,

#### fn type_id(&self) ->
TypeId

Gets the `TypeId` of `self`. Read more

### impl<T> Borrow<T>
for T

where T: ?Sized,

#### fn borrow(&self) ->
&T

Immutably borrows from an owned value. Read more

### impl<T> BorrowMut<T> for T

where T: ?Sized,

#### fn borrow_mut(&mut
self) -> &mut T

Mutably borrows from an owned value. Read more

### impl<T> From<T>
for T

#### fn from(t: T) -> T

Returns the argument unchanged.

### impl<T, U> Into<U>
for T

where U: From<T>,

#### fn into(self) -> U

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
`From<T> for U` chooses to do.

### impl<T, U> TryFrom<U> for T

where U: Into<T>,

#### type Error =
Infallible

The type returned in the event of a conversion error.

#### fn try_from(value: U)
-> Result<T, <T as TryFrom<U>>::Error>

Performs the conversion.

### impl<T, U> TryInto<U> for T

where U: TryFrom<T>,

#### type Error = <U as
TryFrom<T>>::Error

The type returned in the event of a conversion error.

#### fn try_into(self) ->
Result<U, <U as TryFrom<T>>::Error>

Performs the conversion.


