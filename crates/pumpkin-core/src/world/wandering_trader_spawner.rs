use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use pumpkin_data::entity::EntityType;
use pumpkin_data::poi_type::PoiType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use pumpkin_world::world_info::data_files::{
    DEFAULT_WANDERING_TRADER_SPAWN_CHANCE as MIN_SPAWN_CHANCE,
    DEFAULT_WANDERING_TRADER_SPAWN_DELAY as DEFAULT_SPAWN_DELAY,
};
use rand::RngExt;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::spawn::{EntitySpawnReason, finalize_spawn};
use crate::entity::passive::wandering_trader::WanderingTraderEntity;
use crate::entity::r#type::from_type;
use crate::world::World;
use crate::world::natural_spawner::is_spawn_position_ok;

const DEFAULT_TICK_DELAY: i32 = 1200;
const MAX_SPAWN_CHANCE: i32 = 75;
const SPAWN_CHANCE_INCREASE: i32 = 25;
const SPAWN_ONE_IN_X_CHANCE: i32 = 10;
const NUMBER_OF_SPAWN_ATTEMPTS: i32 = 10;
const SEARCH_RADIUS: i32 = 48;
const LLAMA_SPAWN_RADIUS: i32 = 4;
const TRADER_DESPAWN_DELAY: i32 = 48000;
const HOME_RADIUS: i32 = 16;

pub struct WanderingTraderSpawner {
    tick_delay: AtomicI32,
}

impl Default for WanderingTraderSpawner {
    fn default() -> Self {
        Self {
            tick_delay: AtomicI32::new(DEFAULT_TICK_DELAY),
        }
    }
}

impl WanderingTraderSpawner {
    pub fn tick(&self, world: &Arc<World>) {
        if !world.level_info.load().game_rules.spawn_wandering_traders {
            return;
        }
        if self.tick_delay.fetch_sub(1, Relaxed) - 1 > 0 {
            return;
        }
        self.tick_delay.store(DEFAULT_TICK_DELAY, Relaxed);

        let info = world.level_info.load();
        let mut spawn_delay = info.wandering_trader_spawn_delay - DEFAULT_TICK_DELAY;
        let mut spawn_chance = info.wandering_trader_spawn_chance;
        if spawn_delay <= 0 {
            spawn_delay = DEFAULT_SPAWN_DELAY;
            let chance_to_spawn = spawn_chance;
            spawn_chance =
                (chance_to_spawn + SPAWN_CHANCE_INCREASE).clamp(MIN_SPAWN_CHANCE, MAX_SPAWN_CHANCE);
            if rand::rng().random_range(0..100) <= chance_to_spawn && Self::spawn(world) {
                spawn_chance = MIN_SPAWN_CHANCE;
            }
        }
        world.level_info.rcu(|current| {
            let mut new_info = (**current).clone();
            new_info.wandering_trader_spawn_delay = spawn_delay;
            new_info.wandering_trader_spawn_chance = spawn_chance;
            new_info
        });
    }

    fn spawn(world: &Arc<World>) -> bool {
        let Some(player) = world.get_random_player() else {
            return true;
        };
        let mut rng = rand::rng();
        if rng.random_range(0..SPAWN_ONE_IN_X_CHANCE) != 0 {
            return false;
        }

        let player_pos = player.get_entity().block_pos.load();
        let reference_pos = Self::find_meeting_point(world, player_pos).unwrap_or(player_pos);
        let Some(spawn_pos) = Self::find_spawn_position_near(world, reference_pos, SEARCH_RADIUS)
        else {
            return false;
        };
        if !Self::has_enough_space(world, spawn_pos) {
            return false;
        }
        if world
            .get_biome(&spawn_pos)
            .has_tag(&tag::WorldgenBiome::MINECRAFT_WITHOUT_WANDERING_TRADER_SPAWNS)
        {
            return false;
        }

        let trader = Self::spawn_mob(world, &EntityType::WANDERING_TRADER, spawn_pos);
        for _ in 0..2 {
            Self::try_to_spawn_llama_for(world, &trader);
        }
        if let Some(wandering_trader) = trader.cast_any().downcast_ref::<WanderingTraderEntity>() {
            wandering_trader.set_despawn_delay(TRADER_DESPAWN_DELAY);
            wandering_trader.set_wander_target(Some(reference_pos));
            wandering_trader
                .mob_entity
                .set_home_to(reference_pos, HOME_RADIUS);
        }
        true
    }

    fn find_meeting_point(world: &World, center: BlockPos) -> Option<BlockPos> {
        let (pos, _) = world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .find_closest_matching(center, SEARCH_RADIUS, |poi_type| {
                PoiType::from_name(poi_type) == Some(PoiType::Meeting)
            })?;
        (pos.squared_distance(&center) <= SEARCH_RADIUS * SEARCH_RADIUS).then_some(pos)
    }

    fn try_to_spawn_llama_for(world: &Arc<World>, trader: &Arc<dyn EntityBase>) {
        let trader_pos = trader.get_entity().block_pos.load();
        let Some(spawn_pos) = Self::find_spawn_position_near(world, trader_pos, LLAMA_SPAWN_RADIUS)
        else {
            return;
        };
        let llama = Self::spawn_mob(world, &EntityType::TRADER_LLAMA, spawn_pos);
        llama.get_entity().attach_leash(trader);
    }

    fn spawn_mob(
        world: &Arc<World>,
        entity_type: &'static EntityType,
        pos: BlockPos,
    ) -> Arc<dyn EntityBase> {
        let spawn_pos = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y),
            f64::from(pos.0.z) + 0.5,
        );
        let entity = from_type(entity_type, spawn_pos, world, Uuid::new_v4());
        let yaw = rand::rng().random::<f32>() * 360.0;
        entity.get_entity().set_rotation(yaw, 0.0);
        finalize_spawn(&entity, world, EntitySpawnReason::Event, None);
        world.spawn_entity(entity.clone());
        entity
    }

    fn find_spawn_position_near(
        world: &Arc<World>,
        reference_pos: BlockPos,
        radius: i32,
    ) -> Option<BlockPos> {
        let mut rng = rand::rng();
        for _ in 0..NUMBER_OF_SPAWN_ATTEMPTS {
            let x = reference_pos.0.x + rng.random_range(0..radius * 2) - radius;
            let z = reference_pos.0.z + rng.random_range(0..radius * 2) - radius;
            let y =
                world.get_heightmap_height(ChunkHeightmapType::MotionBlockingNoLeaves, x, z) + 1;
            let spawn_pos = BlockPos::new(x, y, z);
            if is_spawn_position_ok(world, &spawn_pos, &EntityType::WANDERING_TRADER) {
                return Some(spawn_pos);
            }
        }
        None
    }

    fn has_enough_space(world: &World, spawn_pos: BlockPos) -> bool {
        for x in 0..=1 {
            for y in 0..=2 {
                for z in 0..=1 {
                    let pos = spawn_pos.offset(Vector3::new(x, y, z));
                    if !world.get_block_state(&pos).collision_shapes.is_empty() {
                        return false;
                    }
                }
            }
        }
        true
    }
}
