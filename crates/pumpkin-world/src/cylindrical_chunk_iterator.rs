use std::num::NonZero;

use pumpkin_util::math::vector2::Vector2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cylindrical {
    pub center: Vector2<i32>,
    pub view_distance: NonZero<u8>,
}

impl Cylindrical {
    #[must_use]
    pub const fn new(center: Vector2<i32>, view_distance: NonZero<u8>) -> Self {
        Self {
            center,
            view_distance,
        }
    }

    #[inline]
    pub fn changed_chunks(
        old: Self,
        new: Self,
    ) -> (
        impl Iterator<Item = Vector2<i32>>,
        impl Iterator<Item = Vector2<i32>>,
    ) {
        let loading = new
            .all_chunks_within()
            .filter(move |c| !old.is_within_distance(c.x, c.y));

        let unloading = old
            .all_chunks_within()
            .filter(move |c| !new.is_within_distance(c.x, c.y));

        (loading, unloading)
    }

    #[must_use]
    #[inline]
    pub const fn left(&self) -> i32 {
        self.center.x - self.view_distance.get() as i32 - 1
    }

    #[must_use]
    #[inline]
    pub const fn bottom(&self) -> i32 {
        self.center.y - self.view_distance.get() as i32 - 1
    }

    #[must_use]
    #[inline]
    pub const fn right(&self) -> i32 {
        self.center.x + self.view_distance.get() as i32 + 1
    }

    #[must_use]
    #[inline]
    pub const fn top(&self) -> i32 {
        self.center.y + self.view_distance.get() as i32 + 1
    }

    #[must_use]
    #[inline]
    pub const fn is_within_distance(&self, x: i32, z: i32) -> bool {
        let vd = self.view_distance.get() as i64;
        if vd == 1 {
            return false;
        }
        let dx = (x - self.center.x).abs() as i64 - 2;
        let dz = (z - self.center.y).abs() as i64 - 2;
        let rel_x = if dx > 0 { dx } else { 0 };
        let rel_z = if dz > 0 { dz } else { 0 };

        let hyp_sqr = rel_x * rel_x + rel_z * rel_z;
        hyp_sqr < vd * vd
    }

    /// Returns a precomputed list of relative chunk offset pairs `(dx, dy)` for a given view distance.
    #[must_use]
    #[inline]
    pub const fn get_offsets(view_distance: u8) -> &'static [(i8, i8)] {
        let vd = view_distance as usize;
        let max = pumpkin_data::chunk_view_lut::MAX_VIEW_DISTANCE as usize;
        let idx = if vd < max { vd } else { max };
        pumpkin_data::chunk_view_lut::CHUNK_VIEW_LUT[idx]
    }

    /// Returns an iterator of all chunks within this cylinder using the precomputed LUT
    #[must_use]
    #[inline]
    pub fn all_chunks_within(self) -> impl ExactSizeIterator<Item = Vector2<i32>> {
        let offsets = Self::get_offsets(self.view_distance.get());
        offsets.iter().map(move |&(dx, dy)| {
            Vector2::new(self.center.x + i32::from(dx), self.center.y + i32::from(dy))
        })
    }
}

#[cfg(test)]
mod test {

    use std::num::NonZero;

    use super::Cylindrical;
    use pumpkin_util::math::vector2::Vector2;

    #[test]
    fn bounds() {
        let mut cylinder = Cylindrical::new(Vector2::new(0, 0), NonZero::new(1).unwrap());

        for view_distance in 1..=32 {
            cylinder.view_distance = NonZero::new(view_distance).unwrap();

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
        let mut cylinder = Cylindrical::new(Vector2::new(0, 0), NonZero::new(1).unwrap());

        for distance in 1..=64 {
            cylinder.view_distance = NonZero::new(distance).unwrap();
            let chunks = cylinder.all_chunks_within();
            let estimated_capacity = ((distance as usize + 3).pow(2) * 3167) >> 10;

            assert!(estimated_capacity >= chunks.len(),);
        }
    }
}
