use std::sync::{
    Arc, Mutex, PoisonError,
    atomic::{AtomicI32, AtomicI64, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{
    Block,
    block_properties::{CopperGolemPose, CopperGolemStatueLikeProperties, EnumVariants},
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    environment_attribute::Activity,
    game_event::GameEvent,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::gamemode::GameMode;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use uuid::Uuid;

use crate::block::{
    blocks::chests::ChestContainer, entities::copper_golem_statue::CopperGolemStatueBlockEntity,
};
use crate::entity::ai::{
    brain::{
        Brain,
        memory::{PackedMemories, types},
    },
    pathfinder::node::PathType,
};
use crate::entity::item::ItemEntity;

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
    player::Player,
};

const IGNORE_WEATHERING_TICK: i64 = -2;
const UNSET_WEATHERING_TICK: i64 = -1;
const WEATHERING_TICK_FROM: i64 = 504_000;
const WEATHERING_TICK_TO: i64 = 552_000;
const TURN_TO_STATUE_CHANCE: f32 = 0.0058;
const SPAWN_COOLDOWN_MIN: i32 = 60;
const SPAWN_COOLDOWN_MAX: i32 = 100;
const TICK_TO_START_ON_REACHED_INTERACTION: i32 = 1;
const TICK_TO_PLAY_ON_REACHED_SOUND: i32 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum WeatherState {
    #[default]
    Unaffected = 0,
    Exposed = 1,
    Weathered = 2,
    Oxidized = 3,
}

impl WeatherState {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Exposed,
            2 => Self::Weathered,
            3 => Self::Oxidized,
            _ => Self::Unaffected,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Unaffected => Self::Exposed,
            Self::Exposed => Self::Weathered,
            Self::Weathered | Self::Oxidized => Self::Oxidized,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Oxidized => Self::Weathered,
            Self::Weathered => Self::Exposed,
            Self::Exposed | Self::Unaffected => Self::Unaffected,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Unaffected => "unaffected",
            Self::Exposed => "exposed",
            Self::Weathered => "weathered",
            Self::Oxidized => "oxidized",
        }
    }

    fn from_name(name: &str) -> Self {
        match name {
            "exposed" => Self::Exposed,
            "weathered" => Self::Weathered,
            "oxidized" => Self::Oxidized,
            _ => Self::Unaffected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum CopperGolemState {
    #[default]
    Idle = 0,
    GettingItem = 1,
    GettingNoItem = 2,
    DroppingItem = 3,
    DroppingNoItem = 4,
}

impl CopperGolemState {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::GettingItem,
            2 => Self::GettingNoItem,
            3 => Self::DroppingItem,
            4 => Self::DroppingNoItem,
            _ => Self::Idle,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

pub struct CopperGolemEntity {
    pub mob_entity: MobEntity,
    pub weather_state: AtomicI32,
    pub state: AtomicI32,
    pub next_weathering_tick: AtomicI64,
    opened_container: Mutex<Option<ChestContainer>>,
    last_lightning_bolt_uuid: Mutex<Option<Uuid>>,
}

impl CopperGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let golem = Self {
            mob_entity,
            weather_state: AtomicI32::new(WeatherState::Unaffected.id()),
            state: AtomicI32::new(CopperGolemState::Idle.id()),
            next_weathering_tick: AtomicI64::new(UNSET_WEATHERING_TICK),
            opened_container: Mutex::new(None),
            last_lightning_bolt_uuid: Mutex::new(None),
        };
        let mob_arc = Arc::new(golem);
        {
            let mut navigation = mob_arc
                .mob_entity
                .navigator
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            navigation.set_required_path_length(48.0);
            navigation.set_can_open_doors(true);
            navigation.set_pathfinding_malus(PathType::DangerFire, 16.0);
            navigation.set_pathfinding_malus(PathType::DangerOther, 16.0);
            navigation.set_pathfinding_malus(PathType::DamageFire, -1.0);
        };
        mob_arc
            .mob_entity
            .persistence_required
            .store(true, Ordering::Relaxed);
        let cooldown = mob_arc
            .get_random()
            .random_range(SPAWN_COOLDOWN_MIN..SPAWN_COOLDOWN_MAX);
        mob_arc.mob_entity.with_brain(mob_arc.as_ref(), |brain| {
            brain.set(types::TRANSPORT_ITEMS_COOLDOWN_TICKS, cooldown);
        });
        mob_arc
    }

    pub fn play_spawn_sound(&self) {
        self.play_golem_sound(Sound::EntityCopperGolemSpawn);
    }

    fn play_golem_sound(&self, sound: Sound) {
        if !self.get_entity().silent.load(Ordering::Relaxed) {
            self.get_entity().play_sound(sound);
        }
    }

    pub(crate) fn close_container(&self) {
        if let Some(container) = self
            .opened_container
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
        {
            container.stop_open();
        }
    }

    pub(crate) fn on_container_interaction(
        &self,
        container: &ChestContainer,
        state: CopperGolemState,
        ticks: i32,
    ) -> bool {
        if ticks == TICK_TO_START_ON_REACHED_INTERACTION {
            self.close_container();
            if !container.try_start_open() {
                return false;
            }
            *self
                .opened_container
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = Some(container.clone());
            self.set_state(state);
        }
        if ticks == TICK_TO_PLAY_ON_REACHED_SOUND {
            let sound = match state {
                CopperGolemState::GettingItem => Some(Sound::EntityCopperGolemNoItemGet),
                CopperGolemState::GettingNoItem => Some(Sound::EntityCopperGolemNoItemNoGet),
                CopperGolemState::DroppingItem => Some(Sound::EntityCopperGolemItemDrop),
                CopperGolemState::DroppingNoItem => Some(Sound::EntityCopperGolemItemNoDrop),
                CopperGolemState::Idle => None,
            };
            if let Some(sound) = sound {
                self.play_golem_sound(sound);
            }
        }
        if ticks == crate::entity::ai::brain::behavior::transport_items_between_containers::TARGET_INTERACTION_TIME {
            self.close_container();
        }
        true
    }

    fn update_weathering(&self) {
        let next = self.next_weathering_tick.load(Ordering::Relaxed);
        if next == IGNORE_WEATHERING_TICK {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let mut random = self.get_random();
        if next == UNSET_WEATHERING_TICK {
            self.next_weathering_tick.store(
                world.get_world_age()
                    + random.random_range(WEATHERING_TICK_FROM..=WEATHERING_TICK_TO),
                Ordering::Relaxed,
            );
            return;
        }
        let state = self.get_weather_state();
        if world.get_world_age() >= next && state != WeatherState::Oxidized {
            let new_state = state.next();
            self.set_weather_state(new_state);
            self.next_weathering_tick.store(
                if new_state == WeatherState::Oxidized {
                    0
                } else {
                    next + random.random_range(WEATHERING_TICK_FROM..=WEATHERING_TICK_TO)
                },
                Ordering::Relaxed,
            );
        }
        if state == WeatherState::Oxidized
            && world.get_block_state(&entity.block_pos.load()).is_air()
            && random.random::<f32>() <= TURN_TO_STATUE_CHANCE
        {
            self.turn_to_statue();
        }
    }

    fn turn_to_statue(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        let block = &Block::OXIDIZED_COPPER_GOLEM_STATUE;
        let mut event =
            crate::plugin::api::events::entity::entity_change_block::EntityChangeBlockEvent::new(
                entity.entity_id,
                pos,
                format!("minecraft:{}", block.name),
            );
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return;
        }
        let mut props = CopperGolemStatueLikeProperties::default(block);
        props.facing = entity.get_horizontal_facing();
        props.copper_golem_pose = CopperGolemPose::from_index(
            self.get_random()
                .random_range(0..CopperGolemPose::variant_count()),
        );
        world.set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        if let Some(block_entity) = world.get_block_entity(&pos)
            && let Some(statue) = block_entity
                .as_any()
                .downcast_ref::<CopperGolemStatueBlockEntity>()
        {
            statue.create_statue(self);
            self.close_container();
            self.drop_preserved_equipment();
            if entity.is_leashed() {
                entity.unleash();
                if !entity.is_leashed() && world.level_info.load().game_rules.entity_drops {
                    world.drop_stack(&pos, ItemStack::new(1, &Item::LEAD));
                }
            }
            entity.remove();
            self.play_golem_sound(Sound::EntityCopperGolemBecomeStatue);
        }
    }

    fn drop_preserved_equipment(&self) {
        let slots: Vec<_> = self
            .mob_entity
            .living_entity
            .equipment_drop_chances
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(|(_, chance)| **chance > 1.0)
            .map(|(slot, _)| slot.clone())
            .collect();
        for slot in slots {
            let stack = self
                .mob_entity
                .set_item_slot(&slot, ItemStack::EMPTY.clone());
            if !stack.is_empty() && self.can_drop_item(&stack) {
                self.get_entity().spawn_at_location(stack);
            }
        }
    }

    fn can_drop_item(&self, stack: &ItemStack) -> bool {
        let mut event =
            crate::plugin::api::events::entity::entity_drop_item::EntityDropItemEvent::new(
                self.get_entity().entity_id,
                stack.item.registry_key.to_owned(),
                stack.item_count,
            );
        let world = self.get_entity().world.load();
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        !event.cancelled
    }

    fn shear(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        let antenna = self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&EquipmentSlot::SADDLE);
        if antenna
            .item
            .has_tag(&tag::Item::MINECRAFT_SHEARABLE_FROM_COPPER_GOLEM)
        {
            let mut event =
                crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent {
                    player: player.clone(),
                    entity_id: entity.entity_id,
                    hand: 0,
                    cancelled: false,
                };
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled || !self.can_drop_item(&antenna) {
                return true;
            }
            let stack = self
                .mob_entity
                .set_item_slot(&EquipmentSlot::SADDLE, ItemStack::EMPTY.clone());
            let pos = entity.pos.load();
            if !world.spawn_entity(Arc::new(ItemEntity::new(
                Entity::new(
                    world.clone(),
                    pos + Vector3::new(0.0, 1.5, 0.0),
                    &EntityType::ITEM,
                ),
                stack.clone(),
            ))) {
                self.mob_entity.set_item_slot(&EquipmentSlot::SADDLE, stack);
                return true;
            }
            world.play_sound(Sound::EntityCopperGolemShear, SoundCategory::Players, &pos);
            world.emit_game_event(GameEvent::Shear.name(), pos);
            Self::damage_player_item(player, item_stack);
            return true;
        }
        false
    }

    fn damage_player_item(player: &Arc<Player>, item_stack: &mut ItemStack) {
        if player.gamemode.load() == GameMode::Creative {
            return;
        }
        let mut event =
            crate::plugin::api::events::player::player_item_damage::PlayerItemDamageEvent::new(
                player.clone(),
                item_stack.item.registry_key.to_owned(),
                1,
            );
        let world = player.world();
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if !event.cancelled {
            let _ = item_stack.damage_item(event.damage);
        }
    }

    #[must_use]
    pub fn get_weather_state(&self) -> WeatherState {
        WeatherState::from_id(self.weather_state.load(Ordering::Relaxed))
    }

    pub fn set_weather_state(&self, state: WeatherState) {
        self.weather_state.store(state.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::copper_golem::WEATHER_STATE,
            VarInt(state.id()),
        );
    }

    #[must_use]
    pub fn get_state(&self) -> CopperGolemState {
        CopperGolemState::from_id(self.state.load(Ordering::Relaxed))
    }

    pub fn set_state(&self, state: CopperGolemState) {
        self.state.store(state.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::copper_golem::COPPER_GOLEM_STATE,
            VarInt(state.id()),
        );
    }

    #[must_use]
    pub fn step_sound(&self) -> Sound {
        match self.get_weather_state() {
            WeatherState::Unaffected | WeatherState::Exposed => Sound::EntityCopperGolemStep,
            WeatherState::Weathered => Sound::EntityCopperGolemWeatheredStep,
            WeatherState::Oxidized => Sound::EntityCopperGolemOxidizedStep,
        }
    }
}

impl CustomSound for CopperGolemEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(match self.get_weather_state() {
            WeatherState::Unaffected | WeatherState::Exposed => Sound::EntityCopperGolemHurt,
            WeatherState::Weathered => Sound::EntityCopperGolemWeatheredHurt,
            WeatherState::Oxidized => Sound::EntityCopperGolemOxidizedHurt,
        })
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(match self.get_weather_state() {
            WeatherState::Unaffected | WeatherState::Exposed => Sound::EntityCopperGolemDeath,
            WeatherState::Weathered => Sound::EntityCopperGolemWeatheredDeath,
            WeatherState::Oxidized => Sound::EntityCopperGolemOxidizedDeath,
        })
    }
}

impl Mob for CopperGolemEntity {
    fn finalize_spawn(
        &self,
        _world: &Arc<crate::world::World>,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        self.play_spawn_sound();
        self.mob_entity.finalize_spawn_base();
        group_data
    }
    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        super::copper_golem_ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        if self.mob_entity.living_entity.health.load() <= 0.0 {
            return;
        }
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, |brain| {
            brain.set_active_activity_to_first_valid(&[Activity::Idle]);
        });
    }

    fn on_removed(&self) {
        self.close_container();
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if self.mob_entity.is_no_ai() || self.mob_entity.living_entity.health.load() <= 0.0 {
            let world = self.get_entity().world.load_full();
            self.mob_entity.with_brain(self, |brain| {
                brain.stop_all(&world, self, world.get_world_age());
            });
            self.close_container();
            self.set_state(CopperGolemState::Idle);
        }
    }

    fn post_tick(&self) {
        if self.mob_entity.living_entity.health.load() > 0.0 && self.get_entity().is_alive() {
            self.update_weathering();
        }
    }

    fn on_damage(&self, _damage_type: DamageType, _source: Option<&dyn EntityBase>) {
        self.set_state(CopperGolemState::Idle);
        if self.mob_entity.living_entity.health.load() <= 0.0 {
            self.close_container();
            self.drop_preserved_equipment();
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_long(
            "next_weather_age",
            self.next_weathering_tick.load(Ordering::Relaxed),
        );
        nbt.put_string("weather_state", self.get_weather_state().name().to_owned());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.next_weathering_tick.store(
            nbt.get_long("next_weather_age")
                .unwrap_or(UNSET_WEATHERING_TICK),
            Ordering::Relaxed,
        );
        if let Some(state) = nbt.get_string("weather_state") {
            self.set_weather_state(WeatherState::from_name(state));
        } else if let Some(state) = nbt.get_int("weather_state") {
            self.set_weather_state(WeatherState::from_id(state));
        } else {
            self.set_weather_state(WeatherState::Unaffected);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_on_lightning_strike(
        &self,
        caller: &dyn EntityBase,
        lightning: &crate::entity::lightning::LightningBoltEntity,
    ) {
        self.mob_entity
            .living_entity
            .on_lightning_strike(caller, lightning);
        let uuid = lightning.get_entity().entity_uuid;
        let mut previous = self
            .last_lightning_bolt_uuid
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if *previous != Some(uuid) {
            *previous = Some(uuid);
            let weather = self.get_weather_state();
            if weather != WeatherState::Unaffected {
                self.next_weathering_tick
                    .store(UNSET_WEATHERING_TICK, Ordering::Relaxed);
                self.set_weather_state(weather.previous());
            }
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::copper_golem::WEATHER_STATE,
            VarInt(self.get_weather_state().id()),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::copper_golem::COPPER_GOLEM_STATE,
            VarInt(self.get_state().id()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        if item_stack.is_empty() && !self.mob_entity.living_entity.held_item(self).is_empty() {
            if !self.can_drop_item(&self.mob_entity.living_entity.held_item(self)) {
                return true;
            }
            let held = self
                .mob_entity
                .set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::EMPTY.clone());
            if !held.is_empty() {
                let pos = entity.pos.load();
                let target = player.get_entity().pos.load();
                let direction = (target - pos).normalize();
                let item = ItemEntity::new_with_velocity(
                    Entity::new(
                        world.clone(),
                        Vector3::new(pos.x, entity.get_eye_y() - 0.3, pos.z),
                        &EntityType::ITEM,
                    ),
                    held.clone(),
                    direction.multiply(0.3, 0.3, 0.3),
                    ItemEntity::DEFAULT_PICKUP_DELAY,
                );
                if !world.spawn_entity(Arc::new(item)) {
                    self.mob_entity
                        .set_item_slot(&EquipmentSlot::MAIN_HAND, held);
                }
                return true;
            }
        }
        if item_stack.item.id == Item::SHEARS.id && self.shear(player, item_stack) {
            return true;
        }
        if item_stack.item.id == Item::HONEYCOMB.id
            && self.next_weathering_tick.load(Ordering::Relaxed) != IGNORE_WEATHERING_TICK
        {
            self.next_weathering_tick
                .store(IGNORE_WEATHERING_TICK, Ordering::Relaxed);
            let pos = entity.pos.load();
            world.sync_world_event(WorldEvent::ParticlesWaxOn, entity.block_pos.load(), 0);
            world.play_sound(Sound::ItemHoneycombWaxOn, SoundCategory::Blocks, &pos);
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            return true;
        }
        if item_stack.item.has_tag(&tag::Item::MINECRAFT_AXES) {
            let current_next = self.next_weathering_tick.load(Ordering::Relaxed);
            if current_next == IGNORE_WEATHERING_TICK {
                self.next_weathering_tick
                    .store(UNSET_WEATHERING_TICK, Ordering::Relaxed);
                let pos = entity.pos.load();
                world.play_sound(Sound::ItemAxeScrape, SoundCategory::Neutral, &pos);
                world.sync_world_event(WorldEvent::ParticlesWaxOff, entity.block_pos.load(), 0);
                Self::damage_player_item(player, item_stack);
                return true;
            }

            let weather_state = self.get_weather_state();
            if weather_state != WeatherState::Unaffected {
                self.set_weather_state(weather_state.previous());
                self.next_weathering_tick
                    .store(UNSET_WEATHERING_TICK, Ordering::Relaxed);
                let pos = entity.pos.load();
                world.play_sound(Sound::ItemAxeScrape, SoundCategory::Neutral, &pos);
                world.sync_world_event(WorldEvent::ParticlesScrape, entity.block_pos.load(), 0);
                Self::damage_player_item(player, item_stack);
                return true;
            }
        }

        self.mob_entity
            .mob_interact(player, item_stack, self.can_be_leashed())
    }
}

impl Drop for CopperGolemEntity {
    fn drop(&mut self) {
        self.close_container();
    }
}
