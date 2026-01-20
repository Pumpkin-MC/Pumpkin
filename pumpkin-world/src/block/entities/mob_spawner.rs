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
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    position::BlockPos,
    vector3::Vector3,
};

use crate::{block::entities::BlockEntity, world::SimpleWorld};

pub struct MobSpawnerBlockEntity {
    pub position: BlockPos,
    pub delay: AtomicI32,
    pub max_delay: i32,
    pub min_delay: i32,
    pub spawn_count: i32,
    pub spawn_range: i32,
    pub entity_type: AtomicCell<Option<&'static EntityType>>,
}

impl MobSpawnerBlockEntity {
    pub const ID: &'static str = "minecraft:mob_spawner";
    pub const DEFAULT_DELAY: i32 = 20;
    pub const DEFAULT_MAX_SPAWN_DELAY: i32 = 800;
    pub const DEFAULT_MIN_SPAWN_DELAY: i32 = 200;
    pub const DEFAULT_SPAWN_COUNT: i32 = 4;
    pub const DEFAULT_SPAWN_RANGE: i32 = 4;

    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            delay: AtomicI32::new(Self::DEFAULT_DELAY),
            max_delay: Self::DEFAULT_MAX_SPAWN_DELAY,
            min_delay: Self::DEFAULT_MIN_SPAWN_DELAY,
            spawn_count: Self::DEFAULT_SPAWN_COUNT,
            spawn_range: Self::DEFAULT_SPAWN_RANGE,
            entity_type: AtomicCell::new(None),
        }
    }
}

impl MobSpawnerBlockEntity {
    async fn update_spawns(&self, world: &Arc<dyn SimpleWorld>) {
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
        world.add_synced_block_event(self.position, 1, 0).await
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

    fn tick<'a>(
        &'a self,
        world: Arc<dyn SimpleWorld>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(entity_type) = &self.entity_type.load() {
                if self.delay.load(Ordering::Relaxed) == -1 {
                    self.update_spawns(&world).await;
                } else {
                    self.delay.fetch_sub(1, Ordering::Relaxed);
                    return;
                }
                let spawn_range = self.spawn_range;
                let mut update_spawns = false;
                for _ in 0..self.spawn_count {
                    let pos = self.position.0;

                    let spawn_pos = Vector3::new(
                        pos.x as f64
                            + (rand::random::<f64>() + rand::random::<f64>()) * spawn_range as f64
                            + 0.5,
                        (pos.y + rand::random_range(0..3) - 1) as f64,
                        pos.z as f64
                            + (rand::random::<f64>() + rand::random::<f64>()) * spawn_range as f64
                            + 0.5,
                    );
                    // TODO: we should use getSpawnBox, but this is only modified for slimes and magma slimes
                    if !world
                        .is_space_empty(BoundingBox::new_from_pos(
                            spawn_pos.x,
                            spawn_pos.y,
                            spawn_pos.z,
                            &EntityDimensions {
                                width: entity_type.dimension[0],
                                height: entity_type.dimension[1],
                                eye_height: entity_type.eye_height,
                            },
                        ))
                        .await
                    {
                        continue;
                    }
                    world.clone().spawn_from_type(entity_type, spawn_pos).await;
                    world
                        .sync_world_event(WorldEvent::SpawnerSpawnsMob, self.position, 0)
                        .await;
                    update_spawns = true;
                }
                if update_spawns {
                    self.update_spawns(&world).await;
                }
            }
        })
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let delay = nbt.get_short("Delay").unwrap_or(Self::DEFAULT_DELAY as i16) as i32;
        let min_delay = nbt
            .get_int("MinSpawnDelay")
            .unwrap_or(Self::DEFAULT_MIN_SPAWN_DELAY);
        let max_delay = nbt
            .get_int("MaxSpawnDelay")
            .unwrap_or(Self::DEFAULT_MAX_SPAWN_DELAY);
        let spawn_count = nbt
            .get_int("SpawnCount")
            .unwrap_or(Self::DEFAULT_SPAWN_COUNT);
        let spawn_range = nbt
            .get_int("SpawnRange")
            .unwrap_or(Self::DEFAULT_SPAWN_RANGE);
        let _spawn_entry = nbt.get_compound("SpawnData");

        Self {
            position,
            delay: AtomicI32::new(delay),
            max_delay,
            min_delay,
            spawn_count,
            spawn_range,
            entity_type: AtomicCell::new(None), // TODO
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(entity_type) = self.entity_type.load() {
                let mut entity_nbt = NbtCompound::new();
                entity_nbt.put_string("id", format!("minecraft:{}", entity_type.resource_name));

                nbt.put_component("entity", entity_nbt);
            }
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut final_nbt = NbtCompound::new();
        if let Some(entity_type) = self.entity_type.load() {
            let mut spawn_entry = NbtCompound::new();

            let mut entity_nbt = NbtCompound::new();
            entity_nbt.put_string("id", format!("minecraft:{}", entity_type.resource_name));

            spawn_entry.put_component("entity", entity_nbt);

            final_nbt.put_component("SpawnData", spawn_entry);
        }
        Some(final_nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
