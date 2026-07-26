use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::{entity::EntityType, world::WorldEvent};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    position::BlockPos,
    vector3::Vector3,
};

use crate::{
    block::entities::BlockEntity,
    entity::r#type::check_spawn_rules,
    world::{World, natural_spawner},
};

pub struct MobSpawnerBlockEntity {
    pub position: BlockPos,
    pub delay: AtomicI32,
    pub max_delay: i32,
    pub min_delay: i32,
    pub spawn_count: i32,
    pub spawn_range: i32,
    pub max_nearby_entities: i32,
    pub required_player_range: i32,
    pub entity_type: AtomicCell<Option<&'static EntityType>>,
}

impl MobSpawnerBlockEntity {
    pub const ID: &'static str = "minecraft:mob_spawner";
    // Vanilla BaseSpawner.java:52-58 defaults.
    pub const DEFAULT_DELAY: i32 = 20;
    pub const DEFAULT_MAX_SPAWN_DELAY: i32 = 800;
    pub const DEFAULT_MIN_SPAWN_DELAY: i32 = 200;
    pub const DEFAULT_SPAWN_COUNT: i32 = 4;
    pub const DEFAULT_MAX_NEARBY_ENTITIES: i32 = 6;
    pub const DEFAULT_REQUIRED_PLAYER_RANGE: i32 = 16;
    pub const DEFAULT_SPAWN_RANGE: i32 = 4;

    #[must_use]
    pub const fn new(position: BlockPos, entity_type: Option<&'static EntityType>) -> Self {
        Self {
            position,
            delay: AtomicI32::new(Self::DEFAULT_DELAY),
            max_delay: Self::DEFAULT_MAX_SPAWN_DELAY,
            min_delay: Self::DEFAULT_MIN_SPAWN_DELAY,
            spawn_count: Self::DEFAULT_SPAWN_COUNT,
            spawn_range: Self::DEFAULT_SPAWN_RANGE,
            max_nearby_entities: Self::DEFAULT_MAX_NEARBY_ENTITIES,
            required_player_range: Self::DEFAULT_REQUIRED_PLAYER_RANGE,
            entity_type: AtomicCell::new(entity_type),
        }
    }

    pub fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) {
        nbt.put_string("id", self.resource_location().to_string());
        let position = self.get_position();
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
        // Vanilla BaseSpawner.save writes all tunables as shorts.
        nbt.put_short("Delay", self.delay.load(Ordering::Relaxed) as i16);
        nbt.put_short("MinSpawnDelay", self.min_delay as i16);
        nbt.put_short("MaxSpawnDelay", self.max_delay as i16);
        nbt.put_short("SpawnCount", self.spawn_count as i16);
        nbt.put_short("MaxNearbyEntities", self.max_nearby_entities as i16);
        nbt.put_short("RequiredPlayerRange", self.required_player_range as i16);
        nbt.put_short("SpawnRange", self.spawn_range as i16);
        if let Some(entity_type) = self.entity_type.load() {
            let mut spawn_entry = NbtCompound::new();

            let mut entity_nbt = NbtCompound::new();
            entity_nbt.put_string("id", format!("minecraft:{}", entity_type.resource_name));

            spawn_entry.put_compound("entity", entity_nbt);

            nbt.put_compound("SpawnData", spawn_entry);
        }
    }
}

/// Read a spawner tunable stored as a short by vanilla, tolerating the int
/// encoding older Pumpkin builds wrote.
fn short_or_int(nbt: &NbtCompound, key: &str, default: i32) -> i32 {
    nbt.get_short(key)
        .map_or_else(|| nbt.get_int(key).unwrap_or(default), i32::from)
}

impl MobSpawnerBlockEntity {
    /// Vanilla `BaseSpawner.delay` — re-roll the spawn delay and broadcast
    /// event 1 (the client spin/reset animation).
    async fn update_spawns(&self, world: &Arc<World>) {
        let min_delay = self.min_delay;
        let max_delay = self.max_delay;

        self.delay.store(
            if max_delay <= min_delay {
                min_delay
            } else {
                min_delay + rand::random_range(0..max_delay - min_delay)
            },
            Ordering::Relaxed,
        );
        world.add_synced_block_event(self.position, 1, 0).await;
    }

    /// Vanilla `BaseSpawner.isNearPlayer` (BaseSpawner.java:77-79): an alive,
    /// non-spectator player within `requiredPlayerRange` of the block center.
    fn is_near_player(&self, world: &Arc<World>) -> bool {
        let center = Vector3::new(
            f64::from(self.position.0.x) + 0.5,
            f64::from(self.position.0.y) + 0.5,
            f64::from(self.position.0.z) + 0.5,
        );
        world
            .get_nearby_players(center, f64::from(self.required_player_range))
            .iter()
            .any(|p| p.gamemode.load() != GameMode::Spectator)
    }

    /// Vanilla BaseSpawner.java:138: entities of the exact spawned type inside
    /// the block's 1x1x1 box inflated by `spawnRange` (position-based
    /// approximation of the AABB intersection test).
    fn count_nearby_same_type(&self, world: &Arc<World>, entity_type: &'static EntityType) -> i32 {
        let min_x = f64::from(self.position.0.x) - f64::from(self.spawn_range);
        let max_x = f64::from(self.position.0.x + 1) + f64::from(self.spawn_range);
        let min_y = f64::from(self.position.0.y) - f64::from(self.spawn_range);
        let max_y = f64::from(self.position.0.y + 1) + f64::from(self.spawn_range);
        let min_z = f64::from(self.position.0.z) - f64::from(self.spawn_range);
        let max_z = f64::from(self.position.0.z + 1) + f64::from(self.spawn_range);

        let mut count = 0;
        for entity in world.entities.load().iter() {
            let base = entity.get_entity();
            if base.entity_type.id != entity_type.id {
                continue;
            }
            let pos = base.pos.load();
            if pos.x >= min_x
                && pos.x <= max_x
                && pos.y >= min_y
                && pos.y <= max_y
                && pos.z >= min_z
                && pos.z <= max_z
            {
                count += 1;
            }
        }
        count
    }

    pub fn set_entity_type(&self, entity_type: &'static EntityType) {
        self.entity_type.store(Some(entity_type));
    }
}

impl BlockEntity for MobSpawnerBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let Some(entity_type) = self.entity_type.load() else {
                return;
            };
            // Vanilla BaseSpawner.serverTick (BaseSpawner.java:101-103):
            // inactive without a player in range.
            if !self.is_near_player(world) {
                return;
            }
            if self.delay.load(Ordering::Relaxed) == -1 {
                self.update_spawns(world).await;
            }
            let delay = self.delay.load(Ordering::Relaxed);
            if delay > 0 {
                self.delay.store(delay - 1, Ordering::Relaxed);
                return;
            }

            let is_thundering = world.weather.lock().await.thundering;
            let spawn_range = self.spawn_range;
            let mut spawned = false;
            for _ in 0..self.spawn_count {
                let pos = self.position.0;

                // Vanilla BaseSpawner.java:122: pos + (nextDouble - nextDouble)
                // * spawnRange + 0.5 — a symmetric spread centered on the block.
                let spawn_pos = Vector3::new(
                    f64::from(pos.x)
                        + (rand::random::<f64>() - rand::random::<f64>()) * f64::from(spawn_range)
                        + 0.5,
                    f64::from(pos.y + rand::random_range(0..3) - 1),
                    f64::from(pos.z)
                        + (rand::random::<f64>() - rand::random::<f64>()) * f64::from(spawn_range)
                        + 0.5,
                );
                // TODO: we should use getSpawnBox, but this is only modified for slimes and magma slimes
                if !world.is_space_empty(BoundingBox::new_from_pos(
                    spawn_pos.x,
                    spawn_pos.y,
                    spawn_pos.z,
                    &EntityDimensions {
                        width: entity_type.dimension[0],
                        height: entity_type.dimension[1],
                        eye_height: entity_type.eye_height,
                    },
                )) {
                    continue;
                }

                // Vanilla BaseSpawner.java:129: SpawnPlacements.checkSpawnRules
                // with EntitySpawnReason.SPAWNER — light rules still apply, so
                // lighting up a dungeon disables its spawner.
                let block_pos = BlockPos::new(
                    spawn_pos.x.floor() as i32,
                    spawn_pos.y.floor() as i32,
                    spawn_pos.z.floor() as i32,
                );
                if !natural_spawner::is_spawn_position_ok(world, &block_pos, entity_type) {
                    continue;
                }
                if !check_spawn_rules(entity_type, world, &block_pos, is_thundering) {
                    continue;
                }

                // Vanilla BaseSpawner.java:138-140: cap same-type entities
                // around the spawner; when full, re-roll the delay and stop.
                if self.count_nearby_same_type(world, entity_type) >= self.max_nearby_entities {
                    self.update_spawns(world).await;
                    return;
                }

                let entity = crate::entity::r#type::from_type(
                    entity_type,
                    spawn_pos,
                    world,
                    uuid::Uuid::new_v4(),
                );
                // Vanilla BaseSpawner.java:141: random yaw on spawn.
                entity
                    .get_entity()
                    .set_rotation(rand::random::<f32>() * 360.0, 0.0);
                world.spawn_entity(entity).await;
                world.sync_world_event(WorldEvent::ParticlesMobblockSpawn, self.position, 0);
                spawned = true;
            }
            if spawned {
                self.update_spawns(world).await;
            }
        })
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let delay = short_or_int(nbt, "Delay", Self::DEFAULT_DELAY);
        let min_delay = short_or_int(nbt, "MinSpawnDelay", Self::DEFAULT_MIN_SPAWN_DELAY);
        let max_delay = short_or_int(nbt, "MaxSpawnDelay", Self::DEFAULT_MAX_SPAWN_DELAY);
        let spawn_count = short_or_int(nbt, "SpawnCount", Self::DEFAULT_SPAWN_COUNT);
        let spawn_range = short_or_int(nbt, "SpawnRange", Self::DEFAULT_SPAWN_RANGE);
        let max_nearby_entities =
            short_or_int(nbt, "MaxNearbyEntities", Self::DEFAULT_MAX_NEARBY_ENTITIES);
        let required_player_range = short_or_int(
            nbt,
            "RequiredPlayerRange",
            Self::DEFAULT_REQUIRED_PLAYER_RANGE,
        );

        let entity_type = nbt
            .get_compound("SpawnData")
            .and_then(|data| data.get_compound("entity"))
            .and_then(|entity| entity.get_string("id"))
            .and_then(|id| {
                let name = id.strip_prefix("minecraft:").unwrap_or(id);
                EntityType::from_name(name)
            });

        Self {
            position,
            delay: AtomicI32::new(delay),
            max_delay,
            min_delay,
            spawn_count,
            spawn_range,
            max_nearby_entities,
            required_player_range,
            entity_type: AtomicCell::new(entity_type),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.write_nbt(nbt);
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut final_nbt = NbtCompound::new();
        if let Some(entity_type) = self.entity_type.load() {
            let mut spawn_entry = NbtCompound::new();

            let mut entity_nbt = NbtCompound::new();
            entity_nbt.put_string("id", format!("minecraft:{}", entity_type.resource_name));

            spawn_entry.put_compound("entity", entity_nbt);

            final_nbt.put_compound("SpawnData", spawn_entry);
        }
        Some(final_nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
