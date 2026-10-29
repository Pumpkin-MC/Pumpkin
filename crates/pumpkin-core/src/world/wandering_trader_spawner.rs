use std::path::PathBuf;
use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::WorldgenBiome::MINECRAFT_WITHOUT_WANDERING_TRADER_SPAWNS;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_world::chunk::ChunkHeightmapType;
use pumpkin_world::world_info::MAXIMUM_SUPPORTED_WORLD_DATA_VERSION;
use pumpkin_world::world_info::data_files::{
    WanderingTraderData, read_wandering_trader, write_wandering_trader,
};
use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};
use tracing::error;

use crate::entity::mob::Mob;
use crate::entity::mob::spawn::finalize_spawn;
use crate::entity::passive::trader_llama::TraderLlamaEntity;
use crate::entity::passive::wandering_trader::WanderingTraderEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use crate::world::natural_spawner;
use crate::world::villager_poi::POI_TYPE_MEETING;

const DEFAULT_TICK_DELAY: i32 = 1200;
pub const DEFAULT_SPAWN_DELAY: i32 = 24_000;
const MIN_SPAWN_CHANCE: i32 = 25;
const MAX_SPAWN_CHANCE: i32 = 75;
const SPAWN_CHANCE_INCREASE: i32 = 25;
const SPAWN_ONE_IN_X_CHANCE: i32 = 10;
const NUMBER_OF_SPAWN_ATTEMPTS: i32 = 10;
const SPAWN_RADIUS: i32 = 48;
const LLAMA_SPAWN_RADIUS: i32 = 4;
const LLAMA_COUNT: i32 = 2;
const TRADER_DESPAWN_DELAY: i32 = 48_000;
const TRADER_HOME_RADIUS: i32 = 16;

/// `SpawnPlacements.getHeightmapType(WANDERING_TRADER)`.
const HEIGHTMAP: ChunkHeightmapType = ChunkHeightmapType::MotionBlockingNoLeaves;

/// Vanilla `WanderingTraderSpawner`, backed by the `wandering_trader` saved data.
pub struct WanderingTraderSpawner {
    tick_delay: i32,
    data: WanderingTraderData,
    data_dir: PathBuf,
}

impl WanderingTraderSpawner {
    #[must_use]
    pub fn new(data_dir: PathBuf) -> Self {
        let mut data = read_wandering_trader(&data_dir);
        data.data_version = MAXIMUM_SUPPORTED_WORLD_DATA_VERSION;
        Self {
            tick_delay: DEFAULT_TICK_DELAY,
            data,
            data_dir,
        }
    }

    pub fn tick(&mut self, world: &Arc<World>) {
        if !world.level_info.load().game_rules.spawn_wandering_traders {
            return;
        }

        let mut rng = rand::rng();
        self.tick_with(&mut rng, |rng| Self::spawn(world, rng));
    }

    /// Vanilla `WanderingTraderSpawner.tick`: the schedule and chance bookkeeping around one
    /// spawn attempt. `spawn` is the level-dependent placement step, split out for tests.
    fn tick_with<R: Rng>(&mut self, rng: &mut R, spawn: impl FnOnce(&mut R) -> bool) -> bool {
        self.tick_delay -= 1;
        if self.tick_delay > 0 {
            return false;
        }
        self.tick_delay = DEFAULT_TICK_DELAY;

        self.set_spawn_delay(self.data.spawn_delay - DEFAULT_TICK_DELAY);
        if self.data.spawn_delay > 0 {
            return false;
        }

        self.set_spawn_delay(DEFAULT_SPAWN_DELAY);
        let spawn_chance = self.data.spawn_chance;
        self.set_spawn_chance(
            (spawn_chance + SPAWN_CHANCE_INCREASE).clamp(MIN_SPAWN_CHANCE, MAX_SPAWN_CHANCE),
        );

        if rng.random_range(0..100) <= spawn_chance && spawn(rng) {
            self.set_spawn_chance(MIN_SPAWN_CHANCE);
            return true;
        }

        false
    }

    fn set_spawn_delay(&mut self, spawn_delay: i32) {
        if self.data.spawn_delay != spawn_delay {
            self.data.spawn_delay = spawn_delay;
            self.save();
        }
    }

    fn set_spawn_chance(&mut self, spawn_chance: i32) {
        if self.data.spawn_chance != spawn_chance {
            self.data.spawn_chance = spawn_chance;
            self.save();
        }
    }

    fn save(&self) {
        if let Err(error) = write_wandering_trader(&self.data_dir, &self.data) {
            error!("Failed to save wandering trader data: {error}");
        }
    }

    fn spawn<R: Rng>(world: &Arc<World>, rng: &mut R) -> bool {
        let player_pos = {
            let players = world.players.load();
            players
                .as_slice()
                .choose(rng)
                .map(|player| player.get_entity().block_pos.load())
        };
        let Some(player_pos) = player_pos else {
            return true;
        };

        if !Self::passes_spawn_gate(rng) {
            return false;
        }

        let reference_pos = Self::meeting_site_near(world, player_pos).unwrap_or(player_pos);
        let Some(spawn_pos) =
            Self::find_spawn_position_near(world, reference_pos, SPAWN_RADIUS, rng)
        else {
            return false;
        };
        if !Self::has_enough_space(world, spawn_pos) {
            return false;
        }
        if world
            .get_biome(&spawn_pos)
            .has_tag(&MINECRAFT_WITHOUT_WANDERING_TRADER_SPAWNS)
        {
            return false;
        }

        let trader = WanderingTraderEntity::new(Entity::new(
            world.clone(),
            spawn_position(spawn_pos),
            &EntityType::WANDERING_TRADER,
        ));
        trader
            .get_entity()
            .set_rotation(wrap_degrees(rng.random::<f32>() * 360.0) % 360.0, 0.0);
        finalize_spawn(&(trader.clone() as Arc<dyn EntityBase>), world, None);
        if !world.spawn_entity(trader.clone() as Arc<dyn EntityBase>) {
            return false;
        }

        for _ in 0..LLAMA_COUNT {
            Self::try_to_spawn_llama_for(world, &trader, LLAMA_SPAWN_RADIUS, rng);
        }

        trader.set_despawn_delay(TRADER_DESPAWN_DELAY);
        trader.set_wander_target(Some(reference_pos));
        trader
            .get_mob_entity()
            .set_home_to(reference_pos, TRADER_HOME_RADIUS);
        true
    }

    fn try_to_spawn_llama_for<R: Rng>(
        world: &Arc<World>,
        trader: &Arc<WanderingTraderEntity>,
        radius: i32,
        rng: &mut R,
    ) {
        let Some(spawn_pos) = Self::find_spawn_position_near(
            world,
            trader.get_entity().block_pos.load(),
            radius,
            rng,
        ) else {
            return;
        };

        let llama = TraderLlamaEntity::new(Entity::new(
            world.clone(),
            spawn_position(spawn_pos),
            &EntityType::TRADER_LLAMA,
        ));
        llama
            .get_entity()
            .set_rotation(wrap_degrees(rng.random::<f32>() * 360.0) % 360.0, 0.0);
        finalize_spawn(&(llama.clone() as Arc<dyn EntityBase>), world, None);
        if world.spawn_entity(llama.clone() as Arc<dyn EntityBase>) {
            llama
                .get_entity()
                .attach_leash(trader.clone() as Arc<dyn EntityBase>);
        }
    }

    /// Vanilla's 1-in-10 gate before a placement attempt.
    fn passes_spawn_gate<R: Rng>(rng: &mut R) -> bool {
        rng.random_range(0..SPAWN_ONE_IN_X_CHANCE) == 0
    }

    fn meeting_site_near(world: &World, player_pos: BlockPos) -> Option<BlockPos> {
        world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .find_closest_matching(player_pos, SPAWN_RADIUS, |poi_type| {
                poi_type == POI_TYPE_MEETING
            })
            .map(|(position, _)| position)
    }

    fn find_spawn_position_near<R: Rng>(
        world: &Arc<World>,
        reference_pos: BlockPos,
        radius: i32,
        rng: &mut R,
    ) -> Option<BlockPos> {
        for _ in 0..NUMBER_OF_SPAWN_ATTEMPTS {
            let x = reference_pos.0.x + rng.random_range(0..radius * 2) - radius;
            let z = reference_pos.0.z + rng.random_range(0..radius * 2) - radius;
            // `World.get_heightmap_height` returns the top block, vanilla `Level.getHeight`
            // the first free position above it.
            let y = world.get_heightmap_height(HEIGHTMAP, x, z) + 1;
            let spawn_pos = BlockPos::new(x, y, z);
            if natural_spawner::is_spawn_position_ok(
                world,
                &spawn_pos,
                &EntityType::WANDERING_TRADER,
            ) {
                return Some(spawn_pos);
            }
        }
        None
    }

    fn has_enough_space(world: &World, spawn_pos: BlockPos) -> bool {
        BlockPos::between_closed(spawn_pos, spawn_pos.offset(Vector3::new(1, 2, 1))).all(|pos| {
            world
                .get_block_state(&pos)
                .get_block_collision_shapes_at(&pos)
                .next()
                .is_none()
        })
    }
}

fn spawn_position(pos: BlockPos) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::TryRng;
    use std::convert::Infallible;
    use std::path::Path;

    /// Returns a constant value, so the chance roll and the one-in-ten gate are predictable.
    struct ConstRng(u32);

    impl TryRng for ConstRng {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Infallible> {
            Ok(self.0)
        }

        fn try_next_u64(&mut self) -> Result<u64, Infallible> {
            Ok(u64::from(self.0))
        }

        fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), Infallible> {
            Ok(())
        }
    }

    /// `u32::MAX` maps to the top of every integer range, so the chance roll always fails.
    const FAILING_ROLL: u32 = u32::MAX;

    fn spawner(temp_dir: &Path, spawn_delay: i32, spawn_chance: i32) -> WanderingTraderSpawner {
        write_wandering_trader(
            temp_dir,
            &WanderingTraderData {
                spawn_delay,
                spawn_chance,
                data_version: MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
            },
        )
        .unwrap();
        WanderingTraderSpawner::new(temp_dir.to_path_buf())
    }

    fn tick_checks(spawner: &mut WanderingTraderSpawner, rng: &mut ConstRng, spawn_succeeds: bool) {
        for tick in 0..DEFAULT_TICK_DELAY {
            let is_last_tick = tick == DEFAULT_TICK_DELAY - 1;
            spawner.tick_with(rng, |_| is_last_tick && spawn_succeeds);
        }
    }

    #[test]
    fn checks_the_schedule_every_1200_ticks() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut spawner = spawner(temp_dir.path(), DEFAULT_SPAWN_DELAY, MIN_SPAWN_CHANCE);
        let mut rng = ConstRng(FAILING_ROLL);

        for _ in 0..DEFAULT_TICK_DELAY - 1 {
            spawner.tick_with(&mut rng, |_| false);
        }
        assert_eq!(spawner.data.spawn_delay, DEFAULT_SPAWN_DELAY);

        spawner.tick_with(&mut rng, |_| false);
        assert_eq!(
            spawner.data.spawn_delay,
            DEFAULT_SPAWN_DELAY - DEFAULT_TICK_DELAY
        );
    }

    #[test]
    fn spawn_delay_resets_to_24000_when_it_reaches_zero() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut spawner = spawner(temp_dir.path(), DEFAULT_TICK_DELAY, MIN_SPAWN_CHANCE);
        let mut rng = ConstRng(FAILING_ROLL);

        tick_checks(&mut spawner, &mut rng, false);

        assert_eq!(spawner.data.spawn_delay, DEFAULT_SPAWN_DELAY);
    }

    #[test]
    fn spawn_chance_rises_by_25_up_to_75() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut spawner = spawner(temp_dir.path(), DEFAULT_TICK_DELAY, MIN_SPAWN_CHANCE);
        let mut rng = ConstRng(FAILING_ROLL);

        tick_checks(&mut spawner, &mut rng, false);
        assert_eq!(spawner.data.spawn_chance, 50);

        spawner.set_spawn_delay(DEFAULT_TICK_DELAY);
        tick_checks(&mut spawner, &mut rng, false);
        assert_eq!(spawner.data.spawn_chance, MAX_SPAWN_CHANCE);

        spawner.set_spawn_delay(DEFAULT_TICK_DELAY);
        tick_checks(&mut spawner, &mut rng, false);
        assert_eq!(spawner.data.spawn_chance, MAX_SPAWN_CHANCE);
    }

    #[test]
    fn spawn_chance_resets_after_a_successful_spawn() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut spawner = spawner(temp_dir.path(), DEFAULT_TICK_DELAY, MIN_SPAWN_CHANCE);
        let mut rng = ConstRng(0);

        tick_checks(&mut spawner, &mut rng, true);

        assert_eq!(spawner.data.spawn_chance, MIN_SPAWN_CHANCE);
    }

    #[test]
    fn failed_spawn_keeps_the_increased_chance() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut spawner = spawner(temp_dir.path(), DEFAULT_TICK_DELAY, MIN_SPAWN_CHANCE);
        let mut rng = ConstRng(0);

        tick_checks(&mut spawner, &mut rng, false);

        assert_eq!(spawner.data.spawn_chance, 50);
    }

    #[test]
    fn spawn_gate_passes_only_the_zero_roll() {
        assert!(WanderingTraderSpawner::passes_spawn_gate(&mut ConstRng(0)));
        assert!(!WanderingTraderSpawner::passes_spawn_gate(&mut ConstRng(
            FAILING_ROLL
        )));
    }

    #[test]
    fn schedule_changes_are_saved_immediately() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut spawner = spawner(temp_dir.path(), DEFAULT_TICK_DELAY, MIN_SPAWN_CHANCE);
        let mut rng = ConstRng(FAILING_ROLL);

        tick_checks(&mut spawner, &mut rng, false);

        let saved = read_wandering_trader(temp_dir.path());
        assert_eq!(saved.spawn_delay, DEFAULT_SPAWN_DELAY);
        assert_eq!(saved.spawn_chance, 50);
    }
}
