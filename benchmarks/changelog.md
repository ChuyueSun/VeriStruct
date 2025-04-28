### Changes

- Agreement: add typeinv version, add test.
- basic_lock1: add typeinv version, add test.
- basic_lock2: add test
- option: add test, remove the deprecated `[is_variant]` header.
- count_to_two: fix syntax errors, remove comments.
- doubly_linked: remove comment
- doubly_xor_linked: remove comment
- even_cell: remove comment, fix TODO
- frac: remove comments
- invariants: looks good
- oneshot: remove comment, add TODO
- log: add TODO
- monotonic_counters: add TODO
- rb_type_invariant: looks good
- rfmig_script: remove irrelevant part, remove comment
- rwlock_vstd: remove irrelevant part
- set_from_vec: looks good

### Problems

- basic_lock2: fail to add type invaraint version (fail to unfold `structs with invaraints`).
- api_server_state: no ground truth
- common: no ground truth
- frac: cannot add type invaraint (`const params not supported yet`, there is a constant param in the type definition, such types cannot establish type invaraint)
- io: compile not succesful (`some libs about kubernetes cannot be imported`, shall we install these libs manually)
- specexec: wierd example, seems used for tutorial.

### TODO

- fix basic_lock2
- add typeinv version for `doubly_linked` and `doubly_linked_xor`
- fix specexec
- `statics.rs`, `vectors.rs` and `vectors_vec.rs` not checked.
