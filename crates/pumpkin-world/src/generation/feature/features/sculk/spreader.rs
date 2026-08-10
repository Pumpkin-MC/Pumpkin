//! Vanilla-faithful `SculkSpreader` and `ChargeCursor`.
//!
//! Reference: `net.minecraft.world.level.block.SculkSpreader` (mc-26_2).

use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::RandomImpl;

use super::growth::GrowthRules;
use super::vein::VeinRules;
use super::NON_CORNER_NEIGHBOURS;
use super::SculkLevel;
use super::is_sculk_behaviour;

/// Maximum number of simultaneous cursors (vanilla: `MAX_CURSORS = 32`).
pub const MAX_CURSORS: usize = 32;
/// Maximum charge a single cursor may carry (vanilla: `MAX_CHARGE = 1000`).
pub const MAX_CHARGE: u16 = 1000;
/// Maximum chessboard distance from origin before a cursor is discarded
/// (vanilla: `MAX_CURSOR_DISTANCE = 1024`).
const MAX_CURSOR_DISTANCE: i32 = 1024;
/// XZ radius limit during world generation (vanilla: `15.0`).
const WORLD_GEN_RADIUS: f64 = 15.0;

/// Configuration values extracted from [`SculkSpreader`] so that cursor
/// updates do not require borrowing the spreader while iterating.
#[derive(Clone, Copy)]
pub struct SpreaderConfig {
    pub is_world_generation: bool,
    pub growth_spawn_cost: i32,
    pub no_growth_radius: i32,
    pub charge_decay_rate: i32,
    pub additional_decay_rate: i32,
}

impl SculkSpreader {
    /// Returns a snapshot of the configuration for cursor updates.
    pub fn config(&self) -> SpreaderConfig {
        SpreaderConfig {
            is_world_generation: self.is_world_generation,
            growth_spawn_cost: self.growth_spawn_cost,
            no_growth_radius: self.no_growth_radius,
            charge_decay_rate: self.charge_decay_rate,
            additional_decay_rate: self.additional_decay_rate,
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
    pub fn new_level_spreader() -> Self {
        Self::new(false, false, 10, 4, 10, 5)
    }

    /// Creates a spreader for world-generation spreading.
    /// Mirrors vanilla `SculkSpreader.createWorldGenSpreader()`.
    #[must_use]
    pub fn new_world_gen_spreader() -> Self {
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

        // Collect into a reusable buffer to avoid re-allocation between calls.
        let mut processed: Vec<ChargeCursor> = Vec::with_capacity(self.cursors.len());
        // Merge map: position -> index into `processed`.
        let mut merge_index: Vec<(BlockPos, usize)> = Vec::new();

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

            // Attempt merge with existing cursor at the same position.
            if let Some(&(_, idx)) =
                merge_index.iter().find(|&&(pos, _)| pos == cursor.pos)
            {
                let existing = &mut processed[idx];
                if !config.is_world_generation {
                    let combined = existing.charge as u32 + cursor.charge as u32;
                    if combined <= MAX_CHARGE as u32 {
                        existing.charge = combined as u16;
                        existing.update_delay =
                        std::cmp::min(existing.update_delay, cursor.update_delay);
                        continue;
                    }
                }
                // Can't merge — add as separate cursor if room.
                if processed.len() < MAX_CURSORS {
                    merge_index.push((cursor.pos, processed.len()));
                    processed.push(cursor);
                }
                // Do NOT add to merge_index: the position is already indexed
                // to the original cursor, so future merges find it correctly.
            } else {
                merge_index.push((cursor.pos, processed.len()));
                processed.push(cursor);
            }
        }

        // Truncate to the mergeable set (processed acts as the new cursor list).
        // We keep only one cursor per position for the merge map, but allow
        // unmerged duplicates. This matches vanilla behaviour.
        self.cursors = processed;
    }
}

/// A single "cursor" — a moving point of sculk charge.
pub struct ChargeCursor {
    pub pos: BlockPos,
    pub charge: u16,
    pub update_delay: u8,
    pub decay_delay: u8,
    /// Bitset of faces this cursor is attached to (from the block below).
    /// Stored as a `u8` bitset: bit 0 = Down … bit 5 = East.
    pub faces: u8,
}

impl ChargeCursor {
    #[must_use]
    pub const fn new(pos: BlockPos, charge: u16) -> Self {
        Self {
            pos,
            charge,
            update_delay: 0,
            decay_delay: 1,
            faces: 0,
        }
    }

    /// Returns the `BlockDirection` bits set in the face bitset.
    pub fn facing_directions(&self) -> impl Iterator<Item = BlockDirection> + '_ {
        BlockDirection::all().into_iter().filter(move |dir| {
            self.faces & (1 << dir.to_index()) != 0
        })
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

        // Attempt vein spreading first (if spread_veins and current block is
        // a SculkBehaviour).
        let current_state = level.sculk_get(self.pos);
        let current_id = current_state.map(|s| s.to_block_id()).unwrap_or(BlockId::AIR);

        if spread_veins && is_sculk_behaviour(current_id) {
            // attempt_spread_vein via the vein rules.
            let faces: Vec<BlockDirection> =
                self.facing_directions().collect();
            let _spread = VeinRules::attempt_spread_vein(
                level,
                self.pos,
                current_state,
                &faces,
            );
            // In vanilla, the block state may change if canChangeBlockStateOnSpread
            // returns true. For sculk_sensor/sculk this is false, so we skip.
        }

        // Apply charge decay / growth logic.
        self.charge = Self::attempt_use_charge(
            self,
            level,
            origin_pos,
            random,
            &config,
            spread_veins,
        );

        if self.charge == 0 {
            return;
        }

        // Attempt movement.
        if let Some(new_pos) = Self::valid_movement_position(level, self.pos, random) {
            self.pos = new_pos;

            // World-gen radius limit (XZ only).
            if config.is_world_generation {
                let dx = (self.pos.0.x - origin_pos.0.x) as f64;
                let dz = (self.pos.0.z - origin_pos.0.z) as f64;
                if (dx * dx + dz * dz).sqrt() > WORLD_GEN_RADIUS {
                    self.charge = 0;
                    return;
                }
            }

            // Update faces from the new position's block.
            let new_state = level.sculk_get(self.pos);
            if let Some(id) = new_state.map(|s| s.to_block_id()) {
                if is_sculk_behaviour(id) {
                    self.faces = Self::available_faces(level, self.pos, id);
                }
            }
        }

        // Update delays.
        self.decay_delay = Self::update_decay_delay(self.decay_delay);
        self.update_delay = 1; // vanilla: getSculkSpreadDelay() == 1
    }

    /// Vanilla `attemptUseCharge` — determines how much charge is consumed
    /// this tick and whether a growth spawns.
    fn attempt_use_charge(
        &self,
        level: &mut dyn SculkLevel,
        origin_pos: BlockPos,
        random: &mut RandomGenerator,
        config: &SpreaderConfig,
        _spread_veins: bool,
    ) -> u16 {
        let charge = self.charge;
        if charge == 0 {
            return 0;
        }
        // Roll for decay.
        if random.next_bounded_i32(config.charge_decay_rate) != 0 {
            return charge;
        }

        let is_close_to_catalyst =
            self.pos.0.x.abs_diff(origin_pos.0.x) <= config.no_growth_radius as u32
                && self.pos.0.y.abs_diff(origin_pos.0.y) <= config.no_growth_radius as u32
                && self.pos.0.z.abs_diff(origin_pos.0.z) <= config.no_growth_radius as u32;

        if !is_close_to_catalyst && GrowthRules::can_place_growth(level, self.pos) {
            let xp_per_growth = config.growth_spawn_cost;
            if random.next_bounded_i32(xp_per_growth) < charge as i32 {
                let growth_pos = self.pos.up();
                let growth_state = GrowthRules::random_growth_state(
                    level,
                    growth_pos,
                    random,
                    config.is_world_generation,
                );
                level.sculk_set(growth_pos, growth_state);
            }
            // Consume charge.
            return charge.saturating_sub(xp_per_growth as u16);
        }

        // No growth — apply additional decay or small decrement.
        if random.next_bounded_i32(config.additional_decay_rate) != 0 {
            return charge;
        }

        if is_close_to_catalyst {
            charge.saturating_sub(1)
        } else {
            let penalty = Self::decay_penalty(
                config,
                self.pos,
                origin_pos,
                charge as i32,
            );
            charge.saturating_sub(penalty as u16)
        }
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
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let outer_distance_sq = (distance - no_growth_radius as f64).powi(2);
        let max_reach_sq = (24 - no_growth_radius).pow(2);
        let factor =
            (outer_distance_sq / max_reach_sq as f64).min(1.0) as f32;
        let penalty = (charge as f64 * factor as f64 * 0.5).ceil() as i32;
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
        if result.is_some() {
            result
        } else {
            None
        }
    }

    /// Vanilla `isMovementUnobstructed`.
    fn is_movement_unobstructed(
        level: &dyn SculkLevel,
        from: BlockPos,
        to: BlockPos,
    ) -> bool {
        let delta = Vector3::new(
            to.0.x - from.0.x,
            to.0.y - from.0.y,
            to.0.z - from.0.z,
        );
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
            Self::is_unobstructed(level, from, dir_y)
                || Self::is_unobstructed(level, from, dir_z)
        } else if delta.y == 0 {
            Self::is_unobstructed(level, from, dir_x)
                || Self::is_unobstructed(level, from, dir_z)
        } else {
            Self::is_unobstructed(level, from, dir_x)
                || Self::is_unobstructed(level, from, dir_y)
        }
    }

    fn is_unobstructed(
        level: &dyn SculkLevel,
        from: BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let test_pos = from.offset(direction.to_offset());
        !level.sculk_is_face_sturdy(test_pos, direction.opposite())
    }

    /// Computes the face bitset for a sculk-behaviour block at `pos`.
    fn available_faces(level: &dyn SculkLevel, pos: BlockPos, id: BlockId) -> u8 {
        let mut faces: u8 = 0;
        for dir in BlockDirection::all() {
            let neighbour = pos.offset(dir.to_offset());
            let Some(state) = level.sculk_get(neighbour) else {
                continue;
            };
            if level.sculk_is_face_sturdy(neighbour, dir.opposite())
                || state.to_block_id() == id
            {
                faces |= 1 << dir.to_index();
            }
        }
        faces
    }

    /// Vanilla `updateDecayDelay` for SculkBehaviour.DEFAULT.
    fn update_decay_delay(current: u8) -> u8 {
        current.saturating_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_corner_neighbours_count() {
        assert_eq!(NON_CORNER_NEIGHBOURS.len(), 18);
    }

    #[test]
    fn non_corner_neighbours_valid() {
        for off in NON_CORNER_NEIGHBOURS {
            // At least one axis must be zero (not a corner).
            let zeros = (off.x == 0) as u8 + (off.y == 0) as u8 + (off.z == 0) as u8;
            assert!(zeros >= 1, "corner offset found: {:?}", off);
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
        // decay_delay saturating_sub
        c.decay_delay = 0;
        assert_eq!(ChargeCursor::update_decay_delay(c.decay_delay), 0);
    }
}
