use crate::WritingError;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::format::LightContainer;
use std::io::Write;

use crate::ser::NetworkWriteExt;

/// Writes an NBT compound tag to the writer, formatted appropriately for the target Minecraft version.
pub fn write_compound_nbt(
    mut write: impl Write,
    comp: &pumpkin_nbt::compound::NbtCompound,
    version: JavaMinecraftVersion,
) -> Result<(), WritingError> {
    write.write_compound_nbt_with_version(Some(comp), &version)
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

/// Bit-packs entries across 64-bit boundaries (Minecraft 1.9 to 1.15.2 legacy format).
#[must_use]
pub fn pack_legacy_data(entries: &[u32], bits_per_entry: usize) -> Vec<i64> {
    let bpe = bits_per_entry;
    if bpe == 0 || entries.is_empty() {
        return Vec::new();
    }
    let total_bits = entries.len() * bpe;
    let long_count = total_bits.div_ceil(64);
    let mut data = vec![0u64; long_count];
    let max_entry_value = if bpe >= 64 {
        u64::MAX
    } else {
        (1u64 << bpe) - 1
    };

    for (index, &value) in entries.iter().enumerate() {
        let val = (value as u64) & max_entry_value;
        let bit_index = index * bpe;
        let start_index = bit_index / 64;
        let end_index = ((index + 1) * bpe - 1) / 64;
        let start_bit_sub_index = bit_index % 64;

        data[start_index] = (data[start_index]
            & !(max_entry_value.wrapping_shl(start_bit_sub_index as u32)))
            | val.wrapping_shl(start_bit_sub_index as u32);
        if start_index != end_index {
            let end_bit_sub_index = 64 - start_bit_sub_index;
            let j1 = bpe - end_bit_sub_index;
            data[end_index] = ((data[end_index] >> j1) << j1) | (val >> end_bit_sub_index);
        }
    }

    data.into_iter().map(|w| w as i64).collect()
}
