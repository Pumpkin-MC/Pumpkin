use std::sync::{Arc, Mutex};

use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::DayTimeline;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::MobEntity;
use crate::entity::mob::spawn::{EntitySpawnReason, finalize_spawn};
use crate::entity::r#type::from_type;
use crate::world::World;

const SIEGE_CHANCE: i32 = 10;
const SIEGE_SIZE: i32 = 20;
const SPAWN_INTERVAL: i32 = 2;
const SPAWN_DISTANCE: f32 = 32.0;
const SETUP_ATTEMPTS: i32 = 10;
const SPAWN_POS_ATTEMPTS: i32 = 10;
const SPAWN_POS_SPREAD: i32 = 16;

#[derive(Default, PartialEq, Eq)]
enum SiegeState {
    SiegeTonight,
    #[default]
    SiegeDone,
}

struct Siege {
    is_set_up: bool,
    state: SiegeState,
    zombies_to_spawn: i32,
    next_spawn_time: i32,
    spawn_pos: BlockPos,
}

impl Default for Siege {
    fn default() -> Self {
        Self {
            is_set_up: false,
            state: SiegeState::default(),
            zombies_to_spawn: 0,
            next_spawn_time: 0,
            spawn_pos: BlockPos::new(0, 0, 0),
        }
    }
}

/// Sends a wave of zombies at a village one night in ten.
#[derive(Default)]
pub struct VillageSiege {
    siege: Mutex<Siege>,
}

impl VillageSiege {
    pub fn tick(&self, world: &Arc<World>, spawn_enemies: bool) {
        let mut siege = self
            .siege
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if world.is_bright_outside() || !spawn_enemies {
            siege.state = SiegeState::SiegeDone;
            siege.is_set_up = false;
            return;
        }
        if !world.dimension.has_fixed_time
            && world
                .get_time_of_day()
                .rem_euclid(i64::from(DayTimeline::PERIOD_TICKS))
                == i64::from(DayTimeline::ROLL_VILLAGE_SIEGE_TIME_MARKER)
        {
            siege.state = if rand::random_range(0..SIEGE_CHANCE) == 0 {
                SiegeState::SiegeTonight
            } else {
                SiegeState::SiegeDone
            };
        }
        if siege.state == SiegeState::SiegeDone {
            return;
        }
        if !siege.is_set_up {
            if !try_to_setup_siege(world, &mut siege) {
                return;
            }
            siege.is_set_up = true;
        }
        if siege.next_spawn_time > 0 {
            siege.next_spawn_time -= 1;
            return;
        }
        siege.next_spawn_time = SPAWN_INTERVAL;
        if siege.zombies_to_spawn > 0 {
            try_spawn(world, siege.spawn_pos);
            siege.zombies_to_spawn -= 1;
        } else {
            siege.state = SiegeState::SiegeDone;
        }
    }
}

fn try_to_setup_siege(world: &Arc<World>, siege: &mut Siege) -> bool {
    let mut rng = rand::rng();
    for player in world.players.load().iter() {
        if player.is_spectator() {
            continue;
        }
        let center = player.get_entity().block_pos.load();
        if !world.is_village(&center)
            || world
                .get_biome(&center)
                .has_tag(&tag::WorldgenBiome::MINECRAFT_WITHOUT_ZOMBIE_SIEGES)
        {
            continue;
        }
        for _ in 0..SETUP_ATTEMPTS {
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
            siege.spawn_pos = BlockPos::new(
                center.0.x + (angle.cos() * SPAWN_DISTANCE).floor() as i32,
                center.0.y,
                center.0.z + (angle.sin() * SPAWN_DISTANCE).floor() as i32,
            );
            if find_random_spawn_pos(world, siege.spawn_pos).is_some() {
                siege.next_spawn_time = 0;
                siege.zombies_to_spawn = SIEGE_SIZE;
                break;
            }
        }
        return true;
    }
    false
}

fn try_spawn(world: &Arc<World>, spawn_pos: BlockPos) {
    let Some(pos) = find_random_spawn_pos(world, spawn_pos) else {
        return;
    };
    let zombie = from_type(
        &EntityType::ZOMBIE,
        Vector3::default(),
        world,
        Uuid::new_v4(),
    );
    finalize_spawn(&zombie, world, EntitySpawnReason::Event, None);
    let entity = zombie.get_entity();
    entity.set_pos(pos);
    entity.set_rotation(rand::random::<f32>() * 360.0, 0.0);
    world.spawn_entity(zombie);
}

fn find_random_spawn_pos(world: &Arc<World>, pos: BlockPos) -> Option<Vector3<f64>> {
    let mut rng = rand::rng();
    for _ in 0..SPAWN_POS_ATTEMPTS {
        let x = pos.0.x + rng.random_range(0..SPAWN_POS_SPREAD) - SPAWN_POS_SPREAD / 2;
        let z = pos.0.z + rng.random_range(0..SPAWN_POS_SPREAD) - SPAWN_POS_SPREAD / 2;
        let y = world.get_heightmap_height(ChunkHeightmapType::WorldSurface, x, z);
        let offset = BlockPos::new(x, y, z);
        if world.is_village(&offset)
            && MobEntity::check_monster_spawn_rules(
                &EntityType::ZOMBIE,
                world,
                &offset,
                world.is_thundering(),
                EntitySpawnReason::Event,
            )
        {
            return Some(Vector3::new(
                f64::from(x) + 0.5,
                f64::from(y),
                f64::from(z) + 0.5,
            ));
        }
    }
    None
}
