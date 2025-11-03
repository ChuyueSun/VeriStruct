pub fn insert(&mut self, v: u64)
ensures
    self@ =~= old(self)@.insert(v),
{
    self.vt.push(v);
    // ========== INFERRED PROOF ==========
    proof {
        broadcast use group_seq_properties;
        assert(self.vt@ =~= old(self).vt@ + seq![v]);
    }
    // ====================================
}
