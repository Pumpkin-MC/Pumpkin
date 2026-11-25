//! Uniform 15 above the highest surface. Section-granular.

use crate::chunk::format::LightContainer;

#[derive(Clone, Copy)]
pub(super) struct SkyFill {
    open_sky_start: usize,
}

impl SkyFill {
    /// `tops`: `WorldSurface` of this chunk's 16x16. Fill starts one section above the surface section.
    pub(super) fn from_surface(
        tops: impl IntoIterator<Item = i32>,
        bottom_y: i32,
        sections: usize,
    ) -> Self {
        let highest = tops.into_iter().max().unwrap_or(bottom_y);
        let top_section = ((highest + 1 - bottom_y).max(0) as usize) >> 4;
        Self {
            open_sky_start: (top_section + 1).min(sections),
        }
    }

    pub(super) const fn fill_end(self) -> usize {
        self.open_sky_start
    }

    pub(super) fn mark(self, sky_light: &mut [LightContainer]) {
        for container in &mut sky_light[self.open_sky_start..] {
            *container = LightContainer::Empty(15);
        }
    }
}
