//! Largest-rectangle search used to measure a portal.
//!
//! Faithful port of `net.minecraft.util.BlockUtil` (`BlockUtil.java:23-108`).
//! Vanilla measures both the entry portal and the exit portal with
//! `BlockUtil.getLargestRectangleAround`, never by walking the obsidian frame
//! (`NetherPortalBlock.java:149` and `NetherPortalBlock.java:170`), so the same
//! algorithm is reproduced here rather than approximated.

use pumpkin_util::math::position::BlockPos;

/// Axis selector, mirroring `Direction.Axis` for the cases the portal code uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RectAxis {
    X,
    Y,
    Z,
}

impl RectAxis {
    /// Vanilla `BlockPos.relative(Direction.Axis, int)`.
    #[must_use]
    pub const fn relative(self, pos: BlockPos, amount: i32) -> BlockPos {
        match self {
            Self::X => pos.add(amount, 0, 0),
            Self::Y => pos.add(0, amount, 0),
            Self::Z => pos.add(0, 0, amount),
        }
    }
}

/// Vanilla `BlockUtil.IntBounds` (`BlockUtil.java:126-138`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntBounds {
    min: i32,
    max: i32,
}

/// Vanilla `BlockUtil.FoundRectangle` (`BlockUtil.java:140-152`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoundRectangle {
    /// Vanilla `minCorner`: lowest corner on both scanned axes.
    pub min_corner: BlockPos,
    /// Vanilla `axis1Size`: extent along the first axis (portal width).
    pub axis1_size: i32,
    /// Vanilla `axis2Size`: extent along the second axis (portal height).
    pub axis2_size: i32,
}

/// Vanilla `BlockUtil.getLimit` (`BlockUtil.java:70-75`).
///
/// Note the vanilla ordering: the cursor moves *before* the test, so the block
/// at `start` itself is never tested and the returned count is the number of
/// consecutive matches strictly beyond it.
fn get_limit<F: FnMut(BlockPos) -> bool>(
    test: &mut F,
    start: BlockPos,
    axis: RectAxis,
    step: i32,
    limit: i32,
) -> i32 {
    let mut max = 0;
    while max < limit && test(axis.relative(start, step * (max + 1))) {
        max += 1;
    }
    max
}

/// Vanilla `BlockUtil.getMaxRectangleLocation` (`BlockUtil.java:78-100`).
///
/// Classic largest-rectangle-in-histogram scan. Returns the inclusive column
/// bounds of the best rectangle plus its height.
fn get_max_rectangle_location(columns: &[i32]) -> (IntBounds, i32) {
    let mut max_start = 0i32;
    let mut max_end = 0i32;
    let mut max_height = 0i32;
    let mut stack: Vec<i32> = Vec::with_capacity(columns.len() + 1);
    stack.push(0);

    let column_at = |index: i32| -> i32 {
        usize::try_from(index)
            .ok()
            .and_then(|i| columns.get(i))
            .copied()
            .unwrap_or(0)
    };

    let len = i32::try_from(columns.len()).unwrap_or(i32::MAX);
    for column in 1..=len {
        let height = if column == len { 0 } else { column_at(column) };
        while let Some(&top) = stack.last() {
            let stack_height = column_at(top);
            if height >= stack_height {
                stack.push(column);
                break;
            }
            stack.pop();
            let start = stack.last().map_or(0, |&t| t + 1);
            if stack_height * (column - start) > max_height * (max_end - max_start) {
                max_end = column;
                max_start = start;
                max_height = stack_height;
            }
        }
        if stack.is_empty() {
            stack.push(column);
        }
    }

    (
        IntBounds {
            min: max_start,
            max: max_end - 1,
        },
        max_height,
    )
}

/// Vanilla `BlockUtil.getLargestRectangleAround` (`BlockUtil.java:23-68`).
///
/// Finds the largest axis-aligned rectangle of blocks satisfying `test` that
/// contains `center`. `limit1` / `limit2` cap how far the scan walks along each
/// axis (vanilla passes 21 for both when measuring nether portals).
pub fn get_largest_rectangle_around<F: FnMut(BlockPos) -> bool>(
    center: BlockPos,
    axis1: RectAxis,
    limit1: i32,
    axis2: RectAxis,
    limit2: i32,
    test: &mut F,
) -> FoundRectangle {
    let negative_delta1 = get_limit(test, center, axis1, -1, limit1);
    let positive_delta1 = get_limit(test, center, axis1, 1, limit1);
    let center_index1 = negative_delta1;

    let len = usize::try_from(center_index1 + 1 + positive_delta1).unwrap_or(1);
    let mut bounds_by_axis1 = vec![IntBounds { min: 0, max: 0 }; len];
    let center_slot = usize::try_from(center_index1).unwrap_or(0);
    bounds_by_axis1[center_slot] = IntBounds {
        min: get_limit(test, center, axis2, -1, limit2),
        max: get_limit(test, center, axis2, 1, limit2),
    };
    let center_index2 = bounds_by_axis1[center_slot].min;

    // Each successive column is capped by its neighbour's extent, exactly as
    // vanilla does (BlockUtil.java:39-48), which keeps the scan convex.
    for i in 1..=negative_delta1 {
        let slot = center_slot - usize::try_from(i).unwrap_or(0);
        let last = bounds_by_axis1[slot + 1];
        let column = axis1.relative(center, -i);
        bounds_by_axis1[slot] = IntBounds {
            min: get_limit(test, column, axis2, -1, last.min),
            max: get_limit(test, column, axis2, 1, last.max),
        };
    }
    for i in 1..=positive_delta1 {
        let slot = center_slot + usize::try_from(i).unwrap_or(0);
        let last = bounds_by_axis1[slot - 1];
        let column = axis1.relative(center, i);
        bounds_by_axis1[slot] = IntBounds {
            min: get_limit(test, column, axis2, -1, last.min),
            max: get_limit(test, column, axis2, 1, last.max),
        };
    }

    let mut min_axis1 = 0;
    let mut min_axis2 = 0;
    let mut size_axis1 = 0;
    let mut size_axis2 = 0;
    let mut columns = vec![0i32; bounds_by_axis1.len()];
    for i2 in (0..=center_index2).rev() {
        for (slot, bounds) in bounds_by_axis1.iter().enumerate() {
            let min2 = center_index2 - bounds.min;
            let max2 = center_index2 + bounds.max;
            columns[slot] = if i2 >= min2 && i2 <= max2 {
                max2 + 1 - i2
            } else {
                0
            };
        }
        let (bounds_axis1, new_size_axis2) = get_max_rectangle_location(&columns);
        let new_size_axis1 = 1 + bounds_axis1.max - bounds_axis1.min;
        if new_size_axis1 * new_size_axis2 > size_axis1 * size_axis2 {
            min_axis1 = bounds_axis1.min;
            min_axis2 = i2;
            size_axis1 = new_size_axis1;
            size_axis2 = new_size_axis2;
        }
    }

    let min_corner = axis2.relative(
        axis1.relative(center, min_axis1 - center_index1),
        min_axis2 - center_index2,
    );
    FoundRectangle {
        min_corner,
        axis1_size: size_axis1,
        axis2_size: size_axis2,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FoundRectangle, RectAxis, get_largest_rectangle_around, get_max_rectangle_location,
    };
    use pumpkin_util::math::position::BlockPos;
    use rustc_hash::FxHashSet;

    /// Builds a solid `width` x `height` slab of portal blocks on the X/Y plane.
    fn slab(origin: BlockPos, width: i32, height: i32) -> FxHashSet<BlockPos> {
        let mut set = FxHashSet::default();
        for dx in 0..width {
            for dy in 0..height {
                set.insert(origin.add(dx, dy, 0));
            }
        }
        set
    }

    #[test]
    fn measures_standard_two_by_three_portal() {
        let origin = BlockPos::new(10, 64, -3);
        let blocks = slab(origin, 2, 3);
        // Probe from every interior block: vanilla must report the same rectangle.
        for probe in &blocks {
            let rect =
                get_largest_rectangle_around(*probe, RectAxis::X, 21, RectAxis::Y, 21, &mut |p| {
                    blocks.contains(&p)
                });
            assert_eq!(
                rect,
                FoundRectangle {
                    min_corner: origin,
                    axis1_size: 2,
                    axis2_size: 3,
                }
            );
        }
    }

    #[test]
    fn measures_single_block() {
        let origin = BlockPos::new(0, 0, 0);
        let blocks = slab(origin, 1, 1);
        let rect =
            get_largest_rectangle_around(origin, RectAxis::X, 21, RectAxis::Y, 21, &mut |p| {
                blocks.contains(&p)
            });
        assert_eq!(rect.axis1_size, 1);
        assert_eq!(rect.axis2_size, 1);
        assert_eq!(rect.min_corner, origin);
    }

    #[test]
    fn measures_max_size_portal_on_z_axis() {
        let origin = BlockPos::new(4, 10, 4);
        let mut blocks = FxHashSet::default();
        for dz in 0..21 {
            for dy in 0..21 {
                blocks.insert(origin.add(0, dy, dz));
            }
        }
        let rect = get_largest_rectangle_around(
            origin.add(0, 10, 10),
            RectAxis::Z,
            21,
            RectAxis::Y,
            21,
            &mut |p| blocks.contains(&p),
        );
        assert_eq!(rect.axis1_size, 21);
        assert_eq!(rect.axis2_size, 21);
        assert_eq!(rect.min_corner, origin);
    }

    #[test]
    fn picks_largest_area_in_irregular_shape() {
        // A 4-wide, 2-tall block with a 1x4 spike on the left column.
        // Largest rectangle by area is the 4x2 body (8) not the spike (4).
        let origin = BlockPos::new(0, 0, 0);
        let mut blocks = slab(origin, 4, 2);
        for dy in 2..6 {
            blocks.insert(origin.add(0, dy, 0));
        }
        let rect = get_largest_rectangle_around(
            origin.add(1, 0, 0),
            RectAxis::X,
            21,
            RectAxis::Y,
            21,
            &mut |p| blocks.contains(&p),
        );
        assert_eq!(rect.axis1_size * rect.axis2_size, 8);
        assert_eq!(rect.axis1_size, 4);
        assert_eq!(rect.axis2_size, 2);
        assert_eq!(rect.min_corner, origin);
    }

    #[test]
    fn scan_is_bounded_by_limits() {
        // An infinite plane clipped by the limits: vanilla walks at most
        // `limit` steps per direction, so width caps at 1 + 2 * limit.
        let rect = get_largest_rectangle_around(
            BlockPos::new(0, 0, 0),
            RectAxis::X,
            3,
            RectAxis::Y,
            2,
            &mut |_| true,
        );
        assert_eq!(rect.axis1_size, 7);
        assert_eq!(rect.axis2_size, 5);
        assert_eq!(rect.min_corner, BlockPos::new(-3, -2, 0));
    }

    #[test]
    fn histogram_matches_vanilla_reference() {
        // Textbook histogram: heights 2,1,5,6,2,3 -> best area 10 (5,6 at h=5).
        let (bounds, height) = get_max_rectangle_location(&[2, 1, 5, 6, 2, 3]);
        assert_eq!(height, 5);
        assert_eq!(bounds.min, 2);
        assert_eq!(bounds.max, 3);
    }

    #[test]
    fn histogram_handles_flat_and_empty() {
        let (bounds, height) = get_max_rectangle_location(&[3, 3, 3]);
        assert_eq!(height, 3);
        assert_eq!(bounds.min, 0);
        assert_eq!(bounds.max, 2);

        let (_, zero) = get_max_rectangle_location(&[0, 0]);
        assert_eq!(zero, 0);
    }
}
