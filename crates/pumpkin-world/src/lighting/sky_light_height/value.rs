//! Packed 24-bit cut height: hex Y, flags, spread. Migration is storage only.

use super::{DECODE_SAFETY_MARGIN, SPREAD_SCALES, SkyLightTier};
use crate::ProtoChunk;
use crate::chunk::{ChunkData, ChunkHeightmapType};
use pumpkin_data::{BlockState, BlockStateId};

/// 24-bit Sky Light Cut Height
///
/// Bytes 0-1 (bits 0-15) -> hexadecimal coarse and fine value
/// Byte 2 (bits 16-23) -> flags half, `has_surface_water`, 4x quadrant divergence, 2 reserved
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkyLightHeight(u32);

impl SkyLightHeight {
    const HEX_APPROX_MASK: u32 = 0x0000_FFFF;
    const HEX_APPROX_STEPS: u32 = 1 << 16;

    const HALF_SHIFT: u32 = 16;
    pub(super) const FLAG_HALF: u32 = 1 << Self::HALF_SHIFT;

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

    /// Bits 22-23: index into [`SPREAD_SCALES`], the width of the tier 3 band.
    const SPREAD_SHIFT: u32 = 22;
    const SPREAD_MASK: u32 = 0b11 << Self::SPREAD_SHIFT;

    /// Width of the uncertain band above the cut for this chunk.
    #[must_use]
    pub const fn spread(self) -> i32 {
        SPREAD_SCALES[((self.0 & Self::SPREAD_MASK) >> Self::SPREAD_SHIFT) as usize]
    }

    #[must_use]
    pub(super) const fn with_spread_index(self, index: usize) -> Self {
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

    /// Bumps the hex approximation by `delta` steps (keeps `raw() == 0` impossible)
    /// 65536 slots per half -> enough headroom for later changes that allow building above the height limit.
    #[must_use]
    pub const fn with_hex_approx_bumped(self, delta: u32) -> Self {
        let hex_approx =
            (self.0 & Self::HEX_APPROX_MASK).wrapping_add(delta) & Self::HEX_APPROX_MASK;
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

    /// AND gate across a chunk border.
    ///
    /// At the border the fast path holds only if both adjoining quadrants carry
    /// it. NAND falls back to the real check.
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

    /// Whether a column ceiling still fits this chunk's band -> the inverse of
    /// [`Self::tier`] and the only place where quadrant divergence is decided
    /// at all.
    #[must_use]
    pub fn ceiling_within_band(self, ceiling: i32, chunk_min_y: i32, chunk_height: i32) -> bool {
        let cut = self.decode(chunk_min_y, chunk_height);
        ceiling >= cut - DECODE_SAFETY_MARGIN
            && ceiling <= cut + self.spread() + DECODE_SAFETY_MARGIN
    }

    /// Whether a block change at `y` can still move a column ceiling of this chunk.
    ///
    /// The lower edge of the band, and it must be the same edge
    /// [`Self::ceiling_within_band`] accepts. Skipping deeper changes is what makes the
    /// invalidation cheap: below the lowest possible ceiling, digging only removes blocks
    /// that were never the ceiling, and placing cannot raise one that already sits higher.
    #[must_use]
    pub fn may_move_a_ceiling(self, y: i32, chunk_min_y: i32, chunk_height: i32) -> bool {
        y >= self.decode(chunk_min_y, chunk_height) - DECODE_SAFETY_MARGIN
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
    fn column_opaque_ceiling(
        chunk: &ChunkData,
        local_x: usize,
        local_z: usize,
        from_y: i32,
    ) -> i32 {
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
            let top_y = min_y
                + (chunk.section.count as i32) * crate::chunk::palette::BlockPalette::SIZE as i32
                - 1;

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

    /// Picks the cut, the band and the divergence flags from the 4 quadrant ceiling
    /// intervals. Shared core of worldgen and runtime
    ///
    /// A quadrant can only use the band if all of its ceilings lie in
    /// `[cut, cut + spread]`, so the cut must lie in `[q_max - spread, q_min]`.
    #[must_use]
    fn solve(q_min: &[i32; 4], q_max: &[i32; 4], min_y: i32, chunk_height: i32) -> Self {
        let mut best_cut = q_min[0];
        let mut best_covered = 0u32;
        let mut best_spread_index = SPREAD_SCALES.len() - 1;

        for (spread_index, &spread) in SPREAD_SCALES.iter().enumerate() {
            for i in 0..4 {
                if q_max[i] - q_min[i] > spread {
                    continue; // quadrant does not fit the band
                }
                let candidate = q_max[i] - spread;
                let covered = (0..4)
                    .filter(|&j| candidate >= q_max[j] - spread && candidate <= q_min[j])
                    .count() as u32;
                // More quadrants > narrower band > high cut (large tier 1)
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

    /// Like [`Self::column_opaque_ceiling`], but on a not yet finished
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

    /// Worldgen variant of [`Self::compute_from_chunk`].
    ///
    /// Must run after carvers and features (stage `Lighting`), so that holes and
    /// ravines already carved into the terrain also land in the quadrant flags.
    /// Source is the `WorldSurface` heightmap of the `ProtoChunk`
    #[must_use]
    pub fn compute_from_proto(proto: &ProtoChunk) -> Self {
        let min_y = i32::from(proto.bottom_y());
        let mut q_min = [i32::MAX; 4];
        let mut q_max = [i32::MIN; 4];

        for local_z in 0..16i32 {
            for local_x in 0..16i32 {
                // `top_block_height_exclusive` is exclusive, the top block sits one below it.
                let surface = proto.top_block_height_exclusive(local_x, local_z) - 1;
                let ceiling = Self::proto_column_opaque_ceiling(proto, local_x, local_z, surface);
                let quadrant = Self::quadrant_index(local_x as usize, local_z as usize);
                q_min[quadrant] = q_min[quadrant].min(ceiling);
                q_max[quadrant] = q_max[quadrant].max(ceiling);
            }
        }

        let computed = Self::solve(&q_min, &q_max, min_y, i32::from(proto.height()));
        // raw() == 0 is the "not cached" sentinel and must never be persisted.
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
