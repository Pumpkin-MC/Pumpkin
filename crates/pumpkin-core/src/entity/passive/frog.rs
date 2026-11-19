mod ai;
mod shoot_tongue;

use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::{
        brain::{Brain, BrainState, memory::PackedMemories, memory::types},
        control::smooth_swimming_move_control::SmoothSwimmingMoveControl,
        pathfinder::{Navigator, node::PathType},
    },
    mob::{Mob, MobEntity, magma_cube::MagmaCubeEntity, slime::SlimeEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

use pumpkin_data::frog_variant::FrogVariant;

const FROG_FALL_DAMAGE_REDUCTION: i32 = 5;
const HEAD_ROT_SPEED: f32 = 35.0;
const MAX_HEAD_Y_ROT: f32 = 5.0;
const MAX_TURN_X: i32 = 85;
const MAX_TURN_Y: i32 = 10;
const IN_WATER_SPEED_MODIFIER: f32 = 0.02;
const OUTSIDE_WATER_SPEED_MODIFIER: f32 = 0.1;
const WATER_DRAG: f64 = 0.9;
const NO_TONGUE_TARGET: i32 = -1;

/// Whether a frog can catch and eat `entity`: frog food, and only the smallest slimes.
#[must_use]
pub fn can_eat(entity: &dyn EntityBase) -> bool {
    let cube_size = entity
        .cast_any()
        .downcast_ref::<SlimeEntity>()
        .map(SlimeEntity::get_size)
        .or_else(|| {
            entity
                .cast_any()
                .downcast_ref::<MagmaCubeEntity>()
                .map(|cube| cube.slime.get_size())
        });
    if cube_size.is_some_and(|size| size != 1) {
        return false;
    }
    entity
        .get_entity()
        .entity_type
        .has_tag(&tag::EntityType::MINECRAFT_FROG_FOOD)
}

/// An amphibious mob that eats small slimes and magma cubes and lays frogspawn after breeding.
///
/// Wiki: <https://minecraft.wiki/w/Frog>
pub struct FrogEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    tongue_target_id: AtomicI32,
}

impl FrogEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        let mut navigator = Navigator::frog();
        navigator.set_pathfinding_malus(PathType::Water, 4.0);
        navigator.set_pathfinding_malus(PathType::Trapdoor, -1.0);
        *mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = navigator;
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingMoveControl::new(
                MAX_TURN_X,
                MAX_TURN_Y,
                IN_WATER_SPEED_MODIFIER,
                OUTSIDE_WATER_SPEED_MODIFIER,
                true,
            ));
        Arc::new(Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(FrogVariant::Temperate.id() as i32),
            tongue_target_id: AtomicI32::new(NO_TONGUE_TARGET),
        })
    }

    #[must_use]
    pub fn get_variant(&self) -> FrogVariant {
        FrogVariant::from_id(self.variant.load(Ordering::Relaxed) as u32)
    }

    pub fn set_variant(&self, variant: FrogVariant) {
        self.set_variant_id(variant.id() as i32);
    }

    pub fn set_variant_id(&self, raw_id: i32) {
        self.variant.store(raw_id, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::frog::VARIANT, VarInt(raw_id));
    }

    #[must_use]
    pub fn get_tongue_target(&self) -> Option<Arc<dyn EntityBase>> {
        let id = self.tongue_target_id.load(Ordering::Relaxed);
        if id == NO_TONGUE_TARGET {
            return None;
        }
        self.get_entity().world.load().get_entity_by_id(id)
    }

    pub fn set_tongue_target(&self, target: &dyn EntityBase) {
        self.store_tongue_target(target.get_entity().entity_id);
    }

    pub fn erase_tongue_target(&self) {
        self.store_tongue_target(NO_TONGUE_TARGET);
    }

    fn store_tongue_target(&self, id: i32) {
        if self.tongue_target_id.swap(id, Ordering::Relaxed) != id {
            // OPTIONAL_UNSIGNED_INT uses zero for absent and id + 1 for present.
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::frog::DATA_TONGUE_TARGET_ID,
                VarInt(id + 1),
            );
        }
    }

    fn select_variant_for_biome(&self, biome_name: &str) {
        self.set_variant(FrogVariant::select_for_biome(biome_name));
    }
}

impl AgeableMob for FrogEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn can_be_a_baby(&self) -> bool {
        false
    }
}

impl Animal for FrogEntity {
    fn play_eating_sound(&self) {
        self.play_entity_sound(Sound::EntityFrogEat, 2.0, 1.0);
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_FROG_FOOD)
    }

    fn get_breed_offspring(&self, _partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let frog = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            entity.entity_type,
        ));
        frog.mob_entity.with_brain(frog.as_ref(), |brain| {
            ai::init_memories(brain, frog.as_ref());
        });
        frog
    }

    fn spawns_offspring(&self) -> bool {
        false
    }

    fn on_bred_without_offspring(&self, brain: Option<&mut BrainState>) {
        match brain {
            Some(brain) => brain.set(types::IS_PREGNANT, ()),
            None => self
                .mob_entity
                .with_brain(self, |brain| brain.set(types::IS_PREGNANT, ())),
        }
    }
}

impl Mob for FrogEntity {
    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        let attack_target = self.mob_entity.with_brain(self, |brain| {
            ai::update_activity(brain);
            brain.get(types::ATTACK_TARGET).cloned()
        });
        // `getTarget` reads the brain's attack target.
        self.mob_entity.set_target(attack_target);
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        self.mob_entity.try_attack(self, target)
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        HEAD_ROT_SPEED
    }

    fn get_max_head_rotation(&self) -> f32 {
        MAX_HEAD_Y_ROT
    }

    fn look_resets_pitch(&self) -> bool {
        self.tongue_target_id.load(Ordering::Relaxed) == NO_TONGUE_TARGET
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn travel_in_water(&self, caller: &dyn EntityBase, _gravity: f64, _falling: bool) {
        let living = &self.mob_entity.living_entity;
        living.entity.update_velocity_from_input(
            living.movement_input.load(),
            f64::from(living.get_speed()),
        );
        living.make_move(caller);
        living
            .entity
            .velocity
            .store(living.entity.velocity.load() * WATER_DRAG);
    }

    fn calculate_fall_damage(&self, fall_distance: f32, damage_modifier: f32) -> i32 {
        self.mob_entity
            .living_entity
            .base_fall_damage(fall_distance, damage_modifier)
            - FROG_FALL_DAMAGE_REDUCTION
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityFrogStep, 0.15, 1.0);
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        let biome = world.get_biome(&self.get_entity().block_pos.load());
        self.select_variant_for_biome(biome.registry_id);
        self.mob_entity
            .with_brain(self, |brain| ai::init_memories(brain, self));
        self.finalize_spawn_super(group_data)
    }

    fn finalize_spawn_in_biome(&self, biome: &'static pumpkin_data::chunk::Biome) {
        self.select_variant_for_biome(biome.registry_id);
        self.mob_entity
            .with_brain(self, |brain| ai::init_memories(brain, self));
    }

    fn mob_get_component(
        &self,
        component: pumpkin_data::data_component::DataComponent,
    ) -> Option<Box<dyn pumpkin_data::data_component_impl::DataComponentImpl>> {
        use pumpkin_data::{data_component::DataComponent, data_component_impl::FrogVariantImpl};
        match component {
            DataComponent::FrogVariant => Some(Box::new(FrogVariantImpl {
                value: format!("minecraft:{}", self.get_variant().to_name()).into(),
            })),
            _ => None,
        }
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let variant_id = self.variant.load(Ordering::Relaxed) as u32;
        let variant_str =
            self.resolve_dynamic_entry_id("minecraft:frog_variant", variant_id, |_id| {
                self.get_variant().asset_id().to_string()
            });
        nbt.put_string("variant", variant_str);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("variant")
            && let Some(id) =
                self.resolve_dynamic_raw_id("minecraft:frog_variant", variant_str, |n| {
                    FrogVariant::from_name(n).map(|v| v.id())
                })
        {
            self.set_variant_id(id as i32);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(id) = self.resolve_dynamic_raw_id("minecraft:frog_variant", name, |n| {
            FrogVariant::from_name(n).map(|v| v.id())
        }) {
            self.set_variant_id(id as i32);
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::frog::VARIANT,
            VarInt(self.variant.load(Ordering::Relaxed)),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}
