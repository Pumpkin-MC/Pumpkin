//! Vanilla-faithful `SculkSpreader` and `ChargeCursor`.
//!
//! Reference: `net.minecraft.world.level.block.SculkSpreader` (mc-26_2).

use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::RandomImpl;
use std::collections::HashMap;

use super::NON_CORNER_NEIGHBOURS;
use super::SculkLevel;
use super::growth::GrowthRules;
use super::is_sculk_behaviour;
use super::is_sculk_replaceable;
use super::is_sculk_replaceable_world_gen;
use super::vein::VeinRules;

/// Maximum number of simultaneous cursors (vanilla: `MAX_CURSORS = 32`).
pub const MAX_CURSORS: usize = 32;
/// Maximum charge a single cursor may carry (vanilla: `MAX_CHARGE = 1000`).
pub const MAX_CHARGE: u16 = 1000;
/// Maximum chessboard distance from origin before a cursor is discarded
/// (vanilla: `MAX_CURSOR_DISTANCE = 1024`).
const MAX_CURSOR_DISTANCE: i32 = 1024;
/// Squared XZ radius limit during world generation (vanilla: cursors are
/// discarded once `distSqr(vec3i(x0, y, z0)) >= Mth.square(15.0)`, i.e.
/// `dx² + dz² >= 225`).
const WORLD_GEN_RADIUS_SQ: i64 = 225;

/// Configuration values extracted from [`SculkSpreader`] so that cursor
/// updates do not require borrowing the spreader while iterating.
#[derive(Clone, Copy)]
pub struct SpreaderConfig {
    pub is_world_generation: bool,
    pub growth_spawn_cost: i32,
    pub no_growth_radius: i32,
    pub charge_decay_rate: i32,
    pub additional_decay_rate: i32,
    /// Whether replaceable blocks are resolved through the world-gen tag
    /// (`minecraft:sculk_replaceable_world_gen`) instead of the level tag
    /// (`minecraft:sculk_replaceable`).
    pub replaceable_world_gen: bool,
}

impl SculkSpreader {
    /// Returns a snapshot of the configuration for cursor updates.
    pub const fn config(&self) -> SpreaderConfig {
        SpreaderConfig {
            is_world_generation: self.is_world_generation,
            growth_spawn_cost: self.growth_spawn_cost,
            no_growth_radius: self.no_growth_radius,
            charge_decay_rate: self.charge_decay_rate,
            additional_decay_rate: self.additional_decay_rate,
            replaceable_world_gen: self.replaceable_tag_world_gen,
        }
    }
}

/// Drives sculk spreading for a single patch.
pub struct SculkSpreader {
    /// Whether this spreader runs during world generation (vs. catalyst-driven).
    is_world_generation: bool,
    /// Tag used to determine which blocks are replaceable.
    replaceable_tag_world_gen: bool,
    /// Cost in charge to spawn a growth (sensor/shrieker).
    growth_spawn_cost: i32,
    /// Radius around the origin within which no growths spawn.
    no_growth_radius: i32,
    /// Charge decay rate — `random.nextInt(rate) == 0` triggers decay.
    charge_decay_rate: i32,
    /// Additional decay rate when no growth spawns.
    additional_decay_rate: i32,
    /// Active cursors. Bounded to [`MAX_CURSORS`].
    cursors: Vec<ChargeCursor>,
}

impl SculkSpreader {
    /// Creates a spreader for catalyst-driven (level) spreading.
    /// Mirrors vanilla `SculkSpreader.createLevelSpreader()`.
    #[must_use]
    pub const fn new_level_spreader() -> Self {
        Self::new(false, false, 10, 4, 10, 5)
    }

    /// Creates a spreader for world-generation spreading.
    /// Mirrors vanilla `SculkSpreader.createWorldGenSpreader()`.
    #[must_use]
    pub const fn new_world_gen_spreader() -> Self {
        Self::new(true, true, 50, 1, 5, 10)
    }

    #[must_use]
    const fn new(
        is_world_generation: bool,
        replaceable_tag_world_gen: bool,
        growth_spawn_cost: i32,
        no_growth_radius: i32,
        charge_decay_rate: i32,
        additional_decay_rate: i32,
    ) -> Self {
        Self {
            is_world_generation,
            replaceable_tag_world_gen,
            growth_spawn_cost,
            no_growth_radius,
            charge_decay_rate,
            additional_decay_rate,
            cursors: Vec::new(),
        }
    }

    #[inline]
    pub const fn is_world_generation(&self) -> bool {
        self.is_world_generation
    }

    #[inline]
    pub const fn growth_spawn_cost(&self) -> i32 {
        self.growth_spawn_cost
    }

    #[inline]
    pub const fn no_growth_radius(&self) -> i32 {
        self.no_growth_radius
    }

    #[inline]
    pub const fn charge_decay_rate(&self) -> i32 {
        self.charge_decay_rate
    }

    #[inline]
    pub const fn additional_decay_rate(&self) -> i32 {
        self.additional_decay_rate
    }

    /// Returns `true` if the given block id is replaceable for this spreader.
    #[inline]
    pub fn is_replaceable(&self, id: BlockId) -> bool {
        if self.replaceable_tag_world_gen {
            super::is_sculk_replaceable_world_gen(id)
        } else {
            super::is_sculk_replaceable(id)
        }
    }

    /// Adds charge at a position, splitting into multiple cursors if
    /// charge exceeds [`MAX_CHARGE`] (vanilla `addCursors`).
    pub fn add_cursors(&mut self, start_pos: BlockPos, mut charge: i32) {
        while charge > 0 {
            let current = std::cmp::min(charge, MAX_CHARGE as i32);
            self.add_cursor(ChargeCursor::new(start_pos, current as u16));
            charge -= current;
        }
    }

    /// Adds a single cursor, respecting the [`MAX_CURSORS`] limit.
    fn add_cursor(&mut self, cursor: ChargeCursor) {
        if self.cursors.len() < MAX_CURSORS {
            self.cursors.push(cursor);
        }
    }

    /// Clears all cursors (called between rounds).
    pub fn clear(&mut self) {
        self.cursors.clear();
    }

    /// Returns a reference to the active cursors (for testing).
    #[cfg(test)]
    pub fn cursors(&self) -> &[ChargeCursor] {
        &self.cursors
    }

    /// Main update loop. Mirrors vanilla `updateCursors`.
    pub fn update_cursors(
        &mut self,
        level: &mut dyn SculkLevel,
        origin_pos: BlockPos,
        random: &mut RandomGenerator,
        spread_veins: bool,
    ) {
        if self.cursors.is_empty() {
            return;
        }

        let mut processed: Vec<ChargeCursor> = Vec::with_capacity(self.cursors.len());
        // Merge map: position -> index into `processed` (vanilla
        // `mergeableCursors`).
        let mut merge_index: HashMap<BlockPos, usize> = HashMap::with_capacity(self.cursors.len());

        let config = self.config();
        for cursor in self.cursors.drain(..) {
            if cursor.pos.0.x.abs_diff(origin_pos.0.x) > MAX_CURSOR_DISTANCE as u32
                || cursor.pos.0.y.abs_diff(origin_pos.0.y) > MAX_CURSOR_DISTANCE as u32
                || cursor.pos.0.z.abs_diff(origin_pos.0.z) > MAX_CURSOR_DISTANCE as u32
            {
                continue; // unreachable position, discard
            }

            let mut cursor = cursor;
            cursor.update(level, origin_pos, random, config, spread_veins);

            if cursor.charge == 0 {
                continue;
            }

            let pos = cursor.pos;
            // Attempt merge with the existing cursor at the same position.
            if let Some(existing_idx) = merge_index.get(&pos).copied() {
                let existing_charge = processed[existing_idx].charge;
                // Vanilla: merge when not world-gen and the combined
                // charge fits within MAX_CHARGE.
                if !config.is_world_generation {
                    let combined = existing_charge as u32 + cursor.charge as u32;
                    if combined <= MAX_CHARGE as u32 {
                        processed[existing_idx].charge = combined as u16;
                        processed[existing_idx].update_delay = processed[existing_idx]
                            .update_delay
                            .min(cursor.update_delay);
                        continue;
                    }
                }
                // Can't merge: keep the new cursor as a separate entry. It
                // is intentionally NOT registered in merge_index — the
                // existing entry must remain the merge target for future
                // lookups — unless it carries less charge than the
                // existing cursor, in which case vanilla replaces the
                // merge target (SculkSpreader.updateCursors).
                processed.push(cursor);
                let new_idx = processed.len() - 1;
                if processed[new_idx].charge < existing_charge {
                    merge_index.insert(pos, new_idx);
                }
            } else {
                merge_index.insert(pos, processed.len());
                processed.push(cursor);
            }
        }

        self.cursors = processed;
    }
}

/// A single "cursor" — a moving point of sculk charge.
pub struct ChargeCursor {
    pub pos: BlockPos,
    pub charge: u16,
    pub update_delay: u8,
    pub decay_delay: u8,
    /// Optional bitset of faces for this cursor. `None` mirrors vanilla's
    /// `null` facings (same-space spreading only), `Some(0)` is the empty set
    /// and `Some(bits)` holds face bits (bit 0 = Down … bit 5 = East).
    pub faces: Option<u8>,
}

impl ChargeCursor {
    #[must_use]
    pub const fn new(pos: BlockPos, charge: u16) -> Self {
        Self {
            pos,
            charge,
            update_delay: 0,
            decay_delay: 1,
            faces: None,
        }
    }

    /// Returns the `BlockDirection` bits set in the face bitset.
    pub fn facing_directions(&self) -> impl Iterator<Item = BlockDirection> + '_ {
        let bits = self.faces.unwrap_or(0);
        BlockDirection::all()
            .into_iter()
            .filter(move |dir| bits & (1 << dir.to_index()) != 0)
    }

    /// Core update logic — mirrors vanilla `ChargeCursor.update`.
    pub fn update(
        &mut self,
        level: &mut dyn SculkLevel,
        origin_pos: BlockPos,
        random: &mut RandomGenerator,
        config: SpreaderConfig,
        spread_veins: bool,
    ) {
        if self.charge == 0 {
            return;
        }
        if self.update_delay > 0 {
            self.update_delay -= 1;
            return;
        }

        let mut current_state = level.sculk_get(self.pos);
        let mut current_id = current_state.map_or(BlockId::AIR, BlockStateId::to_block_id);

        // Attempt vein spreading first. Vanilla dispatches on the block
        // behaviour: sculk behaviour blocks (sculk / sculk vein) use the
        // multiface `spreadAll`, while every other block uses
        // `SculkBehaviour.DEFAULT`, which switches on the cursor's facings
        // (null → same-space, empty → spreadAll, non-empty → regrow).
        if spread_veins {
            let spread = if is_sculk_behaviour(current_id) {
                VeinRules::spread_all(level, self.pos)
            } else {
                VeinRules::attempt_spread_vein(level, self.pos, current_state, self.faces)
            };
            // Vanilla re-reads the state after a successful spread unless the
            // block cannot change state on spread (`canChangeBlockStateOnSpread`
            // is false only for sculk).
            if spread && current_id != BlockId::SCULK {
                current_state = level.sculk_get(self.pos);
                current_id = current_state.map_or(BlockId::AIR, BlockStateId::to_block_id);
            }
        }

        // Apply charge decay / growth logic.
        self.charge = Self::attempt_use_charge(
            self,
            current_id,
            level,
            origin_pos,
            random,
            &config,
            spread_veins,
        );

        if self.charge == 0 {
            // Vanilla: `sculkBehaviour.onDischarged(...)` — only sculk veins
            // actually remove faces (other behaviours are no-ops).
            if current_id == BlockId::SCULK_VEIN {
                VeinRules::on_discharged(level, self.pos);
            }
            return;
        }

        // Attempt movement.
        if let Some(new_pos) = Self::valid_movement_position(level, self.pos, random) {
            if current_id == BlockId::SCULK_VEIN {
                VeinRules::on_discharged(level, self.pos);
            }
            self.pos = new_pos;

            // World-gen radius limit (XZ only). Vanilla:
            // `!closerThan(originX, y, originZ, 15.0)` → `dx² + dz² >= 225`.
            if config.is_world_generation {
                let dx = (self.pos.0.x - origin_pos.0.x) as i64;
                let dz = (self.pos.0.z - origin_pos.0.z) as i64;
                if dx * dx + dz * dz >= WORLD_GEN_RADIUS_SQ {
                    self.charge = 0;
                    return;
                }
            }

            // Update faces from the new position's block.
            if let Some(state) = level.sculk_get(self.pos) {
                let id = state.to_block_id();
                if is_sculk_behaviour(id) {
                    self.faces = Some(Self::available_faces(state, id));
                }
            }
        }

        // Update delays.
        // Vanilla uses the (pre-movement) block behaviour for the decay delay:
        // sculk behaviours reset it to 1, `DEFAULT` decrements it.
        self.decay_delay =
            Self::update_decay_delay(self.decay_delay, is_sculk_behaviour(current_id));
        self.update_delay = 1; // vanilla: getSculkSpreadDelay() == 1
    }

    /// Vanilla `attemptUseCharge` — determines how much charge is consumed
    /// this tick. Dispatches on the block at the cursor position, mirroring
    /// `ChargeCursor.getBlockBehaviour`:
    /// - sculk vein → `SculkVeinBlock.attemptUseCharge`
    /// - sculk → `SculkBlock.attemptUseCharge`
    /// - everything else (including sensors, shriekers and catalysts, which
    ///   do not implement `SculkBehaviour`) →
    ///   `SculkBehaviour.DEFAULT.attemptUseCharge`
    fn attempt_use_charge(
        &self,
        current_id: BlockId,
        level: &mut dyn SculkLevel,
        origin_pos: BlockPos,
        random: &mut RandomGenerator,
        config: &SpreaderConfig,
        spread_veins: bool,
    ) -> u16 {
        let charge = self.charge;
        if charge == 0 {
            return 0;
        }
        match current_id {
            BlockId::SCULK_VEIN => {
                // Vanilla `SculkVeinBlock.attemptUseCharge`.
                let replaceable = |id: BlockId| {
                    if config.replaceable_world_gen {
                        is_sculk_replaceable_world_gen(id)
                    } else {
                        is_sculk_replaceable(id)
                    }
                };
                if spread_veins
                    && VeinRules::attempt_place_sculk(level, self.pos, random, replaceable)
                {
                    return charge.saturating_sub(1);
                }
                if random.next_bounded_i32(config.charge_decay_rate) == 0 {
                    // `Mth.floor(charge * 0.5F)` — halves the charge.
                    charge / 2
                } else {
                    charge
                }
            }
            BlockId::SCULK => {
                Self::sculk_block_use_charge(self.pos, level, origin_pos, random, config, charge)
            }
            // Vanilla `SculkBehaviour.DEFAULT.attemptUseCharge`:
            // `decayDelay > 0 ? charge : 0`.
            _ => {
                if self.decay_delay > 0 {
                    charge
                } else {
                    0
                }
            }
        }
    }

    /// Vanilla `SculkBlock.attemptUseCharge` — growth placement and
    /// distance-based charge decay.
    fn sculk_block_use_charge(
        pos: BlockPos,
        level: &mut dyn SculkLevel,
        origin_pos: BlockPos,
        random: &mut RandomGenerator,
        config: &SpreaderConfig,
        charge: u16,
    ) -> u16 {
        if charge == 0 {
            return 0;
        }
        // Roll for decay.
        if random.next_bounded_i32(config.charge_decay_rate) != 0 {
            return charge;
        }

        let is_close_to_catalyst =
            Self::is_close_to_catalyst(pos, origin_pos, config.no_growth_radius);

        if !is_close_to_catalyst && GrowthRules::can_place_growth(level, pos) {
            let xp_per_growth = config.growth_spawn_cost;
            if random.next_bounded_i32(xp_per_growth) < charge as i32 {
                let growth_pos = pos.up();
                let growth_state = GrowthRules::random_growth_state(
                    level,
                    growth_pos,
                    random,
                    config.is_world_generation,
                );
                level.sculk_set(growth_pos, growth_state);
            }
            // Consume charge (`Math.max(0, charge - xpPerGrowthSpawn)`).
            return charge.saturating_sub(xp_per_growth as u16);
        }

        // No growth — apply additional decay or small decrement.
        if random.next_bounded_i32(config.additional_decay_rate) != 0 {
            return charge;
        }

        if is_close_to_catalyst {
            charge.saturating_sub(1)
        } else {
            let penalty = Self::decay_penalty(config, pos, origin_pos, charge as i32);
            charge.saturating_sub(penalty as u16)
        }
    }

    /// Vanilla `Vec3i.closerThan` — strict squared-Euclidean comparison.
    const fn is_close_to_catalyst(
        pos: BlockPos,
        origin_pos: BlockPos,
        no_growth_radius: i32,
    ) -> bool {
        let dx = (pos.0.x - origin_pos.0.x) as i64;
        let dy = (pos.0.y - origin_pos.0.y) as i64;
        let dz = (pos.0.z - origin_pos.0.z) as i64;
        let radius = no_growth_radius as i64;
        dx * dx + dy * dy + dz * dz < radius * radius
    }

    /// Vanilla `getDecayPenalty` — distance-based charge decay.
    fn decay_penalty(
        config: &SpreaderConfig,
        pos: BlockPos,
        origin_pos: BlockPos,
        charge: i32,
    ) -> i32 {
        let no_growth_radius = config.no_growth_radius;
        let dx = (pos.0.x - origin_pos.0.x) as f64;
        let dy = (pos.0.y - origin_pos.0.y) as f64;
        let dz = (pos.0.z - origin_pos.0.z) as f64;
        // Vanilla: `Mth.square((float)Math.sqrt(distSqr) - noGrowthRadius)`,
        // i.e. the distance is computed in double then cast to float.
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let outer_distance_sq = (distance as f32 - no_growth_radius as f32).powi(2);
        // `Mth.square(24 - noGrowthRadius)` is an int.
        let max_reach_sq = (24 - no_growth_radius).pow(2);
        let factor = (outer_distance_sq / max_reach_sq as f32).min(1.0);
        // `(int)(charge * distanceFactor * 0.5F)` — truncates toward zero.
        let penalty = (charge as f32 * factor * 0.5) as i32;
        penalty.max(1)
    }

    /// Vanilla `getValidMovementPos` — scans non-corner neighbours for a
    /// sculk-behaviour block the cursor can move to.
    fn valid_movement_position(
        level: &dyn SculkLevel,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) -> Option<BlockPos> {
        let mut result = None;

        // Shuffle the 18 non-corner neighbours (Fisher-Yates).
        let mut order: [u8; 18] = core::array::from_fn(|i| i as u8);
        for i in (1..18).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            order.swap(i, j);
        }

        for &idx in &order {
            let offset = NON_CORNER_NEIGHBOURS[idx as usize];
            let neighbour = pos.offset(offset);

            let Some(state) = level.sculk_get(neighbour) else {
                continue;
            };
            let id = state.to_block_id();
            if !is_sculk_behaviour(id) {
                continue;
            }
            if !Self::is_movement_unobstructed(level, pos, neighbour) {
                continue;
            }
            if result.is_none() {
                result = Some(neighbour);
            }
            if VeinRules::has_substrate_access(level, state, neighbour) {
                // Found a substrate-accessible target — take it immediately.
                break;
            }
        }

        // Vanilla returns null if it found no movement AND no substrate access.
        // But if it found a valid target (even without substrate), it returns it.
        if result.is_some() { result } else { None }
    }

    /// Vanilla `isMovementUnobstructed`.
    fn is_movement_unobstructed(level: &dyn SculkLevel, from: BlockPos, to: BlockPos) -> bool {
        let delta = Vector3::new(to.0.x - from.0.x, to.0.y - from.0.y, to.0.z - from.0.z);
        // Manhattan distance == 1 → always unobstructed.
        if delta.x.abs() + delta.y.abs() + delta.z.abs() == 1 {
            return true;
        }
        let dir_x = if delta.x < 0 {
            BlockDirection::West
        } else {
            BlockDirection::East
        };
        let dir_y = if delta.y < 0 {
            BlockDirection::Down
        } else {
            BlockDirection::Up
        };
        let dir_z = if delta.z < 0 {
            BlockDirection::North
        } else {
            BlockDirection::South
        };
        if delta.x == 0 {
            Self::is_unobstructed(level, from, dir_y) || Self::is_unobstructed(level, from, dir_z)
        } else if delta.y == 0 {
            Self::is_unobstructed(level, from, dir_x) || Self::is_unobstructed(level, from, dir_z)
        } else {
            Self::is_unobstructed(level, from, dir_x) || Self::is_unobstructed(level, from, dir_y)
        }
    }

    fn is_unobstructed(level: &dyn SculkLevel, from: BlockPos, direction: BlockDirection) -> bool {
        let test_pos = from.offset(direction.to_offset());
        !level.sculk_is_face_sturdy(test_pos, direction.opposite())
    }

    /// Vanilla `MultifaceBlock.availableFaces` — derives the face bitset from
    /// the block state's own face properties. Only multiface blocks (sculk
    /// vein) expose faces; every other block yields an empty bitset.
    fn available_faces(state: BlockStateId, id: BlockId) -> u8 {
        if id != BlockId::SCULK_VEIN {
            return 0;
        }
        let mut faces: u8 = 0;
        for dir in BlockDirection::all() {
            if VeinRules::has_face(state, dir) {
                faces |= 1 << dir.to_index();
            }
        }
        faces
    }

    /// Vanilla `SculkBehaviour.updateDecayDelay`: sculk behaviours reset the
    /// delay to 1, `DEFAULT` decrements it (`Math.max(age - 1, 0)`).
    const fn update_decay_delay(current: u8, is_sculk: bool) -> u8 {
        if is_sculk {
            1
        } else {
            current.saturating_sub(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;
    use pumpkin_util::random::legacy_rand::LegacyRand;

    use crate::generation::feature::features::sculk::test_utils::MockSculkLevel;

    #[test]
    fn non_corner_neighbours_count() {
        assert_eq!(NON_CORNER_NEIGHBOURS.len(), 18);
    }

    #[test]
    fn non_corner_neighbours_valid() {
        for off in NON_CORNER_NEIGHBOURS {
            // At least one axis must be zero (not a corner).
            let zeros = (off.x == 0) as u8 + (off.y == 0) as u8 + (off.z == 0) as u8;
            assert!(zeros >= 1, "corner offset found: {off:?}");
            // Not the centre.
            assert!(
                !(off.x == 0 && off.y == 0 && off.z == 0),
                "centre offset found"
            );
            // Each axis in -1..=1.
            assert!(off.x >= -1 && off.x <= 1);
            assert!(off.y >= -1 && off.y <= 1);
            assert!(off.z >= -1 && off.z <= 1);
        }
    }

    #[test]
    fn charge_saturation() {
        let mut s = SculkSpreader::new_world_gen_spreader();
        let origin = BlockPos::new(0, 60, 0);
        s.add_cursors(origin, 2500);
        // 2500 = 1000 + 1000 + 500 → 3 cursors.
        assert_eq!(s.cursors().len(), 3);
        assert_eq!(s.cursors()[0].charge, 1000);
        assert_eq!(s.cursors()[1].charge, 1000);
        assert_eq!(s.cursors()[2].charge, 500);
    }

    #[test]
    fn max_cursors_limit() {
        let mut s = SculkSpreader::new_world_gen_spreader();
        let origin = BlockPos::new(0, 60, 0);
        for _ in 0..50 {
            s.add_cursors(origin, 1000);
        }
        assert!(s.cursors().len() <= MAX_CURSORS);
    }

    #[test]
    fn no_underflow() {
        let mut c = ChargeCursor::new(BlockPos::new(0, 60, 0), 0);
        assert_eq!(c.charge, 0);
        // decay_delay saturating_sub for the DEFAULT behaviour.
        c.decay_delay = 0;
        assert_eq!(ChargeCursor::update_decay_delay(c.decay_delay, false), 0);
        // sculk behaviour resets the delay to 1.
        assert_eq!(ChargeCursor::update_decay_delay(0, true), 1);
    }

    #[test]
    fn cursor_faces_default_to_none() {
        let c = ChargeCursor::new(BlockPos::new(0, 60, 0), 100);
        assert_eq!(c.faces, None);
        assert_eq!(c.facing_directions().count(), 0);
    }

    #[test]
    fn close_to_catalyst_is_strict() {
        let origin = BlockPos::new(0, 60, 0);
        // `closerThan` uses `< radius²`: exactly on the boundary is NOT close.
        assert!(!ChargeCursor::is_close_to_catalyst(
            BlockPos::new(4, 60, 0),
            origin,
            4,
        ));
        assert!(ChargeCursor::is_close_to_catalyst(
            BlockPos::new(3, 60, 0),
            origin,
            4,
        ));
        // 3D distance: (2, 2, 2) → √12 < 4.
        assert!(ChargeCursor::is_close_to_catalyst(
            BlockPos::new(2, 62, 2),
            origin,
            4,
        ));
    }

    #[test]
    fn available_faces_reads_block_state() {
        // Only multiface (sculk vein) states expose faces.
        let base = Block::SCULK_VEIN.default_state.id;
        assert_eq!(ChargeCursor::available_faces(base, BlockId::SCULK_VEIN), 0,);
        let with_up = VeinRules::with_face(base, BlockDirection::Up, true);
        assert_eq!(
            ChargeCursor::available_faces(with_up, BlockId::SCULK_VEIN),
            1 << BlockDirection::Up.to_index(),
        );
        // Non-multiface states always yield an empty set.
        assert_eq!(
            ChargeCursor::available_faces(Block::SCULK.default_state.id, BlockId::SCULK,),
            0,
        );
    }

    #[test]
    fn cursor_does_not_move_onto_sensor() {
        // Vanilla `getValidMovementPos` only accepts `SculkBehaviour` blocks;
        // a lone sensor neighbour is not a valid movement target.
        let mut level = MockSculkLevel::new();
        let pos = BlockPos::new(0, 60, 0);
        level.set_id(pos, Block::SCULK.default_state.id);
        level.set_id(pos.up(), Block::SCULK_SENSOR.default_state.id);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(1));
        assert_eq!(
            ChargeCursor::valid_movement_position(&level, pos, &mut random),
            None,
        );
    }

    #[test]
    fn cursor_moves_onto_sculk_not_sensor() {
        let mut level = MockSculkLevel::new();
        let pos = BlockPos::new(0, 60, 0);
        level.set_id(pos, Block::SCULK.default_state.id);
        level.set_id(pos.up(), Block::SCULK_SENSOR.default_state.id);
        level.set_id(pos.east(), Block::SCULK.default_state.id);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(1));
        assert_eq!(
            ChargeCursor::valid_movement_position(&level, pos, &mut random),
            Some(pos.east()),
        );
    }
}
