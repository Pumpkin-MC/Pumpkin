use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

pub use difficulty::Difficulty;
pub use gamemode::GameMode;
pub use permission::PermissionLvl;

pub mod biome;
pub mod difficulty;
pub mod gamemode;
pub mod loot_table;
pub mod math;
pub mod noise;
pub mod permission;
pub mod random;
pub mod registry;
pub mod resource_location;
pub mod serde_enum_as_integer;
pub mod text;
pub mod translation;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeightMap {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[macro_export]
macro_rules! global_path {
    ($path:expr) => {{
        use std::path::Path;
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(file!())
            .parent()
            .unwrap()
            .join($path)
    }};
}

#[macro_export]
macro_rules! read_data_from_file {
    ($path:expr) => {{
        use std::fs;
        use $crate::global_path;
        serde_json::from_str(&fs::read_to_string(global_path!($path)).expect("no data file"))
            .expect("failed to decode data")
    }};
}

/// The minimum number of bits required to represent this number
#[inline]
pub fn encompassing_bits(count: usize) -> u8 {
    if count == 1 {
        1
    } else {
        count.ilog2() as u8 + if count.is_power_of_two() { 0 } else { 1 }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileAction {
    ForcedNameChange,
    UsingBannedSkin,
}

/// Takes a mutable reference of an index and returns a mutable "slice" where we can mutate both at
/// the same time
pub struct MutableSplitSlice<'a, T> {
    start: &'a mut [T],
    end: &'a mut [T],
}

impl<'a, T> MutableSplitSlice<'a, T> {
    pub fn extract_ith(base: &'a mut [T], index: usize) -> (&'a mut T, Self) {
        let (start, end_inclusive) = base.split_at_mut(index);
        let (value, end) = end_inclusive
            .split_first_mut()
            .expect("Index is not in base slice");

        (value, Self { start, end })
    }

    pub fn len(&self) -> usize {
        self.start.len() + self.end.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl<T> Index<usize> for MutableSplitSlice<'_, T> {
    type Output = T;

    #[allow(clippy::comparison_chain)]
    fn index(&self, index: usize) -> &Self::Output {
        if index < self.start.len() {
            &self.start[index]
        } else if index == self.start.len() {
            panic!("We tried to index into the element that was removed");
        } else {
            &self.end[index - self.start.len() - 1]
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct DoublePerlinNoiseParametersCodec {
    #[serde(rename = "firstOctave")]
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

impl<T> IndexMut<usize> for MutableSplitSlice<'_, T> {
    #[allow(clippy::comparison_chain)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.start.len() {
            &mut self.start[index]
        } else if index == self.start.len() {
            panic!("We tried to index into the element that was removed");
        } else {
            &mut self.end[index - self.start.len() - 1]
        }
    }
}

#[macro_export]
macro_rules! assert_eq_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if 2f64 * ($x - $y).abs() > $d * ($x.abs() + $y.abs()) {
            panic!("{} vs {} ({} vs {})", $x, $y, ($x - $y).abs(), $d);
        }
    };
}
