use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
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
            memory::{GlobalPos, PackedMemories, types},
        },
        pathfinder::node::PathType,
        util::land_random_pos,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity, drop_from_gift_loot_table},
    passive::animal::Animal,
    player::Player,
    r#type::from_type,
};

pub(crate) mod ai;

const SNIFFER_DIGGING_LOOT_TABLE: &str = "minecraft:gameplay/sniffer_digging";
pub const SNIFFER_BABY_START_AGE: i32 = -48000;
pub const DIGGING_DROP_SEED_OFFSET_TICKS: i32 = 120;

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
}

/// Represents a Sniffer, a large passive mob that digs up ancient seeds.
///
/// Wiki: <https://minecraft.wiki/w/Sniffer>
pub struct SnifferEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub state: AtomicI32,
    pub drop_seed_at_tick: AtomicI32,
    pub explored_positions: std::sync::Mutex<Vec<BlockPos>>,
}

impl SnifferEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let sniffer = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            state: AtomicI32::new(SnifferState::Idling.id()),
            drop_seed_at_tick: AtomicI32::new(0),
            explored_positions: std::sync::Mutex::new(Vec::new()),
        };
        let mob_arc = Arc::new(sniffer);

        {
            let mut navigator = mob_arc
                .mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_can_float(true);
            navigator.set_pathfinding_malus(PathType::Water, -1.0);
            navigator.set_pathfinding_malus(PathType::PowderSnow, -1.0);
            navigator.set_pathfinding_malus(PathType::DamageCautious, -1.0);
        };

        mob_arc
    }

    #[must_use]
    pub fn get_state(&self) -> SnifferState {
        SnifferState::from_id(self.state.load(Ordering::Relaxed))
    }

    pub fn set_state(&self, state: SnifferState) {
        self.state.store(state.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::sniffer::STATE,
            VarInt(state.id()),
        );
        self.update_dimensions();
    }

    /// Vanilla `Sniffer.getDefaultDimensions`: 0.4 lower while digging, by age otherwise.
    fn update_dimensions(&self) {
        let entity = self.get_entity();
        let baby = entity.age.load(Ordering::Relaxed) < 0;
        let dimensions = if self.get_state() == SnifferState::Digging {
            let size = Entity::type_dimensions(&EntityType::SNIFFER);
            let digging = EntityDimensions::new(size.width, size.height - 0.4, 0.81);
            if baby { digging.scale(0.5) } else { digging }
        } else {
            Entity::age_dimensions(&EntityType::SNIFFER, baby)
        };
        entity.set_default_dimensions(dimensions);
    }

    pub fn transition_to(&self, state: SnifferState) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();

        match state {
            SnifferState::Idling => {
                self.set_state(SnifferState::Idling);
            }
            SnifferState::FeelingHappy => {
                world.play_sound(Sound::EntitySnifferHappy, SoundCategory::Neutral, &pos);
                self.set_state(SnifferState::FeelingHappy);
            }
            SnifferState::Scenting => {
                self.set_state(SnifferState::Scenting);
                self.on_scenting_start();
            }
            SnifferState::Sniffing => {
                world.play_sound(Sound::EntitySnifferSniffing, SoundCategory::Neutral, &pos);
                self.set_state(SnifferState::Sniffing);
            }
            SnifferState::Searching => {
                self.set_state(SnifferState::Searching);
            }
            SnifferState::Digging => {
                self.set_state(SnifferState::Digging);
                self.on_digging_start();
            }
            SnifferState::Rising => {
                world.play_sound(
                    Sound::EntitySnifferDiggingStop,
                    SoundCategory::Neutral,
                    &pos,
                );
                self.set_state(SnifferState::Rising);
            }
        }
    }

    fn on_scenting_start(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let pitch = if self.is_baby() { 1.3 } else { 1.0 };
        world.play_sound_fine(
            Sound::EntitySnifferScenting,
            SoundCategory::Neutral,
            &pos,
            1.0,
            pitch,
        );
    }

    fn on_digging_start(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let current_ticks = world.level_time.try_lock().map_or(0, |t| t.world_age);
        let drop_tick = current_ticks as i32 + DIGGING_DROP_SEED_OFFSET_TICKS;
        self.drop_seed_at_tick.store(drop_tick, Ordering::Relaxed);
        entity.set_synced_data(
            pumpkin_data::tracked_data::sniffer::DROP_SEED_AT_TICK,
            VarInt(drop_tick),
        );
        world.send_entity_status(
            entity,
            EntityStatus::SnifferDiggingSound,
            ActorEventID::None,
        );
    }

    pub fn on_digging_complete(&self, success: bool) {
        if success {
            self.store_explored_position(self.get_entity().get_on_pos());
        }
    }

    #[must_use]
    pub fn get_head_position(&self) -> Vector3<f64> {
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let yaw = f64::from(entity.yaw.load());
        let yaw_rad = yaw.to_radians();
        let forward = Vector3::new(-yaw_rad.sin(), 0.0, yaw_rad.cos());
        pos + forward * 2.25
    }

    #[must_use]
    pub fn get_head_block(&self) -> BlockPos {
        let head_pos = self.get_head_position();
        let entity = self.get_entity();
        let pos = entity.pos.load();
        BlockPos::floored(head_pos.x, pos.y + 0.2, head_pos.z)
    }

    #[must_use]
    pub fn is_tempted(&self) -> bool {
        self.mob_entity
            .brain_has_memory_value(types::TEMPTING_PLAYER.id())
    }

    #[must_use]
    pub fn can_sniff(&self) -> bool {
        let entity = self.get_entity();
        !self.is_tempted()
            && !self.is_panicking()
            && !self.mob_entity.living_entity.is_in_water()
            && !self.mob_entity.is_in_love()
            && entity.on_ground.load(Ordering::Relaxed)
            && !entity.is_passenger()
            && !entity.is_leashed()
    }

    #[must_use]
    pub fn can_sniff_with_brain(&self, state: &BrainState) -> bool {
        let entity = self.get_entity();
        !state.has_memory_value(types::TEMPTING_PLAYER.id())
            && !state.has_memory_value(types::IS_PANICKING.id())
            && !self.mob_entity.living_entity.is_in_water()
            && !self.mob_entity.is_in_love()
            && entity.on_ground.load(Ordering::Relaxed)
            && !entity.is_passenger()
            && !entity.is_leashed()
    }

    #[must_use]
    pub fn can_dig(&self) -> bool {
        let entity = self.get_entity();
        !self.is_panicking()
            && !self.is_tempted()
            && !self.mob_entity.living_entity.is_baby()
            && !self.mob_entity.living_entity.is_in_water()
            && entity.on_ground.load(Ordering::Relaxed)
            && !entity.is_passenger()
            && self.can_dig_at(self.get_head_block().down())
    }

    #[must_use]
    pub fn can_dig_with_brain(&self, state: &BrainState) -> bool {
        let entity = self.get_entity();
        !state.has_memory_value(types::IS_PANICKING.id())
            && !state.has_memory_value(types::TEMPTING_PLAYER.id())
            && !self.mob_entity.living_entity.is_baby()
            && !self.mob_entity.living_entity.is_in_water()
            && entity.on_ground.load(Ordering::Relaxed)
            && !entity.is_passenger()
            && self.can_dig_at_with_brain(state, self.get_head_block().down())
    }

    #[must_use]
    pub fn can_dig_at(&self, pos: BlockPos) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        let block_state = world.get_block_state(&pos);
        let block = pumpkin_data::Block::from_state_id(block_state.id);

        if !block.has_tag(&tag::Block::MINECRAFT_SNIFFER_DIGGABLE_BLOCK) {
            return false;
        }

        let explored = self
            .explored_positions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if explored.contains(&pos) {
            return false;
        }

        let mob_entity = self.get_mob_entity();
        let steered = mob_entity.steered();
        steered
            .navigator()
            .create_path(&mob_entity.living_entity, pos.0.to_f64(), 1)
            .is_some_and(|p| p.can_reach())
    }

    #[must_use]
    pub fn can_dig_at_with_brain(&self, state: &BrainState, pos: BlockPos) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        let block_state = world.get_block_state(&pos);
        let block = pumpkin_data::Block::from_state_id(block_state.id);

        if !block.has_tag(&tag::Block::MINECRAFT_SNIFFER_DIGGABLE_BLOCK) {
            return false;
        }

        if let Some(explored) = state.get(types::SNIFFER_EXPLORED_POSITIONS) {
            let dimension = Dimension::from_name(world.dimension.minecraft_name)
                .unwrap_or(&Dimension::OVERWORLD);
            let global_pos = GlobalPos::new(dimension, pos);
            if explored.contains(&global_pos) {
                return false;
            }
        }

        let mob_entity = self.get_mob_entity();
        let steered = mob_entity.steered();
        steered
            .navigator()
            .create_path(&mob_entity.living_entity, pos.0.to_f64(), 1)
            .is_some_and(|p| p.can_reach())
    }

    #[must_use]
    pub fn calculate_dig_position(&self, state: &BrainState) -> Option<BlockPos> {
        let world = self.get_entity().world.load();
        let border = world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for idx in 0..5 {
            if let Some(target_vec) = land_random_pos::get_pos(self, 10 + 2 * idx, 3) {
                let containing = BlockPos::floored_v(target_vec);
                if border.contains_block(containing.0.x, containing.0.z) {
                    let below = containing.down();
                    if self.can_dig_at_with_brain(state, below) {
                        return Some(below);
                    }
                }
            }
        }
        None
    }

    pub fn store_explored_position(&self, pos: BlockPos) {
        let mut explored = self
            .explored_positions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if explored.len() >= 20 {
            explored.pop();
        }
        explored.insert(0, pos);
    }

    pub fn drop_seed(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let current_tick = world.get_world_age() as i32;

        if self.drop_seed_at_tick.load(Ordering::Relaxed) == current_tick {
            let head_pos = self.get_head_position();
            drop_from_gift_loot_table(self, SNIFFER_DIGGING_LOOT_TABLE, |item_stack| {
                let item_entity = Entity::new(world.clone(), head_pos, &EntityType::ITEM);
                let item_arc = Arc::new(ItemEntity::new(item_entity, item_stack));
                world.spawn_entity_non_save(item_arc as Arc<dyn EntityBase>);
            });

            world.play_sound(
                Sound::EntitySnifferDropSeed,
                SoundCategory::Neutral,
                &head_pos,
            );
        }
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
        self.update_dimensions();
    }
}

impl Animal for SnifferEntity {
    fn play_eating_sound(&self) {
        let pitch = 0.8 + self.get_random().random::<f32>() * 0.4;
        self.play_entity_sound(Sound::EntitySnifferEat, 1.0, pitch);
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_SNIFFER_FOOD)
    }

    fn can_mate(&self, partner: &dyn EntityBase) -> bool {
        let is_valid_state = |state: SnifferState| {
            matches!(
                state,
                SnifferState::Idling | SnifferState::Scenting | SnifferState::FeelingHappy
            )
        };
        partner.get_entity().entity_id != self.get_entity().entity_id
            && self.mob_entity.is_in_love()
            && is_valid_state(self.get_state())
            && partner
                .cast_any()
                .downcast_ref::<Self>()
                .is_some_and(|partner_sniffer| {
                    partner_sniffer.mob_entity.is_in_love()
                        && is_valid_state(partner_sniffer.get_state())
                })
    }

    fn spawns_offspring(&self) -> bool {
        false
    }

    fn on_bred_without_offspring(&self, _brain: Option<&mut crate::entity::ai::brain::BrainState>) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();

        let item_stack = ItemStack::new(1, &Item::SNIFFER_EGG);
        let egg_entity = Entity::new(world.clone(), pos, &EntityType::ITEM);
        let egg_arc = Arc::new(ItemEntity::new(egg_entity, item_stack));
        world.spawn_entity_non_save(egg_arc as Arc<dyn EntityBase>);

        let mut rng = rand::rng();
        let pitch = 0.2f32.mul_add(rng.random::<f32>() - rng.random::<f32>(), 0.5);
        world.play_sound_fine(
            Sound::BlockSnifferEggPlop,
            SoundCategory::Neutral,
            &pos,
            1.0,
            pitch,
        );
    }

    fn get_breed_offspring(&self, _partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        from_type(
            &EntityType::SNIFFER,
            entity.pos.load(),
            &entity.world.load(),
            uuid::Uuid::new_v4(),
        )
    }
}

impl Mob for SnifferEntity {
    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, |brain| {
            ai::update_activity(brain);
        });
    }

    fn mob_support_quad_leash(&self) -> bool {
        true
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntitySnifferStep, 0.15, 1.0);
    }

    fn get_max_head_rotation(&self) -> f32 {
        50.0
    }

    fn on_jump_from_ground(&self) {
        let entity = self.get_entity();
        let steered = self.mob_entity.steered();
        let speed = steered.move_control().get_speed_modifier();
        if speed > 0.0 && entity.velocity.load().horizontal_length_squared() < 0.01 {
            entity.update_velocity_from_input(Vector3::new(0.0, 0.0, 1.0), 0.1);
        }
    }

    fn on_die(&self, _killer: Option<&dyn EntityBase>) {
        self.transition_to(SnifferState::Idling);
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        if matches!(
            self.get_state(),
            SnifferState::Digging | SnifferState::Searching
        ) {
            None
        } else {
            Some(Sound::EntitySnifferIdle)
        }
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("State", self.get_state().id());
        nbt.put_int(
            "DropSeedAtTick",
            self.drop_seed_at_tick.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(state_id) = nbt.get_int("State") {
            self.set_state(SnifferState::from_id(state_id));
        }
        if let Some(drop_tick) = nbt.get_int("DropSeedAtTick") {
            self.drop_seed_at_tick.store(drop_tick, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let in_water_or_fire = self.mob_entity.living_entity.is_in_water()
            || self.mob_entity.living_entity.entity.is_on_fire();
        let target_water_malus = if in_water_or_fire { 0.0 } else { -1.0 };
        if let Ok(mut navigator) = self.mob_entity.navigator.try_lock() {
            navigator.set_pathfinding_malus(PathType::Water, target_water_malus);
        }

        let state = self.get_state();
        match state {
            SnifferState::Searching => {
                let entity = self.get_entity();
                let world = entity.world.load();
                let ticks = world.get_world_age();
                if ticks % 20 == 0 {
                    world.play_sound(
                        Sound::EntitySnifferSearching,
                        SoundCategory::Neutral,
                        &entity.pos.load(),
                    );
                }
            }
            SnifferState::Digging => {
                self.drop_seed();
            }
            _ => {}
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::sniffer::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::sniffer::STATE,
            VarInt(self.get_state().id()),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::sniffer::DROP_SEED_AT_TICK,
            VarInt(self.drop_seed_at_tick.load(Ordering::Relaxed)),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;

    #[tokio::test]
    async fn sniffer_can_mate_requires_valid_states() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity1 = Entity::new(world.clone(), Vector3::default(), &EntityType::SNIFFER);
        let sniffer1 = SnifferEntity::new(entity1);

        let entity2 = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer2 = SnifferEntity::new(entity2);

        // Neither in love -> cannot mate
        assert!(!sniffer1.can_mate(sniffer2.as_ref()));

        // Only one in love -> cannot mate
        sniffer1.mob_entity.set_love_ticks(600, None);
        assert!(!sniffer1.can_mate(sniffer2.as_ref()));

        // Both in love and Idling -> can mate
        sniffer2.mob_entity.set_love_ticks(600, None);
        assert!(sniffer1.can_mate(sniffer2.as_ref()));

        // Cannot mate with self
        assert!(!sniffer1.can_mate(sniffer1.as_ref()));

        // When partner is in Scenting or FeelingHappy -> can mate
        sniffer2.set_state(SnifferState::Scenting);
        assert!(sniffer1.can_mate(sniffer2.as_ref()));
        sniffer2.set_state(SnifferState::FeelingHappy);
        assert!(sniffer1.can_mate(sniffer2.as_ref()));

        // When partner is Digging or Searching -> cannot mate
        sniffer2.set_state(SnifferState::Digging);
        assert!(!sniffer1.can_mate(sniffer2.as_ref()));
        sniffer2.set_state(SnifferState::Searching);
        assert!(!sniffer1.can_mate(sniffer2.as_ref()));

        // When self is Digging -> cannot mate
        sniffer1.set_state(SnifferState::Digging);
        sniffer2.set_state(SnifferState::Idling);
        assert!(!sniffer1.can_mate(sniffer2.as_ref()));
    }

    #[tokio::test]
    async fn sniffer_on_die_resets_state_to_idling() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        sniffer.set_state(SnifferState::Digging);
        assert_eq!(sniffer.get_state(), SnifferState::Digging);

        sniffer.on_die(None);
        assert_eq!(sniffer.get_state(), SnifferState::Idling);
    }

    #[tokio::test]
    async fn sniffer_max_head_rotation_is_50() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        assert!((sniffer.get_max_head_rotation() - 50.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn sniffer_ambient_sound_suppressed_while_digging_or_searching() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        sniffer.set_state(SnifferState::Idling);
        assert_eq!(sniffer.get_ambient_sound(), Some(Sound::EntitySnifferIdle));

        sniffer.set_state(SnifferState::Digging);
        assert_eq!(sniffer.get_ambient_sound(), None);

        sniffer.set_state(SnifferState::Searching);
        assert_eq!(sniffer.get_ambient_sound(), None);

        sniffer.set_state(SnifferState::FeelingHappy);
        assert_eq!(sniffer.get_ambient_sound(), Some(Sound::EntitySnifferIdle));
    }

    #[tokio::test]
    async fn sniffer_on_bred_without_offspring_drops_egg() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world.clone(), Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        assert_eq!(world.entities.load().len(), 0);
        assert!(!sniffer.spawns_offspring());

        sniffer.on_bred_without_offspring(None);
        let entities = world.entities.load();
        assert_eq!(entities.len(), 1);
        let item_entity = entities[0].get_item_entity().expect("expected item entity");
        assert_eq!(
            item_entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .item,
            &pumpkin_data::item::Item::SNIFFER_EGG
        );
    }

    #[tokio::test]
    async fn sniffer_navigator_maluses() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        let navigator = sniffer
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(navigator.get_pathfinding_malus(PathType::Water), -1.0);
        assert_eq!(navigator.get_pathfinding_malus(PathType::PowderSnow), -1.0);
        assert_eq!(
            navigator.get_pathfinding_malus(PathType::DamageCautious),
            -1.0
        );
    }

    #[tokio::test]
    async fn sniffer_reset_sniffing_resets_memories_and_state() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        sniffer.set_state(SnifferState::Digging);
        sniffer.mob_entity.with_brain(sniffer.as_ref(), |brain| {
            brain.set(types::SNIFFER_DIGGING, true);
            brain.set(types::SNIFFER_SNIFFING_TARGET, BlockPos::new(10, 64, 10));
            ai::reset_sniffing(brain, sniffer.as_ref());

            assert_eq!(sniffer.get_state(), SnifferState::Idling);
            assert!(!brain.has_memory_value(types::SNIFFER_DIGGING.id()));
            assert!(!brain.has_memory_value(types::SNIFFER_SNIFFING_TARGET.id()));
        });
    }

    #[tokio::test]
    async fn sniffer_brain_activity_transitions() {
        use crate::entity::ai::brain::memory::walk_target::WalkTarget;
        use pumpkin_data::environment_attribute::Activity;

        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        sniffer.mob_entity.with_brain(sniffer.as_ref(), |brain| {
            // Initial state defaults to Idle
            ai::update_activity(brain);
            assert_eq!(brain.get_active_non_core_activity(), Some(Activity::Idle));

            // With sniffing target and walk target -> Sniff activity
            let target_pos = BlockPos::new(5, 64, 5);
            brain.set(types::SNIFFER_SNIFFING_TARGET, target_pos);
            brain.set(
                types::WALK_TARGET,
                WalkTarget::from_block_pos(target_pos, 1.25, 0),
            );
            ai::update_activity(brain);
            assert_eq!(brain.get_active_non_core_activity(), Some(Activity::Sniff));

            // With digging memory and no walk target -> Dig activity
            brain.erase(types::WALK_TARGET.id());
            brain.erase(types::SNIFFER_SNIFFING_TARGET.id());
            brain.set(types::SNIFFER_DIGGING, true);
            ai::update_activity(brain);
            assert_eq!(brain.get_active_non_core_activity(), Some(Activity::Dig));

            // While panicking -> sniffing is reset, Dig and Sniff are rejected, falls back to Idle
            ai::reset_sniffing(brain, sniffer.as_ref());
            brain.set(types::IS_PANICKING, true);
            ai::update_activity(brain);
            assert_eq!(brain.get_active_non_core_activity(), Some(Activity::Idle));
        });
    }

    #[tokio::test]
    async fn sniffer_store_explored_position_limits_to_20() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        sniffer.mob_entity.with_brain(sniffer.as_ref(), |brain| {
            for i in 0..25 {
                ai::store_explored_position(brain, sniffer.as_ref(), BlockPos::new(i, 64, 0));
            }

            let positions = brain
                .get(types::SNIFFER_EXPLORED_POSITIONS)
                .unwrap_or_else(|| panic!("missing explored positions"));
            assert_eq!(positions.len(), 20);
            assert_eq!(positions[0].pos, BlockPos::new(24, 64, 0));

            let local_explored = sniffer
                .explored_positions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert_eq!(local_explored.len(), 20);
            assert_eq!(local_explored[0], BlockPos::new(24, 64, 0));
        });
    }

    #[tokio::test]
    async fn sniffer_can_sniff_and_can_dig_predicates() {
        let folder = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let world = world(&folder);

        let entity = Entity::new(world, Vector3::default(), &EntityType::SNIFFER);
        let sniffer = SnifferEntity::new(entity);

        sniffer.mob_entity.with_brain(sniffer.as_ref(), |brain| {
            // Grounded, not panicking, not in water, not baby -> can sniff
            sniffer
                .get_entity()
                .on_ground
                .store(true, Ordering::Relaxed);
            assert!(sniffer.can_sniff_with_brain(brain));

            // Panicking -> cannot sniff
            brain.set(types::IS_PANICKING, true);
            assert!(!sniffer.can_sniff_with_brain(brain));
            brain.erase(types::IS_PANICKING.id());

            // Baby cannot dig
            sniffer.get_entity().age.store(-100, Ordering::Relaxed);
            assert!(!sniffer.can_dig_with_brain(brain));
        });
    }
}
