use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI32, AtomicI64, Ordering},
};

use pumpkin_data::block_properties::{
    ChestLikeProperties, ChestType, CopperGolemPose, CopperGolemStatueLikeProperties,
};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::game_rules::{GameRule, GameRuleValue};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockState};
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::{GameMode, Hand};
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use uuid::Uuid;

use crate::block::blocks::chests;
use crate::block::blocks::weathering_copper::WeatherState;
use crate::block::entities::copper_golem_statue::CopperGolemStatueBlockEntity;
use crate::entity::{
    Entity, EntityBase,
    ai::{
        brain::{
            Brain,
            behavior::utils,
            memory::{PackedMemories, types},
        },
        pathfinder::node::PathType,
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
    player::Player,
    shearable::{Shearable, shear_by_player_in_hand},
};

pub(crate) mod ai;

pub const EQUIPMENT_SLOT_ANTENNA: EquipmentSlot = EquipmentSlot::SADDLE;
pub const IGNORE_WEATHERING_TICK: i64 = -2;
pub const UNSET_WEATHERING_TICK: i64 = -1;
pub const WEATHERING_TICK_FROM: i64 = 504_000;
pub const WEATHERING_TICK_TO: i64 = 552_000;
pub const TURN_TO_STATUE_CHANCE: f32 = 0.0058;
pub const SPAWN_COOLDOWN_MIN: i32 = 60;
pub const SPAWN_COOLDOWN_MAX: i32 = 100;
pub const CONTAINER_INTERACTION_RANGE: f64 = 3.0;
const REQUIRED_PATH_LENGTH: f32 = 48.0;
const ABSTRACT_GOLEM_AMBIENT_INTERVAL: i32 = 120;

const STATUE_POSES: [CopperGolemPose; 4] = [
    CopperGolemPose::Standing,
    CopperGolemPose::Sitting,
    CopperGolemPose::Running,
    CopperGolemPose::Star,
];

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
    /// The chest this golem holds open, closed again when the golem stops using it or is removed.
    opened_chest: Mutex<Option<(BlockPos, Arc<dyn Inventory>)>>,
    last_lightning_bolt_uuid: Mutex<Option<Uuid>>,
}

impl CopperGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        mob_entity.set_persistence_required();
        let mut navigator = mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_required_path_length(REQUIRED_PATH_LENGTH);
        navigator.set_can_open_doors(true);
        navigator.set_pathfinding_malus(PathType::DangerFire, 16.0);
        navigator.set_pathfinding_malus(PathType::DangerOther, 16.0);
        navigator.set_pathfinding_malus(PathType::DamageFire, -1.0);
        drop(navigator);

        let golem = Self {
            mob_entity,
            weather_state: AtomicI32::new(WeatherState::Unaffected as i32),
            state: AtomicI32::new(CopperGolemState::Idle.id()),
            next_weathering_tick: AtomicI64::new(UNSET_WEATHERING_TICK),
            opened_chest: Mutex::new(None),
            last_lightning_bolt_uuid: Mutex::new(None),
        };
        let mob_arc = Arc::new(golem);
        let initial_cooldown = mob_arc
            .get_random()
            .random_range(SPAWN_COOLDOWN_MIN..SPAWN_COOLDOWN_MAX);
        mob_arc.mob_entity.with_brain(mob_arc.as_ref(), |brain| {
            brain.set(types::TRANSPORT_ITEMS_COOLDOWN_TICKS, initial_cooldown);
        });

        mob_arc
    }

    pub fn spawn(&self, weather_state: WeatherState) {
        self.set_weather_state(weather_state);
        self.play_spawn_sound();
    }

    pub fn play_spawn_sound(&self) {
        self.play_entity_sound(Sound::EntityCopperGolemSpawn, 1.0, 1.0);
    }

    #[must_use]
    pub fn get_weather_state(&self) -> WeatherState {
        usize::try_from(self.weather_state.load(Ordering::Relaxed))
            .ok()
            .and_then(WeatherState::from_ordinal)
            .unwrap_or(WeatherState::Unaffected)
    }

    pub fn set_weather_state(&self, state: WeatherState) {
        self.weather_state.store(state as i32, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::copper_golem::WEATHER_STATE,
            VarInt(state as i32),
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

    pub fn open_chest(&self, pos: BlockPos, container: Arc<dyn Inventory>) {
        container.on_open();
        let previous = self
            .opened_chest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace((pos, container));
        if let Some((_, previous)) = previous {
            previous.on_close();
        }
    }

    pub fn close_opened_chest(&self) {
        let opened = self
            .opened_chest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some((_, container)) = opened {
            container.on_close();
        }
    }

    #[must_use]
    pub fn has_container_open(&self, pos: &BlockPos) -> bool {
        let Some(opened) = self
            .opened_chest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|(pos, _)| *pos)
        else {
            return false;
        };
        if opened == *pos {
            return true;
        }
        let world = self.get_entity().world.load();
        let (block, state_id) = world.get_block_and_state_id(&opened);
        if block != &Block::CHEST
            && block != &Block::TRAPPED_CHEST
            && !block.has_tag(&tag::Block::MINECRAFT_COPPER_CHESTS)
        {
            return false;
        }
        let props = ChestLikeProperties::from_state_id(state_id);
        props.r#type != ChestType::Single
            && chests::connected_half(&world, block, &opened, props) == Some(*pos)
    }

    #[must_use]
    pub fn step_sound(&self) -> Sound {
        match self.get_weather_state() {
            WeatherState::Unaffected | WeatherState::Exposed => Sound::EntityCopperGolemStep,
            WeatherState::Weathered => Sound::EntityCopperGolemWeatheredStep,
            WeatherState::Oxidized => Sound::EntityCopperGolemOxidizedStep,
        }
    }

    fn update_weathering(&self) {
        let next_tick = self.next_weathering_tick.load(Ordering::Relaxed);
        if next_tick == IGNORE_WEATHERING_TICK {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let game_time = world.get_world_age();

        if next_tick == UNSET_WEATHERING_TICK {
            let delay = self
                .get_random()
                .random_range(WEATHERING_TICK_FROM..=WEATHERING_TICK_TO);
            self.next_weathering_tick
                .store(game_time + delay, Ordering::Relaxed);
            return;
        }

        let weather_state = self.get_weather_state();
        let is_fully_oxidized = weather_state == WeatherState::Oxidized;
        if game_time >= next_tick && !is_fully_oxidized {
            let new_state = weather_state.next().unwrap_or(weather_state);
            self.set_weather_state(new_state);
            let new_next = if new_state == WeatherState::Oxidized {
                0
            } else {
                next_tick
                    + self
                        .get_random()
                        .random_range(WEATHERING_TICK_FROM..=WEATHERING_TICK_TO)
            };
            self.next_weathering_tick.store(new_next, Ordering::Relaxed);
        }

        if is_fully_oxidized && self.can_turn_to_statue(&world) {
            self.turn_to_statue(&world);
        }
    }

    fn can_turn_to_statue(&self, world: &crate::world::World) -> bool {
        let pos = self.get_entity().block_pos.load();
        world.get_block_state(&pos).is_air()
            && self.get_random().random::<f32>() <= TURN_TO_STATUE_CHANCE
    }

    fn turn_to_statue(&self, world: &Arc<crate::world::World>) {
        let entity = self.get_entity();
        let pos = entity.block_pos.load();
        let pose_idx = self.get_random().random_range(0..STATUE_POSES.len());
        let mut props =
            CopperGolemStatueLikeProperties::default(&Block::OXIDIZED_COPPER_GOLEM_STATUE);
        props.copper_golem_pose = STATUE_POSES[pose_idx];
        props.facing = entity.get_horizontal_facing();

        world.set_block_state(
            &pos,
            props.to_state_id(&Block::OXIDIZED_COPPER_GOLEM_STATUE),
            BlockFlags::NOTIFY_ALL,
        );

        let statue_be = world
            .get_block_entity(&pos)
            .filter(|be| be.as_any().is::<CopperGolemStatueBlockEntity>())
            .unwrap_or_else(|| {
                let be = Arc::new(CopperGolemStatueBlockEntity::new(pos));
                world.add_block_entity(be.clone());
                be
            });

        if let Some(statue) = statue_be
            .as_any()
            .downcast_ref::<CopperGolemStatueBlockEntity>()
        {
            statue.set_custom_name(entity.custom_name.load().as_ref().clone());
            self.mob_entity.drop_preserved_equipment();
            if entity.is_leashed() {
                if matches!(
                    world.get_game_rule(&GameRule::EntityDrops),
                    GameRuleValue::Bool(true)
                ) {
                    entity.drop_leash();
                } else {
                    entity.remove_leash();
                }
            }
            self.play_entity_sound(Sound::EntityCopperGolemBecomeStatue, 1.0, 1.0);
            entity.remove();
        }
    }
}

impl Shearable for CopperGolemEntity {
    fn shear(&self, category: SoundCategory, _tool: &ItemStack) -> bool {
        let antenna_item = self
            .mob_entity
            .set_item_slot(&EQUIPMENT_SLOT_ANTENNA, ItemStack::EMPTY.clone());
        if antenna_item.is_empty() {
            return false;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntityCopperGolemShear, category, &pos);
        entity.spawn_at_location_with_offset(antenna_item, Vector3::new(0.0, 1.5, 0.0));
        true
    }

    fn ready_for_shearing(&self) -> bool {
        self.mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EQUIPMENT_SLOT_ANTENNA)
            .item
            .has_tag(&tag::Item::MINECRAFT_SHEARABLE_FROM_COPPER_GOLEM)
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
    fn on_removed(&self) {
        self.close_opened_chest();
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, |brain| {
            ai::update_activity(brain);
        });
    }

    fn as_shearable(&self) -> Option<&dyn Shearable> {
        Some(self)
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        false
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        ABSTRACT_GOLEM_AMBIENT_INTERVAL
    }

    fn finalize_spawn(
        &self,
        world: &Arc<crate::world::World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        self.play_spawn_sound();
        self.default_finalize_spawn(world, group_data)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(self.step_sound(), 1.0, 1.0);
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_long(
            "next_weather_age",
            self.next_weathering_tick.load(Ordering::Relaxed),
        );
        nbt.put_string(
            "weather_state",
            self.get_weather_state().as_str().to_string(),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.next_weathering_tick.store(
            nbt.get_long("next_weather_age")
                .unwrap_or(UNSET_WEATHERING_TICK),
            Ordering::Relaxed,
        );
        let state = nbt
            .get_string("weather_state")
            .and_then(WeatherState::from_name)
            .or_else(|| {
                nbt.get_int("weather_state")
                    .and_then(|id| usize::try_from(id).ok())
                    .and_then(WeatherState::from_ordinal)
            })
            .unwrap_or(WeatherState::Unaffected);
        self.set_weather_state(state);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.update_weathering();
    }

    fn on_damage(
        &self,
        _damage_type: pumpkin_data::damage::DamageType,
        _source: Option<&dyn EntityBase>,
    ) {
        self.set_state(CopperGolemState::Idle);
    }

    fn mob_on_lightning_strike(
        &self,
        caller: &dyn EntityBase,
        lightning: &crate::entity::lightning::LightningBoltEntity,
    ) {
        self.mob_entity
            .living_entity
            .on_lightning_strike(caller, lightning);
        let bolt_uuid = lightning.get_entity().entity_uuid;
        let mut last = self
            .last_lightning_bolt_uuid
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *last != Some(bolt_uuid) {
            *last = Some(bolt_uuid);
            drop(last);
            let weather_state = self.get_weather_state();
            if weather_state != WeatherState::Unaffected {
                self.next_weathering_tick
                    .store(UNSET_WEATHERING_TICK, Ordering::Relaxed);
                self.set_weather_state(weather_state.previous().unwrap_or(weather_state));
            }
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::copper_golem::WEATHER_STATE,
            VarInt(self.get_weather_state() as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::copper_golem::COPPER_GOLEM_STATE,
            VarInt(self.get_state().id()),
        );
    }

    fn mob_interact_with_hand(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        hand: Hand,
    ) -> Option<bool> {
        Some(self.interact_with_hand(player, item_stack, hand))
    }
}

impl CopperGolemEntity {
    fn interact_with_hand(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        hand: Hand,
    ) -> bool {
        if item_stack.is_empty() {
            let equipped = self
                .mob_entity
                .set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::EMPTY.clone());
            if !equipped.is_empty() {
                utils::throw_item(self, equipped, player.position());
                return true;
            }
        }

        if item_stack.item.id == Item::SHEARS.id && self.ready_for_shearing() {
            return shear_by_player_in_hand(self, player, item_stack, hand);
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        let block_pos = entity.block_pos.load();

        if item_stack.item.id == Item::HONEYCOMB.id
            && self.next_weathering_tick.load(Ordering::Relaxed) != IGNORE_WEATHERING_TICK
        {
            world.sync_world_event(WorldEvent::ParticlesWaxOn, block_pos, 0);
            world.play_sound(
                Sound::ItemHoneycombWaxOn,
                SoundCategory::Blocks,
                &block_pos.to_f64(),
            );
            self.next_weathering_tick
                .store(IGNORE_WEATHERING_TICK, Ordering::Relaxed);
            if player.gamemode.load() != GameMode::Creative {
                item_stack.decrement(1);
            }
            return true;
        }

        if item_stack.item.has_tag(&tag::Item::MINECRAFT_AXES) {
            if self.next_weathering_tick.load(Ordering::Relaxed) == IGNORE_WEATHERING_TICK {
                self.play_entity_sound(Sound::ItemAxeScrape, 1.0, 1.0);
                world.sync_world_event(WorldEvent::ParticlesWaxOff, block_pos, 0);
                self.next_weathering_tick
                    .store(UNSET_WEATHERING_TICK, Ordering::Relaxed);
                let _ = player.damage_held_stack(item_stack, 1, hand);
                return true;
            }

            let weather_state = self.get_weather_state();
            if weather_state != WeatherState::Unaffected {
                self.play_entity_sound(Sound::ItemAxeScrape, 1.0, 1.0);
                world.sync_world_event(WorldEvent::ParticlesScrape, block_pos, 0);
                self.next_weathering_tick
                    .store(UNSET_WEATHERING_TICK, Ordering::Relaxed);
                self.set_weather_state(weather_state.previous().unwrap_or(weather_state));
                let _ = player.damage_held_stack(item_stack, 1, hand);
                return true;
            }
        }

        false
    }
}
