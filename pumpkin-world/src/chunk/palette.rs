use std::collections::{HashMap, hash_map::Entry};

use itertools::Itertools;

/// 3d array indexed by y,z,x
type AbstractCube<T, const DIM: usize> = [[[T; DIM]; DIM]; DIM];

/// The minimum number of bits required to represent this number
#[inline]
pub fn encompassing_bits(count: usize) -> usize {
    count.ilog2() as usize + 1
}

#[derive(Debug)]
pub struct SinglePalettedContainer {
    registry_id: u16,
}

impl SinglePalettedContainer {
    fn should_upgrade(&self, registry_id_to_add: u16) -> bool {
        registry_id_to_add != self.registry_id
    }
}

#[derive(Debug)]
struct MappedPalettedConatinerMetadata {
    /// The value that this registry id is mapped to
    mapping: u8,
    /// The count of this id that remain in the cube
    remaining: u16,
}

/// A paletted container that compresses data depending on how many bits are needed to encode ids.
/// `LOWER_BOUND` is the minimum bits per entry used to encode values. `UPPER_BOUND` is the maximum
/// bits per entry used to encode values. `DIM` is the length of the side of the cube.
#[derive(Debug)]
pub struct MappedPalettedContainer<
    const DIM: usize,
    const LOWER_BOUND: usize,
    const UPPER_BOUND: usize,
> {
    data: AbstractCube<u8, DIM>,
    index_to_id_map: Vec<Option<u16>>,
    id_to_data_map: HashMap<u16, MappedPalettedConatinerMetadata>,
}

impl<const DIM: usize, const LOWER_BOUND: usize, const UPPER_BOUND: usize>
    MappedPalettedContainer<DIM, LOWER_BOUND, UPPER_BOUND>
{
    fn build_direct_data(&self) -> AbstractCube<u16, DIM> {
        let mut return_data = [[[0; DIM]; DIM]; DIM];
        return_data
            .iter_mut()
            .zip_eq(self.data)
            .for_each(|(return_yzs, yzs)| {
                return_yzs
                    .iter_mut()
                    .zip_eq(yzs)
                    .for_each(|(return_zs, zs)| {
                        return_zs
                            .iter_mut()
                            .zip_eq(zs)
                            .for_each(|(registry_id, index)| {
                                *registry_id = self.index_to_id_map[index as usize].expect(
                                    "The index to id map should be synchronized with the data",
                                );
                            });
                    });
            });
        return_data
    }

    fn iter_yzx<F>(&self, f: F)
    where
        F: FnMut(u16),
    {
        let mut f = f;
        self.data.iter().for_each(|zxs| {
            zxs.iter().for_each(|xs| {
                xs.iter().for_each(|x| {
                    let registry_id = self.index_to_id_map[*x as usize]
                        .expect("The index to id map should be synchronized with the data");
                    f(registry_id);
                });
            });
        });
    }

    fn set_id(&mut self, x: usize, y: usize, z: usize, registry_id: u16) {
        let original_index = self.data[y][z][x];
        let original_registry_id = self.index_to_id_map[original_index as usize]
            .expect("The index_to_id_map should be synchronized with the id_to_data_map");
        if original_registry_id == registry_id {
            // We don't need to do anything
            return;
        }

        if let Entry::Occupied(mut entry) = self.id_to_data_map.entry(original_registry_id) {
            let metadata = entry.get_mut();
            // We lost a reference in the data cube
            metadata.remaining -= 1;
            if metadata.remaining == 0 {
                // Mark this mapping as unused
                self.index_to_id_map[metadata.mapping as usize] = None;
                // Remove from the map
                let _ = entry.remove();
            }
        }

        let entry = self.id_to_data_map.entry(registry_id).or_insert_with(|| {
            // Get the first empty index
            let empty_index = self
                .index_to_id_map
                .iter()
                .position(|value| value.is_none());

            let index = match empty_index {
                Some(index) => {
                    self.index_to_id_map[index] = Some(registry_id);
                    index
                }
                None => {
                    self.index_to_id_map.push(Some(registry_id));
                    self.index_to_id_map.len() - 1
                }
            };

            debug_assert!(
                index <= 255,
                "This will overflow the map! Entries should be no more than 8 bits!"
            );

            MappedPalettedConatinerMetadata {
                mapping: index as u8,
                remaining: 0,
            }
        });

        // We're adding this to the data
        entry.remaining += 1;
        self.data[y][z][x] = entry.mapping;
    }

    fn get_id(&self, x: usize, y: usize, z: usize) -> u16 {
        let index = self.data[y][z][x];
        self.index_to_id_map[index as usize].expect(
            "The index to id map should only be none if the index is not used in the data cube",
        )
    }

    fn should_upgrade(&self, registry_id_to_add: u16, registry_id_to_remove: u16) -> bool {
        // Adding a new id would overflow our bits per entry
        let new_id_would_create_new_entry = !self.id_to_data_map.contains_key(&registry_id_to_add);
        let old_id_would_remove_entry = self
            .id_to_data_map
            .get(&registry_id_to_remove)
            .expect("The index to data map should be synchronized with the data")
            .remaining
            <= 1;

        if new_id_would_create_new_entry && !old_id_would_remove_entry {
            let base_bpe = encompassing_bits(self.id_to_data_map.len() + 1);
            base_bpe > UPPER_BOUND
        } else {
            false
        }
    }

    /// If this is true, `registry_id_to_add` is the only remaining id
    fn should_downgrade(&self, registry_id_to_add: u16, registry_id_to_remove: u16) -> bool {
        // The new id won't create a new mapping and the registry_id_to_remove isn't what is being
        // added and there is only two (or less) registries left and the registry_id_to_remove being
        // removed would remove that id
        registry_id_to_add != registry_id_to_remove
            && self.id_to_data_map.len() <= 2
            && self.id_to_data_map.contains_key(&registry_id_to_add)
            && self
                .id_to_data_map
                .get(&registry_id_to_remove)
                .is_none_or(|metadata| metadata.remaining <= 1)
    }

    fn bits_per_entry(&self) -> usize {
        let base_bpe = encompassing_bits(self.id_to_data_map.len());

        if base_bpe <= LOWER_BOUND {
            LOWER_BOUND
        } else if base_bpe <= UPPER_BOUND {
            base_bpe
        } else {
            panic!(
                "This should never happen because we will always upgrade to a direct pallete before this"
            );
        }
    }

    fn from_single_entry(registry_id: u16) -> Self {
        // We'll map this initial registry id to the value 0, so insert it into the 0th place in
        // the vec.
        let index_to_id_map = vec![Some(registry_id)];

        // We'll set what id this maps to in the map and intialize its count to the size of the data
        // cube
        let mut id_to_data_map = HashMap::new();
        id_to_data_map.insert(
            registry_id,
            MappedPalettedConatinerMetadata {
                mapping: 0,
                // This is max 4096 so it is safe
                remaining: (DIM * DIM * DIM) as u16,
            },
        );
        // Initialize the data cube to be all 0's
        let data = [[[0; DIM]; DIM]; DIM];

        Self {
            data,
            index_to_id_map,
            id_to_data_map,
        }
    }

    fn from_direct_entry(original_data: &AbstractCube<u16, DIM>) -> Self {
        // Initialize the data cube
        let mut data = [[[0; DIM]; DIM]; DIM];
        // Initialize the index to id map
        let mut index_to_id_map = Vec::new();
        // Initialize the id to index map
        let mut id_to_data_map = HashMap::new();

        data.iter_mut()
            .zip_eq(original_data)
            .for_each(|(yzs, original_yzs)| {
                yzs.iter_mut()
                    .zip_eq(original_yzs)
                    .for_each(|(zs, original_zs)| {
                        zs.iter_mut()
                            .zip_eq(original_zs)
                            .for_each(|(mapped_id, original_id)| {
                                let original_id = *original_id;
                                let entry =
                                    id_to_data_map.entry(original_id).or_insert_with(|| {
                                        // The index that the id is inserted into is the new map id
                                        index_to_id_map.push(Some(original_id));
                                        debug_assert!(
                                            index_to_id_map.len() <= 256,
                                            "mapped id overflow"
                                        );
                                        MappedPalettedConatinerMetadata {
                                            mapping: (index_to_id_map.len() - 1) as u8,
                                            remaining: 0,
                                        }
                                    });

                                // We're adding this to the data
                                entry.remaining += 1;

                                *mapped_id = entry.mapping;
                            });
                    });
            });

        Self {
            data,
            index_to_id_map,
            id_to_data_map,
        }
    }
}

/// A paletted container that directly holds values. `BPE` is the bits to use to encode data
/// entries. `DIM` is the length of the side of the cube.
#[derive(Debug)]
pub struct DirectPalettedContainer<const DIM: usize, const UPPER_BOUND: usize, const BPE: usize> {
    data: AbstractCube<u16, DIM>,
    id_to_count_map: HashMap<u16, u16>,
}

impl<const DIM: usize, const UPPER_BOUND: usize, const BPE: usize>
    DirectPalettedContainer<DIM, UPPER_BOUND, BPE>
{
    fn iter_yzx<F>(&self, f: F)
    where
        F: FnMut(u16),
    {
        let mut f = f;
        self.data.iter().for_each(|zxs| {
            zxs.iter().for_each(|xs| {
                xs.iter().for_each(|x| {
                    f(*x);
                });
            });
        });
    }

    fn from_abstract_cube(data: AbstractCube<u16, DIM>) -> Self {
        let mut id_to_count_map = HashMap::new();

        data.iter().for_each(|yzs| {
            yzs.iter().for_each(|zs| {
                zs.iter().for_each(|registry_id| {
                    id_to_count_map
                        .entry(*registry_id)
                        .and_modify(|count| *count += 1)
                        .or_insert_with(|| 1);
                });
            });
        });

        Self {
            data,
            id_to_count_map,
        }
    }

    fn set_id(&mut self, x: usize, y: usize, z: usize, registry_id: u16) {
        let original_id = self.data[y][z][x];
        self.data[y][z][x] = registry_id;

        self.id_to_count_map
            .entry(registry_id)
            .and_modify(|count| *count += 1)
            .or_insert_with(|| 1);

        if let Entry::Occupied(mut entry) = self.id_to_count_map.entry(original_id) {
            let count = entry.get_mut();
            *count -= 1;
            if *count == 0 {
                let _ = entry.remove();
            }
        }
    }

    fn get_id(&self, x: usize, y: usize, z: usize) -> u16 {
        self.data[y][z][x]
    }

    fn should_downgrade(&self, registry_id_to_add: u16, registry_id_to_remove: u16) -> bool {
        // The new id won't create a new mapping and the registry_id_to_remove isn't what is being
        // added and the registry_id_to_remove being removed would remove that id
        if registry_id_to_add != registry_id_to_remove
            && self.id_to_count_map.contains_key(&registry_id_to_add)
            && self
                .id_to_count_map
                .get(&registry_id_to_remove)
                .is_none_or(|count| *count <= 1)
        {
            let base_bpe = encompassing_bits(self.id_to_count_map.len() - 1);
            base_bpe <= UPPER_BOUND
        } else {
            false
        }
    }
}

/// A paletted container is a cube of registry ids. It uses a custom compression scheme based on how
/// may distinct registry ids are in the cube.
#[derive(Debug)]
pub enum PalettedContainer<
    const DIM: usize,
    const MAP_LOWER_BOUND: usize,
    const MAP_UPPER_BOUND: usize,
    const MAX_BPE: usize,
> {
    Single(SinglePalettedContainer),
    Mapped(Box<MappedPalettedContainer<DIM, MAP_LOWER_BOUND, MAP_UPPER_BOUND>>),
    Direct(Box<DirectPalettedContainer<DIM, MAP_UPPER_BOUND, MAX_BPE>>),
}

impl<
    const DIM: usize,
    const MAP_LOWER_BOUND: usize,
    const MAP_UPPER_BOUND: usize,
    const MAX_BPE: usize,
> PalettedContainer<DIM, MAP_LOWER_BOUND, MAP_UPPER_BOUND, MAX_BPE>
{
    pub const SIZE: usize = DIM;

    pub fn iter_yzx<F>(&self, f: F)
    where
        F: FnMut(u16),
    {
        match self {
            Self::Single(single) => {
                let mut f = f;
                for _ in 0..DIM * DIM * DIM {
                    f(single.registry_id);
                }
            }
            Self::Mapped(mapped) => {
                mapped.iter_yzx(f);
            }
            Self::Direct(direct) => {
                direct.iter_yzx(f);
            }
        }
    }

    #[inline]
    pub fn get_id(&self, x: usize, y: usize, z: usize) -> u16 {
        match self {
            Self::Single(single) => single.registry_id,
            Self::Mapped(mapped) => mapped.get_id(x, y, z),
            Self::Direct(direct) => direct.get_id(x, y, z),
        }
    }

    pub fn set_id(&mut self, x: usize, y: usize, z: usize, registry_id: u16) {
        match self {
            Self::Single(single) => {
                if single.should_upgrade(registry_id) {
                    let mut new_palette =
                        MappedPalettedContainer::from_single_entry(single.registry_id);
                    new_palette.set_id(x, y, z, registry_id);
                    *self = Self::Mapped(Box::new(new_palette));
                }
            }
            Self::Mapped(mapped) => {
                let original_id = mapped.get_id(x, y, z);
                if mapped.should_downgrade(registry_id, original_id) {
                    *self = Self::Single(SinglePalettedContainer { registry_id })
                } else if mapped.should_upgrade(registry_id, original_id) {
                    let mut new_palette =
                        DirectPalettedContainer::from_abstract_cube(mapped.build_direct_data());
                    new_palette.set_id(x, y, z, registry_id);
                    *self = Self::Direct(Box::new(new_palette));
                } else {
                    mapped.set_id(x, y, z, registry_id);
                }
            }
            Self::Direct(direct) => {
                let original_id = direct.get_id(x, y, z);
                direct.set_id(x, y, z, registry_id);
                if direct.should_downgrade(registry_id, original_id) {
                    let new_palette = MappedPalettedContainer::from_direct_entry(&direct.data);
                    *self = Self::Mapped(Box::new(new_palette));
                }
            }
        }
    }
}

impl<
    const DIM: usize,
    const MAP_LOWER_BOUND: usize,
    const MAP_UPPER_BOUND: usize,
    const MAX_BPE: usize,
> Default for PalettedContainer<DIM, MAP_LOWER_BOUND, MAP_UPPER_BOUND, MAX_BPE>
{
    fn default() -> Self {
        Self::Single(SinglePalettedContainer { registry_id: 0 })
    }
}

pub type BlockPalette = PalettedContainer<16, 4, 8, 15>;
pub type BiomePalette = PalettedContainer<4, 1, 3, 6>;

#[cfg(test)]
mod test {
    use std::u16;

    use super::BlockPalette;

    #[test]
    fn test_single() {
        let palette = BlockPalette::default();
        match &palette {
            BlockPalette::Single(single) => {
                assert_eq!(single.registry_id, 0);
            }
            _ => unreachable!(),
        }

        assert_eq!(palette.get_id(0, 0, 0), 0);
    }

    #[test]
    fn test_single_upgrade() {
        let mut palette = BlockPalette::default();
        palette.set_id(0, 0, 0, 1);

        assert!(matches!(palette, BlockPalette::Mapped(_)));

        assert_eq!(palette.get_id(0, 0, 0), 1);
        assert_eq!(palette.get_id(0, 0, 1), 0);
    }

    #[test]
    fn test_single_upgrade_downgrade() {
        let mut palette = BlockPalette::default();
        palette.set_id(0, 0, 0, 1);

        assert!(matches!(palette, BlockPalette::Mapped(_)));

        assert_eq!(palette.get_id(0, 0, 0), 1);
        assert_eq!(palette.get_id(0, 0, 1), 0);

        palette.set_id(0, 0, 0, 0);
        match &palette {
            BlockPalette::Single(single) => {
                assert_eq!(single.registry_id, 0);
            }
            _ => unreachable!(),
        }

        assert_eq!(palette.get_id(0, 0, 0), 0);
    }

    #[test]
    fn test_map_max() {
        // Max mapping for blocks is 8 bits = 255;
        let max_count = 255u8;
        let mut count = 0;

        let mut palette = BlockPalette::default();
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    if count == max_count {
                        break;
                    }
                    palette.set_id(x, y, z, count as u16);
                    count += 1;
                }
            }
        }

        // Make sure its still a map
        assert!(matches!(palette, BlockPalette::Mapped(_)));

        // Make sure everything matches
        let mut count = 0;
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    if count == max_count {
                        break;
                    }
                    assert_eq!(palette.get_id(x, y, z), count as u16);
                    count += 1;
                }
            }
        }

        // Edge case of replacing something to get the same BPE
        palette.set_id(0, 0, 1, u16::MAX);

        // Make sure its still a map
        assert!(matches!(palette, BlockPalette::Mapped(_)));

        // Make sure everything matches
        let mut count = 0;
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    if count == max_count {
                        break;
                    }

                    let check = if count == 1 { u16::MAX } else { count as u16 };
                    assert_eq!(palette.get_id(x, y, z), check);
                    count += 1;
                }
            }
        }
    }

    #[test]
    fn test_direct_threshold() {
        let max_count = 256u16;
        let mut count = 0;

        let mut palette = BlockPalette::default();
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    if count == max_count {
                        break;
                    }
                    palette.set_id(x, y, z, count);
                    count += 1;
                }
            }
        }

        assert!(matches!(palette, BlockPalette::Direct(_)));

        // Make sure everything matches
        let mut count = 0;
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    if count == max_count {
                        break;
                    }
                    assert_eq!(palette.get_id(x, y, z), count);
                    count += 1;
                }
            }
        }
    }

    #[test]
    fn test_fill_to_direct_to_single() {
        let mut count = 0;
        let mut palette = BlockPalette::default();

        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    palette.set_id(x, y, z, count);
                    count += 1;
                }
            }
        }

        assert!(matches!(palette, BlockPalette::Direct(_)));

        // Make sure everything matches
        let mut count = 0;
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    assert_eq!(palette.get_id(x, y, z), count);
                    count += 1;
                }
            }
        }

        // Check downgrade to map
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    if x == 0 && y == 0 && z == 0 {
                        palette.set_id(x, y, z, 0);
                    } else {
                        palette.set_id(x, y, z, u16::MAX);
                    }
                }
            }
        }

        assert!(matches!(palette, BlockPalette::Mapped(_)));
        for x in 0..BlockPalette::SIZE {
            for y in 0..BlockPalette::SIZE {
                for z in 0..BlockPalette::SIZE {
                    if x == 0 && y == 0 && z == 0 {
                        assert_eq!(palette.get_id(x, y, z), 0);
                    } else {
                        assert_eq!(palette.get_id(x, y, z), u16::MAX);
                    }
                }
            }
        }

        palette.set_id(0, 0, 0, u16::MAX);
        assert!(matches!(palette, BlockPalette::Single(_)));
        assert_eq!(palette.get_id(0, 0, 0), u16::MAX);
    }
}
