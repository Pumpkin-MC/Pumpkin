use bytes::BufMut;

use crate::{bytebuf::ByteBufMut, codec::var_int::VarInt};

use super::Codec;

pub type BlockStatePalettedContainer = PalettedContainer<16, 4, 8>;
pub type BiomePalettedContainer = PalettedContainer<4, 1, 3>;

/// Note: These do not have to be u16's, but currently we do not have any
/// block state or biome registry id's that are greater than u16::MAX.
pub type RegistryIdType = u16;

/// An abstract cube of `RegistryIdType` values with a length of `AXIS`. Indexed with [y][z][x].
type RegistryIdCube<const AXIS: usize> = [[[RegistryIdType; AXIS]; AXIS]; AXIS];

/// The notchian maximum bits-per-entry for block state
///
/// TODO: Calculate this at runtime (based off of what is sent in the `Registry Data` packet)
pub const MAX_BPE_BLOCK_STATES: usize = 15;

/// The notchian maximum bits-per-entry for block state
///
/// TODO: Calculate this at runtime (based off of what is sent in the `Registry Data` packet)
pub const MAX_BPE_BIOMES: usize = 6;

/// A flat map representing an abstract cube. `ENTRIES_PER_AXIS` is the number of entries per side
/// of the cube. `LOWER_BOUND` is the minimum possible bits-per-entry for an indirect palette, and
/// `UPPER_BOUND` is the maximum possible bits-per-entry (inclusive) for an indirect palette. These
/// bounds are set by the notchian client, and using different values is undefined behavior.
pub struct PalettedContainer<
    const ENTRIES_PER_AXIS: usize,
    const LOWER_BOUND: usize,
    const UPPER_BOUND: usize,
> {
    /// The bits-per-entry to use if the threshold is above `UPPER_BOUND`. This is determined at
    /// runtime based on the total overall number of entries in the registry.
    maximum_bpe: usize,
    /// The raw data within this container; a cube of registry id's
    data: RegistryIdCube<ENTRIES_PER_AXIS>,
}

impl<const ENTRIES_PER_AXIS: usize, const LOWER_BOUND: usize, const UPPER_BOUND: usize>
    PalettedContainer<ENTRIES_PER_AXIS, LOWER_BOUND, UPPER_BOUND>
{
    /// Creates a new `PalettedContainer` from a slice of `RegistryIdType`. The length of the slice
    /// must be equal to `ENTRIES_PER_AXIS` cubed. `index_map` provides a mapping from the index in
    /// the input slice to the (x, y, z) position in the cube.
    pub fn from_raw_data<M>(raw_data: &[RegistryIdType], index_map: M, maximum_bpe: usize) -> Self
    where
        M: Fn(usize) -> (usize, usize, usize),
    {
        debug_assert!(raw_data.len() == ENTRIES_PER_AXIS * ENTRIES_PER_AXIS * ENTRIES_PER_AXIS);

        let mut data: RegistryIdCube<ENTRIES_PER_AXIS> =
            [[[0; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS];
        for (index, registry_key) in raw_data.iter().enumerate() {
            let (x, y, z) = index_map(index);
            debug_assert!(x < ENTRIES_PER_AXIS);
            debug_assert!(y < ENTRIES_PER_AXIS);
            debug_assert!(z < ENTRIES_PER_AXIS);

            data[y][z][x] = *registry_key;
        }

        Self { data, maximum_bpe }
    }

    /// Gets the registry id from the (x, y, z) position. x, y, and z must be less than
    /// `ENTRIES_PER_AXIS`.
    pub fn get_id(&self, x: usize, y: usize, z: usize) -> RegistryIdType {
        debug_assert!(x < ENTRIES_PER_AXIS);
        debug_assert!(y < ENTRIES_PER_AXIS);
        debug_assert!(z < ENTRIES_PER_AXIS);

        self.data[y][z][x]
    }

    /// Sets the registry id at the (x, y, z) position. x, y, and z must be less than
    /// `ENTRIES_PER_AXIS`.
    pub fn set_id(&mut self, x: usize, y: usize, z: usize, id: RegistryIdType) {
        debug_assert!(x < ENTRIES_PER_AXIS);
        debug_assert!(y < ENTRIES_PER_AXIS);
        debug_assert!(z < ENTRIES_PER_AXIS);

        self.data[y][z][x] = id;
    }

    fn write_registry_cube(&self, write: &mut impl BufMut) {
        let mut unique_registry_ids = self.data.as_flattened().as_flattened().to_vec();
        unique_registry_ids.sort_unstable();
        unique_registry_ids.dedup();

        let smallest_encompassing_power_of_2 = unique_registry_ids.len().next_power_of_two();
        let leading_zeros = if smallest_encompassing_power_of_2 == unique_registry_ids.len() {
            smallest_encompassing_power_of_2
        } else {
            smallest_encompassing_power_of_2 - 1
        };

        let raw_bpe = std::mem::size_of::<usize>() - leading_zeros.leading_zeros() as usize;

        let bpe_to_use = if raw_bpe == 0 {
            0
        } else if raw_bpe <= LOWER_BOUND {
            LOWER_BOUND
        } else if raw_bpe <= UPPER_BOUND {
            raw_bpe
        } else {
            self.maximum_bpe
        };

        if bpe_to_use == 0 {
            self.write_single_value_palette(write)
        } else if (LOWER_BOUND..=UPPER_BOUND).contains(&bpe_to_use) {
            self.write_indirect_palette(bpe_to_use, &unique_registry_ids, write)
        } else {
            self.write_direct_palette(bpe_to_use, write)
        }
    }

    fn read_registry_cube(
        read: &mut impl bytes::Buf,
    ) -> Result<RegistryIdCube<ENTRIES_PER_AXIS>, super::DecodeError> {
        let bpe = read.get_u8() as usize;
        if bpe == 0 {
            Self::read_single_value_palette(read)
        } else if bpe <= UPPER_BOUND {
            Self::read_indirect_palette(bpe, read)
        } else {
            Self::read_direct_palette(bpe, read)
        }
    }

    fn write_single_value_palette(&self, write: &mut impl BufMut) {
        write.put_u8(0);
        write.put_var_int(&self.data[0][0][0].into());
        write.put_u8(0);
    }

    fn read_single_value_palette(
        read: &mut impl bytes::Buf,
    ) -> Result<RegistryIdCube<ENTRIES_PER_AXIS>, super::DecodeError> {
        let value = VarInt::decode(read)?;
        let data: RegistryIdCube<ENTRIES_PER_AXIS> =
            [[[value.0 as u16; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS];

        let data_length = read.get_u8();
        if data_length != 0 {
            return Err(super::DecodeError::Malformed);
        }

        Ok(data)
    }

    fn write_indirect_palette(
        &self,
        bpe: usize,
        unique_registry_ids: &[RegistryIdType],
        write: &mut impl BufMut,
    ) {
        write.put_var_int(&unique_registry_ids.len().into());
        for id in unique_registry_ids {
            write.put_var_int(&*id.into());
        }

        let values_per_long = 64 / bpe;
        let value_mask = (1u64 << bpe) - 1;
        let long_array_length =
            (ENTRIES_PER_AXIS * ENTRIES_PER_AXIS * ENTRIES_PER_AXIS) / values_per_long;

        write.put_var_int(&long_array_length.into());

        let mut value = 0u64;
        let mut packed_count = 0;
        let mut longs_written = 0;
        for xz in self.data {
            for xs in xz {
                for registry_id in xs {
                    let index = unique_registry_ids
                        .binary_search(&registry_id)
                        .expect("We previously computed this");

                    value |= (index as u64 & value_mask) << (bpe * packed_count);
                    if packed_count == values_per_long {
                        write.put_u64(value);
                        longs_written += 1;
                        value = 0;
                        packed_count = 0;
                    }
                }
            }
        }

        debug_assert!(longs_written, long_array_length);
    }

    fn read_indirect_palette(
        bpe: usize,
        read: &mut impl bytes::Buf,
    ) -> Result<RegistryIdCube<ENTRIES_PER_AXIS>, super::DecodeError> {
        let map_length = VarInt::decode(read)?;

        let mut map = vec![0; map_length.0 as usize];
        for entry in map.iter_mut() {
            *entry = VarInt::decode(read)?.0 as RegistryIdType;
        }

        let mut data: RegistryIdCube<ENTRIES_PER_AXIS> =
            [[[0 as u16; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS];
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;

        let data_length = VarInt::decode(read)?.0 as usize;
        let values_per_long = 64 / bpe;
        let value_mask = (1u64 << bpe) - 1;

        for _ in 0..data_length {
            let raw_data = read.get_u64();
            for count in 0..values_per_long {
                let value_mask = value_mask << (bpe * count);
                let value = (raw_data & value_mask) >> (bpe * count);

                data[y][z][x] = *map
                    .get(value as usize)
                    .ok_or_else(|| super::DecodeError::Malformed)?;

                // TODO: Is there a better way to do this?
                x += 1;
                if x == ENTRIES_PER_AXIS {
                    z += 1;
                    if z == ENTRIES_PER_AXIS {
                        y += 1;
                        if y == ENTRIES_PER_AXIS {
                            return Err(super::DecodeError::Malformed);
                        }
                    }
                }
            }
        }

        if x != ENTRIES_PER_AXIS - 1 || z != ENTRIES_PER_AXIS - 1 || y != ENTRIES_PER_AXIS {
            return Err(super::DecodeError::Malformed);
        }

        Ok(data)
    }

    fn write_direct_palette(&self, bpe: usize, write: &mut impl BufMut) {
        let values_per_long = 64 / bpe;
        let value_mask = (1u64 << bpe) - 1;
        let long_array_length =
            (ENTRIES_PER_AXIS * ENTRIES_PER_AXIS * ENTRIES_PER_AXIS) / values_per_long;

        write.put_var_int(&long_array_length.into());

        let mut value = 0u64;
        let mut packed_count = 0;
        let mut longs_written = 0;
        for xz in self.data {
            for xs in xz {
                for registry_id in xs {
                    value |= (registry_id as u64 & value_mask) << (bpe * packed_count);
                    if packed_count == values_per_long {
                        write.put_u64(value);
                        longs_written += 1;
                        value = 0;
                        packed_count = 0;
                    }
                }
            }
        }

        debug_assert!(longs_written, long_array_length);
    }

    fn read_direct_palette(
        bpe: usize,
        read: &mut impl bytes::Buf,
    ) -> Result<RegistryIdCube<ENTRIES_PER_AXIS>, super::DecodeError> {
        let mut data: RegistryIdCube<ENTRIES_PER_AXIS> =
            [[[0 as u16; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS]; ENTRIES_PER_AXIS];
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;

        let data_length = VarInt::decode(read)?.0 as usize;
        let values_per_long = 64 / bpe;
        let value_mask = (1u64 << bpe) - 1;

        for _ in 0..data_length {
            let raw_data = read.get_u64();
            for count in 0..values_per_long {
                let value_mask = value_mask << (bpe * count);
                let value = (raw_data & value_mask) >> (bpe * count);

                data[y][z][x] = value as RegistryIdType;

                // TODO: Is there a better way to do this?
                x += 1;
                if x == ENTRIES_PER_AXIS {
                    z += 1;
                    if z == ENTRIES_PER_AXIS {
                        y += 1;
                        if y == ENTRIES_PER_AXIS {
                            return Err(super::DecodeError::Malformed);
                        }
                    }
                }
            }
        }

        if x != ENTRIES_PER_AXIS - 1 || z != ENTRIES_PER_AXIS - 1 || y != ENTRIES_PER_AXIS {
            return Err(super::DecodeError::Malformed);
        }

        Ok(data)
    }
}
