use crate::block_properties::{Axis, Facing, HorizontalAxis, HorizontalFacing};
use pumpkin_util::{
    math::vector3::{Axis as MathAxis, Vector3},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

#[repr(u8)]
#[derive(PartialEq, Clone, Copy, Debug, Hash, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockDirection {
    Down = 0,
    Up,
    North,
    South,
    West,
    East,
}

impl From<MathAxis> for Axis {
    fn from(a: MathAxis) -> Self {
        match a {
            MathAxis::X => Self::X,
            MathAxis::Y => Self::Y,
            MathAxis::Z => Self::Z,
        }
    }
}
impl From<Axis> for MathAxis {
    fn from(a: Axis) -> Self {
        match a {
            Axis::X => Self::X,
            Axis::Y => Self::Y,
            Axis::Z => Self::Z,
        }
    }
}

pub struct InvalidBlockFace;

impl TryFrom<i32> for BlockDirection {
    type Error = InvalidBlockFace;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Down),
            1 => Ok(Self::Up),
            2 => Ok(Self::North),
            3 => Ok(Self::South),
            4 => Ok(Self::West),
            5 => Ok(Self::East),
            _ => Err(InvalidBlockFace),
        }
    }
}

impl BlockDirection {
    #[must_use]
    pub const fn to_index(&self) -> u8 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Down),
            1 => Some(Self::Up),
            2 => Some(Self::North),
            3 => Some(Self::South),
            4 => Some(Self::West),
            5 => Some(Self::East),
            _ => None,
        }
    }

    pub fn random(random: &mut RandomGenerator) -> Self {
        // Vanilla `Direction.getRandom` = `values[nextInt(values.length)]` over all six
        // directions, in declaration order (DOWN, UP, NORTH, SOUTH, WEST, EAST).
        Self::all()[random.next_bounded_i32(Self::all().len() as i32) as usize]
    }

    /// Vanilla `Direction.Plane.HORIZONTAL.getRandomDirection`: `nextInt(4)` over `[NORTH, EAST,
    /// SOUTH, WEST]`.
    ///
    /// The bound is the full length (a `len() - 1` here made the last direction unreachable) and
    /// the order is [`Self::horizontal_worldgen`], not [`Self::horizontal`], so a given draw picks
    /// the direction vanilla picks.
    pub fn random_horizontal(random: &mut RandomGenerator) -> HorizontalFacing {
        let directions = Self::horizontal_worldgen();
        directions[random.next_bounded_i32(directions.len() as i32) as usize]
    }

    #[must_use]
    pub fn by_index(index: usize) -> Option<Self> {
        Self::all().get(index % Self::all().len()).copied()
    }

    #[must_use]
    pub fn to_offset(&self) -> Vector3<i32> {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
        .into()
    }

    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    #[must_use]
    pub const fn positive(&self) -> bool {
        matches!(self, Self::South | Self::East | Self::Up)
    }

    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Down,
            Self::Up,
            Self::North,
            Self::South,
            Self::West,
            Self::East,
        ]
    }
    #[must_use]
    pub const fn update_order() -> [Self; 6] {
        [
            Self::West,
            Self::East,
            Self::Down,
            Self::Up,
            Self::North,
            Self::South,
        ]
    }

    #[must_use]
    pub const fn abstract_block_update_order() -> [Self; 6] {
        [
            Self::West,
            Self::East,
            Self::North,
            Self::South,
            Self::Down,
            Self::Up,
        ]
    }

    #[must_use]
    pub const fn horizontal() -> [HorizontalFacing; 4] {
        [
            HorizontalFacing::North,
            HorizontalFacing::South,
            HorizontalFacing::West,
            HorizontalFacing::East,
        ]
    }

    /// The four horizontal directions in vanilla `Direction.Plane.HORIZONTAL` order NORTH, EAST,
    /// SOUTH, WEST (matching vanilla's `getRandomDirection(random)` indexes with `nextInt(4)`).
    /// Worldgen-parity code that samples or iterates a random horizontal direction MUST use this,
    /// not [`Self::horizontal`] (whose `[North, South, West, East]` order would pick a different
    /// direction for the same draw).
    #[must_use]
    pub const fn horizontal_worldgen() -> [HorizontalFacing; 4] {
        [
            HorizontalFacing::North,
            HorizontalFacing::East,
            HorizontalFacing::South,
            HorizontalFacing::West,
        ]
    }

    #[must_use]
    pub const fn flow_directions() -> [Self; 5] {
        [Self::Down, Self::North, Self::South, Self::West, Self::East]
    }

    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::North | Self::South | Self::West | Self::East)
    }

    #[must_use]
    pub const fn vertical() -> [Self; 2] {
        [Self::Down, Self::Up]
    }

    #[must_use]
    pub const fn to_horizontal_facing(&self) -> Option<HorizontalFacing> {
        match self {
            Self::North => Some(HorizontalFacing::North),
            Self::South => Some(HorizontalFacing::South),
            Self::West => Some(HorizontalFacing::West),
            Self::East => Some(HorizontalFacing::East),
            _ => None,
        }
    }

    #[must_use]
    pub const fn to_horizontal_axis(&self) -> Option<HorizontalAxis> {
        match self {
            Self::North | Self::South => Some(HorizontalAxis::Z),
            Self::West | Self::East => Some(HorizontalAxis::X),
            _ => None,
        }
    }

    #[must_use]
    pub const fn to_cardinal_direction(&self) -> HorizontalFacing {
        match self {
            Self::South => HorizontalFacing::South,
            Self::West => HorizontalFacing::West,
            Self::East => HorizontalFacing::East,
            _ => HorizontalFacing::North,
        }
    }

    #[must_use]
    pub const fn from_cardinal_direction(direction: HorizontalFacing) -> Self {
        match direction {
            HorizontalFacing::North => Self::North,
            HorizontalFacing::South => Self::South,
            HorizontalFacing::West => Self::West,
            HorizontalFacing::East => Self::East,
        }
    }
    #[must_use]
    pub const fn to_axis(&self) -> Axis {
        match self {
            Self::North | Self::South => Axis::Z,
            Self::West | Self::East => Axis::X,
            Self::Up | Self::Down => Axis::Y,
        }
    }

    #[must_use]
    pub const fn to_facing(&self) -> Facing {
        match self {
            Self::North => Facing::North,
            Self::South => Facing::South,
            Self::West => Facing::West,
            Self::East => Facing::East,
            Self::Up => Facing::Up,
            Self::Down => Facing::Down,
        }
    }

    #[must_use]
    pub const fn rotate_clockwise(&self) -> Self {
        match self {
            Self::East => Self::South,
            Self::West => Self::North,
            Self::Up | Self::North => Self::East,
            Self::Down | Self::South => Self::West,
        }
    }

    #[must_use]
    pub const fn rotate_counter_clockwise(&self) -> Self {
        match self {
            Self::West => Self::South,
            Self::East => Self::North,
            Self::Up | Self::North => Self::West,
            Self::Down | Self::South => Self::East,
        }
    }
}

pub trait FacingExt {
    fn to_block_direction(&self) -> BlockDirection;
    fn to_horizontal_facing(&self) -> Option<HorizontalFacing>;
}

impl FacingExt for Facing {
    fn to_block_direction(&self) -> BlockDirection {
        match self {
            Self::North => BlockDirection::North,
            Self::South => BlockDirection::South,
            Self::West => BlockDirection::West,
            Self::East => BlockDirection::East,
            Self::Up => BlockDirection::Up,
            Self::Down => BlockDirection::Down,
        }
    }
    fn to_horizontal_facing(&self) -> Option<HorizontalFacing> {
        match self {
            Self::North => Some(HorizontalFacing::North),
            Self::South => Some(HorizontalFacing::South),
            Self::West => Some(HorizontalFacing::West),
            Self::East => Some(HorizontalFacing::East),
            _ => None,
        }
    }
}

pub trait HorizontalFacingExt {
    fn to_block_direction(&self) -> BlockDirection;
}

impl HorizontalFacingExt for HorizontalFacing {
    fn to_block_direction(&self) -> BlockDirection {
        match self {
            Self::North => BlockDirection::North,
            Self::South => BlockDirection::South,
            Self::West => BlockDirection::West,
            Self::East => BlockDirection::East,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::random::legacy_rand::LegacyRand;
    use pumpkin_util::random::xoroshiro128::Xoroshiro;

    fn legacy(seed: u64) -> RandomGenerator {
        RandomGenerator::Legacy(LegacyRand::from_seed(seed))
    }

    fn xoroshiro(seed: u64) -> RandomGenerator {
        RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed))
    }

    /// Vanilla `Direction.getRandom` indexes `values[nextInt(6)]` over the declaration order DOWN,
    /// UP, NORTH, SOUTH, WEST, EAST. Pinned against `java.util.Random` with seed 12345.
    #[test]
    fn random_matches_java_util_random() {
        let mut random = legacy(12345);
        let drawn: Vec<BlockDirection> = (0..12)
            .map(|_| BlockDirection::random(&mut random))
            .collect();
        use BlockDirection::{Down, East, South, Up, West};
        assert_eq!(
            drawn,
            [
                Up, West, South, Down, Up, West, Up, Down, Up, South, East, Down
            ]
        );
    }

    /// Vanilla `Direction.Plane.HORIZONTAL.getRandomDirection` indexes `nextInt(4)` over NORTH,
    /// EAST, SOUTH, WEST. Pinned against `java.util.Random` with seed 12345.
    #[test]
    fn random_horizontal_matches_java_util_random() {
        let mut random = legacy(12345);
        let drawn: Vec<HorizontalFacing> = (0..8)
            .map(|_| BlockDirection::random_horizontal(&mut random))
            .collect();
        use HorizontalFacing::{East, North, South, West};
        assert_eq!(drawn, [East, South, West, West, West, North, East, North]);
    }

    /// Every production caller reaches these functions through the `Xoroshiro` variant
    /// (feature-stage worldgen RNG, fire spread, gourd stems), while the parity tests above use
    /// `Legacy`. These sequences are pinned from pumpkin's Xoroshiro implementation (whose
    /// generator core is bit-verified against vanilla in `pumpkin-util`) to guard the enum dispatch
    /// and the small-bound `next_bounded_i32` path against regressions.
    #[test]
    fn random_pinned_through_xoroshiro() {
        let mut random = xoroshiro(12345);
        let drawn: Vec<BlockDirection> = (0..12)
            .map(|_| BlockDirection::random(&mut random))
            .collect();
        use BlockDirection::{Down, East, South, Up, West};
        assert_eq!(
            drawn,
            [
                Down, West, East, Down, South, West, Up, South, Up, East, West, East
            ]
        );

        let mut random = xoroshiro(12345);
        let drawn: Vec<HorizontalFacing> = (0..8)
            .map(|_| BlockDirection::random_horizontal(&mut random))
            .collect();
        use HorizontalFacing as H;
        assert_eq!(
            drawn,
            [
                H::North,
                H::West,
                H::West,
                H::North,
                H::South,
                H::South,
                H::North,
                H::South
            ]
        );
    }

    /// With the old `len() - 1` bound the last entry of each array was unreachable.
    #[test]
    fn every_direction_is_reachable() {
        let mut random = legacy(0);
        let mut seen = [false; 6];
        for _ in 0..512 {
            seen[BlockDirection::random(&mut random) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "unreachable direction: {seen:?}");

        let mut random = legacy(0);
        let mut seen_horizontal = [false; 4];
        for _ in 0..512 {
            let index = match BlockDirection::random_horizontal(&mut random) {
                HorizontalFacing::North => 0,
                HorizontalFacing::East => 1,
                HorizontalFacing::South => 2,
                HorizontalFacing::West => 3,
            };
            seen_horizontal[index] = true;
        }
        assert!(
            seen_horizontal.iter().all(|&s| s),
            "unreachable horizontal direction: {seen_horizontal:?}"
        );
    }
}
