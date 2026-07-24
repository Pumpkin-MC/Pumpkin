//! Vanilla 26.2 `net.minecraft.world.level.redstone.Orientation` (CFR).
//!
//! Distinct from jigsaw `block_properties::Orientation`.
//! 48 orientations = 6 up × 4 front × 2 side-bias.

use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::Axis;

/// Vanilla `Orientation.SideBias`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SideBias {
    Left,
    Right,
}

impl SideBias {
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    #[must_use]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 2] {
        [Self::Left, Self::Right]
    }
}

/// Vanilla redstone `Orientation`: up + front + side bias → ordered neighbor list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedstoneOrientation {
    up: BlockDirection,
    front: BlockDirection,
    side: BlockDirection,
    side_bias: SideBias,
    index: u8,
    /// Precomputed `withFront` / `withUp` / `withSideBias` targets (indices into table).
    with_front: [u8; 6],
    with_up: [u8; 6],
    with_side_bias: [u8; 2],
}

/// All 48 valid orientations, indexed like vanilla `Orientation.ORIENTATIONS`.
static ORIENTATIONS: std::sync::LazyLock<[RedstoneOrientation; 48]> =
    std::sync::LazyLock::new(build_orientations);

fn build_orientations() -> [RedstoneOrientation; 48] {
    let mut table: [Option<RedstoneOrientation>; 48] = [None; 48];
    let seed = make_orientation(BlockDirection::Up, BlockDirection::North, SideBias::Left);
    generate_context(seed, &mut table);
    std::array::from_fn(|i| table[i].expect("orientation table incomplete"))
}

/// Build a raw orientation (maps initially point at self index; rewritten by `generate_context`).
fn make_orientation(
    up: BlockDirection,
    front: BlockDirection,
    side_bias: SideBias,
) -> RedstoneOrientation {
    assert_ne!(
        up.to_axis(),
        front.to_axis(),
        "up and front must be on different axes"
    );
    let side = compute_side(up, front, side_bias);
    let index = generate_index(up, front, side_bias) as u8;
    RedstoneOrientation {
        up,
        front,
        side,
        side_bias,
        index,
        with_front: [index; 6],
        with_up: [index; 6],
        with_side_bias: [index; 2],
    }
}

/// Vanilla `Orientation.generateContext` — fills the 48-slot table and transition maps.
fn generate_context(
    mut self_o: RedstoneOrientation,
    lookup: &mut [Option<RedstoneOrientation>; 48],
) -> RedstoneOrientation {
    let idx = self_o.index as usize;
    if let Some(existing) = lookup[idx] {
        return existing;
    }
    lookup[idx] = Some(self_o);

    for bias in SideBias::all() {
        let child = generate_context(make_orientation(self_o.up, self_o.front, bias), lookup);
        self_o.with_side_bias[bias.ordinal()] = child.index;
    }

    for front in BlockDirection::all() {
        let up = if front == self_o.up {
            self_o.front.opposite()
        } else if front == self_o.up.opposite() {
            self_o.front
        } else {
            self_o.up
        };
        let child = generate_context(make_orientation(up, front, self_o.side_bias), lookup);
        self_o.with_front[direction_ordinal(front)] = child.index;
    }

    for up in BlockDirection::all() {
        let front = if up == self_o.front {
            self_o.up.opposite()
        } else if up == self_o.front.opposite() {
            self_o.up
        } else {
            self_o.front
        };
        let child = generate_context(make_orientation(up, front, self_o.side_bias), lookup);
        self_o.with_up[direction_ordinal(up)] = child.index;
    }

    lookup[idx] = Some(self_o);
    self_o
}

impl RedstoneOrientation {
    #[must_use]
    pub fn of(up: BlockDirection, front: BlockDirection, side_bias: SideBias) -> Self {
        ORIENTATIONS[generate_index(up, front, side_bias)]
    }

    #[must_use]
    pub fn from_index(index: usize) -> Self {
        ORIENTATIONS[index % 48]
    }

    #[must_use]
    pub const fn get_index(self) -> u8 {
        self.index
    }

    #[must_use]
    pub const fn get_front(self) -> BlockDirection {
        self.front
    }

    #[must_use]
    pub const fn get_up(self) -> BlockDirection {
        self.up
    }

    #[must_use]
    pub const fn get_side(self) -> BlockDirection {
        self.side
    }

    #[must_use]
    pub const fn get_side_bias(self) -> SideBias {
        self.side_bias
    }

    /// Vanilla neighbor order:
    /// opposite(front), front, side, opposite(side), opposite(up), up
    #[must_use]
    pub const fn get_directions(self) -> [BlockDirection; 6] {
        [
            self.front.opposite(),
            self.front,
            self.side,
            self.side.opposite(),
            self.up.opposite(),
            self.up,
        ]
    }

    #[must_use]
    pub fn get_horizontal_directions(self) -> Vec<BlockDirection> {
        self.get_directions()
            .into_iter()
            .filter(|d| d.to_axis() != self.up.to_axis())
            .collect()
    }

    #[must_use]
    pub fn get_vertical_directions(self) -> Vec<BlockDirection> {
        self.get_directions()
            .into_iter()
            .filter(|d| d.to_axis() == self.up.to_axis())
            .collect()
    }

    #[must_use]
    pub fn with_front(self, front: BlockDirection) -> Self {
        ORIENTATIONS[self.with_front[direction_ordinal(front)] as usize]
    }

    #[must_use]
    pub fn with_up(self, up: BlockDirection) -> Self {
        ORIENTATIONS[self.with_up[direction_ordinal(up)] as usize]
    }

    #[must_use]
    pub fn with_side_bias(self, bias: SideBias) -> Self {
        ORIENTATIONS[self.with_side_bias[bias.ordinal()] as usize]
    }

    #[must_use]
    pub fn with_mirror(self) -> Self {
        self.with_side_bias(self.side_bias.opposite())
    }

    /// Vanilla `withFrontPreserveUp`.
    #[must_use]
    pub fn with_front_preserve_up(self, front: BlockDirection) -> Self {
        if front.to_axis() == self.up.to_axis() {
            return self;
        }
        self.with_front(front)
    }

    /// Vanilla `withFrontAdjustSideBias`.
    #[must_use]
    pub fn with_front_adjust_side_bias(self, front: BlockDirection) -> Self {
        let with_front = self.with_front(front);
        if self.front == with_front.side {
            return with_front.with_mirror();
        }
        with_front
    }
}

fn compute_side(up: BlockDirection, front: BlockDirection, bias: SideBias) -> BlockDirection {
    // right = front × up (right-handed); LEFT uses opposite of that.
    // Not `const`: `BlockDirection::to_offset` is not const.
    let fv = front.to_offset();
    let uv = up.to_offset();
    let rx = fv.y * uv.z - fv.z * uv.y;
    let ry = fv.z * uv.x - fv.x * uv.z;
    let rz = fv.x * uv.y - fv.y * uv.x;
    let right = nearest_direction(rx, ry, rz);
    match bias {
        SideBias::Right => right,
        SideBias::Left => right.opposite(),
    }
}

fn nearest_direction(x: i32, y: i32, z: i32) -> BlockDirection {
    let ax = x.unsigned_abs();
    let ay = y.unsigned_abs();
    let az = z.unsigned_abs();
    if ax >= ay && ax >= az {
        if x >= 0 {
            BlockDirection::East
        } else {
            BlockDirection::West
        }
    } else if ay >= ax && ay >= az {
        if y >= 0 {
            BlockDirection::Up
        } else {
            BlockDirection::Down
        }
    } else if z >= 0 {
        BlockDirection::South
    } else {
        BlockDirection::North
    }
}

/// Vanilla `Orientation.generateIndex`.
///
/// Index layout packs `up`, `front_key`, and `side_bias` into 0..47.
/// Explicit parens: `((up << 2) + front_key) << 1 + bias`.
const fn generate_index(up: BlockDirection, front: BlockDirection, side_bias: SideBias) -> usize {
    let front_axis_key: usize = match (up.to_axis(), front.to_axis()) {
        (Axis::Y, Axis::X) => 1,
        (Axis::Y, _) => 0,
        (_, Axis::Y) => 1,
        _ => 0,
    };
    // AxisDirection: NEGATIVE=0 (D/N/W), POSITIVE=1 (U/S/E)
    let front_dir: usize = match front {
        BlockDirection::Down | BlockDirection::North | BlockDirection::West => 0,
        BlockDirection::Up | BlockDirection::South | BlockDirection::East => 1,
    };
    let front_key = (front_axis_key << 1) | front_dir;
    let up_ord = direction_ordinal(up);
    (((up_ord << 2) + front_key) << 1) + side_bias.ordinal()
}

const fn direction_ordinal(d: BlockDirection) -> usize {
    // Vanilla Direction.ordinal: DOWN=0 UP=1 NORTH=2 SOUTH=3 WEST=4 EAST=5
    match d {
        BlockDirection::Down => 0,
        BlockDirection::Up => 1,
        BlockDirection::North => 2,
        BlockDirection::South => 3,
        BlockDirection::West => 4,
        BlockDirection::East => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_48_distinct() {
        let mut seen = [false; 48];
        for o in ORIENTATIONS.iter() {
            assert!(!seen[o.index as usize], "duplicate index {}", o.index);
            seen[o.index as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn of_matches_index() {
        for up in BlockDirection::all() {
            for front in BlockDirection::all() {
                if up.to_axis() == front.to_axis() {
                    continue;
                }
                for bias in SideBias::all() {
                    let o = RedstoneOrientation::of(up, front, bias);
                    assert_eq!(o.get_up(), up);
                    assert_eq!(o.get_front(), front);
                    assert_eq!(o.get_side_bias(), bias);
                    assert_eq!(o.get_index() as usize, generate_index(up, front, bias));
                }
            }
        }
    }

    #[test]
    fn with_front_changes() {
        let o = RedstoneOrientation::of(BlockDirection::Up, BlockDirection::North, SideBias::Left);
        let o2 = o.with_front(BlockDirection::East);
        assert_eq!(o2.get_front(), BlockDirection::East);
        assert_eq!(o2.get_up(), BlockDirection::Up);
    }

    #[test]
    fn with_front_when_parallel_to_up_rotates_up() {
        let o = RedstoneOrientation::of(BlockDirection::Up, BlockDirection::North, SideBias::Left);
        let o2 = o.with_front(BlockDirection::Up);
        assert_eq!(o2.get_front(), BlockDirection::Up);
        assert_eq!(o2.get_up(), BlockDirection::South);
    }

    #[test]
    fn directions_order() {
        let o = RedstoneOrientation::of(BlockDirection::Up, BlockDirection::North, SideBias::Left);
        let dirs = o.get_directions();
        assert_eq!(dirs[0], BlockDirection::South);
        assert_eq!(dirs[1], BlockDirection::North);
        assert_eq!(dirs[4], BlockDirection::Down);
        assert_eq!(dirs[5], BlockDirection::Up);
    }

    #[test]
    fn with_mirror_flips_bias() {
        let o = RedstoneOrientation::of(BlockDirection::Up, BlockDirection::North, SideBias::Left);
        assert_eq!(o.with_mirror().get_side_bias(), SideBias::Right);
        assert_eq!(o.with_mirror().get_side(), o.get_side().opposite());
    }
}
