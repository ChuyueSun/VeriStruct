pub fn insert(&mut self, v: u64)
ensures
    self@ =~= old(self)@.insert(v),
{
self.vt.push(v);
// TODO: add proof

}