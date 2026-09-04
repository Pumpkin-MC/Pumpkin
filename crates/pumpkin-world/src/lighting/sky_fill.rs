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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_above_the_highest_column_hold_one_value() {
        let mut sky = vec![LightContainer::Empty(0); 24];
        let fill = SkyFill::from_surface([60, 70, 65], -64, sky.len());

        assert_eq!(fill.fill_end(), 9);

        fill.mark(&mut sky);
        assert!(sky[..9].iter().all(LightContainer::is_empty));
        assert!(sky[9..].iter().all(|c| c.uniform_level() == Some(15)));
    }

    #[test]
    fn a_surface_at_the_world_top_leaves_nothing_uniform() {
        let sections = 24;
        let fill = SkyFill::from_surface([319], -64, sections);
        assert_eq!(fill.fill_end(), sections);

        let mut sky = vec![LightContainer::Empty(0); sections];
        fill.mark(&mut sky);
        assert!(sky.iter().all(|c| c.uniform_level() == Some(0)));
    }
}
