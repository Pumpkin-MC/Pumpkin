pub struct OptionalInt(pub Option<i32>);

impl OptionalInt {
    pub fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        let val = self.0.map_or(0, |id| id + 1);
        write.write_var_int(&crate::codec::var_int::VarInt(val))
    }
}
