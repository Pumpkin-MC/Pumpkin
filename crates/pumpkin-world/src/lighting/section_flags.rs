//! Per-section seed masks. Level chunks: palette. Proto: maintained on write.

use crate::chunk::format::LightContainer;
use crate::chunk_system::Chunk;

/// One bit per section. Indices >= 32 stay set so they are never skipped.
#[derive(Clone, Copy, Default)]
pub struct SectionMask(u32);

impl SectionMask {
    #[inline]
    #[must_use]
    pub const fn contains(self, section: usize) -> bool {
        section >= 32 || (self.0 >> section) & 1 != 0
    }

    #[inline]
    pub const fn set(&mut self, section: usize) {
        if section < 32 {
            self.0 |= 1 << section;
        }
    }
}

/// Emitter sections, plus already-lit rim storage (level 1 cannot spread).
pub(super) fn block_light_seeds(chunk: &Chunk, on_rim: bool) -> SectionMask {
    match chunk {
        Chunk::Proto(proto) => {
            let mut mask = proto.emitter_sections;
            if on_rim {
                add_stored_light(&mut mask, &proto.light.block_light);
            }
            mask
        }
        Chunk::Level(level) => {
            let mut mask = SectionMask::default();
            level.section.with_blocks(|sections| {
                for (index, section) in sections.iter().enumerate() {
                    if section.max_luminance() > 0 {
                        mask.set(index);
                    }
                }
            });

            if on_rim {
                let light = level
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                add_stored_light(&mut mask, &light.block_light);
            }
            mask
        }
    }
}

/// Marks sections whose stored light can still spread.
fn add_stored_light(mask: &mut SectionMask, sections: &[LightContainer]) {
    for (index, container) in sections.iter().enumerate() {
        if container.uniform_level().is_none_or(|level| level > 1) {
            mask.set(index);
        }
    }
}
