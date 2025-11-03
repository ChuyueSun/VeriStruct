pub fn load(&self) -> $value_ty
    // ========== INFERRED SPECIFICATIONS ==========
    requires self.well_formed(),
    ensures self.well_formed(),
    // =============================================
{
    // code here
}
