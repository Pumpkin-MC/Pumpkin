use std::num::NonZeroU8;

use pumpkin_util::math::vector2::Vector2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cylindrical {
    pub center: Vector2<i32>,
    pub view_distance: NonZeroU8,
}

impl Cylindrical {
    #[must_use]
    pub const fn new(center: Vector2<i32>, view_distance: NonZeroU8) -> Self {
        Self {
            center,
            view_distance,
        }
    }

    pub fn for_each_changed_chunk(
        old_cylindrical: Self,
        new_cylindrical: Self,
        newly_included: &mut Vec<Vector2<i32>>,
        just_removed: &mut Vec<Vector2<i32>>,
    ) {
        for new_cylindrical_chunk in new_cylindrical.all_chunks_within() {
            if !old_cylindrical.is_within_distance(new_cylindrical_chunk.x, new_cylindrical_chunk.y)
            {
                newly_included.push(new_cylindrical_chunk);
            }
        }

        for old_cylindrical_chunk in old_cylindrical.all_chunks_within() {
            if !new_cylindrical.is_within_distance(old_cylindrical_chunk.x, old_cylindrical_chunk.y)
            {
                just_removed.push(old_cylindrical_chunk);
            }
        }
    }

    #[allow(dead_code)]
    const fn left(&self) -> i32 {
        self.center.x - self.view_distance.get() as i32 - 1
    }

    #[allow(dead_code)]
    const fn bottom(&self) -> i32 {
        self.center.y - self.view_distance.get() as i32 - 1
    }

    #[allow(dead_code)]
    const fn right(&self) -> i32 {
        self.center.x + self.view_distance.get() as i32 + 1
    }

    #[allow(dead_code)]
    const fn top(&self) -> i32 {
        self.center.y + self.view_distance.get() as i32 + 1
    }

    #[must_use]
    pub fn is_within_distance(&self, x: i32, z: i32) -> bool {
        if self.view_distance.get() == 1 {
            return false;
        }
        let rel_x = ((x - self.center.x).abs() as i64 - 2).max(0);
        let rel_z = ((z - self.center.y).abs() as i64 - 2).max(0);

        let hyp_sqr = rel_x * rel_x + rel_z * rel_z;
        //The view distance should be converted to i64 first because u8 * u8 can overflow
        hyp_sqr < (self.view_distance.get() as i64).pow(2)
    }

    /// Returns a precomputed list of relative chunk offset pairs `(dx, dy)` for a given view distance.
    #[must_use]
    #[inline]
    pub fn get_offsets(view_distance: u8) -> &'static [(i8, i8)] {
        let idx =
            (view_distance as usize).min(pumpkin_data::chunk_view_lut::MAX_VIEW_DISTANCE as usize);
        pumpkin_data::chunk_view_lut::CHUNK_VIEW_LUT[idx]
    }

    /// Returns an iterator of all chunks within this cylinder using the precomputed LUT
    #[must_use]
    pub fn all_chunks_within(self) -> impl ExactSizeIterator<Item = Vector2<i32>> {
        let offsets = Self::get_offsets(self.view_distance.get());
        offsets.iter().map(move |&(dx, dy)| {
            Vector2::new(self.center.x + i32::from(dx), self.center.y + i32::from(dy))
        })
    }
}

#[cfg(test)]
mod test {

    use std::num::NonZeroU8;

    use super::Cylindrical;
    use pumpkin_util::math::vector2::Vector2;

    #[test]
    fn bounds() {
        let mut cylinder = Cylindrical::new(Vector2::new(0, 0), NonZeroU8::new(1).unwrap());

        for view_distance in 1..=32 {
            cylinder.view_distance = NonZeroU8::new(view_distance).unwrap();

            for chunk in cylinder.all_chunks_within() {
                assert!(chunk.x >= cylinder.left() && chunk.x <= cylinder.right());
                assert!(chunk.y >= cylinder.bottom() && chunk.y <= cylinder.top());
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
        let mut cylinder = Cylindrical::new(Vector2::new(0, 0), NonZeroU8::new(1).unwrap());

        for distance in 1..=64 {
            cylinder.view_distance = NonZeroU8::new(distance).unwrap();
            let chunks = cylinder.all_chunks_within();
            let estimated_capacity = ((distance as usize + 3).pow(2) * 3167) >> 10;

            assert!(estimated_capacity >= chunks.len(),);
        }
    }
}
