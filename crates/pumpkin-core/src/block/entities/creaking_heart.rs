use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering::Relaxed};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use pumpkin_data::block_properties::{
    CreakingHeartLikeProperties, CreakingHeartState, ResinClumpProperties,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_status::EntityStatus;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::{
    boundingbox::BoundingBox, position::BlockPos, vector3::Vector3, wrap_degrees,
};
use pumpkin_world::world::BlockFlags;
use rand::{RngExt, seq::SliceRandom};
use rustc_hash::FxHashSet;
use uuid::Uuid;

use crate::block::blocks::creaking_heart::CreakingHeartBlock;
use crate::entity::mob::{Mob, creaking::CreakingEntity, spawn::EntitySpawnReason};
use crate::entity::spawn_util::{SpawnStrategy, try_spawn_mob};
use crate::entity::{Entity, EntityBase};
use crate::world::{World, entity_islands::BLOCK_UPDATE_REACH, game_event::GameEventContext};

use super::BlockEntity;

const PLAYER_DETECTION_RANGE: f64 = 32.0;
const CREAKING_ROAMING_RADIUS: f64 = 32.0;
const DISTANCE_CREAKING_TOO_FAR: f64 = 34.0;
const SPAWN_RANGE_XZ: i32 = 16;
const SPAWN_RANGE_Y: i32 = 8;
const ATTEMPTS_PER_SPAWN: i32 = 5;
const UPDATE_TICKS: i32 = 20;
const UPDATE_TICKS_VARIANCE: i32 = 5;
const HURT_CALL_TOTAL_TICKS: u8 = 100;
const HURT_CALL_INTERVAL: u8 = 10;
const HURT_CALL_PARTICLE_TICKS: u8 = 50;
const CREAKING_ORANGE: i32 = 16_545_810;
const CREAKING_GRAY: i32 = 6_250_335;
const MAX_DEPTH: u8 = 2;
const MAX_COUNT: usize = 64;
const TICKS_GRACE_PERIOD: u8 = 30;

struct Protector {
    uuid: Uuid,
    entity: Option<Weak<dyn EntityBase>>,
    ticks_missing: u8,
}

impl Protector {
    const fn pending(uuid: Uuid) -> Self {
        Self {
            uuid,
            entity: None,
            ticks_missing: 0,
        }
    }

    fn loaded(entity: &Arc<dyn EntityBase>) -> Self {
        Self {
            uuid: entity.get_entity().entity_uuid,
            entity: Some(Arc::downgrade(entity)),
            ticks_missing: 0,
        }
    }
}

struct HurtEmitter {
    remaining_ticks: u8,
    target: Vector3<f64>,
}

struct HeartState {
    protector: Option<Protector>,
    emitter: Option<HurtEmitter>,
    ticker: i32,
    retired: bool,
}

pub struct CreakingHeartBlockEntity {
    position: BlockPos,
    state: Mutex<HeartState>,
    output_signal: AtomicU8,
    comparator_dirty: AtomicBool,
}

impl BlockEntity for CreakingHeartBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self {
        let heart = Self::new(position);
        let creaking_uuid = nbt.get_uuid("creaking").or_else(|| {
            nbt.get_string("creaking")
                .and_then(|uuid| Uuid::parse_str(uuid).ok())
                .or_else(|| {
                    nbt.get_string("creaking_uuid")
                        .and_then(|uuid| Uuid::parse_str(uuid).ok())
                })
        });
        heart.lock_state().protector = creaking_uuid.map(Protector::pending);
        heart
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(uuid) = self.creaking_uuid() {
            nbt.put_uuid("creaking", uuid);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn on_nbt_loaded(&self, _world: &Arc<World>, previous: Option<&dyn BlockEntity>) {
        let Some(previous) = previous.and_then(|previous| previous.as_any().downcast_ref::<Self>())
        else {
            return;
        };
        // Applying NBT replaces the Rust block entity; vanilla retains the live tick state.
        let (ticker, emitter) = {
            let mut state = previous.lock_state();
            state.retired = true;
            (state.ticker, state.emitter.take())
        };
        let mut state = self.lock_state();
        state.ticker = ticker;
        state.emitter = emitter;
        self.output_signal.store(previous.output_signal(), Relaxed);
        self.comparator_dirty
            .store(previous.is_comparator_dirty(), Relaxed);
    }

    fn tick(&self, world: &Arc<World>) {
        let Some(state_id) = world.get_block_state_id_if_loaded(&self.position) else {
            return;
        };
        if state_id.to_block() != &Block::CREAKING_HEART {
            return;
        }
        let mut props = CreakingHeartLikeProperties::from_state_id(state_id);
        if props.creaking_heart_state == CreakingHeartState::Uprooted {
            return;
        }

        let (protector, expired) = {
            let mut state = self.lock_state();
            if state.retired {
                return;
            }
            if let Some(protector) = &mut state.protector {
                protector.ticks_missing = protector.ticks_missing.saturating_add(1);
            }
            let had_protector = state.protector.is_some();
            let protector = Self::resolve_protector(world, &mut state);
            (protector, had_protector && state.protector.is_none())
        };
        if expired {
            self.mark_changed(world);
        }
        let distance = protector
            .as_ref()
            .map(|creaking| self.distance_to_creaking(creaking.as_ref()));
        let signal = distance.map_or(0, |distance| {
            15 - (distance.clamp(0.0, CREAKING_ROAMING_RADIUS) / CREAKING_ROAMING_RADIUS * 15.0)
                .floor() as u8
        });
        if self.output_signal.swap(signal, Relaxed) != signal {
            self.comparator_dirty.store(true, Relaxed);
        }
        self.tick_hurt_emitter(world, protector.as_deref());

        let has_protector = {
            let mut state = self.lock_state();
            let tick_due = state.ticker < 0;
            state.ticker -= 1;
            if !tick_due || state.retired {
                return;
            }
            state.ticker = UPDATE_TICKS + rand::rng().random_range(0..UPDATE_TICKS_VARIANCE);
            state.protector.is_some()
        };
        let active = CreakingHeartBlock::active_state(world, &self.position);
        props.creaking_heart_state = if has_protector
            || CreakingHeartBlock::has_required_logs(world, &self.position, props.axis)
        {
            active
        } else {
            CreakingHeartState::Uprooted
        };
        let updated = props.to_state_id(&Block::CREAKING_HEART);
        if updated != state_id
            && world
                .set_block_state_if(&self.position, updated, BlockFlags::NOTIFY_ALL, |id| {
                    id == state_id
                })
                .is_none()
        {
            return;
        }

        if let Some(creaking) = protector.as_deref().and_then(Self::as_creaking) {
            let persistent = creaking.get_mob_entity().persistence_required.load(Relaxed);
            if (active == CreakingHeartState::Dormant && !persistent)
                || distance.is_some_and(|distance| distance > DISTANCE_CREAKING_TOO_FAR)
                || creaking.player_is_stuck_in_you()
            {
                self.remove_protector(world, None, None, None, false);
            }
        } else if !has_protector && props.creaking_heart_state == CreakingHeartState::Awake {
            self.try_spawn_protector(world);
        }
    }

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, _position: &BlockPos) {
        let _ = self.destroy_protector(world, None, None, None);
    }

    fn is_comparator_dirty(&self) -> bool {
        self.comparator_dirty.load(Relaxed)
    }

    fn clear_comparator_dirty(&self) {
        self.comparator_dirty.store(false, Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CreakingHeartBlockEntity {
    pub const ID: &'static str = "minecraft:creaking_heart";

    pub(crate) fn resin_write_reach(position: BlockPos) -> BoundingBox {
        // The search finds a log, then writes to one adjacent block and notifies its neighbors.
        let reach = f64::from(MAX_DEPTH) + 1.0 + BLOCK_UPDATE_REACH;
        BoundingBox::from_block(&position).expand_all(reach)
    }

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            state: Mutex::new(HeartState {
                protector: None,
                emitter: None,
                ticker: 0,
                retired: false,
            }),
            output_signal: AtomicU8::new(0),
            comparator_dirty: AtomicBool::new(false),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, HeartState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn is_current(&self, world: &World) -> bool {
        world
            .get_block_entity(&self.position)
            .is_some_and(|current| {
                current
                    .as_any()
                    .downcast_ref::<Self>()
                    .is_some_and(|heart| std::ptr::eq(self, heart))
            })
    }

    fn mark_changed(&self, world: &World) {
        // Periodic saves serialize the chunk copy, not the live block entity.
        world.update_block_entity_nbt(self, |nbt| {
            nbt.child_tags.remove("creaking_uuid");
            if let Some(uuid) = self.creaking_uuid() {
                nbt.put_uuid("creaking", uuid);
            } else {
                nbt.child_tags.remove("creaking");
            }
        });
    }

    fn is_current_and_awake(&self, world: &World) -> bool {
        world
            .get_block_state_id_if_loaded(&self.position)
            .is_some_and(|id| {
                id.to_block() == &Block::CREAKING_HEART
                    && CreakingHeartLikeProperties::from_state_id(id).creaking_heart_state
                        == CreakingHeartState::Awake
            })
            && self.is_current(world)
    }

    #[must_use]
    pub fn output_signal(&self) -> u8 {
        self.output_signal.load(Relaxed)
    }

    fn as_creaking(entity: &dyn EntityBase) -> Option<&CreakingEntity> {
        entity.cast_any().downcast_ref()
    }

    fn resolve_protector(world: &World, state: &mut HeartState) -> Option<Arc<dyn EntityBase>> {
        let protector = state.protector.as_mut()?;
        if let Some(loaded) = protector.entity.as_ref() {
            if let Some(creaking) = loaded.upgrade()
                && !creaking.get_entity().is_removed()
                && creaking.get_entity().world.load().uuid == world.uuid
            {
                return Some(creaking);
            }
            protector.entity = None;
            protector.ticks_missing = 0;
        }
        if let Some(creaking) = world.get_entity_by_uuid(protector.uuid)
            && !creaking.get_entity().is_removed()
            && creaking.get_entity().world.load().uuid == world.uuid
            && Self::as_creaking(creaking.as_ref()).is_some()
        {
            *protector = Protector::loaded(&creaking);
            return Some(creaking);
        }
        if protector.ticks_missing >= TICKS_GRACE_PERIOD {
            state.protector = None;
        }
        None
    }

    #[must_use]
    pub fn creaking_uuid(&self) -> Option<Uuid> {
        self.lock_state()
            .protector
            .as_ref()
            .map(|protector| protector.uuid)
    }

    #[must_use]
    pub fn is_protector(&self, creaking_uuid: Uuid) -> bool {
        self.creaking_uuid() == Some(creaking_uuid)
    }

    pub(crate) fn try_adopt_legacy_protector(
        &self,
        world: &World,
        candidate: &CreakingEntity,
    ) -> bool {
        let entity = candidate.get_entity();
        if candidate.get_home_pos() != Some(self.position)
            || entity.is_removed()
            || !self.is_current(world)
        {
            return false;
        }
        let mut state = self.lock_state();
        if state.retired {
            return false;
        }
        if let Some(protector) = &state.protector {
            return protector.uuid == entity.entity_uuid;
        }
        state.protector = Some(Protector::pending(entity.entity_uuid));
        drop(state);
        self.mark_changed(world);
        true
    }

    fn distance_to_creaking(&self, creaking: &dyn EntityBase) -> f64 {
        let bottom_center = self.position.to_f64();
        creaking
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&bottom_center)
            .sqrt()
    }

    fn try_spawn_protector(&self, world: &Arc<World>) {
        let position = self.position.0.to_f64();
        if !self.is_current_and_awake(world)
            || !world.should_spawn_monsters()
            || world
                .get_nearest_player(position, PLAYER_DETECTION_RANGE, |player| {
                    !player.is_spectator()
                        && player
                            .get_entity()
                            .pos
                            .load()
                            .squared_distance_to_vec(&position)
                            < PLAYER_DETECTION_RANGE * PLAYER_DETECTION_RANGE
                })
                .is_none()
        {
            return;
        }
        let Some(creaking) = try_spawn_mob(
            &EntityType::CREAKING,
            Self::create_protector,
            world,
            &self.position,
            ATTEMPTS_PER_SPAWN,
            SPAWN_RANGE_XZ,
            SPAWN_RANGE_Y,
            SpawnStrategy::OnTopOfColliderNoLeaves,
            true,
        ) else {
            return;
        };
        self.finish_spawn_protector(world, &creaking);
    }

    fn validate_spawn(&self, world: &World, creaking: &CreakingEntity) -> bool {
        if !creaking.is_live_in_world(world)
            || creaking
                .get_home_pos()
                .is_some_and(|home| home != self.position)
        {
            return false;
        }
        if self.is_current_and_awake(world) {
            return true;
        }
        creaking.tear_down();
        false
    }

    fn finish_spawn_protector(&self, world: &Arc<World>, creaking: &Arc<CreakingEntity>) {
        if !self.validate_spawn(world, creaking) {
            return;
        }
        creaking.play_ambient_sound();
        if !self.validate_spawn(world, creaking) {
            return;
        }
        let entity = creaking.get_entity();
        world.emit_game_event_with_context(
            "entity_place",
            entity.pos.load(),
            &GameEventContext {
                source_entity: Some(creaking.as_ref()),
            },
        );
        if !self.validate_spawn(world, creaking) {
            return;
        }
        world.broadcast_entity_event(entity, EntityStatus::Poof, ActorEventID::None);
        if !self.validate_spawn(world, creaking) {
            return;
        }
        creaking.set_transient(self.position);
        if !self.validate_spawn(world, creaking) {
            return;
        }

        let protector: Arc<dyn EntityBase> = creaking.clone();
        let linked = {
            let mut state = self.lock_state();
            if !state.retired && state.protector.is_none() {
                state.protector = Some(Protector::loaded(&protector));
                true
            } else {
                false
            }
        };
        if !linked {
            creaking.tear_down();
            return;
        }
        self.mark_changed(world);
        creaking.make_sound(Sound::EntityCreakingSpawn);
        if self.can_notify_protector(world, creaking) {
            world.play_sound(
                Sound::BlockCreakingHeartSpawn,
                SoundCategory::Blocks,
                &self.position.to_centered_f64(),
            );
        }
    }

    fn create_protector(entity: Entity) -> Arc<CreakingEntity> {
        let world = entity.world.load_full();
        entity.set_rotation(wrap_degrees(rand::rng().random_range(0.0..360.0)), 0.0);
        entity.body_yaw.store(entity.yaw.load());
        let creaking = CreakingEntity::new(entity);
        creaking.finalize_spawn(&world, EntitySpawnReason::Spawner, None);
        creaking
    }

    #[must_use]
    pub fn destroy_protector(
        &self,
        world: &World,
        damage_type: Option<DamageType>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        self.remove_protector(world, damage_type, source, cause, true)
    }

    fn remove_protector(
        &self,
        world: &World,
        damage_type: Option<DamageType>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
        retire: bool,
    ) -> bool {
        let (protector, changed) = {
            let mut state = self.lock_state();
            if state.retired {
                return false;
            }
            state.retired = retire;
            let had_protector = state.protector.is_some();
            let protector = Self::resolve_protector(world, &mut state);
            if protector.is_some() {
                state.protector = None;
            }
            (protector, had_protector && state.protector.is_none())
        };
        if changed {
            self.mark_changed(world);
        }
        if let Some(creaking) = protector.as_deref().and_then(Self::as_creaking) {
            if let Some(damage_type) = damage_type {
                creaking.start_teardown(damage_type, source, cause);
            } else {
                creaking.tear_down();
            }
        }
        true
    }

    pub(crate) fn can_notify_protector(&self, world: &World, creaking: &CreakingEntity) -> bool {
        creaking.is_live_in_world(world)
            && creaking.get_home_pos() == Some(self.position)
            && self.is_protector(creaking.get_entity().entity_uuid)
            && self.is_current(world)
    }

    pub fn creaking_hurt(&self, world: &Arc<World>, creaking: &CreakingEntity) {
        let entity = creaking.get_entity();
        let bounds = entity.bounding_box.load();
        let state_id = world.get_block_state_id(&self.position);
        if state_id.to_block() != &Block::CREAKING_HEART {
            return;
        }
        let mut state = self.lock_state();
        if state.retired
            || state.emitter.is_some()
            || state
                .protector
                .as_ref()
                .is_none_or(|protector| protector.uuid != entity.entity_uuid)
        {
            return;
        }
        state.emitter = Some(HurtEmitter {
            remaining_ticks: HURT_CALL_TOTAL_TICKS,
            target: bounds.get_center(),
        });
        drop(state);

        self.emit_particles(world, bounds, 20, false);
        if CreakingHeartLikeProperties::from_state_id(state_id).creaking_heart_state
            == CreakingHeartState::Awake
        {
            let clumps = rand::rng().random_range(2..=3);
            for _ in 0..clumps {
                if !self.can_notify_protector(world, creaking) || !self.is_current_and_awake(world)
                {
                    break;
                }
                if let Some(pos) = self.spread_resin(world) {
                    world.play_sound(
                        Sound::BlockResinPlace,
                        SoundCategory::Blocks,
                        &pos.to_centered_f64(),
                    );
                    world.emit_game_event("block_place", pos.to_centered_f64());
                }
            }
        }
    }

    fn tick_hurt_emitter(&self, world: &World, protector: Option<&dyn EntityBase>) {
        let bounds = protector.map(|creaking| creaking.get_entity().bounding_box.load());
        let (remaining, sound_target) = {
            let mut state = self.lock_state();
            let Some(emitter) = &mut state.emitter else {
                return;
            };
            let remaining = emitter.remaining_ticks;
            let sound_target = remaining.is_multiple_of(HURT_CALL_INTERVAL).then(|| {
                if let Some(bounds) = bounds {
                    emitter.target = bounds.get_center();
                }
                emitter.target
            });
            emitter.remaining_ticks -= 1;
            if emitter.remaining_ticks == 0 {
                state.emitter = None;
            }
            (remaining, sound_target)
        };
        if remaining > HURT_CALL_PARTICLE_TICKS
            && let Some(bounds) = bounds
        {
            self.emit_particles(world, bounds, 1, true);
            self.emit_particles(world, bounds, 1, false);
        }
        if let Some(target) = sound_target {
            let progress = 0.2
                + 0.8 * f32::from(HURT_CALL_TOTAL_TICKS - remaining)
                    / f32::from(HURT_CALL_TOTAL_TICKS);
            let location =
                target + (self.position.to_centered_f64() - target) * f64::from(progress);
            let volume = f32::from(remaining) / 2.0 / f32::from(HURT_CALL_TOTAL_TICKS) + 0.5;
            world.play_sound_fine(
                Sound::BlockCreakingHeartHurt,
                SoundCategory::Blocks,
                &BlockPos::floored_v(location).to_centered_f64(),
                volume,
                1.0,
            );
        }
    }

    fn emit_particles(
        &self,
        world: &World,
        bounds: BoundingBox,
        count: usize,
        towards_creaking: bool,
    ) {
        let mut rng = rand::rng();
        let color = if towards_creaking {
            CREAKING_ORANGE
        } else {
            CREAKING_GRAY
        };
        for _ in 0..count {
            let creaking = Vector3::new(
                bounds.min.x + rng.random::<f64>() * (bounds.max.x - bounds.min.x),
                bounds.min.y + rng.random::<f64>() * (bounds.max.y - bounds.min.y),
                bounds.min.z + rng.random::<f64>() * (bounds.max.z - bounds.min.z),
            );
            let heart = self.position.0.to_f64()
                + Vector3::new(
                    rng.random::<f64>(),
                    rng.random::<f64>(),
                    rng.random::<f64>(),
                );
            let (source, target) = if towards_creaking {
                (heart, creaking)
            } else {
                (creaking, heart)
            };
            world.spawn_trail_particle(source, target, color, rng.random_range(10..50));
        }
    }

    fn spread_resin(&self, world: &Arc<World>) -> Option<BlockPos> {
        let mut rng = rand::rng();
        let mut pending = std::collections::VecDeque::from([(self.position, 0)]);
        let mut visited = FxHashSet::default();
        while let Some((pos, depth)) = pending.pop_front() {
            if !visited.insert(pos) {
                continue;
            }
            if world
                .get_block_state_id(&pos)
                .to_block()
                .has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS)
            {
                let mut faces = BlockDirection::all();
                faces.shuffle(&mut rng);
                for face in faces {
                    let target = pos.offset(face.to_offset());
                    if Self::place_resin(world, target, face.opposite()) {
                        return Some(target);
                    }
                }
            }
            if visited.len() >= MAX_COUNT {
                break;
            }
            if depth < MAX_DEPTH {
                let mut faces = BlockDirection::all();
                faces.shuffle(&mut rng);
                for face in faces {
                    let neighbor = pos.offset(face.to_offset());
                    if world
                        .get_block_state_id(&neighbor)
                        .to_block()
                        .has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS)
                    {
                        pending.push_back((neighbor, depth + 1));
                    }
                }
            }
        }
        None
    }

    fn place_resin(world: &Arc<World>, pos: BlockPos, face: BlockDirection) -> bool {
        loop {
            let Some(state) = world.get_block_state_id_if_loaded(&pos) else {
                return false;
            };
            let Some(updated) = Self::resin_with_face(state, face) else {
                return false;
            };
            if world
                .set_block_state_if(&pos, updated, BlockFlags::NOTIFY_ALL, |id| id == state)
                .is_some()
            {
                return true;
            }
            if !world.is_in_build_limit(pos) {
                return false;
            }
        }
    }

    fn resin_with_face(state: BlockStateId, face: BlockDirection) -> Option<BlockStateId> {
        let block = state.to_block();
        let mut resin = if block == &Block::RESIN_CLUMP {
            ResinClumpProperties::from_state_id(state)
        } else if state.to_state().is_air() || state == Block::WATER.default_state.id {
            let mut default = ResinClumpProperties::default(&Block::RESIN_CLUMP);
            default.waterlogged = block == &Block::WATER;
            default
        } else {
            return None;
        };
        let attached = match face {
            BlockDirection::Down => &mut resin.down,
            BlockDirection::Up => &mut resin.up,
            BlockDirection::North => &mut resin.north,
            BlockDirection::South => &mut resin.south,
            BlockDirection::West => &mut resin.west,
            BlockDirection::East => &mut resin.east,
        };
        if *attached {
            return None;
        }
        *attached = true;
        Some(resin.to_state_id(&Block::RESIN_CLUMP))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use pumpkin_data::dimension::Dimension;
    use pumpkin_world::chunk::{ChunkData, format::anvil::SingleChunkDataSerializer};

    use super::*;
    use crate::{
        entity::ai::test_support::{server, world},
        plugin::{
            BoxFuture, EventHandler,
            api::events::{EventPriority, world::generic_game::GenericGameEvent},
        },
        server::Server,
    };

    struct RemoveProtector {
        world: Arc<World>,
        entity_id: i32,
        event_key: &'static str,
        calls: Arc<AtomicUsize>,
    }

    impl EventHandler<GenericGameEvent> for RemoveProtector {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.event_key == self.event_key {
                    self.calls.fetch_add(1, Relaxed);
                    if let Some(protector) = self.world.get_entity_by_id(self.entity_id) {
                        self.world.remove_entity(protector.as_ref());
                    }
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn heart_effects_stop_when_a_game_event_removes_the_protector()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let position = BlockPos::new(8, 64, 8);
        let chunk_pos = position.chunk_position();
        world
            .level
            .loaded_chunks
            .insert(chunk_pos, ChunkData::empty_sync(chunk_pos.x, chunk_pos.y));
        let awake = CreakingHeartLikeProperties {
            creaking_heart_state: CreakingHeartState::Awake,
            ..CreakingHeartLikeProperties::from_state_id(Block::CREAKING_HEART.default_state.id)
        };
        for log in [position.up(), position.down()] {
            world.set_block_state(
                &log,
                Block::PALE_OAK_LOG.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
        }
        world.set_block_state(
            &position,
            awake.to_state_id(&Block::CREAKING_HEART),
            BlockFlags::NOTIFY_ALL,
        );
        let block_entity = world.get_block_entity(&position).ok_or("missing heart")?;
        let heart: &CreakingHeartBlockEntity = block_entity
            .as_any()
            .downcast_ref()
            .ok_or("wrong heart type")?;

        let create = || {
            CreakingEntity::new(Entity::new(
                world.clone(),
                Vector3::new(10.5, 64.0, 8.5),
                &EntityType::CREAKING,
            ))
        };
        let interrupted = create();
        let make_listener = |event_key, protector: &CreakingEntity| {
            let calls = Arc::new(AtomicUsize::new(0));
            server.plugin_manager.register::<GenericGameEvent, _>(
                Arc::new(RemoveProtector {
                    world: world.clone(),
                    entity_id: protector.get_entity().entity_id,
                    event_key,
                    calls: calls.clone(),
                }),
                EventPriority::Normal,
                true,
            );
            calls
        };
        let spawn_calls = make_listener("entity_place", &interrupted);
        assert!(world.spawn_entity(interrupted.clone()));
        heart.finish_spawn_protector(&world, &interrupted);
        assert_eq!(spawn_calls.load(Relaxed), 1);
        assert!(interrupted.get_entity().is_removed());
        assert_eq!(interrupted.get_home_pos(), None);
        assert_eq!(heart.creaking_uuid(), None);

        let protector = create();
        assert!(world.spawn_entity(protector.clone()));
        heart.finish_spawn_protector(&world, &protector);
        assert_eq!(
            heart.creaking_uuid(),
            Some(protector.get_entity().entity_uuid)
        );
        let resin_calls = make_listener("block_place", &protector);
        heart.creaking_hurt(&world, &protector);
        assert_eq!(resin_calls.load(Relaxed), 1);
        assert!(protector.get_entity().is_removed());
        server.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn adopted_protector_is_in_the_chunk_save_until_the_link_expires()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let position = BlockPos::new(0, 64, 0);
        let chunk_pos = position.chunk_position();
        let chunk = ChunkData::empty_sync(chunk_pos.x, chunk_pos.y);
        world.level.loaded_chunks.insert(chunk_pos, chunk.clone());
        world.set_block_state(
            &position,
            Block::CREAKING_HEART.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        let heart = Arc::new(CreakingHeartBlockEntity::new(position));
        world.add_block_entity(heart.clone());
        let mut original = NbtCompound::new();
        heart.write_internal(&mut original);
        original.put_int("other_data", 42);
        world.add_block_entity_nbt(position, &original);
        let candidate = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.5, 64.0, 0.5),
            &EntityType::CREAKING,
        ));
        candidate.set_transient(position);
        let uuid = candidate.get_entity().entity_uuid;
        assert!(heart.try_adopt_legacy_protector(&world, &candidate));
        let saved_nbt = || -> Result<NbtCompound, Box<dyn std::error::Error>> {
            let stored = ChunkData::from_bytes(&chunk.to_pump_bytes()?, chunk_pos)?;
            let entries = stored
                .pending_block_entities
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            entries
                .get(&position)
                .cloned()
                .ok_or_else(|| "Missing saved heart".into())
        };
        assert_eq!(saved_nbt()?.get_uuid("creaking"), Some(uuid));
        heart
            .lock_state()
            .protector
            .as_mut()
            .ok_or("Missing live link")?
            .ticks_missing = TICKS_GRACE_PERIOD;
        assert!(heart.remove_protector(&world, None, None, None, false));
        let cleared = saved_nbt()?;
        assert!(cleared.get("creaking").is_none());
        assert_eq!(cleared.get_int("other_data"), Some(42));
        world.level.shutdown().await;
        Ok(())
    }
}
