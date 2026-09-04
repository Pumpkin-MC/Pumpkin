pub mod engine;

use pumpkin_data::BlockStateId;

/// Opacity of a block state, with air short-circuited before the state lookup.
#[inline]
#[must_use]
pub fn opacity_of(state_id: BlockStateId) -> u8 {
    if state_id == BlockStateId::AIR {
        0
    } else {
        state_id.to_state().opacity
    }
}

/// Emitted block light of a block state, with air short-circuited before the state lookup.
#[inline]
#[must_use]
pub fn luminance_of(state_id: BlockStateId) -> u8 {
    if state_id == BlockStateId::AIR {
        0
    } else {
        state_id.to_state().luminance
    }
}

/// Light level after one propagation step through a block of `opacity`.
///
/// Vanilla `LayerLightEngine`: `level - max(1, opacity)`.
#[inline]
#[must_use]
pub const fn decayed(level: u8, opacity: u8) -> u8 {
    level.saturating_sub(if opacity > 1 { opacity } else { 1 })
}

/// Sky light one block further down: 15 passes through transparent blocks undimmed,
/// anything below that decays like a normal step.
#[inline]
#[must_use]
pub const fn sky_descended(level: u8, opacity: u8) -> u8 {
    if level == 15 && opacity == 0 {
        15
    } else {
        decayed(level, opacity)
    }
}

pub use engine::LightEngine;

mod chunk_access;
pub mod runtime;
mod stats;
pub use runtime::DynamicLightEngine;
pub use stats::LightPassStats;

mod occlusion;
pub mod section_flags;
mod sky_fill;
pub mod sky_light_height;
pub use sky_light_height::{SkyLightHeight, SkyLightHeightMigration};
