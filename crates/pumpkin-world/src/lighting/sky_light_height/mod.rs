//! Sky Light Cut Height caching
//!
//! Stores per chunk the lowest Y below which no open sky exists, so that sky light
//! updates below that height can be answered at runtime instead of being computed
//! from a full column scan.
//!
//! A single chunk-wide value would be dragged down by one
//! 1x1 hole or a single ravine cutting through it,
//! so the value is paired with 4 quadrants (NW, NE, SW, SE), each carrying its own flag.
//! A set flag means that quadrant deviates further
//! than the threshold from the chunk value and falls back to a real check.

mod migration;
mod value;

#[cfg(test)]
mod tests;

pub use migration::SkyLightHeightMigration;
pub use value::SkyLightHeight;

/// Width of the uncertain tier 3 band, chosen per chunk through the 2 reserve bits.
///
/// The band sits exactly on the surface, where players build and mine, and is the
/// only region that still pays for the expensive column scan.
///
/// Theory: flat terrain gets by with 4 blocks, only real mountains need 32.
/// A fixed value would force the worst case onto every chunk.
pub const SPREAD_SCALES: [i32; 4] = [4, 8, 16, 32];

const DECODE_SAFETY_MARGIN: i32 = 1;

/// Answer from the chunk cache for the open-sky question
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyLightTier {
    /// Tier 1: below the cut
    NoOpenSky,
    /// Tier 2: above the cut plus spread
    OpenSky,
    /// Tier 3: inside the uncertain band, or the quadrant diverged. real check.
    Unknown,
}
