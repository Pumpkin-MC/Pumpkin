use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{CustomNameImpl, EquipmentSlot, PotionContentsImpl};
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{
    Hand,
    math::{position::BlockPos, vector3::Vector3},
};
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::{
        brain::{
            Brain, BrainState,
            memory::{PackedMemories, types},
        },
        control::flying_move_control::FlyingMoveControl,
        pathfinder::Navigator,
    },
    custom_sound::CustomSound,
    item::ItemEntity,
    living::prevents_equipment_drop,
    mob::{Mob, MobEntity},
    player::{Player, advancement::trigger::AdvancementTrigger},
};

use crate::world::game_event::GameEventListener;

mod ai;
mod chunk_loading;
mod vibration;

const DUPLICATION_COOLDOWN_TICKS: i64 = 6000;
const ITEM_PICKUP_REACH: Vector3<f64> = Vector3::new(1.0, 1.0, 1.0);
const AMBIENT_SOUND_INTERVAL: i32 = 80;
const THROW_SOUND_PITCHES: &[f32] = &[
    0.5625, 0.625, 0.75, 0.9375, 1.0, 1.0, 1.125, 1.25, 1.5, 1.875, 2.0, 2.25, 2.5, 3.0, 3.75, 4.0,
];

pub struct AllayEntity {
    pub mob_entity: MobEntity,
    dancing: AtomicBool,
    duplication_cooldown: AtomicI64,
    inventory: Mutex<ItemStack>,
    jukebox_pos: AtomicCell<Option<BlockPos>>,
    vibration: Mutex<vibration::VibrationData>,
    ambient_sound_time: AtomicI32,
    noteblock_chunk: Mutex<Option<chunk_loading::NoteBlockChunk>>,
}

impl AllayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(20, true));
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *navigator = Navigator::flying();
        navigator.set_can_open_doors(false);
        navigator.set_can_float(true);
        navigator.set_required_path_length(48.0);
        Arc::new(Self {
            mob_entity,
            dancing: AtomicBool::new(false),
            duplication_cooldown: AtomicI64::new(0),
            inventory: Mutex::new(ItemStack::EMPTY.clone()),
            jukebox_pos: AtomicCell::new(None),
            vibration: Mutex::new(vibration::VibrationData::default()),
            ambient_sound_time: AtomicI32::new(0),
            noteblock_chunk: Mutex::new(None),
        })
    }

    #[must_use]
    pub fn is_dancing(&self) -> bool {
        self.dancing.load(Ordering::Relaxed)
    }

    pub fn set_dancing(&self, dancing: bool) {
        if self.mob_entity.is_no_ai() || (dancing && self.is_panicking()) {
            return;
        }
        self.dancing.store(dancing, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::allay::DATA_DANCING, dancing);
    }

    #[must_use]
    pub fn can_duplicate(&self) -> bool {
        self.duplication_cooldown.load(Ordering::Relaxed) == 0
    }

    fn set_duplication_cooldown(&self, ticks: i64) {
        let previous = self.duplication_cooldown.swap(ticks, Ordering::Relaxed);
        if (previous == 0) != (ticks == 0) {
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::allay::DATA_CAN_DUPLICATE,
                ticks == 0,
            );
        }
    }

    fn is_panicking(&self) -> bool {
        self.mob_entity.with_brain(self, |brain| {
            brain.has_memory_value(types::IS_PANICKING.id())
        })
    }

    fn sample(&self) -> ItemStack {
        self.mob_entity.living_entity.held_item(self)
    }

    fn set_sample(&self, stack: ItemStack) {
        let living = &self.mob_entity.living_entity;
        living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(&EquipmentSlot::MAIN_HAND, stack.clone());
        living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, stack)]);
    }

    fn considers_item_equal(first: &ItemStack, second: &ItemStack) -> bool {
        first.item == second.item
            && first.get_data_component::<PotionContentsImpl>()
                == second.get_data_component::<PotionContentsImpl>()
    }

    fn can_add_item(&self, stack: &ItemStack) -> bool {
        let stored = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stored.is_empty()
            || (stored.are_items_and_components_equal(stack)
                && stored.item_count < stored.get_max_stack_size())
    }

    fn duplicate_allay(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let child = Self::new(Entity::new(
            world.clone(),
            entity.pos.load(),
            &EntityType::ALLAY,
        ));
        child
            .mob_entity
            .persistence_required
            .store(true, Ordering::Relaxed);
        child.set_duplication_cooldown(DUPLICATION_COOLDOWN_TICKS);
        self.set_duplication_cooldown(DUPLICATION_COOLDOWN_TICKS);
        world.spawn_entity(child);
    }

    fn add_item(&self, stack: &mut ItemStack) {
        let mut stored = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if stored.is_empty() {
            *stored = stack.split(stack.get_max_stack_size());
        } else if stored.are_items_and_components_equal(stack) {
            let count = stack.item_count.min(
                stored
                    .get_max_stack_size()
                    .saturating_sub(stored.item_count),
            );
            stack.decrement(count);
            stored.increment(count);
        }
    }

    fn pick_up_items(&self) {
        if self.mob_entity.living_entity.is_dead_or_dying() || !self.can_pick_up_loot(None) {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let area = entity.bounding_box.load().expand(
            ITEM_PICKUP_REACH.x,
            ITEM_PICKUP_REACH.y,
            ITEM_PICKUP_REACH.z,
        );
        for other in world.get_entities_at_box(&area) {
            let Some(item) = other.cast_any().downcast_ref::<ItemEntity>() else {
                continue;
            };
            if item.get_pickup_delay() != 0 || item.get_entity().is_removed() {
                continue;
            }
            let stack = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if !self.wants_to_pick_up(&stack) {
                continue;
            }
            let count = {
                let stored = self
                    .inventory
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                stack.item_count.min(if stored.is_empty() {
                    stack.get_max_stack_size()
                } else {
                    stored.get_max_stack_size() - stored.item_count
                })
            };
            if count == 0
                || !self
                    .mob_entity
                    .living_entity
                    .pickup(item.get_entity(), u32::from(count))
            {
                continue;
            }
            let mut ground = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.add_item(&mut ground);
            let empty = ground.is_empty();
            drop(ground);
            if empty {
                item.get_entity().remove();
            } else {
                item.init_data_tracker();
            }
        }
    }

    fn throw_item(
        &self,
        stack: ItemStack,
        target: Vector3<f64>,
        velocity: Vector3<f64>,
        eye_offset: f64,
    ) {
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let item = ItemEntity::new(
            Entity::new(
                entity.world.load_full(),
                Vector3::new(pos.x, self.get_eye_pos().y - eye_offset, pos.z),
                &EntityType::ITEM,
            ),
            stack,
        );
        item.set_thrower(entity.entity_uuid);
        item.get_entity().velocity.store(
            (target - pos)
                .normalize()
                .multiply(velocity.x, velocity.y, velocity.z),
        );
        entity.world.load().spawn_entity(Arc::new(item));
    }

    fn throw_one_item(
        &self,
        target: Vector3<f64>,
        velocity: Vector3<f32>,
        liked_player: Option<Arc<Player>>,
    ) {
        let stack = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .split(1);
        if stack.is_empty() {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        self.throw_item(
            stack.clone(),
            target + Vector3::new(0.0, 1.0, 0.0),
            velocity.to_f64(),
            f64::from(0.2f32),
        );
        if let Some(player) = liked_player {
            player.trigger_advancement(AdvancementTrigger::AllayDropItemOnBlock {
                block_id: format!(
                    "minecraft:{}",
                    world.get_block(&BlockPos::floored_v(target).down()).name
                ),
                item_id: format!("minecraft:{}", stack.item.registry_key),
            });
        }
        let mut rng = self.get_random();
        if world.get_world_age() % 7 == 0 && rng.random::<f64>() < 0.9 {
            world.play_sound_raw(
                Sound::EntityAllayItemThrown as u16,
                SoundCategory::Neutral,
                &entity.pos.load(),
                1.0,
                THROW_SOUND_PITCHES[rng.random_range(0..THROW_SOUND_PITCHES.len())],
            );
        }
    }

    pub fn set_jukebox_playing(&self, pos: BlockPos, playing: bool) {
        if playing {
            if !self.is_dancing() {
                self.jukebox_pos.store(Some(pos));
                self.set_dancing(true);
            }
        } else if self.jukebox_pos.load().is_none_or(|current| current == pos) {
            self.jukebox_pos.store(None);
            self.set_dancing(false);
        }
    }

    fn interact_with_item(&self, player: &Arc<Player>, stack: &mut ItemStack, hand: Hand) -> bool {
        if self.mob_entity.living_entity.is_dead_or_dying() || self.get_entity().is_removed() {
            return true;
        }
        if stack.item == &pumpkin_data::item::Item::NAME_TAG
            && stack.get_data_component::<CustomNameImpl>().is_some()
        {
            return false;
        }
        if self
            .mob_entity
            .mob_interact(player, stack, self.can_be_leashed())
        {
            return true;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let sound;
        if self.is_dancing()
            && self.can_duplicate()
            && stack.item.has_tag(&tag::Item::MINECRAFT_DUPLICATES_ALLAYS)
        {
            self.duplicate_allay();
            world.send_entity_status(entity, EntityStatus::InLoveHearts, None);
            sound = Sound::BlockAmethystBlockChime;
            stack.decrement_unless_creative(player.gamemode.load(), 1);
        } else if self.sample().is_empty() && !stack.is_empty() {
            self.set_sample(stack.copy_with_count(1));
            stack.decrement_unless_creative(player.gamemode.load(), 1);
            self.mob_entity.with_brain(self, |brain| {
                brain.set(types::LIKED_PLAYER, player.gameprofile.id);
            });
            sound = Sound::EntityAllayItemGiven;
        } else if !self.sample().is_empty() && hand == Hand::Right && stack.is_empty() {
            let sample = self.sample();
            self.set_sample(ItemStack::EMPTY.clone());
            self.mob_entity.living_entity.swing_hand();
            let stored = std::mem::replace(
                &mut *self
                    .inventory
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                ItemStack::EMPTY.clone(),
            );
            if !stored.is_empty() {
                self.throw_item(
                    stored,
                    entity.pos.load(),
                    Vector3::new(f64::from(0.3f32), f64::from(0.3f32), f64::from(0.3f32)),
                    f64::from(0.3f32),
                );
            }
            self.mob_entity
                .with_brain(self, |brain| brain.erase(types::LIKED_PLAYER.id()));
            let mut given = sample;
            player.inventory().insert_stack_anywhere(&mut given);
            *stack = player.inventory().held_item();
            sound = Sound::EntityAllayItemTaken;
        } else {
            return false;
        }
        world.play_sound_raw_expect(
            player,
            sound as u16,
            SoundCategory::Neutral,
            &entity.pos.load(),
            2.0,
            1.0,
        );
        true
    }
}

impl CustomSound for AllayEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityAllayHurt)
    }
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityAllayDeath)
    }
    fn sound_volume(&self) -> f32 {
        0.4
    }
    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Neutral
    }
    fn on_hurt_sound(&self) {
        self.ambient_sound_time
            .store(-AMBIENT_SOUND_INTERVAL, Ordering::Relaxed);
    }
}

impl Mob for AllayEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn base_tick(&self) {
        if !self.mob_entity.living_entity.is_dead_or_dying()
            && self.get_random().random_range(0..1000)
                < self.ambient_sound_time.fetch_add(1, Ordering::Relaxed)
        {
            self.ambient_sound_time
                .store(-AMBIENT_SOUND_INTERVAL, Ordering::Relaxed);
            let entity = self.get_entity();
            if !entity.is_silent() {
                let sound = if self.sample().is_empty() {
                    Sound::EntityAllayAmbientWithoutItem
                } else {
                    Sound::EntityAllayAmbientWithItem
                };
                entity.world.load().play_sound_fine(
                    sound,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                    self.sound_volume(),
                    self.mob_entity.living_entity.get_pitch(),
                );
            }
        }
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        false
    }

    fn can_dispenser_equip_into_slot(&self, _slot: &EquipmentSlot) -> bool {
        false
    }

    fn should_check_fall_damage(&self) -> bool {
        false
    }

    fn has_global_tick_reach(&self) -> bool {
        true
    }

    fn can_pick_up_loot(&self, brain: Option<&BrainState>) -> bool {
        let cooldown = types::ITEM_PICKUP_COOLDOWN_TICKS.id();
        !self.sample().is_empty()
            && !brain.map_or_else(
                || {
                    self.mob_entity
                        .with_brain(self, |brain| brain.has_memory_value(cooldown))
                },
                |brain| brain.has_memory_value(cooldown),
            )
    }

    fn for_each_game_event_listener(&self, visit: &mut dyn FnMut(&dyn GameEventListener)) {
        visit(&vibration::VibrationListener(self));
        visit(&vibration::JukeboxListener(self));
    }

    fn mob_considers_entity_as_ally(
        &self,
        other: &dyn EntityBase,
        brain: Option<&BrainState>,
    ) -> Option<bool> {
        let liked = |brain: &BrainState| {
            other.get_player().is_some()
                && brain
                    .get(types::LIKED_PLAYER)
                    .is_some_and(|uuid| *uuid == other.get_entity().entity_uuid)
        };
        Some(
            brain.map_or_else(
                || self.mob_entity.with_brain(self, |brain| liked(brain)),
                liked,
            ) || self.is_on_same_team(other),
        )
    }

    fn travel(&self, caller: &dyn EntityBase) -> bool {
        self.mob_entity.living_entity.travel_flying(caller);
        true
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, |brain| {
            brain.set_active_activity_to_first_valid(&[
                pumpkin_data::environment_attribute::Activity::Idle,
            ]);
        });
    }

    fn on_removed(&self) {
        self.noteblock_chunk
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    fn pre_damage_with_context(
        &self,
        _damage_type: DamageType,
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        !cause.is_some_and(|other| {
            other.get_player().is_some()
                && self.mob_entity.with_brain(self, |brain| {
                    brain
                        .get(types::LIKED_PLAYER)
                        .is_some_and(|liked| *liked == other.get_entity().entity_uuid)
                })
        })
    }

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        let sample = self.sample();
        !sample.is_empty()
            && !stack.is_empty()
            && self
                .get_entity()
                .world
                .load()
                .level_info
                .load()
                .game_rules
                .mob_griefing
            && self.can_add_item(stack)
            && Self::considers_item_equal(&sample, stack)
    }

    fn post_tick(&self) {
        self.finish_noteblock_chunk();
        self.pick_up_items();
        let entity = self.get_entity();
        let tick = entity.tick_count.load(Ordering::Relaxed);
        if !self.mob_entity.living_entity.is_dead_or_dying() && tick % 10 == 0 {
            self.mob_entity.living_entity.heal(1.0);
        }
        if self.is_dancing()
            && tick % 20 == 0
            && self.jukebox_pos.load().is_none_or(|pos| {
                pos.to_centered_f64()
                    .squared_distance_to_vec(&entity.pos.load())
                    >= f64::from(
                        pumpkin_data::game_event::GameEvent::JukeboxPlay.notification_radius(),
                    )
                    .powi(2)
                    || entity.world.load().get_block(&pos) != &pumpkin_data::Block::JUKEBOX
            })
        {
            self.set_dancing(false);
            self.jukebox_pos.store(None);
        }
        let cd = self.duplication_cooldown.load(Ordering::Relaxed);
        if cd > 0 {
            self.set_duplication_cooldown(cd - 1);
        }
        self.tick_vibration();
        if self.is_panicking() {
            self.set_dancing(false);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_long(
            "DuplicationCooldown",
            self.duplication_cooldown.load(Ordering::Relaxed),
        );
        let stack = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let items = if stack.is_empty() {
            Vec::new()
        } else {
            let mut item = NbtCompound::new();
            stack.write_item_stack(&mut item);
            vec![NbtTag::Compound(item)]
        };
        nbt.put_list("Inventory", items);
        nbt.put_compound(
            "listener",
            self.vibration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .write_nbt(),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_duplication_cooldown(
            nbt.get_long("DuplicationCooldown")
                .or_else(|| nbt.get_int("DuplicationCooldown").map(i64::from))
                .unwrap_or(0),
        );
        *self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
        if let Some(items) = nbt.get_list("Inventory") {
            for item in items {
                if let Some(mut stack) =
                    item.extract_compound().and_then(ItemStack::read_item_stack)
                {
                    self.add_item(&mut stack);
                }
            }
        }
        *self
            .vibration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = nbt
            .get_compound("listener")
            .map(vibration::VibrationData::read_nbt)
            .unwrap_or_default();
    }

    fn drop_inventory(&self) {
        let stack = std::mem::replace(
            &mut *self
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ItemStack::EMPTY.clone(),
        );
        if !stack.is_empty() {
            self.get_entity()
                .world
                .load()
                .drop_stack(&self.get_entity().block_pos.load(), stack);
        }
        let sample = self.sample();
        if !sample.is_empty() {
            if !prevents_equipment_drop(&sample) {
                self.get_entity()
                    .world
                    .load()
                    .drop_stack(&self.get_entity().block_pos.load(), sample);
            }
            self.set_sample(ItemStack::EMPTY.clone());
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::allay::DATA_DANCING,
            self.is_dancing(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::allay::DATA_CAN_DUPLICATE,
            self.can_duplicate(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, stack: &mut ItemStack) -> bool {
        self.interact_with_item(player, stack, Hand::Right)
    }

    fn mob_interact_with_hand(
        &self,
        player: &Arc<Player>,
        stack: &mut ItemStack,
        hand: Hand,
    ) -> Option<bool> {
        Some(self.interact_with_item(player, stack, hand))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::item::Item;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn duplication_cools_down_both_allays_and_persists_the_child() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let allay = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::ALLAY,
        ));
        allay.duplicate_allay();
        let entities = world.entities.load_full();
        assert_eq!(entities.len(), 1);
        let child = entities[0]
            .cast_any()
            .downcast_ref::<AllayEntity>()
            .ok_or("missing duplicated allay")?;
        assert!(!allay.can_duplicate());
        assert!(!child.can_duplicate());
        assert!(
            child
                .mob_entity
                .persistence_required
                .load(Ordering::Relaxed)
        );
        let mut nbt = NbtCompound::new();
        child.mob_write_nbt(&mut nbt);
        assert_eq!(nbt.get_long("DuplicationCooldown"), Some(6000));
        let restored = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.0, 64.0, 0.0),
            &EntityType::ALLAY,
        ));
        restored.mob_read_nbt(&nbt);
        assert!(!restored.can_duplicate());
        restored.set_duplication_cooldown(1);
        restored.post_tick();
        assert!(restored.can_duplicate());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn going_to_a_wanted_item_reads_the_cooldown_from_the_ticking_brain() -> TestResult {
        use crate::entity::ai::brain::{activity::ActivityData, behavior::go_to_wanted_item};

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let allay = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::ALLAY,
        ));
        allay.set_sample(ItemStack::new(1, &Item::DIAMOND));
        let item: Arc<dyn EntityBase> = Arc::new(ItemEntity::new(
            Entity::new(
                world.clone(),
                Vector3::new(3.5, 64.0, 0.5),
                &EntityType::ITEM,
            ),
            ItemStack::new(1, &Item::DIAMOND),
        ));
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            pumpkin_data::environment_attribute::Activity::Core,
            0,
            vec![Box::new(go_to_wanted_item::create(1.75, true, 32))],
        ));
        brain.set(types::NEAREST_VISIBLE_WANTED_ITEM, item);
        allay.mob_entity.set_brain(brain);
        allay.mob_entity.tick_brain(allay.as_ref());
        let walking = allay.mob_entity.with_brain(allay.as_ref(), |brain| {
            brain.has_memory_value(types::WALK_TARGET.id())
        });
        world.level.shutdown().await;
        assert!(walking);
        Ok(())
    }

    #[tokio::test]
    async fn no_ai_pickup_respects_capacity_cooldown_and_living_state() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let allay = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::ALLAY,
        ));
        allay.set_sample(ItemStack::new(1, &Item::DIAMOND));
        allay.mob_entity.set_no_ai(true);
        *allay
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            ItemStack::new(63, &Item::DIAMOND);
        let item = Arc::new(ItemEntity::new(
            Entity::new(
                world.clone(),
                Vector3::new(0.5, 64.0, 0.5),
                &EntityType::ITEM,
            ),
            ItemStack::new(4, &Item::DIAMOND),
        ));
        item.set_pickup_delay(0);
        world.add_entity_silent(item.clone());
        allay.pick_up_items();
        assert_eq!(
            allay
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .item_count,
            64
        );
        assert_eq!(
            item.get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .item_count,
            3
        );
        *allay
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
        allay.mob_entity.with_brain(allay.as_ref(), |brain| {
            brain.set(types::ITEM_PICKUP_COOLDOWN_TICKS, 60);
        });
        allay.pick_up_items();
        assert!(
            allay
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );
        allay.mob_entity.with_brain(allay.as_ref(), |brain| {
            brain.erase(types::ITEM_PICKUP_COOLDOWN_TICKS.id());
        });
        allay.mob_entity.living_entity.health.store(0.0);
        allay.post_tick();
        assert_eq!(allay.mob_entity.living_entity.health.load(), 0.0);
        assert!(
            allay
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );
        world.level.shutdown().await;
        Ok(())
    }
}
