use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;

use crate::{
    block::entities::mob_spawner::{
        MobSpawnerBlockEntity, SpawnerParams, parse_spawner_nbt, tick_spawner,
        write_spawner_params_nbt,
    },
    entity::Entity,
};

/// Vanilla `MinecartSpawner`.
pub(super) struct SpawnerMinecart {
    delay: AtomicI32,
    min_delay: AtomicI32,
    max_delay: AtomicI32,
    spawn_count: AtomicI32,
    spawn_range: AtomicI32,
    max_nearby_entities: AtomicI32,
    required_player_range: AtomicI32,
    entity_type: AtomicCell<Option<&'static EntityType>>,
}

impl SpawnerMinecart {
    pub(super) const fn new() -> Self {
        Self {
            delay: AtomicI32::new(MobSpawnerBlockEntity::DEFAULT_DELAY),
            min_delay: AtomicI32::new(MobSpawnerBlockEntity::DEFAULT_MIN_SPAWN_DELAY),
            max_delay: AtomicI32::new(MobSpawnerBlockEntity::DEFAULT_MAX_SPAWN_DELAY),
            spawn_count: AtomicI32::new(MobSpawnerBlockEntity::DEFAULT_SPAWN_COUNT),
            spawn_range: AtomicI32::new(MobSpawnerBlockEntity::DEFAULT_SPAWN_RANGE),
            max_nearby_entities: AtomicI32::new(MobSpawnerBlockEntity::DEFAULT_MAX_NEARBY_ENTITIES),
            required_player_range: AtomicI32::new(
                MobSpawnerBlockEntity::DEFAULT_REQUIRED_PLAYER_RANGE,
            ),
            entity_type: AtomicCell::new(None),
        }
    }

    fn params(&self) -> SpawnerParams {
        SpawnerParams {
            delay: self.delay.load(Ordering::Relaxed),
            min_delay: self.min_delay.load(Ordering::Relaxed),
            max_delay: self.max_delay.load(Ordering::Relaxed),
            spawn_count: self.spawn_count.load(Ordering::Relaxed),
            spawn_range: self.spawn_range.load(Ordering::Relaxed),
            max_nearby_entities: self.max_nearby_entities.load(Ordering::Relaxed),
            required_player_range: self.required_player_range.load(Ordering::Relaxed),
            entity_type: self.entity_type.load(),
        }
    }

    pub(super) fn tick(&self, entity: &Entity) {
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        tick_spawner(&world, pos, &self.delay, &self.params(), || {
            // Vanilla `MinecartSpawner.spawner.broadcastEvent`: broadcasts entity event 1.
            world.broadcast_entity_event(entity, EntityStatus::Jump, ActorEventID::None);
        });
    }

    pub(super) fn write_nbt(&self, nbt: &mut NbtCompound) {
        write_spawner_params_nbt(&self.params(), nbt);
    }

    pub(super) fn read_nbt(&self, nbt: &NbtCompound) {
        let params = parse_spawner_nbt(nbt);
        self.delay.store(params.delay, Ordering::Relaxed);
        self.min_delay.store(params.min_delay, Ordering::Relaxed);
        self.max_delay.store(params.max_delay, Ordering::Relaxed);
        self.spawn_count
            .store(params.spawn_count, Ordering::Relaxed);
        self.spawn_range
            .store(params.spawn_range, Ordering::Relaxed);
        self.max_nearby_entities
            .store(params.max_nearby_entities, Ordering::Relaxed);
        self.required_player_range
            .store(params.required_player_range, Ordering::Relaxed);
        self.entity_type.store(params.entity_type);
    }
}
