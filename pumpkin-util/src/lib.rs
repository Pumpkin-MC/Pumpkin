use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

pub use serde_json;

pub use difficulty::Difficulty;
pub use gamemode::GameMode;
pub use permission::PermissionLvl;

use crate::math::vector3::Axis;

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
pub mod version;
pub mod world_seed;
pub mod y_offset;

pub mod jwt;

/// Represents the different types of height maps used for terrain generation and collision checks.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeightMap {
    /// Topmost block including plants, snow layers, and surface features (used during world generation).
    WorldSurfaceWg,
    /// Topmost solid or liquid block counting as surface.
    WorldSurface,
    /// Lowest solid block in oceans including underwater terrain features (used during world generation).
    OceanFloorWg,
    /// Lowest solid block in oceans, ignoring non-solid features like kelp.
    OceanFloor,
    /// Topmost block that blocks entity motion (ignores leaves).
    MotionBlocking,
    /// Topmost block that blocks entity motion, ignoring leaf blocks.
    MotionBlockingNoLeaves,
}

/// Constructs a global file system path relative to the project root.
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

/// Reads JSON files from the disk. Don't use this for static files!
#[macro_export]
macro_rules! read_data_from_file {
    ($path:expr) => {{
        use $crate::global_path;
        $crate::serde_json::from_str(
            &std::fs::read_to_string(global_path!($path)).expect("no data file"),
        )
        .expect("failed to decode data")
    }};
}

/// Asserts that two floating-point numbers are approximately equal within a given delta.
#[macro_export]
macro_rules! assert_eq_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if 2f64 * ($x - $y).abs() > $d * ($x.abs() + $y.abs()) {
            panic!("{} vs {} ({} vs {})", $x, $y, ($x - $y).abs(), $d);
        }
    };
}

/// The minimum number of bits required to represent this number
#[inline]
#[must_use]
pub fn encompassing_bits(count: usize) -> u8 {
    if count == 1 {
        1
    } else {
        count.ilog2() as u8 + u8::from(!count.is_power_of_two())
    }
}

/// Represents actions applied to a player profile that may require moderation or restrictions.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileAction {
    /// The player's name was forcibly changed by the server or an administrator.
    ForcedNameChange,
    /// The player attempted to use a skin that is banned or not allowed on the server.
    UsingBannedSkin,
}

/// Represents the six possible block-facing directions in a 3D world.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockDirection {
    /// Points downward along the Y axis; often used for blocks attached to the ceiling.
    Down = 0,
    /// Points upward along the Y axis; commonly used for blocks resting on the ground.
    Up,
    /// Points toward the negative Z axis; typically toward the front of a structure.
    North,
    /// Points toward the positive Z axis; typically toward the back of a structure.
    South,
    /// Points toward the negative X axis; commonly used for left-facing orientation.
    West,
    /// Points toward the positive X axis; commonly used for right-facing orientation.
    East,
}

impl BlockDirection {
    /// Returns the principal axis (`X`, `Y`, or `Z`) associated with this direction.
    #[must_use]
    pub const fn get_axis(&self) -> Axis {
        match self {
            Self::Up | Self::Down => Axis::Y,
            Self::North | Self::South => Axis::Z,
            Self::East | Self::West => Axis::X,
        }
    }
}

/// A mutable slice split into three parts: the element at the split index, the start, and the end.
///
/// This allows modifying the selected element while still having access to the surrounding slices.
pub struct MutableSplitSlice<'a, T> {
    /// Elements before the split index.
    start: &'a mut [T],
    /// Elements after the split index.
    end: &'a mut [T],
}

impl<'a, T> MutableSplitSlice<'a, T> {
    /// Extracts the `index`-th element as a mutable reference along with a `MutableSplitSlice` representing the remaining elements.
    ///
    /// # Panics
    /// * if `index` is out of bounds of the base slice.
    pub const fn extract_ith(base: &'a mut [T], index: usize) -> (&'a mut T, Self) {
        let (start, end_inclusive) = base.split_at_mut(index);
        let (value, end) = end_inclusive
            .split_first_mut()
            .expect("Index is not in base slice");

        (value, Self { start, end })
    }

    /// Returns the total number of elements in the split slice (start + removed element + end).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.start.len() + self.end.len() + 1
    }

    /// Returns `false` since the split slice always contains at least the removed element.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl<T> Index<usize> for MutableSplitSlice<'_, T> {
    type Output = T;

    #[expect(clippy::comparison_chain)]
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

/// Codec for deserializing parameters of a double Perlin noise sampler.
#[derive(Deserialize, Clone)]
pub struct DoublePerlinNoiseParametersCodec {
    /// The first octave index (can be negative for lower frequencies).
    #[serde(rename = "firstOctave")]
    pub first_octave: i32,
    /// Amplitude values for each octave, determining the weight of each frequency layer.
    pub amplitudes: Vec<f64>,
}

impl<T> IndexMut<usize> for MutableSplitSlice<'_, T> {
    #[expect(clippy::comparison_chain)]
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

/// Represents the player's dominant hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    /// Usually the player's off-hand.
    Left,
    /// Usually the player's primary hand.
    Right,
}

impl Hand {
    #[must_use]
    pub const fn all() -> [Self; 2] {
        [Self::Right, Self::Left]
    }
}

/// Error type for invalid hand conversion.
pub struct InvalidHand;

impl TryFrom<i32> for Hand {
    type Error = InvalidHand;

    /// Converts an integer into a `Hand`.
    ///
    /// # Parameters
    /// - `0`: `Left`
    /// - `1`: `Right`
    ///
    /// # Errors
    /// Returns `InvalidHand` if the value is not 0 or 1.
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            _ => Err(InvalidHand),
        }
    }
}
