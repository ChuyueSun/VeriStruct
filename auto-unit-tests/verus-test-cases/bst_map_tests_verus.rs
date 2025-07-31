/* TEST CODE BELOW */
pub fn main() {
    // Test new: A newly created TreeMap should have an empty view.
    let tm_new: TreeMap<u64> = TreeMap::new();
    assert(tm_new@ == Map::<u64, u64>::empty());

    // Test insert: Inserting a (key, value) pair should update the view accordingly.
    let mut tm_ins: TreeMap<u64> = TreeMap::new();
    tm_ins.insert(10, 100);
    assert(tm_ins@ == Map::<u64, u64>::empty().insert(10, 100));

    // Test insert update: Inserting a key that already exists should update its value.
    let mut tm_update: TreeMap<u64> = TreeMap::new();
    tm_update.insert(40, 400);
    tm_update.insert(40, 450);
    assert(tm_update@ == Map::<u64, u64>::empty().insert(40, 450));

    // Test delete: Delete one key from a tree with several keys.
    let mut tm_del: TreeMap<u64> = TreeMap::new();
    tm_del.insert(20, 200);
    tm_del.insert(30, 300);
    tm_del.insert(25, 250);
    tm_del.delete(30);
    assert(tm_del@ == Map::<u64, u64>::empty().insert(20, 200).insert(25, 250));

    // Test get: Inserting a key and then retrieving it should yield the correct value.
    let mut tm_get: TreeMap<u64> = TreeMap::new();
    tm_get.insert(15, 150);
    // Since tm_get.get returns an Option<&u64>, check that it is Some and the value is correct.
    if let Some(val) = tm_get.get(15) {
        assert(*val == 150);
    } else {
        assert(false);
    }

    // Test get: Retrieving a key that was never inserted should return None.
    assert(tm_get.get(99).is_none());
}