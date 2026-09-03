//! Section-granular sky fill.
//!
//! Every cell above a chunk's highest surface block holds a full 15, so those sections carry one
//! value instead of a 2 KiB nibble array.

use crate::chunk::format::LightContainer;

/// Splits a chunk's sky sections at the first one that clears every column.
#[derive(Clone, Copy)]
pub(super) struct SkyFill {
    open_sky_start: usize,
}

impl SkyFill {
    /// `tops` are the `WorldSurface` heights of the chunk's own 16x16 columns.
    pub(super) fn from_surface(
        tops: impl IntoIterator<Item = i32>,
        bottom_y: i32,
        sections: usize,
    ) -> Self {
        let highest = tops.into_iter().max().unwrap_or(bottom_y);
        // A column fills its own section from `top + 1`, so only the section above that one is
        // guaranteed uniform.
        let top_section = ((highest + 1 - bottom_y).max(0) as usize) >> 4;
        Self {
            open_sky_start: (top_section + 1).min(sections),
        }
    }

    /// Where the per-column fills can stop; everything from here up is already open sky.
    pub(super) const fn fill_end(self) -> usize {
        self.open_sky_start
    }

    /// Lowest block Y that is uniformly open sky.
    pub(super) const fn open_sky_y(self, bottom_y: i32) -> i32 {
        bottom_y + (self.open_sky_start as i32) * 16
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
        // Surface at y = 70 with bottom_y = -64 -> local 134, section 8.
        let fill = SkyFill::from_surface([60, 70, 65], -64, sky.len());

        assert_eq!(fill.fill_end(), 9);
        assert_eq!(fill.open_sky_y(-64), 80);

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
