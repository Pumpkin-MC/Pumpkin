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

use crate::ProtoChunk;
use crate::chunk::{ChunkData, ChunkHeightmapType};
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_nbt::tag::NbtTag;
use std::sync::atomic::Ordering;

/// Breite des unsicheren Tier-3-Bands, waehlbar pro Chunk über die 2 Reserve-Bits.
///
/// Das Band liegt genau auf der Oberflaeche, wo gebaut und abgebaut wird, und ist der
/// einzige Bereich der noch den teuren Spaltenscan hat.
/// 
/// Theorie: Flaches Terrain kommt mit 4 Blöcken aus, nur echtes Bergterrain braucht 32.
/// Ein fester Wert  könnte jedem Chunk das Band aufzwingen.
pub const SPREAD_SCALES: [i32; 4] = [4, 8, 16, 32];

/// Groesstes Band das ein Quadrant fürs nutzen
pub const QUADRANT_DIVERGENCE_THRESHOLD: i32 = SPREAD_SCALES[3];

const DECODE_SAFETY_MARGIN: i32 = 1;

/// Informationen vom Chunk Cache, die für Skylight
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyLightTier {
    /// Tier 1: below the cut
    NoOpenSky,
    /// Tier 2: above the cut plus spread
    OpenSky,
    /// Tier 3: inside the uncertain band, or the quadrant diverged. real check.
    Unknown,
}

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

    /// Bits 22-23: Index in [`SPREAD_SCALES`], die Breite des Tier-3-Bands.
    const SPREAD_SHIFT: u32 = 22;
    const SPREAD_MASK: u32 = 0b11 << Self::SPREAD_SHIFT;

    /// Breite des unsicheren Bands ueber dem Cut fuer diesen Chunk.
    #[must_use]
    pub const fn spread(self) -> i32 {
        SPREAD_SCALES[((self.0 & Self::SPREAD_MASK) >> Self::SPREAD_SHIFT) as usize]
    }

    #[must_use]
    const fn with_spread_index(self, index: usize) -> Self {
        Self((self.0 & !Self::SPREAD_MASK) | ((index as u32) << Self::SPREAD_SHIFT))
    }

    /// Wraps a raw encoded value from NBT or `AtomicCache`
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The raw encoded value as stored in the `AtomicCache` and persisted to NBT
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

    /// Bumps the hex approximation by `delta` steps (kein `raw() == 0`)
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
    /// by more than [`SkyLightHeight::spread`] and needs a real check (`false`).
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

    /// AND-Gatter ueber eine Chunk-Grenze.
    ///
    /// An der Grenze traegt der hot Pfad nur, wenn beide angrenzende Quadranten ihn
    /// tragen. NAND fällt auf den echten Check zurück.
    #[must_use]
    pub const fn border_uses_limit(
        self,
        neighbor: Self,
        local_x: i32,
        local_z: i32,
        neighbor_local_x: i32,
        neighbor_local_z: i32,
    ) -> bool {
        self.quadrant_uses_limit(local_x, local_z)
            & neighbor.quadrant_uses_limit(neighbor_local_x, neighbor_local_z)
    }

    /// 3-Tier culling lookup. A non-diverged quadrant guarantees every one of its column
    /// ceilings lies in, below the cut nothing sees sky and above cut+spread everything does.
    /// In between -> Unknown.
    #[must_use]
    pub fn tier(
        self,
        y: i32,
        local_x: i32,
        local_z: i32,
        chunk_min_y: i32,
        chunk_height: i32,
    ) -> SkyLightTier {
        if !self.quadrant_uses_limit(local_x, local_z) {
            return SkyLightTier::Unknown;
        }
        let cut = self.decode(chunk_min_y, chunk_height);
        if y < cut - DECODE_SAFETY_MARGIN {
            SkyLightTier::NoOpenSky
        } else if y > cut + self.spread() + DECODE_SAFETY_MARGIN {
            SkyLightTier::OpenSky
        } else {
            SkyLightTier::Unknown
        }
    }

    const fn quadrant_index(local_x: usize, local_z: usize) -> usize {
        match (local_x < 8, local_z < 8) {
            (true, true) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (false, false) => 3,
        }
    }

    const fn quadrant_flag_by_index(index: usize) -> u32 {
        match index {
            0 => Self::FLAG_QUADRANT_NW,
            1 => Self::FLAG_QUADRANT_NE,
            2 => Self::FLAG_QUADRANT_SW,
            _ => Self::FLAG_QUADRANT_SE,
        }
    }

    /// Highest light-blocking block in a column, or `min_y - 1` if the column has none.
    ///
    /// Starts from the already-maintained `WorldSurface` heightmap and walks down. The
    /// heightmap value alone is not usable:
    /// not air -> light needs "opacity > 0"
    /// glass and leaves sit above their real ceiling and would make the cut too high
    fn column_opaque_ceiling(chunk: &ChunkData, local_x: usize, local_z: usize, from_y: i32) -> i32 {
        let min_y = chunk.section.min_y;
        let mut y = from_y;
        while y >= min_y {
            let id = chunk.section.get_block_absolute_y(local_x, y, local_z);
            if let Some(id) = id
                && BlockState::from_id(id).opacity > 0
            {
                return y;
            }
            y -= 1;
        }
        min_y - 1
    }

    /// Derives the cut height and the 4 quadrant divergence flags from this chunk.
    ///
    /// A quadrant may use the chunk cut only if all of its ceilings fit into
    /// `[cut, cut + spread]`, i.e. the cut must lie in
    /// `[q_max - THRESHOLD, q_min]`. The cut is picked as the point covered by the most
    /// of those 4 intervals (ties resolved upwards, for the largest Tier 1 region)
    #[must_use]
    pub fn compute_from_chunk(chunk: &ChunkData) -> Self {
        let min_y = chunk.section.min_y;
        let mut q_min = [i32::MAX; 4];
        let mut q_max = [i32::MIN; 4];

        {
            let heightmap = chunk
                .heightmap
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let top_y =
                min_y + (chunk.section.count as i32) * crate::chunk::palette::BlockPalette::SIZE as i32 - 1;

            for local_z in 0..16usize {
                for local_x in 0..16usize {
                    let surface = heightmap
                        .get(
                            ChunkHeightmapType::WorldSurface,
                            local_x as i32,
                            local_z as i32,
                            min_y,
                        )
                        .min(top_y);
                    let ceiling = Self::column_opaque_ceiling(chunk, local_x, local_z, surface);
                    let quadrant = Self::quadrant_index(local_x, local_z);
                    q_min[quadrant] = q_min[quadrant].min(ceiling);
                    q_max[quadrant] = q_max[quadrant].max(ceiling);
                }
            }
        }

        Self::solve(&q_min, &q_max, min_y, Self::chunk_height(chunk))
    }

    /// Waehlt aus den 4 Quadranten-Deckenintervallen den Cut, das Band und die
    /// Divergenz-Flags. Gemeinsamer Kern von Worldgen und Laufzeit
    ///
    /// Ein Quadrant kann das Band nur nutzen, wenn alle seine Decken in
    /// `[cut, cut + spread]` liegen, der Cut also in `[q_max - spread, q_min]`.
    #[must_use]
    fn solve(q_min: &[i32; 4], q_max: &[i32; 4], min_y: i32, chunk_height: i32) -> Self {
        let mut best_cut = q_min[0];
        let mut best_covered = 0u32;
        let mut best_spread_index = SPREAD_SCALES.len() - 1;

        for (spread_index, &spread) in SPREAD_SCALES.iter().enumerate() {
            for i in 0..4 {
                if q_max[i] - q_min[i] > spread {
                    continue; // Quadrant passt nicht in Band
                }
                let candidate = q_max[i] - spread;
                let covered = (0..4)
                    .filter(|&j| candidate >= q_max[j] - spread && candidate <= q_min[j])
                    .count() as u32;
                // Mehr Quadranten > schmalere Band > high Cut (large Tier-1)
                let better = covered > best_covered
                    || (covered == best_covered
                        && (spread_index < best_spread_index
                            || (spread_index == best_spread_index && candidate > best_cut)));
                if better {
                    best_covered = covered;
                    best_cut = candidate;
                    best_spread_index = spread_index;
                }
            }
        }

        let spread = SPREAD_SCALES[best_spread_index];
        let mut encoded =
            Self::encode(best_cut, min_y, chunk_height).with_spread_index(best_spread_index);
        for i in 0..4 {
            let usable = best_cut >= q_max[i] - spread && best_cut <= q_min[i];
            if !usable {
                encoded = Self(encoded.0 | Self::quadrant_flag_by_index(i));
            }
        }

        encoded
    }

    /// Wie [`Self::column_opaque_ceiling`], aber auf einem noch nicht fertigen
    /// `ProtoChunk`
    fn proto_column_opaque_ceiling(
        proto: &ProtoChunk,
        local_x: i32,
        local_z: i32,
        from_y: i32,
    ) -> i32 {
        let min_y = i32::from(proto.bottom_y());
        let mut y = from_y.min(min_y + i32::from(proto.height()) - 1);
        while y >= min_y {
            let id = proto.get_block_state_raw(local_x, y - min_y, local_z);
            if id != BlockStateId::AIR && BlockState::from_id(id).opacity > 0 {
                return y;
            }
            y -= 1;
        }
        min_y - 1
    }

    /// Worldgen-Variante von [`Self::compute_from_chunk`].
    ///
    /// muss nach Carvern und Features (Stage `Lighting`), damit Löcher und
    /// Schluchten die bereits im Terrain sind auch in den Quadranten-Flags landen.
    /// Quelle ist `WorldSurface`-Heightmap des `ProtoChunk`
    #[must_use]
    pub fn compute_from_proto(proto: &ProtoChunk) -> Self {
        let min_y = i32::from(proto.bottom_y());
        let mut q_min = [i32::MAX; 4];
        let mut q_max = [i32::MIN; 4];

        for local_z in 0..16i32 {
            for local_x in 0..16i32 {
                // `top_block_height_exclusive` ist exklusiv, der oberste Block liegt eins darunter.
                let surface = proto.top_block_height_exclusive(local_x, local_z) - 1;
                let ceiling = Self::proto_column_opaque_ceiling(proto, local_x, local_z, surface);
                let quadrant = Self::quadrant_index(local_x as usize, local_z as usize);
                q_min[quadrant] = q_min[quadrant].min(ceiling);
                q_max[quadrant] = q_max[quadrant].max(ceiling);
            }
        }

        let computed = Self::solve(&q_min, &q_max, min_y, i32::from(proto.height()));
        // raw() == 0 ist der "nicht gecached"-Sentinel und darf nie persistiert werden.
        if computed.raw() == 0 {
            computed.with_hex_approx_bumped(1)
        } else {
            computed
        }
    }

    /// Highest light-blocking block in one chunk-local column, starting from its
    /// `WorldSurface` heightmap entry. Used to re-check a single column after a block change.
    #[must_use]
    pub fn column_ceiling_at(chunk: &ChunkData, local_x: i32, local_z: i32) -> i32 {
        let min_y = chunk.section.min_y;
        let top_y = min_y + Self::chunk_height(chunk) - 1;
        let surface = {
            let heightmap = chunk
                .heightmap
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            heightmap
                .get(ChunkHeightmapType::WorldSurface, local_x, local_z, min_y)
                .min(top_y)
        };
        Self::column_opaque_ceiling(chunk, local_x as usize, local_z as usize, surface)
    }

    /// Block height of the chunk's section stack
    #[must_use]
    pub const fn chunk_height(chunk: &ChunkData) -> i32 {
        (chunk.section.count as i32) * crate::chunk::palette::BlockPalette::SIZE as i32
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

    /// Lazy runtime: computes from the chunk itself on first access.
    pub fn get(chunk: &ChunkData) -> SkyLightHeight {
        Self::ensure_lazy(chunk, || SkyLightHeight::compute_from_chunk(chunk))
    }

    /// Marks a quadrant as diverged and writes it through to cache and NBT. No-op while
    /// nothing is cached
    /// nächste Berechnung sieht die Divergenz eh
    pub fn mark_quadrant_diverged(chunk: &ChunkData, local_x: i32, local_z: i32) {
        let cached = chunk.sky_light_height_cache.load(Ordering::Relaxed);
        if cached == 0 {
            return;
        }
        let height = SkyLightHeight::from_raw(cached).with_quadrant_diverged(local_x, local_z);
        if height.raw() == cached {
            return; // Already diverged.
        }
        chunk
            .sky_light_height_cache
            .store(height.raw(), Ordering::Relaxed);
        Self::persist(chunk, height);
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
    use pumpkin_data::Block;

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

    /// Fills every column of the chunk with stone from `min_y` up to and including `top`.
    fn fill_terrain(chunk: &ChunkData, top: i32) {
        let min_y = chunk.section.min_y;
        for local_z in 0..16usize {
            for local_x in 0..16usize {
                for y in min_y..=top {
                    chunk.set_block_absolute_y(local_x, y, local_z, Block::STONE.default_state.id);
                }
            }
        }
    }

    fn tier_at(chunk: &ChunkData, height: SkyLightHeight, y: i32, x: i32, z: i32) -> SkyLightTier {
        height.tier(
            y,
            x,
            z,
            chunk.section.min_y,
            SkyLightHeight::chunk_height(chunk),
        )
    }

    #[test]
    fn flat_terrain_splits_into_three_tiers() {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, 60);
        let height = SkyLightHeight::compute_from_chunk(&chunk);

        // Flat: no quadrant diverges, everything can use the chunk cut.
        assert!(height.quadrant_uses_limit(0, 0));
        assert!(height.quadrant_uses_limit(15, 15));

        assert_eq!(tier_at(&chunk, height, 20, 8, 8), SkyLightTier::NoOpenSky);
        assert_eq!(tier_at(&chunk, height, 60, 8, 8), SkyLightTier::Unknown);
        assert_eq!(tier_at(&chunk, height, 200, 8, 8), SkyLightTier::OpenSky);
    }

    /// Flaches Terrain hat keine Streuung, also muss das teure Tier-3-Band auf die
    /// kleinste Stufe schrumpfen
    #[test]
    fn flat_terrain_picks_the_tightest_band() {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, 60);
        let height = SkyLightHeight::compute_from_chunk(&chunk);

        assert_eq!(height.spread(), SPREAD_SCALES[0]);
        // Knapp ueber der Oberflaeche ist bereits Tier 2, nicht mehr das Band.
        assert_eq!(tier_at(&chunk, height, 67, 8, 8), SkyLightTier::OpenSky);
    }

    /// schweizer-Käse Terrain braucht ein breiteres Band, sonst wuerden die Quadranten
    /// alle als abweichend markiert
    #[test]
    fn rough_terrain_widens_the_band_instead_of_diverging() {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, 60);
        // Saeulen bis y=72: Streuung 12, passt in keine der beiden kleinsten Stufen.
        for local_z in (0..16usize).step_by(4) {
            for local_x in (0..16usize).step_by(4) {
                for y in 61..=72 {
                    chunk.set_block_absolute_y(local_x, y, local_z, Block::STONE.default_state.id);
                }
            }
        }

        let height = SkyLightHeight::compute_from_chunk(&chunk);
        assert!(height.spread() >= 12, "band {} too narrow", height.spread());
        assert!(
            height.quadrant_uses_limit(2, 2),
            "widening the band must keep the quadrants usable"
        );
    }

    #[test]
    fn spread_survives_a_round_trip_through_nbt() {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, 60);
        let height = SkyLightHeightMigration::get(&chunk);
        let spread = height.spread();

        chunk.sky_light_height_cache.store(0, Ordering::Relaxed);
        assert_eq!(SkyLightHeightMigration::get(&chunk).spread(), spread);
    }

    /// The cut must follow the highest light-blocking block, not the `WorldSurface`
    /// heightmap: glass is "not air" but transmits, so a surface-derived cut would
    /// trivially reject positions under glass that really do see the sky.
    #[test]
    fn glass_does_not_raise_the_cut() {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, 60);
        for local_z in 0..16usize {
            for local_x in 0..16usize {
                chunk.set_block_absolute_y(local_x, 100, local_z, Block::GLASS.default_state.id);
            }
        }

        let height = SkyLightHeight::compute_from_chunk(&chunk);
        let cut = height.decode(chunk.section.min_y, SkyLightHeight::chunk_height(&chunk));
        assert!(
            cut <= 61,
            "cut {cut} follows the glass at y=100 instead of the stone ceiling at y=60"
        );
        assert_ne!(tier_at(&chunk, height, 80, 8, 8), SkyLightTier::NoOpenSky);
    }

    #[test]
    fn shaft_only_degrades_its_own_quadrant() {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, 60);
        // Dig a 1x1 shaft in the NW quadrant from the surface down to the bottom.
        for y in chunk.section.min_y..=60 {
            chunk.set_block_absolute_y(2, y, 2, Block::AIR.default_state.id);
        }

        let height = SkyLightHeight::compute_from_chunk(&chunk);
        assert!(
            !height.quadrant_uses_limit(2, 2),
            "the shaft's own quadrant must lose its fast path"
        );
        for (x, z) in [(12, 2), (2, 12), (12, 12)] {
            assert!(
                height.quadrant_uses_limit(x, z),
                "quadrant ({x},{z}) must keep the fast path"
            );
        }

        // The shaft column must never be trivially rejected; the untouched ones still are.
        assert_eq!(tier_at(&chunk, height, 10, 2, 2), SkyLightTier::Unknown);
        assert_eq!(tier_at(&chunk, height, 10, 12, 12), SkyLightTier::NoOpenSky);
    }

    /// AND-Gatter: der schnelle Pfad an Grenze nur, wenn beide
    /// Quadranten ihn tragen. NAND (einer weicht ab) -> echter Check.
    #[test]
    fn border_gate_needs_both_sides() {
        let flat = SkyLightHeight::encode(56, -64, 384);
        let diverged = flat.with_quadrant_diverged(15, 8);

        // Ostkante von uns (x=15) trifft die Westkante des Nachbarn (x=0).
        assert!(
            flat.border_uses_limit(flat, 15, 8, 0, 8),
            "schneller Pfad"
        );
        assert!(
            !flat.border_uses_limit(diverged.with_quadrant_diverged(0, 8), 15, 8, 0, 8),
            "Nachbar -> echter Check"
        );
        assert!(
            !flat
                .with_quadrant_diverged(15, 8)
                .border_uses_limit(flat, 15, 8, 0, 8),
            "master -> echter Check"
        );

        // Nur der grenznahe Quadrant des Nachbarn zaehlt: eine Abweichung auf dessen
        // gegenueberliegender Seite (x=15) darf uns nicht ausbremsen.
        assert!(
            flat.border_uses_limit(flat.with_quadrant_diverged(15, 8), 15, 8, 0, 8),
            "Abweichung auf der fernen Seite des Nachbarn ist irrelevant"
        );
    }

    #[test]
    fn marking_a_quadrant_diverged_writes_through_to_nbt() {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, 60);
        let height = SkyLightHeightMigration::get(&chunk);
        assert!(height.quadrant_uses_limit(2, 2));

        SkyLightHeightMigration::mark_quadrant_diverged(&chunk, 2, 2);

        let updated = SkyLightHeight::from_raw(chunk.sky_light_height_cache.load(Ordering::Relaxed));
        assert!(!updated.quadrant_uses_limit(2, 2));
        assert!(updated.quadrant_uses_limit(12, 12));

        // Drop the in-memory cache: the divergence must survive in NBT.
        chunk.sky_light_height_cache.store(0, Ordering::Relaxed);
        let reloaded = SkyLightHeightMigration::get(&chunk);
        assert!(!reloaded.quadrant_uses_limit(2, 2));
    }

    ///`ProtoChunk` ohne  Generierung.
    fn proto_chunk() -> ProtoChunk {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
        use pumpkin_data::dimension::Dimension;
        use pumpkin_util::world_seed::Seed;

        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(42),
            Dimension::OVERWORLD,
        )));
        ProtoChunk::new(0, 0, &world_gen)
    }

    /// Fuellt jede Spalte bis `top` mit Stein und zieht die `WorldSurface`-Heightmap nach.
    fn fill_proto_terrain(proto: &mut ProtoChunk, top: i32) {
        let min_y = i32::from(proto.bottom_y());
        for local_z in 0..16 {
            for local_x in 0..16 {
                for y in min_y..=top {
                    proto.set_block_state(local_x, y, local_z, Block::STONE.default_state);
                }
            }
        }
    }

    #[test]
    fn worldgen_flat_terrain_is_usable_in_all_quadrants() {
        let mut proto = proto_chunk();
        fill_proto_terrain(&mut proto, 60);

        let height = SkyLightHeight::compute_from_proto(&proto);
        let min_y = i32::from(proto.bottom_y());
        let cut = height.decode(min_y, i32::from(proto.height()));

        // Der Cut ist die *Unterkante* des Bands, nicht die Deckenhoehe selbst: alle
        // Decken (hier 60) muessen in [cut, cut + spread] liegen.
        assert!(
            cut - DECODE_SAFETY_MARGIN <= 60 && 60 <= cut + height.spread(),
            "Decke 60 liegt nicht im Band [{cut}, {}]",
            cut + height.spread()
        );
        for (x, z) in [(2, 2), (12, 2), (2, 12), (12, 12)] {
            assert!(
                height.quadrant_uses_limit(x, z),
                "flaches Terrain: Quadrant ({x},{z}) muss den schnellen Pfad behalten"
            );
        }
        // Flach heisst schmalstes Band.
        assert_eq!(height.spread(), SPREAD_SCALES[0]);
        assert_ne!(height.raw(), 0, "Sentinel darf nie entstehen");
    }

    /// Ein Schacht (Carver/Ravine-Fall) darf nur sein eigenes 8x8-Quadrant degradieren.
    #[test]
    fn worldgen_shaft_only_degrades_its_own_quadrant() {
        let mut proto = proto_chunk();
        fill_proto_terrain(&mut proto, 60);

        // Spalte (2,2) bis weit unter den Cut ausgraeumt.
        let min_y = i32::from(proto.bottom_y());
        for y in min_y..=60 {
            proto.set_block_state(2, y, 2, Block::AIR.default_state);
        }

        let height = SkyLightHeight::compute_from_proto(&proto);
        assert!(
            !height.quadrant_uses_limit(2, 2),
            "das Quadrant des Schachts muss den schnellen Pfad verlieren"
        );
        for (x, z) in [(12, 2), (2, 12), (12, 12)] {
            assert!(
                height.quadrant_uses_limit(x, z),
                "Quadrant ({x},{z}) muss den schnellen Pfad behalten"
            );
        }
    }

    /// Glas ist nicht Luft, aber lichtdurchlaessig: `WorldSurface` steht hoch, der Cut
    /// darf trotzdem nicht mitwandern, sonst gilt eine belichtete Spalte als "kein Himmel".
    #[test]
    fn worldgen_glass_does_not_raise_the_cut() {
        let mut proto = proto_chunk();
        fill_proto_terrain(&mut proto, 60);
        for local_z in 0..16 {
            for local_x in 0..16 {
                proto.set_block_state(local_x, 80, local_z, Block::GLASS.default_state);
            }
        }

        let height = SkyLightHeight::compute_from_proto(&proto);
        let cut = height.decode(i32::from(proto.bottom_y()), i32::from(proto.height()));
        assert!(
            cut + height.spread() < 80,
            "Band [{cut}, {}] folgt dem Glas auf 80 statt dem Stein auf 60",
            cut + height.spread()
        );
        assert!(
            cut - DECODE_SAFETY_MARGIN <= 60 && 60 <= cut + height.spread(),
            "der Stein auf 60 muss weiterhin die Decke sein"
        );
    }

    /// Upgrade zum Level-Chunk ueberleben -> im Cache und in NBT, ohne Neuberechnung.
    #[test]
    fn worldgen_value_survives_upgrade_to_level_chunk() {
        use crate::chunk_system::chunk_state::Chunk;
        use pumpkin_config::lighting::LightingEngineConfig;
        use pumpkin_data::dimension::Dimension;

        let mut proto = proto_chunk();
        fill_proto_terrain(&mut proto, 60);
        let computed = SkyLightHeight::compute_from_proto(&proto);
        proto.sky_light_height = computed.raw();

        let mut chunk = Chunk::Proto(Box::new(proto));
        chunk.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
        let Chunk::Level(level) = chunk else {
            panic!("upgrade did not produce a level chunk");
        };

        assert_eq!(
            level.sky_light_height_cache.load(Ordering::Relaxed),
            computed.raw(),
            "der Worldgen-Wert muss im Cache ankommen"
        );
        assert!(
            SkyLightHeightMigration::fast_load_flag(&level),
            "und direkt persistiert sein, ohne ersten Lazy-Zugriff"
        );
        assert_eq!(SkyLightHeightMigration::get(&level), computed);
    }
}
