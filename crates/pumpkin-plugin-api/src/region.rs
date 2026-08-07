//! Ergonomic helpers for the bulk region block operations on the world.
//!
//! [`World::get_region`](crate::world::World) and
//! [`World::set_region`](crate::world::World) work with a flat array of
//! block-state ids. [`Region`] wraps that array together with the region's
//! bounds so plugins can read and write blocks by coordinate instead of
//! computing flat indices by hand, and [`WorldExt`] bridges the two.

use crate::wit::pumpkin::plugin::common::BlockPos;
use crate::wit::pumpkin::plugin::world::{BlockFlags, World};

/// Returns the corner-wise minimum and maximum of two positions, so callers can
/// pass the two corners of a region in any order.
fn normalize(a: BlockPos, b: BlockPos) -> (BlockPos, BlockPos) {
    (
        BlockPos {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            z: a.z.min(b.z),
        },
        BlockPos {
            x: a.x.max(b.x),
            y: a.y.max(b.y),
            z: a.z.max(b.z),
        },
    )
}

/// Inclusive length along one axis. `hi` is assumed `>= lo` (true after
/// [`normalize`]); the widening avoids overflow on large spans.
fn span(lo: i32, hi: i32) -> u32 {
    ((i64::from(hi) - i64::from(lo)) + 1) as u32
}

/// A cuboid of block-state ids, addressable by coordinate.
///
/// The states are stored in the same order the host uses: `x` outermost, then
/// `z`, then `y` innermost. Read one with [`WorldExt::read_region`], modify it
/// with [`Region::set`], and write it back with [`WorldExt::write_region`].
pub struct Region {
    min: BlockPos,
    max: BlockPos,
    states: Vec<u16>,
}

impl Region {
    /// Creates a region covering the cuboid between `a` and `b` (inclusive, in
    /// any corner order) with every block set to `state`.
    #[must_use]
    pub fn filled(a: BlockPos, b: BlockPos, state: u16) -> Self {
        let (min, max) = normalize(a, b);
        let len = Self::volume(min, max);
        Self {
            min,
            max,
            states: vec![state; len],
        }
    }

    /// Wraps a flat `states` array that was read for the cuboid between `a` and
    /// `b`. Returns `None` if `states` is not exactly one entry per block.
    #[must_use]
    pub fn from_states(a: BlockPos, b: BlockPos, states: Vec<u16>) -> Option<Self> {
        let (min, max) = normalize(a, b);
        if states.len() != Self::volume(min, max) {
            return None;
        }
        Some(Self { min, max, states })
    }

    fn volume(min: BlockPos, max: BlockPos) -> usize {
        (u64::from(span(min.x, max.x))
            * u64::from(span(min.y, max.y))
            * u64::from(span(min.z, max.z))) as usize
    }

    /// The minimum (corner) position of the region.
    #[must_use]
    pub const fn min(&self) -> BlockPos {
        self.min
    }

    /// The maximum (corner) position of the region.
    #[must_use]
    pub const fn max(&self) -> BlockPos {
        self.max
    }

    /// The size of the region along x, y and z.
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32, u32) {
        (
            span(self.min.x, self.max.x),
            span(self.min.y, self.max.y),
            span(self.min.z, self.max.z),
        )
    }

    /// The flat index for an absolute position, if it lies in the region.
    ///
    /// This must match the host's `x` then `z` then `y` iteration order.
    fn index(&self, pos: BlockPos) -> Option<usize> {
        if pos.x < self.min.x
            || pos.x > self.max.x
            || pos.y < self.min.y
            || pos.y > self.max.y
            || pos.z < self.min.z
            || pos.z > self.max.z
        {
            return None;
        }
        let (_, sy, sz) = self.dimensions();
        let x_off = u64::from((pos.x - self.min.x) as u32);
        let y_off = u64::from((pos.y - self.min.y) as u32);
        let z_off = u64::from((pos.z - self.min.z) as u32);
        Some(((x_off * u64::from(sz) + z_off) * u64::from(sy) + y_off) as usize)
    }

    /// Absolute position for an offset from the minimum corner, if in range.
    fn offset(&self, dx: u32, dy: u32, dz: u32) -> Option<BlockPos> {
        let (sx, sy, sz) = self.dimensions();
        if dx >= sx || dy >= sy || dz >= sz {
            return None;
        }
        Some(BlockPos {
            x: self.min.x + dx as i32,
            y: self.min.y + dy as i32,
            z: self.min.z + dz as i32,
        })
    }

    /// The block-state id at an absolute position, if inside the region.
    #[must_use]
    pub fn get(&self, pos: BlockPos) -> Option<u16> {
        self.index(pos).map(|i| self.states[i])
    }

    /// The block-state id at an offset from the region's minimum corner.
    #[must_use]
    pub fn get_relative(&self, dx: u32, dy: u32, dz: u32) -> Option<u16> {
        self.get(self.offset(dx, dy, dz)?)
    }

    /// Sets the block-state id at an absolute position. Returns `false` if the
    /// position lies outside the region.
    pub fn set(&mut self, pos: BlockPos, state: u16) -> bool {
        match self.index(pos) {
            Some(i) => {
                self.states[i] = state;
                true
            }
            None => false,
        }
    }

    /// Sets the block-state id at an offset from the minimum corner. Returns
    /// `false` if the offset lies outside the region.
    pub fn set_relative(&mut self, dx: u32, dy: u32, dz: u32, state: u16) -> bool {
        match self.offset(dx, dy, dz) {
            Some(pos) => self.set(pos, state),
            None => false,
        }
    }

    /// Iterates over every `(position, state)` in the region, in the host's
    /// `x` then `z` then `y` order.
    pub fn iter(&self) -> impl Iterator<Item = (BlockPos, u16)> + '_ {
        let (_, sy, sz) = self.dimensions();
        let plane = u64::from(sy) * u64::from(sz);
        self.states.iter().enumerate().map(move |(i, &state)| {
            let i = i as u64;
            let x_off = i / plane;
            let rem = i % plane;
            let z_off = rem / u64::from(sy);
            let y_off = rem % u64::from(sy);
            let pos = BlockPos {
                x: self.min.x + x_off as i32,
                y: self.min.y + y_off as i32,
                z: self.min.z + z_off as i32,
            };
            (pos, state)
        })
    }

    /// Consumes the region, returning its flat states array in the order
    /// [`World::set_region`](crate::world::World) expects.
    #[must_use]
    pub fn into_states(self) -> Vec<u16> {
        self.states
    }
}

/// Region-oriented convenience methods on [`World`].
pub trait WorldExt {
    /// Reads the cuboid between `a` and `b` (inclusive) into a [`Region`].
    ///
    /// # Errors
    /// Returns the host error string if the region is too large or the read
    /// otherwise fails.
    fn read_region(&self, a: BlockPos, b: BlockPos) -> Result<Region, String>;

    /// Writes a [`Region`] back into the world with the given update flags,
    /// returning the number of blocks that actually changed.
    ///
    /// # Errors
    /// Returns the host error string if the write fails.
    fn write_region(&self, region: &Region, flags: BlockFlags) -> Result<u32, String>;
}

impl WorldExt for World {
    fn read_region(&self, a: BlockPos, b: BlockPos) -> Result<Region, String> {
        let states = self.get_region(a, b)?;
        Region::from_states(a, b, states)
            .ok_or_else(|| "get-region returned an unexpected number of states".to_string())
    }

    fn write_region(&self, region: &Region, flags: BlockFlags) -> Result<u32, String> {
        self.set_region(region.min, region.max, &region.states, flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    #[test]
    fn index_matches_host_x_then_z_then_y_order() {
        // sx=2, sy=3, sz=4 -> 24 blocks; each cell holds its own flat index.
        let region = Region::from_states(pos(0, 0, 0), pos(1, 2, 3), (0..24).collect()).unwrap();
        assert_eq!(region.dimensions(), (2, 3, 4));

        let (_, sy, sz) = region.dimensions();
        for x in 0..2 {
            for z in 0..4 {
                for y in 0..3 {
                    let expected = (x * sz + z) * sy + y;
                    assert_eq!(
                        region.get(pos(x as i32, y as i32, z as i32)),
                        Some(expected as u16)
                    );
                }
            }
        }
    }

    #[test]
    fn wrong_state_count_is_rejected() {
        assert!(Region::from_states(pos(0, 0, 0), pos(1, 1, 1), vec![0; 7]).is_none());
        assert!(Region::from_states(pos(0, 0, 0), pos(1, 1, 1), vec![0; 8]).is_some());
    }

    #[test]
    fn set_and_relative_addressing_roundtrip() {
        let mut region = Region::filled(pos(10, 64, -5), pos(12, 66, -3), 0);
        assert!(region.set(pos(11, 65, -4), 42));
        assert_eq!(region.get(pos(11, 65, -4)), Some(42));
        // Relative (1,1,1) from min (10,64,-5) is the same block.
        assert_eq!(region.get_relative(1, 1, 1), Some(42));
        // Out of bounds is rejected, not panicking.
        assert!(!region.set(pos(100, 0, 0), 1));
        assert!(region.get(pos(100, 0, 0)).is_none());
        assert!(!region.set_relative(9, 0, 0, 1));
    }

    #[test]
    fn iter_yields_positions_in_host_order() {
        let region = Region::filled(pos(0, 0, 0), pos(1, 1, 1), 7);
        let coords: Vec<_> = region.iter().map(|(p, _)| (p.x, p.y, p.z)).collect();
        assert_eq!(
            coords,
            vec![
                (0, 0, 0),
                (0, 1, 0),
                (0, 0, 1),
                (0, 1, 1),
                (1, 0, 0),
                (1, 1, 0),
                (1, 0, 1),
                (1, 1, 1),
            ]
        );
    }
}
