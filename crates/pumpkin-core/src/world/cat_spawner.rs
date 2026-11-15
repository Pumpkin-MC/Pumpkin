use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use pumpkin_data::entity::EntityType;
use pumpkin_data::poi_type::PoiType;
use pumpkin_data::tag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::poi::Occupancy;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::spawn::{EntitySpawnReason, finalize_spawn};
use crate::entity::r#type::from_type;
use crate::world::World;
use crate::world::natural_spawner::is_spawn_position_ok;

const TICK_DELAY: i32 = 1200;
const MIN_SPAWN_DISTANCE: i32 = 8;
const SPAWN_DISTANCE_SPREAD: i32 = 24;
const LOADED_RADIUS: i32 = 10;
const VILLAGE_SECTION_DISTANCE: i32 = 2;
const VILLAGE_RADIUS: i32 = 48;
const VILLAGE_MIN_OCCUPIED_HOMES: usize = 4;
const VILLAGE_MAX_CATS: usize = 5;
const HUT_RADIUS: i32 = 16;
const CAT_SEARCH_HEIGHT: f64 = 8.0;

/// Spawns stray cats in villages with enough claimed beds, and one cat per witch hut.
pub struct CatSpawner {
    next_tick: AtomicI32,
}

impl Default for CatSpawner {
    fn default() -> Self {
        Self {
            next_tick: AtomicI32::new(0),
        }
    }
}

impl CatSpawner {
    pub fn tick(&self, world: &Arc<World>) {
        if self.next_tick.fetch_sub(1, Relaxed) - 1 > 0 {
            return;
        }
        self.next_tick.store(TICK_DELAY, Relaxed);
        let Some(player) = world.get_random_player() else {
            return;
        };
        let mut rng = rand::rng();
        let mut offset = || {
            let distance = MIN_SPAWN_DISTANCE + rng.random_range(0..SPAWN_DISTANCE_SPREAD);
            if rng.random::<bool>() {
                -distance
            } else {
                distance
            }
        };
        let x = offset();
        let z = offset();
        let spawn_pos = player
            .get_entity()
            .block_pos
            .load()
            .offset(Vector3::new(x, 0, z));
        if !world.has_chunks_at(
            spawn_pos.0.x - LOADED_RADIUS,
            spawn_pos.0.z - LOADED_RADIUS,
            spawn_pos.0.x + LOADED_RADIUS,
            spawn_pos.0.z + LOADED_RADIUS,
        ) || !is_spawn_position_ok(world, &spawn_pos, &EntityType::CAT)
        {
            return;
        }
        if world.is_close_to_village(&spawn_pos, VILLAGE_SECTION_DISTANCE) {
            Self::spawn_in_village(world, spawn_pos);
        } else if world
            .has_structure_piece_at(spawn_pos, &tag::WorldgenStructure::MINECRAFT_CATS_SPAWN_IN)
        {
            Self::spawn_in_hut(world, spawn_pos);
        }
    }

    fn spawn_in_village(world: &Arc<World>, spawn_pos: BlockPos) {
        let occupied_homes = world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_records_in_range(
                &|poi_type| poi_type == PoiType::Home,
                spawn_pos,
                VILLAGE_RADIUS,
                Occupancy::IsOccupied,
            )
            .len();
        if occupied_homes <= VILLAGE_MIN_OCCUPIED_HOMES {
            return;
        }
        let area = BoundingBox::from_block(&spawn_pos).expand(
            f64::from(VILLAGE_RADIUS),
            CAT_SEARCH_HEIGHT,
            f64::from(VILLAGE_RADIUS),
        );
        let cats =
            world.get_entities_of_type_at_box(&area, |entity_type| entity_type == &EntityType::CAT);
        if cats.len() < VILLAGE_MAX_CATS {
            Self::spawn_cat(world, spawn_pos, false);
        }
    }

    fn spawn_in_hut(world: &Arc<World>, spawn_pos: BlockPos) {
        let area = BoundingBox::from_block(&spawn_pos).expand(
            f64::from(HUT_RADIUS),
            CAT_SEARCH_HEIGHT,
            f64::from(HUT_RADIUS),
        );
        if world
            .get_entities_of_type_at_box(&area, |entity_type| entity_type == &EntityType::CAT)
            .is_empty()
        {
            Self::spawn_cat(world, spawn_pos, true);
        }
    }

    fn spawn_cat(world: &Arc<World>, spawn_pos: BlockPos, make_persistent: bool) {
        // The cat is finalized before it is moved, so its variant is picked at the origin.
        let cat = from_type(&EntityType::CAT, Vector3::default(), world, Uuid::new_v4());
        finalize_spawn(&cat, world, EntitySpawnReason::Natural, None);
        if make_persistent && let Some(mob) = cat.get_mob() {
            mob.get_mob_entity().set_persistence_required();
        }
        let entity = cat.get_entity();
        entity.set_pos(Vector3::new(
            f64::from(spawn_pos.0.x) + 0.5,
            f64::from(spawn_pos.0.y),
            f64::from(spawn_pos.0.z) + 0.5,
        ));
        entity.set_rotation(0.0, 0.0);
        world.spawn_entity(cat);
    }
}
