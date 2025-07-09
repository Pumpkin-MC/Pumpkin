use std::{
    cmp::Ordering,
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use pumpkin_data::{Block, block_properties::get_state_by_state_id, chunk::Biome};
use pumpkin_util::encompassing_bits;

use crate::block::BlockStateCodec;

use super::format::{ChunkSectionBiomes, ChunkSectionBlockStates, PaletteBiomeEntry};

/// 3d array indexed by y,z,x
type AbstractCube<T, const DIM: u64> = [[[T; DIM]; DIM]; DIM];

#[derive(Debug, Clone)]
pub struct HeterogeneousPaletteData<V: Hash + Eq + Copy, const DIM: usize> {
    cube: Box<AbstractCube<V, DIM>>,
    counts: HashMap<V, u16>,
}

impl<V: Hash + Eq + Copy, const DIM: usize> HeterogeneousPaletteData<V, DIM> {
    fn get(&self, x: usize, y: usize, z: usize) -> V {
        debug_assert!(x < DIM);
        debug_assert!(y < DIM);
        debug_assert!(z < DIM);

        self.cube[y][z][x]
    }

    fn set(&mut self, x: usize, y: usize, z: usize, value: V) {
        debug_assert!(x < DIM);
        debug_assert!(y < DIM);
        debug_assert!(z < DIM);

        let original = self.cube[y][z][x];
        if let Entry::Occupied(mut entry) = self.counts.entry(original) {
            let count = entry.get_mut();
            *count -= 1;
            if *count == 0 {
                let _ = entry.remove();
            }
        }

        self.cube[y][z][x] = value;
        self.counts
            .entry(value)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
}

/// A paletted container is a cube of registry ids. It uses a custom compression scheme based on how
/// may distinct registry ids are in the cube.
#[derive(Debug, Clone)]
pub enum PalettedContainer<V: Hash + Eq + Copy + Default, const DIM: usize> {
    Homogeneous(V),
    Heterogeneous(Box<HeterogeneousPaletteData<V, DIM>>),
}

impl<V: Hash + Eq + Copy + Default, const DIM: usize> PalettedContainer<V, DIM> {
    pub const SIZE: usize = DIM;
    pub const VOLUME: usize = DIM * DIM * DIM;

    fn from_cube(cube: Box<AbstractCube<V, DIM>>) -> Self {
        let counts =
            cube.as_flattened()
                .as_flattened()
                .iter()
                .fold(HashMap::new(), |mut acc, key| {
                    acc.entry(*key).and_modify(|count| *count += 1).or_insert(1);
                    acc
                });

        if counts.len() == 1 {
            Self::Homogeneous(*counts.keys().next().unwrap())
        } else {
            Self::Heterogeneous(Box::new(HeterogeneousPaletteData { cube, counts }))
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

                let palette: Box<[V]> = data.counts.keys().copied().collect();
                let key_to_index_map: HashMap<V, usize> = palette
                    .iter()
                    .enumerate()
                    .map(|(index, key)| (*key, index))
                    .collect();

                let blocks_per_i64 = 64 / bits_per_entry;

                let packed_indices = data
                    .cube
                    .as_flattened()
                    .as_flattened()
                    .chunks(blocks_per_i64 as usize)
                    .map(|chunk| {
                        chunk.iter().enumerate().fold(0, |acc, (index, key)| {
                            let key_index = key_to_index_map.get(key).unwrap();
                            debug_assert!((1 << bits_per_entry) > *key_index);

                            let packed_offset_index =
                                (*key_index as u64) << (bits_per_entry as u64 * index as u64);
                            acc | packed_offset_index as i64
                        })
                    })
                    .collect();

                (palette, packed_indices)
            }
        }
    }

    pub fn from_palette_and_packed_data(
        palette: &[V],
        packed_data: &[i64],
        minimum_bits_per_entry: u8,
    ) -> Self {
        if palette.is_empty() {
            log::warn!("No palette data! Defaulting...");
            Self::Homogeneous(V::default())
        } else if palette.len() == 1 {
            Self::Homogeneous(palette[0])
        } else {
            let bits_per_key = encompassing_bits(palette.len()).max(minimum_bits_per_entry);
            let index_mask = (1 << bits_per_key) - 1;
            let keys_per_i64 = 64 / bits_per_key;

            let expected_i64_count = Self::VOLUME.div_ceil(keys_per_i64 as usize);

            match packed_data.len().cmp(&expected_i64_count) {
                Ordering::Greater => {
                    // Handled by the zip
                    log::warn!("Filled the section but there is still more data! Ignoring...");
                }
                Ordering::Less => {
                    // Handled by the array initialization and zip
                    log::warn!(
                        "Ran out of packed indices, but did not fill the section ({} vs {} for {}). Defaulting...",
                        packed_data.len() * keys_per_i64 as usize,
                        Self::VOLUME,
                        palette.len(),
                    );
                }
                // This is what we want!
                Ordering::Equal => {}
            }

            // TODO: Can we do this all with an `array::from_fn` or something?
            let mut cube = Box::new([[[V::default(); DIM]; DIM]; DIM]);
            cube.as_flattened_mut()
                .as_flattened_mut()
                .chunks_mut(keys_per_i64 as usize)
                .zip(packed_data)
                .for_each(|(values, packed)| {
                    values.iter_mut().enumerate().for_each(|(index, value)| {
                        let lookup_index =
                            (*packed as u64 >> (index as u64 * bits_per_key as u64)) & index_mask;

                        if let Some(v) = palette.get(lookup_index as usize) {
                            *value = *v;
                        } else {
                            // The cube is already initialized to the default
                            log::warn!("Lookup index out of bounds! Defaulting...");
                        }
                    });
                });

            Self::from_cube(cube)
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> V {
        match self {
            Self::Homogeneous(value) => *value,
            Self::Heterogeneous(data) => data.get(x, y, z),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: V) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Homogeneous(original) => {
                if value != *original {
                    let mut cube = Box::new([[[*original; DIM]; DIM]; DIM]);
                    cube[y][z][x] = value;
                    *self = Self::from_cube(cube);
                }
            }
            Self::Heterogeneous(data) => {
                data.set(x, y, z, value);
                if data.counts.len() == 1 {
                    *self = Self::Homogeneous(*data.counts.keys().next().unwrap());
                }
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

    pub fn non_air_block_count(&self) -> u16 {
        match self {
            Self::Homogeneous(registry_id) => {
                if !get_state_by_state_id(*registry_id).is_air() {
                    Self::VOLUME as u16
                } else {
                    0
                }
            }
            Self::Heterogeneous(data) => data
                .counts
                .iter()
                .map(|(registry_id, count)| {
                    if !get_state_by_state_id(*registry_id).is_air() {
                        *count
                    } else {
                        0
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
            properties: block.properties(registry_id).map(|p| p.to_props()),
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
