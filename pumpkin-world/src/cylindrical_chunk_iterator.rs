use std::num::NonZeroU8;

use pumpkin_util::math::vector2::Vector2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cylindrical {
    pub center: Vector2<i32>,
    pub view_distance: NonZeroU8,
}

impl Cylindrical {
    pub fn new(center: Vector2<i32>, view_distance: NonZeroU8) -> Self {
        Self {
            center,
            view_distance,
        }
    }

    pub fn for_each_changed_chunk(
        old_cylindrical: Cylindrical,
        new_cylindrical: Cylindrical,
        mut newly_included: impl FnMut(Vector2<i32>),
        mut just_removed: impl FnMut(Vector2<i32>),
    ) {
        for new_cylindrical_chunk in new_cylindrical.all_chunks_within() {
            if !old_cylindrical.is_within_distance(new_cylindrical_chunk.x, new_cylindrical_chunk.z)
            {
                newly_included(new_cylindrical_chunk);
            }
        }

        for old_cylindrical_chunk in old_cylindrical.all_chunks_within() {
            if !new_cylindrical.is_within_distance(old_cylindrical_chunk.x, old_cylindrical_chunk.z)
            {
                just_removed(old_cylindrical_chunk);
            }
        }
    }

    fn left(&self) -> i32 {
        self.center.x - self.view_distance.get() as i32 - 1
    }

    fn bottom(&self) -> i32 {
        self.center.z - self.view_distance.get() as i32 - 1
    }

    fn right(&self) -> i32 {
        self.center.x + self.view_distance.get() as i32 + 1
    }

    fn top(&self) -> i32 {
        self.center.z + self.view_distance.get() as i32 + 1
    }

    pub fn is_within_distance(&self, x: i32, z: i32) -> bool {
        let rel_x = ((x - self.center.x).abs() as i64 - 2).max(0);
        let rel_z = ((z - self.center.z).abs() as i64 - 2).max(0);

        let hyp_sqr = rel_x * rel_x + rel_z * rel_z;
        //The view distance should be converted to i64 first because u8 * u8 can overflow
        hyp_sqr < (self.view_distance.get() as i64).pow(2)
    }

    /// Returns an iterator of all chunks within this cylinder
    pub fn all_chunks_within(&self) -> Vec<Vector2<i32>> {
        // I came up with this values by testing
        // for view distances 2-32 it usually gives 5 - 20 chunks more than needed if the player is on ground
        // this looks scary but this few calculations are definitely faster than ~5 reallocations
        // this part "3167) >> 10" is a replacement for flointing point multiplication
        let estimated_capacity = ((self.view_distance.get() as usize + 3).pow(2) * 3167) >> 10;
        let mut all_chunks = Vec::with_capacity(estimated_capacity);

        for x in self.left()..=self.right() {
            let mut was_in = false;
            'inner: for z in self.bottom()..=self.top() {
                if self.is_within_distance(x, z) {
                    all_chunks.push(Vector2::new(x, z));
                    was_in = true;
                } else if was_in {
                    break 'inner;
                }
            }
        }

        all_chunks
    }
}

#[cfg(test)]
mod test {

    use std::num::NonZeroU8;

    use super::Cylindrical;
    use pumpkin_util::math::vector2::Vector2;

    #[test]
    fn test_bounds() {
        let mut cylinder =
            Cylindrical::new(Vector2::new(0, 0), unsafe { NonZeroU8::new_unchecked(1) });

        for view_distance in 1..=32 {
            cylinder.view_distance = unsafe { NonZeroU8::new_unchecked(view_distance) };

            for chunk in cylinder.all_chunks_within() {
                assert!(chunk.x >= cylinder.left() && chunk.x <= cylinder.right());
                assert!(chunk.z >= cylinder.bottom() && chunk.z <= cylinder.top());
            }

            for x in (cylinder.left() - 2)..=(cylinder.right() + 2) {
                for z in (cylinder.bottom() - 2)..=(cylinder.top() + 2) {
                    if cylinder.is_within_distance(x, z) {
                        assert!(x >= cylinder.left() && x <= cylinder.right());
                        assert!(z >= cylinder.bottom() && z <= cylinder.top());
                    }
                }
            }
        }
    }

    #[test]
    fn all_chunks_within_capacity_estimation() {
        let mut cylinder =
            Cylindrical::new(Vector2::new(0, 0), unsafe { NonZeroU8::new_unchecked(1) });

        for distance in 1..=64 {
            cylinder.view_distance = unsafe { NonZeroU8::new_unchecked(distance) };
            let chunks = cylinder.all_chunks_within();
            let estimated_capacity = ((distance as usize + 3).pow(2) * 3167) >> 10;

            if estimated_capacity < chunks.len() {
                panic!()
            }
        }
    }
}
