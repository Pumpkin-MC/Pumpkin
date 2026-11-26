use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_protocol::ser::WritingError;
use pumpkin_world::chunk::format::LightContainer;
use std::io::Write;

/// Writes an NBT compound tag in the `CURRENT_MC_VERSION` format.
pub fn write_compound_nbt(
    mut write: impl Write,
    comp: &pumpkin_nbt::compound::NbtCompound,
) -> Result<(), WritingError> {
    write.write_compound_nbt(comp)
}

/// Retrieves the 2048-byte nibble array from a light container, filling with `default_val` if empty.
#[must_use]
pub fn get_light_bytes(container: Option<&LightContainer>, default_val: u8) -> [u8; 2048] {
    let mut buf = [default_val << 4 | default_val; 2048];
    if let Some(LightContainer::Full(data)) = container
        && data.len() == 2048
    {
        buf.copy_from_slice(data);
    }
    buf
}

/// Bit-packs entries without spanning across 64-bit boundaries (Minecraft 1.16+ format).
#[must_use]
pub fn pack_modern_data(entries: &[u32], bits_per_entry: usize) -> Vec<i64> {
    if bits_per_entry == 0 {
        return Vec::new();
    }
    let values_per_i64 = 64 / bits_per_entry;
    let long_count = entries.len().div_ceil(values_per_i64);
    let mut data = Vec::with_capacity(long_count);
    let mut current_idx = 0;
    while current_idx < entries.len() {
        let mut acc = 0u64;
        for i in 0..values_per_i64 {
            if current_idx + i < entries.len() {
                let value = entries[current_idx + i] as u64;
                acc |= value << (bits_per_entry * i);
            }
        }
        data.push(acc as i64);
        current_idx += values_per_i64;
    }
    data
}
