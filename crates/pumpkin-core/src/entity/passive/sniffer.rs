mod ai;

use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::BlockState;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::boundingbox::EntityDimensions;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::{
        brain::{
            Brain, BrainState,
            memory::{PackedMemories, global_pos::GlobalPos, types},
        },
        pathfinder::{GroundPathNavigation, Navigator, PathNavigationTrait, node::PathType},
        util::land_random_pos,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity},
    passive::animal::{Animal, PARENT_AGE_AFTER_BREEDING},
    player::Player,
};
use crate::plugin::api::events::entity::entity_drop_item::EntityDropItemEvent;
use crate::world::{
    game_event::GameEventContext,
    loot::{LootContextParameters, generate_loot_from_handle},
};

pub const SNIFFER_BABY_START_AGE: i32 = -48000;
pub const DIGGING_DROP_SEED_OFFSET_TICKS: i32 = 120;
const DIGGING_BB_HEIGHT_OFFSET: f32 = 0.4;
const DIGGING_DIMENSIONS_EYE_HEIGHT: f32 = 0.81;
const MAX_EXPLORED_POSITIONS: usize = 20;
const HEAD_DISTANCE: f64 = 2.25;
const MAX_HEAD_Y_ROT: f32 = 50.0;
const DIGGING_LOOT_TABLE: &str = "minecraft:gameplay/sniffer_digging";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum SnifferState {
    #[default]
    Idling = 0,
    FeelingHappy = 1,
    Scenting = 2,
    Sniffing = 3,
    Searching = 4,
    Digging = 5,
    Rising = 6,
}

impl SnifferState {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::FeelingHappy,
            2 => Self::Scenting,
            3 => Self::Sniffing,
            4 => Self::Searching,
            5 => Self::Digging,
            6 => Self::Rising,
            _ => Self::Idling,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    const fn can_mate(self) -> bool {
        matches!(self, Self::Idling | Self::Scenting | Self::FeelingHappy)
    }
}

pub struct SnifferEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub state: AtomicI32,
    pub drop_seed_at_tick: AtomicI32,
}

impl SnifferEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        let mut navigation = GroundPathNavigation::new();
        navigation.set_can_float(true);
        navigation.set_pathfinding_malus(PathType::Water, -1.0);
        navigation.set_pathfinding_malus(PathType::DangerPowderSnow, -1.0);
        navigation.set_pathfinding_malus(PathType::DamageCautious, -1.0);
        navigation.inner.allow_water_when_burning_or_immersed = true;
        mob_entity.navigator = std::sync::Mutex::new(Navigator::new(navigation));
        Arc::new(Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            state: AtomicI32::new(SnifferState::Idling as i32),
            drop_seed_at_tick: AtomicI32::new(0),
        })
    }

    #[must_use]
    pub fn get_state(&self) -> SnifferState {
        SnifferState::from_id(self.state.load(Relaxed))
    }

    fn refresh_state_dimensions(&self) {
        let entity = self.get_entity();
        let mut dimensions = Entity::age_dimensions(entity.entity_type, self.is_baby());
        if self.get_state() == SnifferState::Digging {
            let age_scale = dimensions.width / entity.entity_type.dimension[0];
            dimensions = EntityDimensions {
                width: dimensions.width,
                height: (entity.entity_type.dimension[1] - DIGGING_BB_HEIGHT_OFFSET) * age_scale,
                eye_height: DIGGING_DIMENSIONS_EYE_HEIGHT * age_scale,
            };
        }
        entity.set_default_dimensions(dimensions);
    }

    pub fn transition_to(&self, state: SnifferState) {
        match state {
            SnifferState::FeelingHappy => {
                self.play_entity_sound(Sound::EntitySnifferHappy, 1.0, 1.0);
            }
            SnifferState::Sniffing => {
                self.play_entity_sound(Sound::EntitySnifferSniffing, 1.0, 1.0);
            }
            SnifferState::Rising => {
                self.play_entity_sound(Sound::EntitySnifferDiggingStop, 1.0, 1.0);
            }
            _ => {}
        }
        let entity = self.get_entity();
        if self.state.swap(state as i32, Relaxed) != state as i32 {
            entity.set_synced_data(
                pumpkin_data::tracked_data::sniffer::STATE,
                VarInt(state as i32),
            );
            self.refresh_state_dimensions();
        }
        match state {
            SnifferState::Scenting => self.play_entity_sound(
                Sound::EntitySnifferScenting,
                1.0,
                if self.is_baby() { 1.3 } else { 1.0 },
            ),
            SnifferState::Digging => {
                let drop_tick = entity
                    .tick_count
                    .load(Relaxed)
                    .wrapping_add(DIGGING_DROP_SEED_OFFSET_TICKS);
                self.drop_seed_at_tick.store(drop_tick, Relaxed);
                entity.set_synced_data(
                    pumpkin_data::tracked_data::sniffer::DROP_SEED_AT_TICK,
                    VarInt(drop_tick),
                );
                entity.world.load().send_entity_status(
                    entity,
                    EntityStatus::SnifferDiggingSound,
                    ActorEventID::None,
                );
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn get_head_position(&self) -> Vector3<f64> {
        let entity = self.get_entity();
        entity.pos.load() + entity.rotation().to_f64() * HEAD_DISTANCE
    }

    #[must_use]
    pub fn get_head_block(&self) -> BlockPos {
        let entity = self.get_entity();
        let head = self.get_head_position();
        BlockPos::floored(head.x, entity.pos.load().y + f64::from(0.2f32), head.z)
    }

    fn can_sniff(&self, brain: &BrainState) -> bool {
        let entity = self.get_entity();
        !brain.has_memory_value(types::TEMPTING_PLAYER.id())
            && !brain.has_memory_value(types::IS_PANICKING.id())
            && !entity.is_in_water()
            && !self.mob_entity.is_in_love()
            && entity.on_ground.load(Relaxed)
            && !entity.has_vehicle()
            && !entity.is_leashed()
    }

    fn can_dig(&self, brain: &BrainState) -> bool {
        let entity = self.get_entity();
        !brain.has_memory_value(types::IS_PANICKING.id())
            && !brain.has_memory_value(types::TEMPTING_PLAYER.id())
            && !self.is_baby()
            && !entity.is_in_water()
            && entity.on_ground.load(Relaxed)
            && !entity.has_vehicle()
            && self.can_dig_at(brain, self.get_head_block().down())
    }

    fn can_dig_at(&self, brain: &BrainState, pos: BlockPos) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        if !world
            .get_block(&pos)
            .has_tag(&tag::Block::MINECRAFT_SNIFFER_DIGGABLE_BLOCK)
            || brain
                .get(types::SNIFFER_EXPLORED_POSITIONS)
                .is_some_and(|explored| {
                    explored
                        .iter()
                        .any(|past| past.dimension.id == world.dimension.id && past.pos == pos)
                })
        {
            return false;
        }
        self.mob_entity
            .steered()
            .navigator()
            .create_path(&self.mob_entity.living_entity, pos.0.to_f64(), 1)
            .is_some_and(|path| path.can_reach())
    }

    fn calculate_dig_position(&self, brain: &BrainState) -> Option<BlockPos> {
        let world = self.get_entity().world.load();
        for index in 0..5 {
            let Some(pos) = land_random_pos::get_pos(self, 10 + 2 * index, 3) else {
                continue;
            };
            let pos = BlockPos::floored_v(pos);
            let within_border = world
                .worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains(f64::from(pos.0.x), f64::from(pos.0.z));
            if within_border && self.can_dig_at(brain, pos.down()) {
                return Some(pos.down());
            }
        }
        None
    }

    fn on_digging_complete(&self, brain: &mut BrainState, success: bool) {
        if !success {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let Some(dimension) = Dimension::from_name(world.dimension.minecraft_name) else {
            return;
        };
        let pos = entity.get_supporting_block_pos().unwrap_or_else(|| {
            let pos = entity.pos.load();
            BlockPos::floored(pos.x, pos.y - f64::from(1.0e-5f32), pos.z)
        });
        let mut explored: Vec<_> = brain
            .get(types::SNIFFER_EXPLORED_POSITIONS)
            .into_iter()
            .flatten()
            .take(MAX_EXPLORED_POSITIONS)
            .copied()
            .collect();
        // Vanilla retains twenty old entries before prepending the new supporting block.
        explored.insert(0, GlobalPos::new(dimension, pos));
        brain.set(types::SNIFFER_EXPLORED_POSITIONS, explored);
    }

    pub fn drop_seed(&self) {
        let entity = self.get_entity();
        if self.drop_seed_at_tick.load(Relaxed) != entity.tick_count.load(Relaxed) {
            return;
        }
        let world = entity.world.load_full();
        if let Some(table) = world.get_loot_table(DIGGING_LOOT_TABLE) {
            let params = LootContextParameters {
                world: Some(&world),
                position: Some(entity.pos.load()),
                this_entity: Some(self),
                ..Default::default()
            };
            let head = self.get_head_block().0.to_f64();
            for stack in generate_loot_from_handle(&table, self.get_random().random(), &params) {
                let mut event = EntityDropItemEvent::new(
                    entity.entity_id,
                    format!("minecraft:{}", stack.item.registry_key),
                    stack.item_count,
                );
                if let Some(server) = world.server.upgrade() {
                    server.plugin_manager.fire_blocking(&server, &mut event);
                }
                if !event.cancelled {
                    world.spawn_entity(Arc::new(ItemEntity::new(
                        Entity::new(world.clone(), head, &EntityType::ITEM),
                        stack,
                    )));
                }
            }
        }
        self.play_entity_sound(Sound::EntitySnifferDropSeed, 1.0, 1.0);
    }
}

impl AgeableMob for SnifferEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn get_baby_start_age(&self) -> i32 {
        SNIFFER_BABY_START_AGE
    }

    fn age_boundary_reached(&self) {
        self.refresh_state_dimensions();
    }
}

impl Animal for SnifferEntity {
    fn play_eating_sound(&self) {
        // Vanilla Sniffer.mobInteract adds a second sound after a successful feed.
        for _ in 0..2 {
            self.play_entity_sound(
                Sound::EntitySnifferEat,
                1.0,
                0.8 + self.get_random().random::<f32>() * 0.4,
            );
        }
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_SNIFFER_FOOD)
    }

    fn can_mate(&self, partner: &dyn EntityBase) -> bool {
        partner
            .cast_any()
            .downcast_ref::<Self>()
            .is_some_and(|partner| {
                self.get_entity().entity_id != partner.get_entity().entity_id
                    && self.get_state().can_mate()
                    && partner.get_state().can_mate()
                    && self.mob_entity.is_in_love()
                    && partner.mob_entity.is_in_love()
            })
    }

    fn spawns_offspring(&self) -> bool {
        false
    }

    fn on_bred_without_offspring(&self, mate: &dyn EntityBase, _brain: Option<&mut BrainState>) {
        self.set_age(PARENT_AGE_AFTER_BREEDING);
        mate.get_entity().set_age(PARENT_AGE_AFTER_BREEDING);

        let entity = self.get_entity();
        let world = entity.world.load_full();
        let egg = Arc::new(ItemEntity::new(
            Entity::new(world.clone(), entity.pos.load(), &EntityType::ITEM),
            ItemStack::new(1, &Item::SNIFFER_EGG),
        ));
        let mut random = self.get_random();
        self.play_entity_sound(
            Sound::BlockSnifferEggPlop,
            1.0,
            (random.random::<f32>() - random.random::<f32>()) * 0.2 + 0.5,
        );
        world.spawn_entity(egg);
    }

    fn on_accepted_damage(&self) {
        self.mob_entity.reset_love_ticks();
    }
}

impl Mob for SnifferEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, |brain| {
            brain.set_active_activity_to_first_valid(&[
                pumpkin_data::environment_attribute::Activity::Dig,
                pumpkin_data::environment_attribute::Activity::Sniff,
                pumpkin_data::environment_attribute::Activity::Idle,
            ]);
        });
        if self.get_age() != 0 {
            self.mob_entity.reset_love_ticks();
        }
    }

    fn mob_ai_step(&self) {
        if self.get_age() != 0 {
            self.mob_entity.reset_love_ticks();
        }
    }

    fn has_global_tick_reach(&self) -> bool {
        // Breeding retains another brain and can credit an arbitrarily distant breeder.
        true
    }

    fn mob_support_quad_leash(&self) -> bool {
        true
    }

    fn get_max_head_rotation(&self) -> f32 {
        MAX_HEAD_Y_ROT
    }

    fn on_jump_from_ground(&self) {
        let speed = self
            .mob_entity
            .steered()
            .move_control()
            .get_speed_modifier();
        let entity = self.get_entity();
        let mut velocity = entity.velocity.load();
        if speed > 0.0 && velocity.x * velocity.x + velocity.z * velocity.z < 0.01 {
            let yaw = f64::from(entity.yaw.load()).to_radians();
            velocity.x -= yaw.sin() * f64::from(0.1f32);
            velocity.z += yaw.cos() * f64::from(0.1f32);
            entity.velocity.store(velocity);
        }
    }

    fn on_die(&self, _source: Option<&dyn EntityBase>) {
        self.transition_to(SnifferState::Idling);
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        (!matches!(
            self.get_state(),
            SnifferState::Digging | SnifferState::Searching
        ))
        .then_some(Sound::EntitySnifferIdle)
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        120
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntitySnifferStep, 0.15, 1.0);
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if self.get_state() == SnifferState::Digging {
            let entity = self.get_entity();
            if entity.tick_count.load(Relaxed) % 10 == 0 {
                entity.world.load().emit_game_event_with_context(
                    "minecraft:entity_action",
                    self.get_head_block().to_centered_f64(),
                    &GameEventContext {
                        source_entity: Some(self),
                    },
                );
            }
            self.drop_seed();
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::sniffer::BABY_ID, self.is_baby());
        entity.set_synced_data(
            pumpkin_data::tracked_data::sniffer::STATE,
            VarInt(self.state.load(Relaxed)),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::sniffer::DROP_SEED_AT_TICK,
            VarInt(self.drop_seed_at_tick.load(Relaxed)),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        // The positive age survives reload, unlike the common transient cooldown.
        if self.get_age() > 0 {
            return self
                .mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed());
        }
        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn sniffer(world: &Arc<crate::world::World>) -> Arc<SnifferEntity> {
        SnifferEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 64.0, 8.5),
            &EntityType::SNIFFER,
        ))
    }

    #[tokio::test]
    async fn digging_drops_gift_loot_at_the_head_on_the_entity_clock() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let sniffer = sniffer(&world);
        sniffer.get_entity().tick_count.store(50, Relaxed);
        sniffer.transition_to(SnifferState::Digging);
        for tick in [169, 170, 171] {
            sniffer.get_entity().tick_count.store(tick, Relaxed);
            sniffer.mob_tick(sniffer.as_ref());
            let entities = world.entities.load_full();
            assert_eq!(entities.len(), usize::from(tick >= 170));
        }
        let entities = world.entities.load_full();
        let item = entities[0]
            .cast_any()
            .downcast_ref::<ItemEntity>()
            .ok_or("missing dig loot")?;
        let head = sniffer.get_head_block().0.to_f64();
        assert_eq!(item.get_entity().pos.load(), head);
        assert_eq!(item.get_pickup_delay(), ItemEntity::DEFAULT_PICKUP_DELAY);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn completed_digs_keep_explored_sites_and_cooldown_after_nbt_reload() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let original = sniffer(&world);
        let entity = original.get_entity();
        entity.on_ground.store(true, Relaxed);
        original.mob_entity.with_brain(original.as_ref(), |brain| {
            for x in 0..23 {
                entity
                    .supporting_block_pos
                    .store(Some(BlockPos::new(x, 63, 8)));
                original.on_digging_complete(brain, true);
            }
            brain.set_with_expiry(types::SNIFF_COOLDOWN, (), 9137);
        });
        original.transition_to(SnifferState::Digging);

        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
        original.write_custom_nbt(&mut nbt);
        let restored = sniffer(&world);
        restored.read_custom_nbt(&nbt);
        assert_eq!(restored.get_state(), SnifferState::Idling);
        restored.mob_entity.with_brain(restored.as_ref(), |brain| {
            let positions = brain
                .get(types::SNIFFER_EXPLORED_POSITIONS)
                .ok_or("missing explored positions")?;
            assert_eq!(positions.len(), 21);
            assert_eq!(
                positions[0],
                GlobalPos::new(&Dimension::OVERWORLD, BlockPos::new(22, 63, 8))
            );
            assert_eq!(positions[20].pos, BlockPos::new(2, 63, 8));
            assert_eq!(
                brain.time_until_expiry(types::SNIFF_COOLDOWN.id()),
                Some(9137)
            );
            Ok::<(), Box<dyn std::error::Error>>(())
        })?;
        world.level.shutdown().await;
        Ok(())
    }
}
