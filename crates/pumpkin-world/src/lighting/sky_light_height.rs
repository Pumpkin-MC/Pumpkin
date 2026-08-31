//! Sky Light Cut Height caching 
//!
//! Speichert pro Chunk den niedrigsten Y-Wert, unterhalb dessen kein offener Himmel
//! existiert, sodass Aktualisierungen des Skylights unterhalb dieser Höhe zur Laufzeit
//! verworfen werden, anstatt berechnet zu werden.
//! 
//! Ein einzelner, chunkweiter Wert würde durch einen
//! einzigen 1x1 Loch oder eine Schlucht heruntergezogen werden,
//! daher wird der Wert mit 4 Quadranten (NW, NE, SW, SE) gepaart, die jeweils eine eigene Flag haben.
//! Ein Flag bedeutet, dass der Quadrant stärker
//! als der Schwellenwert vom Chunk abweicht und auf eine tatsächliche Überprüfung zurückfällt.

use crate::chunk::ChunkData;
use pumpkin_nbt::tag::NbtTag;
use std::sync::atomic::Ordering;

/// Ab wie vielen Blöcken wird ein Quadrant unoptimiert.
pub const QUADRANT_DIVERGENCE_THRESHOLD: i32 = 30;

/// 24-bit Sky Light Cut Height
///
/// Bytes 0-1 (bits 0-15) -> hexadecimal grob und fein Wert
/// Byte 2 (bits 16-23) -> Flags half, `has_surface_water`, 4x quadrant divergence, 2 reserviert
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkyLightHeight(u32);

impl SkyLightHeight {
    const HEX_APPROX_MASK: u32 = 0x0000_FFFF;
    const HEX_APPROX_STEPS: u32 = 1 << 16;

    const HALF_SHIFT: u32 = 16;
    const FLAG_HALF: u32 = 1 << Self::HALF_SHIFT;

    const SURFACE_WATER_SHIFT: u32 = 17;
    const FLAG_SURFACE_WATER: u32 = 1 << Self::SURFACE_WATER_SHIFT;

    const QUADRANT_NW_SHIFT: u32 = 18;
    const QUADRANT_NE_SHIFT: u32 = 19;
    const QUADRANT_SW_SHIFT: u32 = 20;
    const QUADRANT_SE_SHIFT: u32 = 21;
    const FLAG_QUADRANT_NW: u32 = 1 << Self::QUADRANT_NW_SHIFT;
    const FLAG_QUADRANT_NE: u32 = 1 << Self::QUADRANT_NE_SHIFT;
    const FLAG_QUADRANT_SW: u32 = 1 << Self::QUADRANT_SW_SHIFT;
    const FLAG_QUADRANT_SE: u32 = 1 << Self::QUADRANT_SE_SHIFT;

    /// Wraps a raw encoded value from NBT or AtomicCache
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The raw encoded value as stored in the AtomicCache and persisted to NBT
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Encodes Y into a chunk-relative approximation
    #[must_use]
    pub fn encode(y: i32, chunk_min_y: i32, chunk_height: i32) -> Self {
        let relative_y = (y - chunk_min_y).max(0) as u32;
        let chunk_half = (chunk_height / 2).max(1) as u32;

        let half = u32::from(relative_y >= chunk_half);
        let y_in_half = relative_y % chunk_half;
        let hex_approx = (y_in_half * Self::HEX_APPROX_STEPS / chunk_half) & Self::HEX_APPROX_MASK;

        Self(hex_approx | (half << Self::HALF_SHIFT))
    }

    /// Decodes back to an absolute world Y
    #[must_use]
    pub fn decode(self, chunk_min_y: i32, chunk_height: i32) -> i32 {
        let hex_approx = self.0 & Self::HEX_APPROX_MASK;
        let half = i32::from(self.0 & Self::FLAG_HALF != 0);

        let chunk_half = chunk_height / 2;
        let base_y = chunk_min_y + half * chunk_half;
        let y_in_half = (hex_approx * chunk_half as u32 / Self::HEX_APPROX_STEPS) as i32;

        base_y + y_in_half
    }

    /// Bumps the hex approximation by `delta` steps (kein "raw() == 0")
    /// 65536 Möglickeiten mit half -> genug für spätere Anpassungen die das Bauen über dem Highlimit erlauben.
    #[must_use]
    pub const fn with_hex_approx_bumped(self, delta: u32) -> Self {
        let hex_approx = (self.0 & Self::HEX_APPROX_MASK).wrapping_add(delta) & Self::HEX_APPROX_MASK;
        Self((self.0 & !Self::HEX_APPROX_MASK) | hex_approx)
    }

    #[must_use]
    pub const fn has_surface_water(self) -> bool {
        (self.0 & Self::FLAG_SURFACE_WATER) != 0
    }

    #[must_use]
    pub const fn with_surface_water(self, has_water: bool) -> Self {
        if has_water {
            Self(self.0 | Self::FLAG_SURFACE_WATER)
        } else {
            Self(self.0 & !Self::FLAG_SURFACE_WATER)
        }
    }

    const fn quadrant_flag(local_x: i32, local_z: i32) -> u32 {
        match (local_x < 8, local_z < 8) {
            (true, true) => Self::FLAG_QUADRANT_NW,
            (false, true) => Self::FLAG_QUADRANT_NE,
            (true, false) => Self::FLAG_QUADRANT_SW,
            (false, false) => Self::FLAG_QUADRANT_SE,
        }
    }

    /// Whether the quadrant containing chunk-local `(local_x, local_z)` may safely use
    /// the chunk-wide cut height for trivial rejection (`true`), or has diverged from it
    /// by more than [`QUADRANT_DIVERGENCE_THRESHOLD`] and needs a real check (`false`).
    #[must_use]
    pub const fn quadrant_uses_limit(self, local_x: i32, local_z: i32) -> bool {
        (self.0 & Self::quadrant_flag(local_x, local_z)) == 0
    }

    /// Marks the quadrant containing chunk-local `(local_x, local_z)` as diverged from
    /// the chunk-wide cut height, disabling trivial rejection for that quadrant only.
    #[must_use]
    pub const fn with_quadrant_diverged(self, local_x: i32, local_z: i32) -> Self {
        Self(self.0 | Self::quadrant_flag(local_x, local_z))
    }
}

/// Lazy migration for the sky light cut height.
///
/// Beim ersten Zugriff einmalig berechnet, anschließend im RAM zwischengespeichert und in
/// `PumpkinCustomData` dauerhaft gespeichert.
/// NBT selbst zeigt an, dass die Funktionalität bereits einmal beim Chunk geladen wurde.
/// kein neuer flag.
pub struct SkyLightHeightMigration;

impl SkyLightHeightMigration {
    const NAMESPACE: &'static str = "pumpkin:optimization";
    const KEY: &'static str = "sky_light_height_v1";

    /// Fast flag check on chunk load (0.01ms)
    /// Gibt es gepsiecherten Wert? RAM nicht berührt.
    #[must_use]
    pub fn fast_load_flag(chunk: &ChunkData) -> bool {
        chunk.has_custom_data(Self::NAMESPACE, Self::KEY)
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

        if let Some(NbtTag::Int(v)) = chunk.get_custom_data(Self::NAMESPACE, Self::KEY) {
            let height = SkyLightHeight::from_raw(v as u32);
            chunk
                .sky_light_height_cache
                .store(height.raw(), Ordering::Relaxed);
            return height;
        }

        let mut height = compute();
        // raw() == 0 collides with the "not cached" sentinel; nudge it off zero.
        if height.raw() == 0 {
            height = height.with_hex_approx_bumped(1);
        }

        chunk
            .sky_light_height_cache
            .store(height.raw(), Ordering::Relaxed);
        Self::persist(chunk, height);

        height
    }

    /// Persists the given cut height to `PumpkinCustomData`.
    pub fn persist(chunk: &ChunkData, height: SkyLightHeight) {
        chunk.set_custom_data(Self::NAMESPACE, Self::KEY, NbtTag::Int(height.raw() as i32));
    }

    /// Persistäns Wert speichern wenn etwas berechnet wurde.
    pub fn ensure_persisted(chunk: &ChunkData) {
        let cached = chunk.sky_light_height_cache.load(Ordering::Relaxed);
        if cached == 0 {
            return;
        }
        Self::persist(chunk, SkyLightHeight::from_raw(cached));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip_lower_half() {
        let height = SkyLightHeight::encode(-32, -64, 384);
        assert_eq!(height.raw() & SkyLightHeight::FLAG_HALF, 0);
        let decoded = height.decode(-64, 384);
        assert!((decoded - -32).abs() <= 1);
    }

    #[test]
    fn encode_decode_round_trip_upper_half() {
        let height = SkyLightHeight::encode(200, -64, 384);
        assert!(height.raw() & SkyLightHeight::FLAG_HALF != 0);
        let decoded = height.decode(-64, 384);
        assert!((decoded - 200).abs() <= 1);
    }

    #[test]
    fn surface_water_flag_round_trips() {
        let height = SkyLightHeight::encode(64, -64, 384).with_surface_water(true);
        assert!(height.has_surface_water());
        let height = height.with_surface_water(false);
        assert!(!height.has_surface_water());
    }

    #[test]
    fn quadrant_flags_are_independent() {
        let height = SkyLightHeight::encode(64, -64, 384);
        assert!(height.quadrant_uses_limit(0, 0));
        assert!(height.quadrant_uses_limit(15, 15));

        let height = height.with_quadrant_diverged(3, 3);
        assert!(!height.quadrant_uses_limit(0, 0));
        assert!(height.quadrant_uses_limit(12, 0));
        assert!(height.quadrant_uses_limit(0, 12));
        assert!(height.quadrant_uses_limit(12, 12));
    }

    #[test]
    fn sentinel_zero_is_never_produced_by_ensure_lazy() {
        let chunk = ChunkData::empty(0, 0);
        let height = SkyLightHeightMigration::ensure_lazy(&chunk, || SkyLightHeight::from_raw(0));
        assert_ne!(height.raw(), 0);
        assert_eq!(
            chunk.sky_light_height_cache.load(Ordering::Relaxed),
            height.raw()
        );
    }

    #[test]
    fn ensure_lazy_persists_and_reloads() {
        let chunk = ChunkData::empty(0, 0);
        assert!(!SkyLightHeightMigration::fast_load_flag(&chunk));

        let computed = SkyLightHeight::encode(10, -64, 384);
        let height = SkyLightHeightMigration::ensure_lazy(&chunk, || computed);
        assert!(SkyLightHeightMigration::fast_load_flag(&chunk));

        // Reset the in-memory cache to force the NBT-backed reload path.
        chunk.sky_light_height_cache.store(0, Ordering::Relaxed);
        let reloaded = SkyLightHeightMigration::ensure_lazy(&chunk, || {
            panic!("should not recompute once persisted")
        });
        assert_eq!(reloaded, height);
    }
}
