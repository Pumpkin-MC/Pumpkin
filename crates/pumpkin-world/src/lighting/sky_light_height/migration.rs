//! Persist cut height in `PumpkinCustomData`. v1 key + geometry tag; unusable values recompute.

use super::SkyLightHeight;
use crate::chunk::ChunkData;
use pumpkin_nbt::tag::NbtTag;
use std::sync::atomic::Ordering;

/// Lazy migration for the sky light cut height.
///
/// Computed once on first access, then cached in RAM and stored persistently in
/// `PumpkinCustomData`.
/// The NBT value itself shows that the feature has already run once for this chunk.
/// No extra flag.
pub struct SkyLightHeightMigration;

impl SkyLightHeightMigration {
    const NAMESPACE: &'static str = "pumpkin:optimization";

    /// Format version, lives in the key name. Other versions are overwritten on the next chunk update.
    pub const VERSION: u8 = 1;
    pub(super) const KEY: &'static str = "sky_light_height_v1";

    /// Keys of earlier versions that need to be discarded.
    ///
    /// Only [`Self::KEY`] is ever read -> no fallback chain.
    ///
    /// On a v2, add `"sky_light_height_v1"` here.
    const LEGACY_KEYS: [&'static str; 0] = [];

    /// Only the low 24 bits are the value; bits 24-31 of the persisted `Int` carry
    /// the geometry tag.
    const VALUE_MASK: u32 = 0x00FF_FFFF;
    const GEOMETRY_SHIFT: u32 = 24;

    /// Packs the chunk geometry a value was computed under into 8 bits.
    ///
    /// The cut is encoded relative to `min_y` and the chunk height (half + fraction).
    ///
    /// `None` = geometry not representable
    pub(super) fn geometry_tag(min_y: i32, chunk_height: i32) -> Option<u8> {
        if min_y % 16 != 0 || chunk_height % 16 != 0 {
            return None;
        }
        let sections = chunk_height / 16;
        let base = min_y / 16;
        if !(1..=31).contains(&sections) || !(-4..=3).contains(&base) {
            return None;
        }
        Some((sections as u8) | (((base + 4) as u8) << 5))
    }

    fn chunk_geometry_tag(chunk: &ChunkData) -> Option<u8> {
        Self::geometry_tag(chunk.section.min_y, SkyLightHeight::chunk_height(chunk))
    }

    /// Fast flag check on chunk load (0.01ms)
    /// Is there a stored value? Does not touch RAM state.
    #[must_use]
    pub fn fast_load_flag(chunk: &ChunkData) -> bool {
        chunk.has_custom_data(Self::NAMESPACE, Self::KEY)
    }

    /// Reads the persisted value, if one is there and valid for this chunk.
    ///
    /// None or other versions are ignored and overwritten on the next chunk update
    pub fn load_persisted(chunk: &ChunkData) -> Option<SkyLightHeight> {
        let expected = Self::chunk_geometry_tag(chunk)?;
        let Some(NbtTag::Int(v)) = chunk.get_custom_data(Self::NAMESPACE, Self::KEY) else {
            return None;
        };
        let stored = v as u32;
        if (stored >> Self::GEOMETRY_SHIFT) as u8 != expected {
            return None; // Different world height: the value is no longer readable.
        }
        let value = stored & Self::VALUE_MASK;
        if value == 0 {
            return None;
        }
        Some(SkyLightHeight::from_raw(value))
    }

    /// Returns the cached/loaded/computed sky light cut height for this chunk,
    /// computing and persisting it on first access only.
    pub fn ensure_lazy(
        chunk: &ChunkData,
        compute: impl FnOnce() -> SkyLightHeight,
    ) -> SkyLightHeight {
        let cached = chunk.sky_light_height_cache.load(Ordering::Relaxed);
        if cached != 0 {
            return SkyLightHeight::from_raw(cached);
        }

        if let Some(height) = Self::load_persisted(chunk) {
            return Self::install(chunk, height);
        }

        let mut height = compute();
        // raw() == 0 collides with the "not cached" sentinel; nudge it off zero.
        if height.raw() == 0 {
            height = height.with_hex_approx_bumped(1);
        }

        let installed = Self::install(chunk, height);
        if installed == height {
            Self::persist(chunk, height);
        }

        installed
    }

    /// Publishes a derived value unless another worker got there first. A plain store
    /// would let a thread that began computing before a divergence was found overwrite
    /// that flag; both views are equally right, so first one wins.
    fn install(chunk: &ChunkData, height: SkyLightHeight) -> SkyLightHeight {
        match chunk.sky_light_height_cache.compare_exchange(
            0,
            height.raw(),
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => height,
            Err(existing) => SkyLightHeight::from_raw(existing),
        }
    }

    /// Persists the given cut height to `PumpkinCustomData`, with geometry tag in bits 24-31.
    ///
    /// Writes without marking the chunk dirty -> the value is fully
    /// derivable from the chunk.
    pub fn persist(chunk: &ChunkData, height: SkyLightHeight) {
        let Some(tag) = Self::chunk_geometry_tag(chunk) else {
            return; // Unverifiable geometry: better to persist nothing at all.
        };
        let stored = (height.raw() & Self::VALUE_MASK) | (u32::from(tag) << Self::GEOMETRY_SHIFT);
        chunk.set_derived_custom_data(Self::NAMESPACE, Self::KEY, NbtTag::Int(stored as i32));

        // Discard leftovers of older format versions instead of carrying them forever.
        for legacy in Self::LEGACY_KEYS {
            if chunk.has_custom_data(Self::NAMESPACE, legacy) {
                chunk.remove_custom_data(Self::NAMESPACE, legacy);
            }
        }
    }

    /// Lazy runtime: computes from the chunk itself on first access.
    pub fn get(chunk: &ChunkData) -> SkyLightHeight {
        Self::ensure_lazy(chunk, || SkyLightHeight::compute_from_chunk(chunk))
    }

    /// Marks a quadrant as diverged and writes it through to cache and NBT. No-op while
    /// nothing is cached
    /// the next computation sees the divergence anyway
    pub fn mark_quadrant_diverged(chunk: &ChunkData, local_x: i32, local_z: i32) {
        // Read-modify-write on one word: load and store would keep only one of two
        // quadrants discovered in the same moment.
        let previous = chunk.sky_light_height_cache.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |cached| {
                if cached == 0 {
                    return None; // Nothing cached; the first computation sees it.
                }
                let marked =
                    SkyLightHeight::from_raw(cached).with_quadrant_diverged(local_x, local_z);
                (marked.raw() != cached).then_some(marked.raw())
            },
        );

        if let Ok(previous) = previous {
            let marked =
                SkyLightHeight::from_raw(previous).with_quadrant_diverged(local_x, local_z);
            Self::persist(chunk, marked);
        }
    }

    /// Persist the value if something has been computed.
    pub fn ensure_persisted(chunk: &ChunkData) {
        let cached = chunk.sky_light_height_cache.load(Ordering::Relaxed);
        if cached == 0 {
            return;
        }
        Self::persist(chunk, SkyLightHeight::from_raw(cached));
    }
}
