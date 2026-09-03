//! Per-section dirty masks for the light passes.
//!
//! Level chunks answer from their block palette, which holds a handful of states instead of
//! 4096 blocks. Proto chunks have no palette and instead track the mask as blocks are written.

use crate::ProtoChunk;
use crate::chunk_system::Chunk;

/// One bit per section, set when the section can seed a light pass.
#[derive(Clone, Copy, Default)]
pub struct SectionMask(u32);

impl SectionMask {
    /// Sections beyond bit 31 are never skipped: a mask cannot describe them, and claiming
    /// they are empty would drop light.
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

/// Sections that can seed the block light pass.
///
/// A section seeds when it holds a light emitting block. Rim sections also seed from the
/// light they already carry, so a uniform container at or below level 1 is what clears them:
/// level 1 cannot spread, and the pass never lowers a level.
pub(super) fn block_light_seeds(chunk: &Chunk, on_rim: bool) -> SectionMask {
    match chunk {
        Chunk::Proto(proto) => proto_emitters(proto),
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
                for (index, container) in light.block_light.iter().enumerate() {
                    if container.uniform_level().is_none_or(|level| level > 1) {
                        mask.set(index);
                    }
                }
            }
            mask
        }
    }
}

/// Proto chunks keep the mask up to date as blocks are written, so there is nothing to scan.
#[inline]
const fn proto_emitters(proto: &ProtoChunk) -> SectionMask {
    proto.emitter_sections
}
