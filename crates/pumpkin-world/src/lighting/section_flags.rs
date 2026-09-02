//! Per-section dirty masks for the light passes.
//!
//! A column walk that runs the full world height Level
//! chunks answer from their block palette, which holds a handful of states
//! instead of 4096 blocks, proto chunks have no palette and are
//! swept once per chunk rather than once per column.

use super::luminance_of;
use crate::ProtoChunk;
use crate::chunk_system::Chunk;

/// One bit per section, set when the section can seed a light pass.
#[derive(Clone, Copy, Default)]
pub(super) struct SectionMask(u32);

impl SectionMask {
    /// Sections beyond bit 31 are never skipped: a mask cannot describe them, and claiming
    /// they are empty would drop light.
    #[inline]
    pub(super) const fn contains(self, section: usize) -> bool {
        section >= 32 || (self.0 >> section) & 1 != 0
    }

    #[inline]
    const fn set(&mut self, section: usize) {
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

/// One sweep over the flat block map instead of one per column.
///
/// The map is `x` major, then `y`, with `z` contiguous, so a section is 16 runs of 16 per
/// column of `x`.
fn proto_emitters(proto: &ProtoChunk) -> SectionMask {
    let mut mask = SectionMask::default();
    let height = proto.height() as usize;
    let map = &proto.flat_block_map;

    for x in 0..16usize {
        let column = x * height * 16;
        for y in 0..height {
            let section = y >> 4;
            if mask.contains(section) {
                continue;
            }
            let row = column + y * 16;
            if map[row..row + 16].iter().any(|&id| luminance_of(id) > 0) {
                mask.set(section);
            }
        }
    }
    mask
}
