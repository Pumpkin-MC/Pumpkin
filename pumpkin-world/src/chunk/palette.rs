use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use pumpkin_data::{block::Block, chunk::Biome};
use pumpkin_macros::block_state;

use crate::block::ChunkBlockState;

use super::format::{
    ChunkSectionBiomes, ChunkSectionBlockStates, PaletteBiomeEntry, PaletteBlockEntry,
};

/// 3d array indexed by y,z,x
type AbstractCube<T, const DIM: usize> = [[[T; DIM]; DIM]; DIM];

/// The minimum number of bits required to represent this number
#[inline]
fn encompassing_bits(count: usize) -> u8 {
    count.ilog2() as u8 + 1
}

// TODO: Verify the default state for these blocks is the only state
const AIR: ChunkBlockState = block_state!("air");
const CAVE_AIR: ChunkBlockState = block_state!("cave_air");
const VOID_AIR: ChunkBlockState = block_state!("void_air");

#[inline]
fn is_not_air_block(state_id: u16) -> bool {
    state_id != AIR.state_id && state_id != CAVE_AIR.state_id && state_id != VOID_AIR.state_id
}

#[derive(Debug)]
pub struct HeterogeneousPaletteData<V: Hash + Eq + Copy, const DIM: usize> {
    cube: AbstractCube<V, DIM>,
    counts: HashMap<V, u16>,
}

impl<V: Hash + Eq + Copy, const DIM: usize> HeterogeneousPaletteData<V, DIM> {
    fn from_cube(cube: AbstractCube<V, DIM>) -> Self {
        let mut counts = HashMap::new();
        cube.iter().for_each(|zxs| {
            zxs.iter().for_each(|xs| {
                xs.iter().for_each(|value| {
                    counts
                        .entry(*value)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                });
            });
        });

        Self { cube, counts }
    }

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
#[derive(Debug)]
pub enum PalettedContainer<V: Hash + Eq + Copy + Default, const DIM: usize> {
    Homogeneous(V),
    Heterogeneous(Box<HeterogeneousPaletteData<V, DIM>>),
}

impl<V: Hash + Eq + Copy + Default, const DIM: usize> PalettedContainer<V, DIM> {
    pub const SIZE: usize = DIM;
    pub const VOLUME: usize = DIM * DIM * DIM;

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
                let mut palette = Vec::new();
                let mut id_to_index_map = HashMap::new();
                data.counts.keys().for_each(|registry_id| {
                    let _ = id_to_index_map.entry(*registry_id).or_insert_with(|| {
                        palette.push(*registry_id);
                        palette.len() - 1
                    });
                });

                let blocks_per_i64 = 64 / bits_per_entry;
                let expected_len = Self::VOLUME.div_ceil(blocks_per_i64 as usize);
                let mut packed_indices = Vec::with_capacity(expected_len);

                let mut packed_data = 0i64;
                let mut pack_count = 0;
                data.cube.iter().for_each(|zxs| {
                    zxs.iter().for_each(|xs| {
                        xs.iter().for_each(|registry_id| {
                            packed_data |= (*id_to_index_map.get(registry_id).unwrap() as i64)
                                << (bits_per_entry * pack_count);
                            if pack_count == blocks_per_i64 {
                                packed_indices.push(packed_data);
                                packed_data = 0;
                                pack_count = 0;
                            }
                        });
                    });
                });

                (
                    palette.into_boxed_slice(),
                    packed_indices.into_boxed_slice(),
                )
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
            let bits_per_block = encompassing_bits(palette.len()).max(minimum_bits_per_entry);
            let index_mask = (1 << bits_per_block) - 1;
            let blocks_per_i64 = 64 / bits_per_block;

            // TODO: Can we do this all with an `array::from_fn` or something?
            let mut cube = [[[V::default(); DIM]; DIM]; DIM];

            let mut index_count = 0;
            'outer: for packed_index in packed_data {
                for block in 0..blocks_per_i64 {
                    if index_count == Self::VOLUME {
                        log::warn!("Filled the section but there is still more data! Ignoring...");
                        break 'outer;
                    }

                    let relative_x = index_count % Self::SIZE;
                    let relative_y = index_count / (Self::SIZE * Self::SIZE);
                    let relative_z = (index_count % (Self::SIZE * Self::SIZE)) / Self::SIZE;

                    let index = (packed_index >> (bits_per_block * block)) & index_mask;
                    let registry_id = palette.get(index as usize).copied().unwrap_or_else(|| {
                        log::warn!("Palette index out of bounds! Defaulting...");
                        V::default()
                    });

                    cube[relative_y][relative_z][relative_x] = registry_id;

                    index_count += 1;
                }
            }

            if index_count < Self::VOLUME {
                // We pre-filled with defaults
                log::warn!(
                    "Ran out of packed indices, but did not fill the section. Defaulting..."
                );
            }

            Self::Heterogeneous(Box::new(HeterogeneousPaletteData::from_cube(cube)))
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
                    let mut cube = [[[*original; DIM]; DIM]; DIM];
                    cube[y][z][x] = value;
                    let data = HeterogeneousPaletteData::from_cube(cube);
                    *self = Self::Heterogeneous(Box::new(data));
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
                data.cube.iter().for_each(|zxs| {
                    zxs.iter().for_each(|xs| {
                        xs.iter().for_each(|registry_id| {
                            f(*registry_id);
                        });
                    });
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
                    debug_assert!((1 << bits_per_entry) > data.counts.len());

                    let blocks_per_i64 = 64 / bits_per_entry;
                    let expected_len = Self::VOLUME.div_ceil(blocks_per_i64 as usize);
                    let mut packed_datas = Vec::with_capacity(expected_len);

                    let mut packed_data = 0i64;
                    let mut pack_count = 0;
                    data.cube.iter().for_each(|zxs| {
                        zxs.iter().for_each(|xs| {
                            xs.iter().for_each(|registry_id| {
                                packed_data |=
                                    (*registry_id as i64) << (bits_per_entry * pack_count);
                                if pack_count == blocks_per_i64 {
                                    packed_datas.push(packed_data);
                                    packed_data = 0;
                                    pack_count = 0;
                                }
                            });
                        });
                    });

                    NetworkSerialization {
                        bits_per_entry,
                        palette: NetworkPalette::Direct,
                        packed_data: packed_datas.into_boxed_slice(),
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
            nbt.data.as_ref().unwrap_or(&vec![].into_boxed_slice()),
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
                    debug_assert!((1 << bits_per_entry) > data.counts.len());

                    let blocks_per_i64 = 64 / bits_per_entry;
                    let expected_len = Self::VOLUME.div_ceil(blocks_per_i64 as usize);
                    let mut packed_datas = Vec::with_capacity(expected_len);

                    let mut packed_data = 0i64;
                    let mut pack_count = 0;
                    data.cube.iter().for_each(|zxs| {
                        zxs.iter().for_each(|xs| {
                            xs.iter().for_each(|registry_id| {
                                packed_data |=
                                    (*registry_id as i64) << (bits_per_entry * pack_count);
                                if pack_count == blocks_per_i64 {
                                    packed_datas.push(packed_data);
                                    packed_data = 0;
                                    pack_count = 0;
                                }
                            });
                        });
                    });

                    NetworkSerialization {
                        bits_per_entry,
                        palette: NetworkPalette::Direct,
                        packed_data: packed_datas.into_boxed_slice(),
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
                if is_not_air_block(*registry_id) {
                    Self::VOLUME as u16
                } else {
                    0
                }
            }
            Self::Heterogeneous(data) => data
                .counts
                .iter()
                .map(|(registry_id, count)| {
                    if is_not_air_block(*registry_id) {
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
            .map(|entry| ChunkBlockState::from_palette(&entry).get_id())
            .collect::<Vec<_>>();

        Self::from_palette_and_packed_data(
            &palette,
            nbt.data.as_ref().unwrap_or(&vec![].into_boxed_slice()),
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

    fn block_state_id_to_palette_entry(registry_id: u16) -> PaletteBlockEntry {
        let block = Block::from_state_id(registry_id).unwrap();

        PaletteBlockEntry {
            name: block.name.into(),
            properties: {
                if let Some(properties) = block.properties(registry_id) {
                    let props = properties.to_props();
                    let mut props_map = HashMap::new();
                    for prop in props {
                        props_map.insert(prop.0.clone(), prop.1.clone());
                    }
                    Some(props_map)
                } else {
                    None
                }
            },
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

pub type BlockPalette = PalettedContainer<u16, 16>;
const BLOCK_DISK_MIN_BITS: u8 = 4;
const BLOCK_NETWORK_MIN_MAP_BITS: u8 = 4;
const BLOCK_NETWORK_MAX_MAP_BITS: u8 = 8;
const BLOCK_NETWORK_MAX_BITS: u8 = 15;

pub type BiomePalette = PalettedContainer<u8, 4>;
const BIOME_DISK_MIN_BITS: u8 = 0;
const BIOME_NETWORK_MIN_MAP_BITS: u8 = 1;
const BIOME_NETWORK_MAX_MAP_BITS: u8 = 3;
const BIOME_NETWORK_MAX_BITS: u8 = 6;
