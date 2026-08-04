//! `SculkBlock` port (`net/minecraft/world/level/block/SculkBlock.java`, 107 lines).
//!
//! Design doc `designs/sculk-and-block-social.md`, Step 3. Verified against
//! `Blocks.java`'s `SCULK` registration and the full 107-line `SculkBlock.java`: no
//! `.randomTicks()`/no `randomTick` override — sculk growth is driven entirely by
//! `SculkSpreader::update_cursors` (Step 4) calling `attempt_use_charge` below, never by
//! a block random tick. Nothing in this codebase drives that yet, so this block is
//! inert with respect to the catalyst; the algorithm is unit-tested directly.

use pumpkin_data::block_properties::{
    BlockProperties, SculkSensorLikeProperties, SculkShriekerLikeProperties,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::Sound;
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use pumpkin_world::world::BlockAccessor;

use crate::block::sculk_behaviour::{
    ChargeCursor, SculkBehaviour, SculkSpreaderConfig, SculkWorld,
};
use crate::block::{BlockBehaviour, BlockFuture};

#[pumpkin_block("minecraft:sculk")]
pub struct SculkBlock;

impl BlockBehaviour for SculkBlock {}

/// `pos.distSqr(other)`: plain integer `dx^2+dy^2+dz^2`, not a center-offset overload.
fn distance_squared(a: BlockPos, b: BlockPos) -> i64 {
    let dx = i64::from(a.0.x - b.0.x);
    let dy = i64::from(a.0.y - b.0.y);
    let dz = i64::from(a.0.z - b.0.z);
    dx * dx + dy * dy + dz * dz
}

/// `BlockPos.closerThan(pos, distance)`: strict Euclidean `<`, not chessboard/manhattan
/// (that's `isPosUnreasonable`, a different check in a different step).
fn closer_than(a: BlockPos, b: BlockPos, distance: i32) -> bool {
    distance_squared(a, b) < i64::from(distance) * i64::from(distance)
}

/// `SculkBlock.getDecayPenalty` (lines 60-66). Precision matters: the sqrt is double,
/// cast down to `f32` before subtracting `noGrowthRadius`, and the rest of the formula
/// is `f32` arithmetic to match `Mth.square`/the `float` return type in vanilla.
fn get_decay_penalty(
    spreader: &SculkSpreaderConfig,
    pos: BlockPos,
    origin_pos: BlockPos,
    charge: i32,
) -> i32 {
    let no_growth_radius = spreader.no_growth_radius();
    #[allow(clippy::cast_precision_loss)]
    let dist_sq = distance_squared(pos, origin_pos) as f64;
    let outer_distance = dist_sq.sqrt() as f32 - no_growth_radius as f32;
    let outer_distance_squared = outer_distance * outer_distance;
    let max_reach = 24 - no_growth_radius;
    #[allow(clippy::cast_precision_loss)]
    let max_reach_squared = (max_reach * max_reach) as f32;
    let distance_factor = (outer_distance_squared / max_reach_squared).min(1.0);
    #[allow(clippy::cast_possible_truncation)]
    let penalty = (charge as f32 * distance_factor * 0.5) as i32;
    penalty.max(1)
}

/// `SculkBlock.canPlaceGrowth` (lines 81-100): the block directly above `pos` must be
/// air or water (any fluid state, not source-only), and at most 2 sensors/shriekers may
/// already exist in the 9x3x9 box `pos.offset(-4,0,-4)..=pos.offset(4,2,4)`.
fn can_place_growth(accessor: &dyn BlockAccessor, pos: BlockPos) -> bool {
    let above = pos.offset(BlockDirection::Up.to_offset());
    let above_block = accessor.get_block(&above);
    let above_state = accessor.get_block_state(&above);
    let above_ok = above_state.is_air() || above_block == &Block::WATER;
    if !above_ok {
        return false;
    }

    let mut growth_count = 0;
    for dx in -4..=4 {
        for dy in 0..=2 {
            for dz in -4..=4 {
                let check_pos = pos.offset(pumpkin_util::math::vector3::Vector3::new(dx, dy, dz));
                let block = accessor.get_block(&check_pos);
                if block == &Block::SCULK_SENSOR || block == &Block::SCULK_SHRIEKER {
                    growth_count += 1;
                    if growth_count > 2 {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// `SculkBlock.getRandomGrowthState` (lines 68-79): 1/11 chance of a shrieker
/// (`can_summon = isWorldGen`), else a sensor; waterlogged if any fluid (not just a
/// source) occupies the growth position.
fn roll_growth_state(
    accessor: &dyn BlockAccessor,
    pos: BlockPos,
    random: &mut RandomGenerator,
    is_world_gen: bool,
) -> (BlockStateId, Sound) {
    let waterlogged = accessor.get_fluid(&pos) != Fluid::EMPTY;
    if random.next_bounded_i32(11) == 0 {
        let mut props = SculkShriekerLikeProperties::default(&Block::SCULK_SHRIEKER);
        props.r#can_summon = is_world_gen;
        props.r#waterlogged = waterlogged;
        (
            props.to_state_id(&Block::SCULK_SHRIEKER),
            Sound::BlockSculkShriekerPlace,
        )
    } else {
        let mut props = SculkSensorLikeProperties::default(&Block::SCULK_SENSOR);
        props.r#waterlogged = waterlogged;
        (
            props.to_state_id(&Block::SCULK_SENSOR),
            Sound::BlockSculkSensorPlace,
        )
    }
}

impl SculkBehaviour for SculkBlock {
    /// `SculkBlock.canChangeBlockStateOnSpread` (lines 103-106): sculk blocks never turn
    /// into something else via `attemptSpreadVein` (only vein blocks do).
    fn can_change_block_state_on_spread(&self) -> bool {
        false
    }

    /// `SculkBlock.attemptUseCharge` (lines 27-58).
    fn attempt_use_charge<'a>(
        &'a self,
        cursor: &'a ChargeCursor,
        world: &'a dyn SculkWorld,
        origin_pos: BlockPos,
        random: &'a mut RandomGenerator,
        spreader: &'a SculkSpreaderConfig,
        _spread_veins: bool,
    ) -> BlockFuture<'a, i32> {
        Box::pin(async move {
            let charge = cursor.charge();
            if charge == 0 || random.next_bounded_i32(spreader.charge_decay_rate()) != 0 {
                return charge;
            }

            let charge_pos = cursor.pos();
            let is_close_to_catalyst =
                closer_than(charge_pos, origin_pos, spreader.no_growth_radius());

            if !is_close_to_catalyst && can_place_growth(world.accessor(), charge_pos) {
                let xp_per_growth_spawn = spreader.growth_spawn_cost();
                if random.next_bounded_i32(xp_per_growth_spawn) < charge {
                    let growth_pos = charge_pos.offset(BlockDirection::Up.to_offset());
                    let (growth_state_id, place_sound) = roll_growth_state(
                        world.accessor(),
                        growth_pos,
                        random,
                        spreader.is_world_generation(),
                    );
                    world.set_block(growth_pos, growth_state_id).await;
                    world.play_block_sound(charge_pos, place_sound);
                }
                (charge - xp_per_growth_spawn).max(0)
            } else if random.next_bounded_i32(spreader.additional_decay_rate()) != 0 {
                charge
            } else if is_close_to_catalyst {
                charge - 1
            } else {
                charge - get_decay_penalty(spreader, charge_pos, origin_pos, charge)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::blocks::multiface_spreader::{SpreadPos, SpreadTarget};
    use pumpkin_data::BlockState;
    use pumpkin_util::random::xoroshiro128::Xoroshiro;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[test]
    fn no_random_ticks_in_vanilla_data() {
        // `Blocks.java` registers SCULK with no `.randomTicks()`, and `SculkBlock.java`
        // (107 lines, read in full) has no `randomTick` override.
        for state in Block::SCULK.states {
            assert!(!state.has_random_ticks());
        }
    }

    #[test]
    fn closer_than_matches_strict_euclidean() {
        let origin = BlockPos::new(0, 0, 0);
        assert!(closer_than(BlockPos::new(3, 0, 0), origin, 4));
        assert!(!closer_than(BlockPos::new(4, 0, 0), origin, 4));
        assert!(!closer_than(BlockPos::new(0, 0, 0), origin, 0));
    }

    #[test]
    fn decay_penalty_is_clamped_to_at_least_one() {
        let spreader = SculkSpreaderConfig::level_spreader();
        let origin = BlockPos::new(0, 0, 0);
        // Exactly at the no-growth-radius boundary: outer distance 0, penalty floor 1.
        let pos = BlockPos::new(spreader.no_growth_radius(), 0, 0);
        assert_eq!(get_decay_penalty(&spreader, pos, origin, 1000), 1);
    }

    #[test]
    fn decay_penalty_grows_toward_the_max_growth_radius() {
        let spreader = SculkSpreaderConfig::level_spreader();
        let origin = BlockPos::new(0, 0, 0);
        // At the full MAX_GROWTH_RATE_RADIUS (24), distanceFactor saturates at 1.0, so
        // penalty = charge * 0.5, floored.
        let pos = BlockPos::new(24, 0, 0);
        assert_eq!(get_decay_penalty(&spreader, pos, origin, 1000), 500);
    }

    struct FakeAccessor {
        states: HashMap<BlockPos, &'static BlockState>,
        fluids: HashMap<BlockPos, Fluid>,
        default: &'static BlockState,
    }

    impl FakeAccessor {
        fn new(default: &'static BlockState) -> Self {
            Self {
                states: HashMap::new(),
                fluids: HashMap::new(),
                default,
            }
        }

        fn with_block(mut self, pos: BlockPos, state: &'static BlockState) -> Self {
            self.states.insert(pos, state);
            self
        }

        fn with_fluid(mut self, pos: BlockPos, fluid: Fluid) -> Self {
            self.fluids.insert(pos, fluid);
            self
        }
    }

    impl BlockAccessor for FakeAccessor {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            Block::from_state_id(self.get_block_state(position).id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.states.get(position).copied().unwrap_or(self.default)
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let state = self.get_block_state(position);
            (Block::from_state_id(state.id), state)
        }

        fn get_fluid(&self, position: &BlockPos) -> Fluid {
            self.fluids.get(position).cloned().unwrap_or(Fluid::EMPTY)
        }
    }

    #[test]
    fn can_place_growth_true_over_air_with_no_existing_growths() {
        let pos = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::AIR.default_state);
        assert!(can_place_growth(&accessor, pos));
    }

    #[test]
    fn can_place_growth_false_when_above_is_solid() {
        let pos = BlockPos::new(0, 0, 0);
        let above = pos.offset(BlockDirection::Up.to_offset());
        let accessor = FakeAccessor::new(Block::AIR.default_state)
            .with_block(above, Block::STONE.default_state);
        assert!(!can_place_growth(&accessor, pos));
    }

    #[test]
    fn can_place_growth_true_over_water() {
        let pos = BlockPos::new(0, 0, 0);
        let above = pos.offset(BlockDirection::Up.to_offset());
        let accessor = FakeAccessor::new(Block::AIR.default_state)
            .with_block(above, Block::WATER.default_state);
        assert!(can_place_growth(&accessor, pos));
    }

    #[test]
    fn can_place_growth_false_when_more_than_two_growths_nearby() {
        let pos = BlockPos::new(0, 0, 0);
        let mut accessor = FakeAccessor::new(Block::AIR.default_state);
        for i in 0..3 {
            accessor =
                accessor.with_block(BlockPos::new(i, 0, 0), Block::SCULK_SENSOR.default_state);
        }
        assert!(!can_place_growth(&accessor, pos));
    }

    #[test]
    fn can_place_growth_true_with_exactly_two_growths_nearby() {
        let pos = BlockPos::new(0, 0, 0);
        let mut accessor = FakeAccessor::new(Block::AIR.default_state);
        for i in 0..2 {
            accessor =
                accessor.with_block(BlockPos::new(i, 0, 0), Block::SCULK_SENSOR.default_state);
        }
        assert!(can_place_growth(&accessor, pos));
    }

    #[test]
    fn roll_growth_state_picks_sensor_or_shrieker_by_rng() {
        let pos = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::AIR.default_state);

        // Seed search: find seeds hitting each branch of `nextInt(11) == 0`.
        let mut saw_sensor = false;
        let mut saw_shrieker = false;
        for seed in 0..64u64 {
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
            let (state_id, sound) = roll_growth_state(&accessor, pos, &mut random, false);
            let block = Block::from_state_id(state_id);
            if block == &Block::SCULK_SENSOR {
                saw_sensor = true;
                assert_eq!(sound, Sound::BlockSculkSensorPlace);
            } else if block == &Block::SCULK_SHRIEKER {
                saw_shrieker = true;
                assert_eq!(sound, Sound::BlockSculkShriekerPlace);
                let props =
                    SculkShriekerLikeProperties::from_state_id(state_id, &Block::SCULK_SHRIEKER);
                assert!(!props.r#can_summon);
            }
        }
        assert!(saw_sensor, "expected at least one sensor roll in 64 seeds");
        assert!(
            saw_shrieker,
            "expected at least one shrieker roll in 64 seeds"
        );
    }

    #[test]
    fn roll_growth_state_sets_waterlogged_from_fluid_at_growth_pos() {
        let pos = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::AIR.default_state).with_fluid(pos, Fluid::WATER);
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(1));
        let (state_id, _) = roll_growth_state(&accessor, pos, &mut random, false);
        let block = Block::from_state_id(state_id);
        if block == &Block::SCULK_SENSOR {
            assert!(
                SculkSensorLikeProperties::from_state_id(state_id, &Block::SCULK_SENSOR)
                    .r#waterlogged
            );
        } else {
            assert!(
                SculkShriekerLikeProperties::from_state_id(state_id, &Block::SCULK_SHRIEKER)
                    .r#waterlogged
            );
        }
    }

    #[test]
    fn shrieker_can_summon_matches_world_gen_flag() {
        let pos = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::AIR.default_state);
        // Find a seed that rolls the shrieker branch under world-gen and check the flag.
        for seed in 0..64u64 {
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
            let (state_id, _) = roll_growth_state(&accessor, pos, &mut random, true);
            let block = Block::from_state_id(state_id);
            if block == &Block::SCULK_SHRIEKER {
                let props =
                    SculkShriekerLikeProperties::from_state_id(state_id, &Block::SCULK_SHRIEKER);
                assert!(props.r#can_summon);
                return;
            }
        }
        panic!("expected at least one shrieker roll in 64 seeds");
    }

    /// Minimal `SculkWorld` test double: records writes/sounds in-memory, no spreading.
    struct RecordingSculkWorld {
        states: Mutex<HashMap<BlockPos, &'static BlockState>>,
        fluids: HashMap<BlockPos, Fluid>,
        default: &'static BlockState,
        sounds_played: Mutex<Vec<(BlockPos, Sound)>>,
    }

    impl RecordingSculkWorld {
        fn new(default: &'static BlockState) -> Self {
            Self {
                states: Mutex::new(HashMap::new()),
                fluids: HashMap::new(),
                default,
                sounds_played: Mutex::new(Vec::new()),
            }
        }

        fn state_at(&self, pos: BlockPos) -> &'static BlockState {
            self.states
                .lock()
                .unwrap()
                .get(&pos)
                .copied()
                .unwrap_or(self.default)
        }
    }

    impl BlockAccessor for RecordingSculkWorld {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            Block::from_state_id(self.get_block_state(position).id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.state_at(*position)
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let state = self.get_block_state(position);
            (Block::from_state_id(state.id), state)
        }

        fn get_fluid(&self, position: &BlockPos) -> Fluid {
            self.fluids.get(position).cloned().unwrap_or(Fluid::EMPTY)
        }
    }

    impl SpreadTarget for RecordingSculkWorld {
        fn accessor(&self) -> &dyn BlockAccessor {
            self
        }

        fn place(&self, _spread_pos: SpreadPos) -> BlockFuture<'_, bool> {
            Box::pin(async { false })
        }
    }

    impl SculkWorld for RecordingSculkWorld {
        fn set_block(&self, pos: BlockPos, state_id: BlockStateId) -> BlockFuture<'_, ()> {
            Box::pin(async move {
                self.states.lock().unwrap().insert(pos, state_id.to_state());
            })
        }

        fn play_block_sound(&self, pos: BlockPos, sound: Sound) {
            self.sounds_played.lock().unwrap().push((pos, sound));
        }

        fn push_entities_up(&self, _pos: BlockPos) {}
    }

    #[tokio::test]
    async fn attempt_use_charge_holds_charge_when_charge_is_zero() {
        let behaviour = SculkBlock;
        let cursor = ChargeCursor::new(BlockPos::new(0, 0, 0), 0, 1);
        let world = RecordingSculkWorld::new(Block::AIR.default_state);
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(1));
        let spreader = SculkSpreaderConfig::level_spreader();

        let new_charge = behaviour
            .attempt_use_charge(
                &cursor,
                &world,
                BlockPos::new(0, 0, 0),
                &mut random,
                &spreader,
                true,
            )
            .await;
        assert_eq!(new_charge, 0);
    }

    #[tokio::test]
    async fn attempt_use_charge_never_grows_too_close_to_the_catalyst() {
        let behaviour = SculkBlock;
        let origin = BlockPos::new(0, 0, 0);
        // Well within `no_growth_radius` (4): growth is impossible regardless of RNG,
        // charge only ever decays (or holds).
        let cursor = ChargeCursor::new(BlockPos::new(1, 0, 0), 1000, 1);
        let world = RecordingSculkWorld::new(Block::AIR.default_state);
        let spreader = SculkSpreaderConfig::level_spreader();

        for seed in 0..32u64 {
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
            let new_charge = behaviour
                .attempt_use_charge(&cursor, &world, origin, &mut random, &spreader, true)
                .await;
            assert!(new_charge <= 1000);
            assert!(world.states.lock().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn attempt_use_charge_places_growth_and_spends_full_cost_when_growth_rolls_succeed() {
        let behaviour = SculkBlock;
        let origin = BlockPos::new(0, 0, 0);
        // Far from the catalyst, high charge: `nextInt(growthSpawnCost) < charge` is
        // guaranteed (growthSpawnCost is 10, charge is 1000), so a growth block must
        // always be placed once the outer `nextInt(chargeDecayRate) == 0` roll hits,
        // regardless of seed.
        let charge_pos = BlockPos::new(100, 64, 0);
        let cursor = ChargeCursor::new(charge_pos, 1000, 1);
        let world = RecordingSculkWorld::new(Block::AIR.default_state);
        let spreader = SculkSpreaderConfig::level_spreader();

        // Search for a seed where the outer decay-rate roll hits (1/10 chance) so growth
        // is actually attempted.
        for seed in 0..64u64 {
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
            let new_charge = behaviour
                .attempt_use_charge(&cursor, &world, origin, &mut random, &spreader, true)
                .await;
            let growth_pos = charge_pos.offset(BlockDirection::Up.to_offset());
            let placed = world.state_at(growth_pos);
            if Block::from_state_id(placed.id) == &Block::SCULK_SENSOR
                || Block::from_state_id(placed.id) == &Block::SCULK_SHRIEKER
            {
                assert_eq!(new_charge, 990);
                assert!(
                    world
                        .sounds_played
                        .lock()
                        .unwrap()
                        .iter()
                        .any(|(pos, _)| *pos == charge_pos)
                );
                return;
            }
        }
        panic!("expected at least one seed in 0..64 to place growth");
    }

    #[test]
    fn can_change_block_state_on_spread_is_false() {
        assert!(!SculkBlock.can_change_block_state_on_spread());
    }
}
