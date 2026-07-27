//! Immutable snapshot of the world border, used by portal placement.
//!
//! Portal searches interleave block reads (which await chunk loads) with border
//! checks. Holding the `worldborder` mutex across those awaits is not allowed, so
//! the bounds are copied out once and consulted synchronously afterwards.
//!
//! The checks here follow `WorldBorder`'s own overloads rather than Pumpkin's
//! [`Worldborder::contains_block`], which tests both the block and its `+1`
//! diagonal neighbour. Vanilla's `isWithinBounds(BlockPos)`
//! (`WorldBorder.java:62-64`) tests only the block's own coordinates, and that is
//! the overload every call in `PortalForcer` uses.

use pumpkin_util::math::position::BlockPos;

use crate::world::{World, border::Worldborder};

/// Copy of the border extent, mirroring `WorldBorder.getMinX/getMaxX/...`.
#[derive(Clone, Copy, Debug)]
pub struct BorderSnapshot {
    min_x: f64,
    max_x: f64,
    min_z: f64,
    max_z: f64,
}

impl BorderSnapshot {
    /// Vanilla clamps against `getMaxX() - 1.0E-5` (`WorldBorder.java:107`).
    const EPSILON: f64 = 1.0e-5;

    /// Copies the current bounds out of the world's border.
    ///
    /// The lock is released before this returns, so callers may await freely.
    pub async fn capture(world: &World) -> Self {
        Self::from_border(&*world.worldborder.lock().await)
    }

    #[must_use]
    pub fn from_border(border: &Worldborder) -> Self {
        let half = border.new_diameter / 2.0;
        Self {
            min_x: border.center_x - half,
            max_x: border.center_x + half,
            min_z: border.center_z - half,
            max_z: border.center_z + half,
        }
    }

    /// Vanilla `WorldBorder.isWithinBounds(double x, double z)`
    /// (`WorldBorder.java:86-88`) with a zero margin.
    #[must_use]
    pub fn contains(&self, x: f64, z: f64) -> bool {
        x >= self.min_x && x < self.max_x && z >= self.min_z && z < self.max_z
    }

    /// Vanilla `WorldBorder.isWithinBounds(BlockPos)` (`WorldBorder.java:62-64`).
    #[must_use]
    pub fn contains_block(&self, x: i32, z: i32) -> bool {
        self.contains(f64::from(x), f64::from(z))
    }

    /// Vanilla `WorldBorder.clampToBounds(double, double, double)`
    /// (`WorldBorder.java:98-108`): clamp X/Z into range, leave Y untouched, then
    /// floor to a block position.
    #[must_use]
    pub fn clamp_to_bounds(&self, pos: BlockPos) -> BlockPos {
        self.clamp_coords(f64::from(pos.0.x), f64::from(pos.0.y), f64::from(pos.0.z))
    }

    /// `clampToBounds` for a not-yet-floored position, matching
    /// `NetherPortalBlock.java:138` where the scaled coordinates are still `double`.
    #[must_use]
    pub fn clamp_coords(&self, x: f64, y: f64, z: f64) -> BlockPos {
        let clamped_x = x.clamp(self.min_x, self.max_x - Self::EPSILON);
        let clamped_z = z.clamp(self.min_z, self.max_z - Self::EPSILON);
        BlockPos::floored(clamped_x, y, clamped_z)
    }
}

#[cfg(test)]
mod tests {
    use super::BorderSnapshot;
    use crate::world::border::Worldborder;
    use pumpkin_util::math::position::BlockPos;

    fn bounds(center_x: f64, center_z: f64, diameter: f64) -> BorderSnapshot {
        BorderSnapshot::from_border(&Worldborder::new(center_x, center_z, diameter, 0, 5, 300))
    }

    #[test]
    fn contains_matches_half_open_range() {
        let b = bounds(0.0, 0.0, 100.0);
        assert!(b.contains(0.0, 0.0));
        assert!(b.contains(-50.0, -50.0));
        // Upper edge is exclusive, lower edge inclusive.
        assert!(!b.contains(50.0, 0.0));
        assert!(!b.contains(0.0, 50.0));
        assert!(!b.contains(-50.1, 0.0));
    }

    #[test]
    fn clamp_pulls_outside_positions_inside() {
        let b = bounds(0.0, 0.0, 100.0);
        let clamped = b.clamp_coords(500.0, 70.0, -500.0);
        assert!(b.contains_block(clamped.0.x, clamped.0.z));
        // Y is never clamped by the border.
        assert_eq!(clamped.0.y, 70);
        assert_eq!(clamped.0.x, 49);
        assert_eq!(clamped.0.z, -50);
    }

    #[test]
    fn clamp_leaves_inside_positions_alone() {
        let b = bounds(0.0, 0.0, 100.0);
        let pos = BlockPos::new(10, 64, -20);
        assert_eq!(b.clamp_to_bounds(pos), pos);
    }

    #[test]
    fn clamp_respects_offset_center() {
        let b = bounds(1000.0, -2000.0, 32.0);
        let clamped = b.clamp_coords(0.0, 64.0, 0.0);
        assert!(b.contains_block(clamped.0.x, clamped.0.z));
        assert_eq!(clamped.0.x, 984);
        assert_eq!(clamped.0.z, -2016);
    }

    #[test]
    fn nether_scaling_then_clamp_stays_inside() {
        // Overworld -> Nether is 8:1, so a far-out overworld position maps to an
        // eighth of the distance and must still land inside a small border.
        let b = bounds(0.0, 0.0, 2000.0);
        let clamped = b.clamp_coords(24_000.0 / 8.0, 64.0, -24_000.0 / 8.0);
        assert!(b.contains_block(clamped.0.x, clamped.0.z));
        assert_eq!(clamped.0.x, 999);
        assert_eq!(clamped.0.z, -1000);
    }
}
