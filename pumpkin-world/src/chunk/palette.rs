use std::{collections::HashMap, hash::Hash};

use pumpkin_data::{Block, BlockState, chunk::Biome};
use pumpkin_util::encompassing_bits;

use crate::block::BlockStateCodec;

use super::format::{ChunkSectionBiomes, ChunkSectionBlockStates, PaletteBiomeEntry};

/// 3d array indexed by y,z,x
type AbstractCube<T, const DIM: usize> = [[[T; DIM]; DIM]; DIM];

#[derive(Clone)]
pub struct HeterogeneousPaletteData<V: Hash + Eq + Copy, const DIM: usize> {
    cube: Box<AbstractCube<V, DIM>>,
    palette: Vec<V>,
    counts: Vec<u16>,
}

impl<V: Hash + Eq + Copy, const DIM: usize> HeterogeneousPaletteData<V, DIM> {
    fn get(&self, x: usize, y: usize, z: usize) -> V {
        debug_assert!(x < DIM);
        debug_assert!(y < DIM);
        debug_assert!(z < DIM);

        self.cube[y][z][x]
    }

    /// Returns the Original
    fn set(&mut self, x: usize, y: usize, z: usize, value: V) -> V {
        debug_assert!(x < DIM);
        debug_assert!(y < DIM);
        debug_assert!(z < DIM);

        let original = self.cube[y][z][x];
        let original_index = self.palette.iter().position(|v| v == &original).unwrap();
        self.counts[original_index] -= 1;

        if self.counts[original_index] == 0 {
            // Remove from palette and counts Vecs if the count hits zero.
            self.palette.swap_remove(original_index);
            self.counts.swap_remove(original_index);
        }

        // Set the new value in the cube
        self.cube[y][z][x] = value;

        // Find or add the new value to the palette.
        if let Some(new_index) = self.palette.iter().position(|v| v == &value) {
            self.counts[new_index] += 1;
        } else {
            self.palette.push(value);
            self.counts.push(1);
        }

        original
    }
}

/// A paletted container is a cube of registry ids. It uses a custom compression scheme based on how
/// may distinct registry ids are in the cube.
#[derive(Clone)]
pub enum PalettedContainer<V: Hash + Eq + Copy + Default, const DIM: usize> {
    Homogeneous(V),
    Heterogeneous(Box<HeterogeneousPaletteData<V, DIM>>),
}

impl<V: Hash + Eq + Copy + Default, const DIM: usize> PalettedContainer<V, DIM> {
    pub const SIZE: usize = DIM;
    pub const VOLUME: usize = DIM * DIM * DIM;

    fn from_cube(cube: Box<AbstractCube<V, DIM>>) -> Self {
        let mut palette: Vec<V> = Vec::new();
        let mut counts: Vec<u16> = Vec::new();

        // Iterate over the flattened cube to populate the palette and counts
        for val in cube.as_flattened().as_flattened().iter() {
            if let Some(index) = palette.iter().position(|v| v == val) {
                // Value already exists, increment its count
                counts[index] += 1;
            } else {
                // New value, add it to the palette and start its count
                palette.push(*val);
                counts.push(1);
            }
        }

        if palette.len() == 1 {
            // Fast path: the cube is homogeneous, so we can store just one value
            Self::Homogeneous(palette[0])
        } else {
            // Heterogeneous cube, store the full data
            Self::Heterogeneous(Box::new(HeterogeneousPaletteData {
                cube,
                palette,
                counts,
            }))
        }
    }

    fn bits_per_entry(&self) -> u8 {
        match self {
            Self::Homogeneous(_) => 0,
            Self::Heterogeneous(data) => encompassing_bits(data.counts.len()),
        }
    }

    pub fn to_palette_and_packed_data(&self, bits_per_entry: u8) -> (Box<[V]>, Box<[i64]>) {
        match self {
            Self::Homogeneous(registry_id) => (Box::new([*registry_id]), Box::new([])),
            Self::Heterogeneous(data) => {
                debug_assert!(bits_per_entry >= encompassing_bits(data.counts.len()));
                debug_assert!(bits_per_entry <= 15);

                // Don't use HashMap's here, because its slow
                let blocks_per_i64 = 64 / bits_per_entry;

                let packed_indices: Box<[i64]> = data
                    .cube
                    .as_flattened()
                    .as_flattened()
                    .chunks(blocks_per_i64 as usize)
                    .map(|chunk| {
                        chunk.iter().enumerate().fold(0, |acc, (index, key)| {
                            let key_index = data.palette.iter().position(|&x| x == *key).unwrap();
                            debug_assert!((1 << bits_per_entry) > key_index);

                            let packed_offset_index =
                                (key_index as u64) << (bits_per_entry as u64 * index as u64);
                            acc | packed_offset_index as i64
                        })
                    })
                    .collect();

                (data.palette.clone().into_boxed_slice(), packed_indices)
            }
        }
    }

    pub fn from_palette_and_packed_data(
        palette_slice: &[V],
        packed_data: &[i64],
        minimum_bits_per_entry: u8,
    ) -> Self {
        if palette_slice.is_empty() {
            log::warn!("No palette data! Defaulting...");
            return Self::Homogeneous(V::default());
        }

        if palette_slice.len() == 1 {
            return Self::Homogeneous(palette_slice[0]);
        }

        let bits_per_key = encompassing_bits(palette_slice.len()).max(minimum_bits_per_entry);
        let index_mask = (1 << bits_per_key) - 1;
        let keys_per_i64 = 64 / bits_per_key;

        let mut decompressed_values = Vec::with_capacity(Self::VOLUME);

        // We already have the palette from the input `palette_slice`.
        // The counts will be created in the next step.

        let mut packed_data_iter = packed_data.iter();
        let mut current_packed_word = *packed_data_iter.next().unwrap_or(&0);

        for i in 0..Self::VOLUME {
            let bit_index_in_word = i % keys_per_i64 as usize;

            if bit_index_in_word == 0 && i > 0 {
                current_packed_word = *packed_data_iter.next().unwrap_or(&0);
            }

            let lookup_index = (current_packed_word as u64
                >> (bit_index_in_word as u64 * bits_per_key as u64))
                & index_mask;

            let value = palette_slice
                .get(lookup_index as usize)
                .copied()
                .unwrap_or_else(|| {
                    log::warn!("Lookup index out of bounds! Defaulting...");
                    V::default()
                });

            decompressed_values.push(value);
        }

        // Now, with all decompressed values, build the counts.
        let mut counts = vec![0; palette_slice.len()];

        for &value in &decompressed_values {
            // This is the key optimization: find the index in the palette Vec
            // and increment the corresponding count.
            if let Some(index) = palette_slice.iter().position(|v| v == &value) {
                counts[index] += 1;
            } else {
                // This case should ideally not happen if the palette is complete.
                log::warn!("Decompressed value not found in palette!");
            }
        }

        let mut cube = Box::new([[[V::default(); DIM]; DIM]; DIM]);
        cube.as_flattened_mut()
            .as_flattened_mut()
            .copy_from_slice(&decompressed_values);

        let palette_vec: Vec<V> = palette_slice.to_vec();

        Self::Heterogeneous(Box::new(HeterogeneousPaletteData {
            cube,
            palette: palette_vec,
            counts,
        }))
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> V {
        match self {
            Self::Homogeneous(value) => *value,
            Self::Heterogeneous(data) => data.get(x, y, z),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: V) -> V {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Homogeneous(original) => {
                let original = *original;
                if value != original {
                    let mut cube = Box::new([[[original; DIM]; DIM]; DIM]);
                    cube[y][z][x] = value;
                    *self = Self::from_cube(cube);
                }
                original
            }
            Self::Heterogeneous(data) => {
                let original = data.set(x, y, z, value);
                if data.counts.len() == 1 {
                    *self = Self::Homogeneous(data.palette[0]);
                }
                original
            }
        }
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(V),
    {
        match self {
            Self::Homogeneous(registry_id) => {
                for _ in 0..Self::VOLUME {
                    f(*registry_id);
                }
            }
            Self::Heterogeneous(data) => {
                data.cube
                    .as_flattened()
                    .as_flattened()
                    .iter()
                    .for_each(|value| {
                        f(*value);
                    });
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Homogeneous(value) => *value == V::default(),
            Self::Heterogeneous(_) => false,
        }
    }
}

impl<V: Default + Hash + Eq + Copy, const DIM: usize> Default for PalettedContainer<V, DIM> {
    fn default() -> Self {
        Self::Homogeneous(V::default())
    }
}

impl BiomePalette {
    pub fn convert_network(&self) -> NetworkSerialization<u8> {
        match self {
            Self::Homogeneous(registry_id) => NetworkSerialization {
                bits_per_entry: 0,
                palette: NetworkPalette::Single(*registry_id),
                packed_data: Box::new([]),
            },
            Self::Heterogeneous(data) => {
                let raw_bits_per_entry = encompassing_bits(data.counts.len());
                if raw_bits_per_entry > BIOME_NETWORK_MAX_MAP_BITS {
                    let bits_per_entry = BIOME_NETWORK_MAX_BITS;
                    let values_per_i64 = 64 / bits_per_entry;
                    let packed_data = data
                        .cube
                        .as_flattened()
                        .as_flattened()
                        .chunks(values_per_i64 as usize)
                        .map(|chunk| {
                            chunk.iter().enumerate().fold(0, |acc, (index, value)| {
                                debug_assert!((1 << bits_per_entry) > *value);
                                let packed_offset_index =
                                    (*value as u64) << (bits_per_entry as u64 * index as u64);
                                acc | packed_offset_index as i64
                            })
                        })
                        .collect();

                    NetworkSerialization {
                        bits_per_entry,
                        palette: NetworkPalette::Direct,
                        packed_data,
                    }
                } else {
                    let bits_per_entry = raw_bits_per_entry.max(BIOME_NETWORK_MIN_MAP_BITS);
                    let (palette, packed) = self.to_palette_and_packed_data(bits_per_entry);

                    NetworkSerialization {
                        bits_per_entry,
                        palette: NetworkPalette::Indirect(palette),
                        packed_data: packed,
                    }
                }
            }
        }
    }

    pub fn from_disk_nbt(nbt: ChunkSectionBiomes) -> Self {
        let palette = nbt
            .palette
            .into_iter()
            .map(|entry| Biome::from_name(&entry.name).unwrap_or(&Biome::PLAINS).id)
            .collect::<Vec<_>>();

        Self::from_palette_and_packed_data(
            &palette,
            nbt.data.as_ref().unwrap_or(&Box::default()),
            BIOME_DISK_MIN_BITS,
        )
    }

    pub fn to_disk_nbt(&self) -> ChunkSectionBiomes {
        #[allow(clippy::unnecessary_min_or_max)]
        let bits_per_entry = self.bits_per_entry().max(BIOME_DISK_MIN_BITS);
        let (palette, packed_data) = self.to_palette_and_packed_data(bits_per_entry);
        ChunkSectionBiomes {
            data: if packed_data.is_empty() {
                None
            } else {
                Some(packed_data)
            },
            palette: palette
                .into_iter()
                .map(|registry_id| PaletteBiomeEntry {
                    name: Biome::from_id(registry_id).unwrap().registry_id.into(),
                })
                .collect(),
        }
    }
}

impl BlockPalette {
    pub fn convert_network(&self) -> NetworkSerialization<u16> {
        match self {
            Self::Homogeneous(registry_id) => NetworkSerialization {
                bits_per_entry: 0,
                palette: NetworkPalette::Single(*registry_id),
                packed_data: Box::new([]),
            },
            Self::Heterogeneous(data) => {
                let raw_bits_per_entry = encompassing_bits(data.counts.len());
                if raw_bits_per_entry > BLOCK_NETWORK_MAX_MAP_BITS {
                    let bits_per_entry = BLOCK_NETWORK_MAX_BITS;
                    let values_per_i64 = 64 / bits_per_entry;
                    let packed_data = data
                        .cube
                        .as_flattened()
                        .as_flattened()
                        .chunks(values_per_i64 as usize)
                        .map(|chunk| {
                            chunk.iter().enumerate().fold(0, |acc, (index, value)| {
                                debug_assert!((1 << bits_per_entry) > *value);

                                let packed_offset_index =
                                    (*value as i64) << (bits_per_entry as u64 * index as u64);
                                acc | packed_offset_index
                            })
                        })
                        .collect();

                    NetworkSerialization {
                        bits_per_entry,
                        palette: NetworkPalette::Direct,
                        packed_data,
                    }
                } else {
                    let bits_per_entry = raw_bits_per_entry.max(BLOCK_NETWORK_MIN_MAP_BITS);
                    let (palette, packed) = self.to_palette_and_packed_data(bits_per_entry);

                    NetworkSerialization {
                        bits_per_entry,
                        palette: NetworkPalette::Indirect(palette),
                        packed_data: packed,
                    }
                }
            }
        }
    }

    pub fn convert_be_network(&self) -> BeNetworkSerialization<u16> {
        match self {
            Self::Homogeneous(registry_id) => BeNetworkSerialization {
                bits_per_entry: 0,
                palette: NetworkPalette::Single(BlockState::to_be_network_id(*registry_id)),
                packed_data: Box::new([]),
            },
            Self::Heterogeneous(data) => {
                let bits_per_entry = encompassing_bits(data.palette.len());

                let key_to_index_map: HashMap<_, usize> = data
                    .palette
                    .iter()
                    .enumerate()
                    .map(|(index, key)| (*key, index))
                    .collect();

                let blocks_per_word = 32 / bits_per_entry;
                let expected_word_count = Self::VOLUME.div_ceil(blocks_per_word as usize);
                let mut packed_data = Vec::with_capacity(expected_word_count);

                let mut current_word: u32 = 0;
                let mut current_index_in_word = 0;

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            // Java has it in y, z, x order, so we need to convert it back to x, y, z
                            // Please test your code on bedrock before merging
                            let key = data.get(x, z, y);
                            let key_index = key_to_index_map.get(&key).unwrap();
                            debug_assert!((1 << bits_per_entry) > *key_index);

                            current_word |= (*key_index as u32)
                                << (bits_per_entry as u32 * current_index_in_word);
                            current_index_in_word += 1;

                            if current_index_in_word == blocks_per_word as u32 {
                                packed_data.push(current_word);
                                current_word = 0;
                                current_index_in_word = 0;
                            }
                        }
                    }
                }

                // Push any remaining bits if the volume isn't a multiple of blocks_per_word
                if current_index_in_word > 0 {
                    packed_data.push(current_word);
                }

                BeNetworkSerialization {
                    bits_per_entry,
                    palette: NetworkPalette::Indirect(
                        data.palette
                            .iter()
                            .map(|&id| BlockState::to_be_network_id(id))
                            .collect(),
                    ),
                    packed_data: packed_data.into_boxed_slice(),
                }
            }
        }
    }

    pub fn non_air_block_count(&self) -> u16 {
        match self {
            Self::Homogeneous(registry_id) => {
                if !BlockState::from_id(*registry_id).is_air() {
                    Self::VOLUME as u16
                } else {
                    0
                }
            }
            Self::Heterogeneous(data) => data
                .palette
                .iter()
                .zip(data.counts.iter())
                .filter_map(|(registry_id, count)| {
                    if !BlockState::from_id(*registry_id).is_air() {
                        Some(*count)
                    } else {
                        None
                    }
                })
                .sum(),
        }
    }

    pub fn from_disk_nbt(nbt: ChunkSectionBlockStates) -> Self {
        let palette = nbt
            .palette
            .into_iter()
            .map(|entry| entry.get_state_id())
            .collect::<Vec<_>>();

        Self::from_palette_and_packed_data(
            &palette,
            nbt.data.as_ref().unwrap_or(&Box::default()),
            BLOCK_DISK_MIN_BITS,
        )
    }

    pub fn to_disk_nbt(&self) -> ChunkSectionBlockStates {
        let bits_per_entry = self.bits_per_entry().max(BLOCK_DISK_MIN_BITS);
        let (palette, packed_data) = self.to_palette_and_packed_data(bits_per_entry);
        ChunkSectionBlockStates {
            data: if packed_data.is_empty() {
                None
            } else {
                Some(packed_data)
            },
            palette: palette
                .into_iter()
                .map(Self::block_state_id_to_palette_entry)
                .collect(),
        }
    }

    fn block_state_id_to_palette_entry(registry_id: u16) -> BlockStateCodec {
        let block = Block::from_state_id(registry_id);

        BlockStateCodec {
            name: block,
            properties: block.properties(registry_id).map(|p| {
                p.to_props()
                    .into_iter()
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    .collect()
            }),
        }
    }
}

pub enum NetworkPalette<V> {
    Single(V),
    Indirect(Box<[V]>),
    Direct,
}

pub struct NetworkSerialization<V> {
    pub bits_per_entry: u8,
    pub palette: NetworkPalette<V>,
    pub packed_data: Box<[i64]>,
}

pub struct BeNetworkSerialization<V> {
    pub bits_per_entry: u8,
    pub palette: NetworkPalette<V>,
    pub packed_data: Box<[u32]>,
}

// According to the wiki, palette serialization for disk and network is different. Disk
// serialization always uses a palette if greater than one entry. Network serialization packs ids
// directly instead of using a palette above a certain bits-per-entry

// TODO: Do our own testing; do we really need to handle network and disk serialization differently?
pub type BlockPalette = PalettedContainer<u16, 16>;
const BLOCK_DISK_MIN_BITS: u8 = 4;
const BLOCK_NETWORK_MIN_MAP_BITS: u8 = 4;
const BLOCK_NETWORK_MAX_MAP_BITS: u8 = 8;
pub(crate) const BLOCK_NETWORK_MAX_BITS: u8 = 15;

pub type BiomePalette = PalettedContainer<u8, 4>;
const BIOME_DISK_MIN_BITS: u8 = 0;
const BIOME_NETWORK_MIN_MAP_BITS: u8 = 1;
const BIOME_NETWORK_MAX_MAP_BITS: u8 = 3;
pub(crate) const BIOME_NETWORK_MAX_BITS: u8 = 7;
