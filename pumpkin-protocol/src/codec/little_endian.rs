pub struct Le64(pub i64);

impl Le64 {
    pub fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.0.to_le_bytes())
    }
}

pub struct Le32(pub i32);

impl Le32 {
    pub fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.0.to_le_bytes())
    }
}

pub struct Le16(pub i16);

impl Le16 {
    pub fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.0.to_le_bytes())
    }
}

pub struct LeU64(pub u64);

impl LeU64 {
    pub fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.0.to_le_bytes())
    }
}

pub struct LeU32(pub u32);

impl LeU32 {
    pub fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.0.to_le_bytes())
    }
}

pub struct LeU16(pub u16);

impl LeU16 {
    pub fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.0.to_le_bytes())
    }
}
