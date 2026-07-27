//! Outward square spiral used when placing a new portal.
//!
//! Faithful port of `BlockPos.spiralAround` (`BlockPos.java:439-478`).
//! `PortalForcer.createPortal` walks `spiralAround(origin, 16, EAST, SOUTH)`
//! (`PortalForcer.java:61`), so candidate columns are visited nearest-first in a
//! specific order. Reproducing the order matters: the first acceptable column
//! wins ties, so a box scan would place portals somewhere vanilla would not.

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

/// Iterator over the spiral positions around a center.
pub struct SpiralAround {
    directions: [Vector3<i32>; 4],
    cursor: BlockPos,
    legs: i32,
    leg: i32,
    leg_size: i32,
    leg_index: i32,
}

impl SpiralAround {
    /// Vanilla `BlockPos.spiralAround(center, radius, first, second)`.
    ///
    /// `first` and `second` must lie on different axes, matching vanilla's
    /// `Validate.validState` check (`BlockPos.java:440`).
    #[must_use]
    pub fn new(center: BlockPos, radius: i32, first: Vector3<i32>, second: Vector3<i32>) -> Self {
        debug_assert!(
            (first.x != 0) != (second.x != 0),
            "the two directions cannot be on the same axis"
        );
        Self {
            directions: [
                first,
                second,
                Vector3::new(-first.x, -first.y, -first.z),
                Vector3::new(-second.x, -second.y, -second.z),
            ],
            // Vanilla seeds the cursor one step along `second` so that the first
            // computeNext (which steps by `second.opposite`) lands on `center`.
            cursor: center.offset(second),
            legs: 4 * radius,
            leg: -1,
            leg_size: 0,
            leg_index: 0,
        }
    }
}

impl Iterator for SpiralAround {
    type Item = BlockPos;

    fn next(&mut self) -> Option<BlockPos> {
        // Vanilla moves the cursor before deciding whether the leg is finished
        // (BlockPos.java:462-473); keep that ordering.
        let direction = self.directions[usize::try_from((self.leg + 4) % 4).unwrap_or(0)];
        self.cursor = self.cursor.offset(direction);
        if self.leg_index >= self.leg_size {
            if self.leg >= self.legs {
                return None;
            }
            self.leg += 1;
            self.leg_index = 0;
            self.leg_size = self.leg / 2 + 1;
        }
        self.leg_index += 1;
        Some(self.cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::SpiralAround;
    use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
    use rustc_hash::FxHashSet;

    const EAST: Vector3<i32> = Vector3::new(1, 0, 0);
    const SOUTH: Vector3<i32> = Vector3::new(0, 0, 1);

    fn spiral(center: BlockPos, radius: i32) -> Vec<BlockPos> {
        SpiralAround::new(center, radius, EAST, SOUTH).collect()
    }

    #[test]
    fn starts_at_center() {
        let center = BlockPos::new(7, 64, -3);
        let positions = spiral(center, 16);
        assert_eq!(positions.first().copied(), Some(center));
    }

    #[test]
    fn stays_on_the_starting_plane() {
        let center = BlockPos::new(0, 64, 0);
        for pos in spiral(center, 8) {
            assert_eq!(pos.0.y, 64, "spiral must not change Y");
        }
    }

    #[test]
    fn visits_each_position_once() {
        let positions = spiral(BlockPos::new(0, 0, 0), 6);
        let unique: FxHashSet<BlockPos> = positions.iter().copied().collect();
        assert_eq!(unique.len(), positions.len());
    }

    #[test]
    fn expands_outward_monotonically() {
        // Chebyshev distance from the center must never decrease: that is what
        // makes the "first acceptable column" the nearest one.
        let center = BlockPos::new(0, 0, 0);
        let mut furthest = 0;
        for pos in spiral(center, 10) {
            let ring = pos.0.x.abs().max(pos.0.z.abs());
            assert!(
                ring >= furthest,
                "spiral moved inward: ring {ring} after {furthest}"
            );
            furthest = furthest.max(ring);
        }
        assert_eq!(furthest, 10);
    }

    #[test]
    fn covers_the_full_square_for_small_radius() {
        // A radius-2 spiral must reach every column of the 5x5 square.
        let center = BlockPos::new(0, 0, 0);
        let visited: FxHashSet<BlockPos> = spiral(center, 2).into_iter().collect();
        for dx in -2..=2 {
            for dz in -2..=2 {
                let pos = center.add(dx, 0, dz);
                assert!(visited.contains(&pos), "missing {pos:?}");
            }
        }
    }

    #[test]
    fn second_position_follows_first_direction() {
        // Vanilla's first step out of the center is along `first` (EAST here).
        let center = BlockPos::new(0, 0, 0);
        let positions = spiral(center, 4);
        assert_eq!(positions[1], center.add(1, 0, 0));
    }
}
