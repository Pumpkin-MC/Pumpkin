use super::{Entity, EntityBase, ai::pathfinder::Navigator, living::LivingEntity};
use crate::entity::ai::control::MoveControlTrait;
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::control::move_control::MoveControl;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::{CHeadRot, CUpdateEntityRot, Metadata};
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, get_seed};
use pumpkin_util::version::JavaMinecraftVersion;
use rand::RngExt;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};
use uuid::Uuid;

pub mod bat;
pub mod blaze;
pub mod breeze;
pub mod cave_spider;
pub mod creaking;
pub mod creeper;
pub mod crossbow_attack_mob;
pub mod elder_guardian;
pub mod enderman;
pub mod endermite;
pub mod equipment;
pub mod evoker;
pub mod ghast;
pub mod giant;
pub mod guardian;
pub mod hoglin;
pub mod illusioner;
pub mod magma_cube;
pub mod patrol;
pub mod phantom;
pub mod piglin;
pub mod piglin_ai;
pub mod piglin_brute;
pub mod pillager;
pub mod raider;
pub mod ravager;
pub mod shulker;
pub mod silverfish;
pub mod skeleton;
pub mod slime;
pub mod spider;
pub mod vex;
pub mod vindicator;
pub mod warden;
pub mod witch;
pub mod zoglin;
pub mod zombie;
pub mod zombified_piglin;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    pub goals_selector: std::sync::Mutex<GoalSelector>,
    pub target_selector: std::sync::Mutex<GoalSelector>,
    pub navigator: std::sync::Mutex<Navigator>,
    pub target: std::sync::Mutex<Option<Arc<dyn EntityBase>>>,
    pub look_control: std::sync::Mutex<LookControl>,
    pub move_control: std::sync::Mutex<Box<dyn MoveControlTrait>>,
    pub position_target: AtomicCell<BlockPos>,
    pub position_target_range: AtomicI32,
    pub love_ticks: AtomicI32,
    pub breeding_cooldown: AtomicI32,
    pub breeder: AtomicCell<Option<Uuid>>,
    pub persistence_required: AtomicBool,
    mob_flags: AtomicU8,
    last_sent_yaw: AtomicU8,
    last_sent_pitch: AtomicU8,
    last_sent_head_yaw: AtomicU8,
}

/// Tick boundaries (both inclusive) when monsters do not burn in sunlight (26.1).
///
/// Sourced from `data/minecraft/timeline/day.json` — `monsters_burn` keyframes:
/// `value=false` at tick 12542 (dusk), `value=true` at tick 23460 (dawn).
///
/// TODO: Replace with `EnvironmentAttributes::MONSTERS_BURN` lookup once the
/// `EnvironmentAttributeSystem` is implemented in `pumpkin-data`.
pub(crate) const NIGHT_START: i64 = 12542;
pub(crate) const NIGHT_END: i64 = 23459;

impl MobEntity {
    const AI_DISABLED_FLAG: u8 = 1;
    const LEFT_HANDED_FLAG: u8 = 2;
    const ATTACKING_FLAG: u8 = 4;
    const CAN_PICK_UP_LOOT_FLAG: u8 = 8;

    pub const MAX_WEARING_ARMOR_CHANCE: f32 = 0.15;
    pub const WEARING_ARMOR_UPGRADE_MATERIAL_CHANCE: f32 = 0.1087;
    pub const WEARING_ARMOR_UPGRADE_MATERIAL_ATTEMPTS: f32 = 3.0;
    pub const MAX_PICKUP_LOOT_CHANCE: f32 = 0.55;
    pub const MAX_ENCHANTED_ARMOR_CHANCE: f32 = 0.5;
    pub const MAX_ENCHANTED_WEAPON_CHANCE: f32 = 0.25;
    pub const EQUIPMENT_POPULATION_ORDER: [EquipmentSlot; 4] = [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
    ];

    #[must_use]
    pub const fn get_equipment_for_slot(
        slot: &EquipmentSlot,
        armor_type: i32,
    ) -> Option<&'static Item> {
        match slot {
            EquipmentSlot::Head(_) => match armor_type {
                0 => Some(&Item::LEATHER_HELMET),
                1 => Some(&Item::COPPER_HELMET),
                2 => Some(&Item::GOLDEN_HELMET),
                3 => Some(&Item::CHAINMAIL_HELMET),
                4 => Some(&Item::IRON_HELMET),
                5 => Some(&Item::DIAMOND_HELMET),
                _ => None,
            },
            EquipmentSlot::Chest(_) => match armor_type {
                0 => Some(&Item::LEATHER_CHESTPLATE),
                1 => Some(&Item::COPPER_CHESTPLATE),
                2 => Some(&Item::GOLDEN_CHESTPLATE),
                3 => Some(&Item::CHAINMAIL_CHESTPLATE),
                4 => Some(&Item::IRON_CHESTPLATE),
                5 => Some(&Item::DIAMOND_CHESTPLATE),
                _ => None,
            },
            EquipmentSlot::Legs(_) => match armor_type {
                0 => Some(&Item::LEATHER_LEGGINGS),
                1 => Some(&Item::COPPER_LEGGINGS),
                2 => Some(&Item::GOLDEN_LEGGINGS),
                3 => Some(&Item::CHAINMAIL_LEGGINGS),
                4 => Some(&Item::IRON_LEGGINGS),
                5 => Some(&Item::DIAMOND_LEGGINGS),
                _ => None,
            },
            EquipmentSlot::Feet(_) => match armor_type {
                0 => Some(&Item::LEATHER_BOOTS),
                1 => Some(&Item::COPPER_BOOTS),
                2 => Some(&Item::GOLDEN_BOOTS),
                3 => Some(&Item::CHAINMAIL_BOOTS),
                4 => Some(&Item::IRON_BOOTS),
                5 => Some(&Item::DIAMOND_BOOTS),
                _ => None,
            },
            _ => None,
        }
    }

    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self {
            living_entity: LivingEntity::new(entity),
            goals_selector: std::sync::Mutex::new(GoalSelector::default()),
            target_selector: std::sync::Mutex::new(GoalSelector::default()),
            navigator: std::sync::Mutex::new(Navigator::default()),
            target: std::sync::Mutex::new(None),
            look_control: std::sync::Mutex::new(LookControl::default()),
            move_control: std::sync::Mutex::new(Box::new(MoveControl::default())),
            position_target: AtomicCell::new(BlockPos::ZERO),
            position_target_range: AtomicI32::new(-1),
            love_ticks: AtomicI32::new(0),
            breeding_cooldown: AtomicI32::new(0),
            breeder: AtomicCell::new(None),
            persistence_required: AtomicBool::new(false),
            mob_flags: AtomicU8::new(0),
            last_sent_yaw: AtomicU8::new(0),
            last_sent_pitch: AtomicU8::new(0),
            last_sent_head_yaw: AtomicU8::new(0),
        }
    }

    pub fn has_position_target(&self) -> bool {
        self.position_target_range.load(Relaxed) != -1
    }

    pub fn is_in_position_target_range(&self) -> bool {
        self.is_in_position_target_range_pos(&self.living_entity.entity.block_pos.load())
    }

    pub fn is_in_position_target_range_pos(&self, block_pos: &BlockPos) -> bool {
        let position_target_range = self.position_target_range.load(Relaxed);
        if position_target_range == -1 {
            true
        } else {
            self.position_target.load().squared_distance(block_pos)
                < position_target_range * position_target_range
        }
    }

    pub fn set_attacking(&self, attacking: bool) {
        self.set_mob_flag(Self::ATTACKING_FLAG, attacking);
    }

    pub fn is_attacking(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::ATTACKING_FLAG) != 0
    }

    pub fn set_left_handed(&self, left_handed: bool) {
        self.set_mob_flag(Self::LEFT_HANDED_FLAG, left_handed);
    }

    pub fn can_pick_up_loot(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::CAN_PICK_UP_LOOT_FLAG) != 0
    }

    pub fn set_can_pick_up_loot(&self, value: bool) {
        self.set_mob_flag(Self::CAN_PICK_UP_LOOT_FLAG, value);
    }

    pub fn is_left_handed(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::LEFT_HANDED_FLAG) != 0
    }

    pub fn set_no_ai(&self, no_ai: bool) {
        self.set_mob_flag(Self::AI_DISABLED_FLAG, no_ai);
    }

    pub fn is_no_ai(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::AI_DISABLED_FLAG) != 0
    }

    pub fn clear_ai_goals(&self, mob: &dyn Mob) {
        let running_goals = self
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        for mut goal in running_goals {
            goal.goal.stop(mob);
        }

        let running_target_goals = self
            .target_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        for mut goal in running_target_goals {
            goal.goal.stop(mob);
        }
    }

    pub fn write_mob_nbt(&self, nbt: &mut NbtCompound) {
        if self.is_no_ai() {
            nbt.put_bool("NoAI", true);
        }
        if self.is_left_handed() {
            nbt.put_bool("LeftHanded", true);
        }
        if self.can_pick_up_loot() {
            nbt.put_bool("CanPickUpLoot", true);
        }
        if self.persistence_required.load(Relaxed) {
            nbt.put_bool("PersistenceRequired", true);
        }
    }

    pub fn read_mob_nbt(&self, nbt: &NbtCompound) {
        if let Some(no_ai) = nbt.get_bool("NoAI") {
            self.set_no_ai(no_ai);
        }
        if let Some(left_handed) = nbt.get_bool("LeftHanded") {
            self.set_left_handed(left_handed);
        }
        if let Some(can_pick_up_loot) = nbt.get_bool("CanPickUpLoot") {
            self.set_can_pick_up_loot(can_pick_up_loot);
        }
        if let Some(persistence_required) = nbt.get_bool("PersistenceRequired") {
            self.persistence_required
                .store(persistence_required, Relaxed);
        }
    }

    pub fn add_goal<G: crate::entity::ai::goal::Goal + 'static>(&self, priority: u8, goal: G) {
        self.goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_goal(priority, Box::new(goal));
    }

    pub fn add_target_goal<G: crate::entity::ai::goal::Goal + 'static>(
        &self,
        priority: u8,
        goal: G,
    ) {
        self.target_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_goal(priority, Box::new(goal));
    }

    pub fn set_target(&self, target: Option<Arc<dyn EntityBase>>) {
        let mut t = self
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *t = target;
    }

    pub fn get_target(&self) -> Option<Arc<dyn EntityBase>> {
        self.target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn set_mob_flag(&self, flag: u8, value: bool) {
        let old_b = self.mob_flags.load(Ordering::Relaxed);

        let new_b = if value { old_b | flag } else { old_b & !flag };

        if new_b != old_b {
            self.mob_flags.store(new_b, Ordering::Relaxed);

            self.living_entity.entity.send_meta_data(
                &[Metadata::new(tracked_data::mob::DATA_MOB_FLAGS_ID, new_b)],
                None,
            );
        }
    }

    pub fn is_in_love(&self) -> bool {
        self.love_ticks.load(Relaxed) > 0
    }

    pub fn set_love_ticks(&self, ticks: i32, breeder: Option<Uuid>) {
        self.love_ticks.store(ticks, Relaxed);
        self.breeder.store(breeder);
    }

    pub fn reset_love_ticks(&self) {
        self.love_ticks.store(0, Relaxed);
    }

    pub fn is_breeding_ready(&self) -> bool {
        self.living_entity.entity.age.load(Relaxed) >= 0
            && self.breeding_cooldown.load(Relaxed) <= 0
    }

    pub fn is_in_attack_range(&self, target: &dyn EntityBase) -> bool {
        const DEFAULT_ATTACK_RANGE: f64 = 0.828_427_12; // sqrt(2.04) - 0.6

        // TODO: Implement DataComponent lookup for ATTACK_RANGE when components are ready
        let max_range = DEFAULT_ATTACK_RANGE;
        let min_range = 0.0;

        let target_hitbox = target.get_entity().bounding_box.load();

        if !self.get_attack_box(max_range).intersects(&target_hitbox) {
            return false;
        }

        min_range <= 0.0 || !self.get_attack_box(min_range).intersects(&target_hitbox)
    }

    pub fn is_dark_enough_to_spawn(world: &World, pos: &BlockPos, is_thundering: bool) -> bool {
        let sky_light = world.get_sky_light_level(pos);
        if sky_light > rand::random_range(0..32) {
            return false;
        }

        let dimension = &world.dimension;
        let block_light_limit = dimension.monster_spawn_block_light_limit;

        let block_light = world.get_block_light_level(pos).unwrap_or(0);
        if block_light_limit < 15 && block_light > block_light_limit {
            return false;
        }

        let current_brightness = if is_thundering {
            (sky_light - 10).max(block_light)
        } else {
            sky_light.max(block_light)
        };

        // TODO
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        current_brightness <= dimension.monster_spawn_light_level.get(&mut random) as u8
    }

    pub fn check_mob_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        let below = pos.down();
        let state = world.get_block_state(&below);
        state.is_side_solid(BlockDirection::Up)
    }

    pub fn check_monster_spawn_rules(world: &World, pos: &BlockPos, is_thundering: bool) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }

        if !Self::is_dark_enough_to_spawn(world, pos, is_thundering) {
            return false;
        }

        Self::check_mob_spawn_rules(world, pos)
    }

    pub fn check_any_light_monster_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }

        Self::check_mob_spawn_rules(world, pos)
    }

    pub fn check_surface_monsters_spawn_rules(
        world: &World,
        pos: &BlockPos,
        is_thundering: bool,
    ) -> bool {
        Self::check_monster_spawn_rules(world, pos, is_thundering) && world.can_see_sky(pos)
    }

    pub fn check_animal_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        let below = pos.down();
        world
            .get_block(&below)
            .has_tag(&tag::Block::MINECRAFT_ANIMALS_SPAWNABLE_ON)
            && Self::is_bright_enough_to_spawn(world, pos)
    }

    pub fn is_bright_enough_to_spawn(world: &World, pos: &BlockPos) -> bool {
        world.get_max_local_raw_brightness(pos) > 8
    }

    pub fn check_surface_water_animal_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        let sea_level = world.sea_level;
        let min_spawn_level = sea_level - 13;
        pos.0.y >= min_spawn_level
            && pos.0.y <= sea_level
            && world
                .get_fluid(&pos.down())
                .has_tag(&tag::Fluid::MINECRAFT_WATER)
            && (world.get_block(&pos.up()) == &Block::WATER
                || world
                    .get_fluid(&pos.up())
                    .has_tag(&tag::Fluid::MINECRAFT_WATER))
    }

    pub fn check_surface_ageable_water_creature_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        Self::check_surface_water_animal_spawn_rules(world, pos)
    }

    pub fn try_attack(&self, caller: &dyn EntityBase, target: &dyn EntityBase) {
        if self.living_entity.dead.load(Relaxed) {
            return;
        }

        let attack_damage: f32 =
            self.living_entity
                .get_attribute_value(&Attributes::ATTACK_DAMAGE) as f32;

        let damaged = target.damage_with_context(
            target,
            attack_damage,
            DamageType::MOB_ATTACK,
            None,
            Some(caller),
            Some(caller),
        );

        if damaged {
            self.living_entity
                .last_attacking_id
                .store(target.get_entity().entity_id, Relaxed);
            self.living_entity
                .last_attack_time
                .store(self.living_entity.entity.age.load(Relaxed), Relaxed);
        }
    }

    fn get_attack_box(&self, attack_range: f64) -> BoundingBox {
        let vehicle_opt = self.living_entity.entity.get_vehicle();

        let base_box = vehicle_opt.as_ref().map_or_else(
            || self.living_entity.entity.bounding_box.load(),
            |vehicle| {
                let vehicle_box = vehicle.get_entity().bounding_box.load();
                let my_box = self.living_entity.entity.bounding_box.load();

                BoundingBox {
                    min: Vector3::new(
                        my_box.min.x.min(vehicle_box.min.x),
                        my_box.min.y,
                        my_box.min.z.min(vehicle_box.min.z),
                    ),
                    max: Vector3::new(
                        my_box.max.x.max(vehicle_box.max.x),
                        my_box.max.y,
                        my_box.max.z.max(vehicle_box.max.z),
                    ),
                }
            },
        );

        base_box.expand(attack_range, 0.0, attack_range)
    }

    pub fn tick_sun_burn(&self) {
        if !self
            .living_entity
            .entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_BURN_IN_DAYLIGHT)
        {
            return;
        }
        if !self.is_sun_burn_tick() {
            return;
        }
        self.apply_sun_burn();
    }

    fn is_sun_burn_tick(&self) -> bool {
        let entity = &self.living_entity.entity;

        let world_arc = entity.world.load();
        let world = world_arc.as_ref();

        // Night boundary from data/minecraft/timeline/day.json — monsters_burn keyframes:
        // value=false at tick 12542 (dusk), value=true at tick 23460 (dawn).
        // TODO: read directly from EnvironmentAttributes::MONSTERS_BURN once implemented.

        let day_time = world.get_time_of_day() % 24000;
        if (NIGHT_START..=NIGHT_END).contains(&day_time) {
            return false;
        }

        // Vanilla: getLightLevelDependentMagicValue() — sky light at eye pos, scaled 0–1.
        let eye_block_pos = entity.get_eye_pos();
        let brightness = world
            .level
            .light_engine
            .get_sky_light_level(&world.level, &eye_block_pos.to_block_pos())
            as f32
            / 15.0;

        if brightness <= 0.5 {
            return false;
        }

        let is_in_non_burnable = entity.touching_water.load(Relaxed)
            || world.is_raining()
            || entity.is_in_powder_snow()
            || entity.was_in_powder_snow.load(Relaxed);

        if is_in_non_burnable {
            return false;
        }

        let pos = entity.pos.load();
        let top_y = world.get_top_block(Vector2::new(pos.x as i32, pos.z as i32));
        if (entity.get_eye_y() as i32) < top_y {
            return false;
        }

        let mut rng = rand::rng();
        rng.random::<f32>() * 30.0 < (brightness - 0.4) * 2.0
    }

    fn apply_sun_burn(&self) {
        let entity = &self.living_entity.entity;
        entity.set_on_fire_for(8.0);
    }

    pub fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let entity = &self.living_entity.entity;

        // If already leashed to player, right-clicking unleashes the mob
        let currently_leashed = entity
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some();

        if currently_leashed {
            entity.unleash();
            let lead_item =
                pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::LEAD);
            entity
                .world
                .load()
                .drop_stack(&entity.block_pos.load(), lead_item);
            return true;
        }

        // If holding a lead, leash the mob to the player
        if item_stack.item.registry_key == "lead"
            || item_stack.item.registry_key == "minecraft:lead"
        {
            let diff = entity.pos.load() - player.get_entity().pos.load();
            let dist_sq = diff.length_squared();
            if dist_sq <= Entity::LEASH_SNAP_DISTANCE * Entity::LEASH_SNAP_DISTANCE {
                entity.leash_to(player.clone() as Arc<dyn EntityBase>);
                if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                    item_stack.decrement(1);
                }
                return true;
            }
        }

        false
    }

    pub fn check_despawn(&self, mob: &dyn Mob) {
        let entity = &self.living_entity.entity;

        if self.persistence_required.load(Relaxed) {
            return;
        }

        if (**entity.custom_name.load()).is_some() {
            return;
        }

        let world = entity.world.load();
        let pos = entity.pos.load();
        let players = world.players.load();

        let nearest_dist_sq = players
            .iter()
            .filter(|p| p.gamemode.load() != pumpkin_util::GameMode::Spectator)
            .map(|p| {
                let pp = p.get_entity().pos.load();
                let dx = pp.x - pos.x;
                let dy = pp.y - pos.y;
                let dz = pp.z - pos.z;
                dx * dx + dy * dy + dz * dz
            })
            .fold(f64::MAX, f64::min);

        if nearest_dist_sq == f64::MAX {
            mob.get_entity().remove();
            return;
        }

        if nearest_dist_sq > 128.0 * 128.0 {
            mob.get_entity().remove();
            return;
        }

        if nearest_dist_sq > 32.0 * 32.0 && rand::random::<i32>().wrapping_abs() % 800 == 0 {
            mob.get_entity().remove();
        }
    }
}

pub trait Mob: EntityBase + Send + Sync {
    fn get_random(&self) -> rand::rngs::ThreadRng {
        rand::rng()
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        10.0
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        40.0
    }

    fn get_max_head_rotation(&self) -> f32 {
        75.0
    }

    fn get_mob_entity(&self) -> &MobEntity;

    fn mob_bedrock_identifier(&self) -> Option<&'static str> {
        None
    }

    /// Metadata which must accompany this mob whenever it is spawned for a Java client.
    fn mob_java_spawn_metadata(&self, _version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        None
    }

    /// Metadata which must accompany this mob whenever it is spawned for a Bedrock client.
    fn mob_bedrock_spawn_metadata(
        &self,
    ) -> Option<pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList> {
        None
    }

    fn get_job_site(&self) -> Option<BlockPos> {
        None
    }

    fn is_job_site_pending(&self) -> bool {
        false
    }

    fn release_pending_job_site(&self, _position: BlockPos) {}

    fn get_trading_player(&self) -> Option<Arc<Player>> {
        None
    }

    fn get_home(&self) -> Option<BlockPos> {
        None
    }

    fn get_path_aware_entity(&self) -> Option<&dyn PathAwareEntity> {
        None
    }

    fn get_item_steerable(&self) -> Option<&dyn crate::entity::item_steerable::ItemSteerable> {
        None
    }

    fn is_saddled(&self) -> bool {
        false
    }

    fn can_be_saddled(&self) -> bool {
        false
    }

    fn set_saddled(&self, _saddled: bool) {}

    /// Per-mob tick hook called each tick before AI runs. Override for mob-specific logic.
    fn mob_tick(&self, _caller: &dyn EntityBase) {}

    fn post_tick(&self) {}

    /// Called before damage is applied. Return `false` to cancel the damage entirely.
    /// Used by endermen to dodge projectiles via teleportation.
    fn pre_damage(&self, _damage_type: DamageType, _source: Option<&dyn EntityBase>) -> bool {
        true
    }

    fn on_damage(&self, _damage_type: DamageType, _source: Option<&dyn EntityBase>) {}

    fn on_attack(&self, _target: &dyn EntityBase) {}

    fn on_eating_grass(&self) {}

    fn modify_incoming_damage(&self, amount: f32, _damage_type: DamageType) -> f32 {
        amount
    }

    fn can_attack_with_owner(&self, _target: &dyn EntityBase, _owner: &dyn EntityBase) -> bool {
        true
    }

    fn get_mob_gravity(&self) -> f64 {
        self.get_mob_entity().living_entity.get_gravity()
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        None
    }

    fn as_ageable(&self) -> Option<&dyn crate::entity::ageable::AgeableMob> {
        None
    }

    fn as_animal(&self) -> Option<&dyn crate::entity::passive::animal::Animal> {
        None
    }

    fn as_tamable(&self) -> Option<&dyn crate::entity::passive::tamable::TamableAnimal> {
        None
    }

    fn as_patrolling_monster(&self) -> Option<&dyn patrol::PatrollingMonster> {
        None
    }

    fn as_raider(&self) -> Option<&dyn raider::Raider> {
        None
    }

    fn as_iron_golem(&self) -> Option<&crate::entity::passive::iron_golem::IronGolemEntity> {
        None
    }

    fn as_crossbow_attack_mob(&self) -> Option<&dyn crossbow_attack_mob::CrossbowAttackMob> {
        None
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        difficulty: &crate::entity::mob::equipment::RegionalDifficulty,
    ) {
        if rand::random::<f32>()
            < MobEntity::MAX_WEARING_ARMOR_CHANCE * difficulty.special_multiplier
        {
            let mut armor_type = rand::random_range(0..3);
            for _ in 1..=3 {
                if rand::random::<f32>() < MobEntity::WEARING_ARMOR_UPGRADE_MATERIAL_CHANCE {
                    armor_type += 1;
                }
            }

            let partial_chance = if difficulty.base_difficulty == Difficulty::Hard {
                0.1f32
            } else {
                0.25f32
            };

            let living = &self.get_mob_entity().living_entity;
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut first = true;

            for slot in &MobEntity::EQUIPMENT_POPULATION_ORDER {
                let current = equipment.get(slot);
                if !first && rand::random::<f32>() < partial_chance {
                    break;
                }
                first = false;
                if current.is_empty()
                    && let Some(item) = MobEntity::get_equipment_for_slot(slot, armor_type)
                {
                    equipment.put(slot, ItemStack::new(1, item));
                }
            }
        }
    }

    fn populate_default_equipment_enchantments(
        &self,
        difficulty: &crate::entity::mob::equipment::RegionalDifficulty,
    ) {
        self.enchant_spawned_weapon(difficulty);
        for slot in &MobEntity::EQUIPMENT_POPULATION_ORDER {
            self.enchant_spawned_armor(slot, difficulty);
        }
    }

    fn enchant_spawned_weapon(
        &self,
        difficulty: &crate::entity::mob::equipment::RegionalDifficulty,
    ) {
        self.enchant_spawned_equipment(
            &EquipmentSlot::MAIN_HAND,
            MobEntity::MAX_ENCHANTED_WEAPON_CHANCE,
            difficulty,
        );
    }

    fn enchant_spawned_armor(
        &self,
        slot: &EquipmentSlot,
        difficulty: &crate::entity::mob::equipment::RegionalDifficulty,
    ) {
        self.enchant_spawned_equipment(slot, MobEntity::MAX_ENCHANTED_ARMOR_CHANCE, difficulty);
    }

    fn enchant_spawned_equipment(
        &self,
        slot: &EquipmentSlot,
        chance: f32,
        difficulty: &crate::entity::mob::equipment::RegionalDifficulty,
    ) {
        let living = &self.get_mob_entity().living_entity;
        let mut equipment = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(stack) = equipment.equipment.get_mut(slot)
            && !stack.is_empty()
            && rand::random::<f32>() < chance * difficulty.special_multiplier
        {
            crate::entity::mob::equipment::apply_vanilla_enchantments(
                stack,
                slot,
                difficulty.special_multiplier,
            );
        }
    }

    fn mob_write_nbt(&self, _nbt: &mut NbtCompound) {}

    fn mob_read_nbt(&self, _nbt: &NbtCompound) {}

    /// Set or clear the mob's target. Override to add side effects when targeting changes.
    fn set_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        let mob = self.get_mob_entity();
        let target_id = target.as_ref().map(|t| t.get_entity().entity_id);
        *mob.target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = target;
        let world = mob.living_entity.entity.world.load_full();
        let entity_id = mob.living_entity.entity.entity_id;
        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_target::EntityTargetEvent::new(
                    entity_id, target_id,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.get_mob_entity().mob_interact(player, item_stack)
    }

    fn tame(&self, player: &Arc<Player>) {
        let mob = self.get_mob_entity();
        let mut event = crate::plugin::api::events::entity::entity_tame::EntityTameEvent::new(
            mob.living_entity.entity.entity_id,
            player.clone(),
        );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn breed(&self, father_id: i32, mother_id: i32, child_id: i32) {
        let mob = self.get_mob_entity();
        let mut event = crate::plugin::api::events::entity::entity_breed::EntityBreedEvent::new(
            father_id, mother_id, child_id,
        );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn dye(
        &self,
        color: crate::plugin::api::events::entity::entity_dye::DyeColor,
        player: Option<&Arc<Player>>,
    ) {
        let mob = self.get_mob_entity();
        let mut event = crate::plugin::api::events::entity::entity_dye::EntityDyeEvent::new(
            mob.living_entity.entity.entity_id,
            color,
            player.cloned(),
        );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn enter_love_mode(&self, human_entity_id: Option<i32>, ticks_in_love: i32) {
        let mob = self.get_mob_entity();
        let mut event = crate::plugin::api::events::entity::entity_enter_love_mode::EntityEnterLoveModeEvent::new(
            mob.living_entity.entity.entity_id,
            human_entity_id,
            ticks_in_love,
        );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn transform(&self, new_entity_id: i32, transform_reason: String) {
        let mob = self.get_mob_entity();
        let mut event =
            crate::plugin::api::events::entity::entity_transform::EntityTransformEvent::new(
                mob.living_entity.entity.entity_id,
                new_entity_id,
                transform_reason,
            );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn break_door(&self, block_pos: BlockPos) {
        let mob = self.get_mob_entity();
        let mut event =
            crate::plugin::api::events::entity::entity_break_door::EntityBreakDoorEvent::new(
                mob.living_entity.entity.entity_id,
                block_pos,
            );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn enter_block(&self, block_pos: BlockPos) {
        let mob = self.get_mob_entity();
        let mut event =
            crate::plugin::api::events::entity::entity_enter_block::EntityEnterBlockEvent::new(
                mob.living_entity.entity.entity_id,
                block_pos,
            );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn interact(&self, block_pos: BlockPos) {
        let mob = self.get_mob_entity();
        let mut event =
            crate::plugin::api::events::entity::entity_interact::EntityInteractEvent::new(
                mob.living_entity.entity.entity_id,
                block_pos,
            );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn place_block(&self, block_pos: BlockPos, block_name: String) {
        let mob = self.get_mob_entity();
        let mut event = crate::plugin::api::events::entity::entity_place::EntityPlaceEvent::new(
            mob.living_entity.entity.entity_id,
            block_pos,
            block_name,
        );
        if let Some(server) = mob.living_entity.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    fn mob_player_collision(&self, _player: &Arc<Player>) {}

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.as_tamable()
            .and_then(crate::entity::passive::tamable::TamableAnimal::get_owner)
    }

    fn is_sitting(&self) -> bool {
        self.as_tamable()
            .is_some_and(crate::entity::passive::tamable::TamableAnimal::is_in_sitting_pose)
    }

    fn is_tamed(&self) -> bool {
        self.as_tamable()
            .is_some_and(crate::entity::passive::tamable::TamableAnimal::is_tame)
    }

    fn get_base_experience_reward(&self) -> u32 {
        self.get_entity().entity_type.experience_reward
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(std::sync::atomic::Ordering::Relaxed) < 0;
        if is_baby {
            entity.send_meta_data(
                &[Metadata::new(tracked_data::ageable_mob::DATA_BABY_ID, true)],
                None,
            );
        }
    }

    fn mob_set_variant_name(&self, _name: &str) {}

    fn mob_on_lightning_strike(
        &self,
        caller: &dyn EntityBase,
        lightning: &crate::entity::lightning::LightningBoltEntity,
    ) {
        self.get_mob_entity()
            .living_entity
            .on_lightning_strike(caller, lightning);
    }
}
impl<T: Mob + Send + 'static> EntityBase for T {
    fn get_mob(&self) -> Option<&dyn Mob> {
        Some(self)
    }

    fn on_lightning_strike(
        &self,
        caller: &dyn EntityBase,
        lightning: &crate::entity::lightning::LightningBoltEntity,
    ) {
        self.mob_on_lightning_strike(caller, lightning);
    }

    fn get_item_steerable(&self) -> Option<&dyn crate::entity::item_steerable::ItemSteerable> {
        Mob::get_item_steerable(self)
    }

    fn init_data_tracker(&self) {
        self.mob_init_data_tracker();
        let world = self.get_mob_entity().living_entity.entity.world.load();
        crate::entity::mob::equipment::equip_mob_on_spawn(self as &dyn EntityBase, &world);

        let entity_name = self.get_entity().entity_type.resource_name;
        if let Some(def) = crate::entity::mob::equipment::EQUIPMENT_REGISTRY.get(entity_name)
            && def.can_pick_up_loot
        {
            let difficulty = crate::entity::mob::equipment::RegionalDifficulty::at(
                &world,
                self.get_entity().pos.load(),
            );
            let pickup_chance = 0.55 * difficulty.special_multiplier;
            self.get_mob_entity()
                .set_can_pick_up_loot(rand::random::<f32>() < pickup_chance);
        }
    }

    fn set_variant_name(&self, name: &str) {
        self.mob_set_variant_name(name);
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        let mob_entity = self.get_mob_entity();
        mob_entity.living_entity.entity.tick_leash();
        mob_entity.tick_sun_burn();

        if mob_entity.breeding_cooldown.load(Relaxed) > 0 {
            mob_entity.breeding_cooldown.fetch_sub(1, Relaxed);
        }

        if mob_entity.love_ticks.load(Relaxed) > 0 {
            let ticks = mob_entity.love_ticks.fetch_sub(1, Relaxed);
            if ticks % 10 == 0 {
                let entity = &mob_entity.living_entity.entity;
                let pos = entity.pos.load();
                let world = entity.world.load();
                world.spawn_particle(
                    pos + Vector3::new(0.0, f64::from(entity.height()) + 0.5, 0.0),
                    Vector3::new(0.5, 0.5, 0.5),
                    1.0,
                    1,
                    pumpkin_data::particle::Particle::Heart,
                );
            }
        }

        mob_entity.check_despawn(self);

        self.mob_tick(caller);

        let age = mob_entity.living_entity.entity.age.load(Relaxed);
        let entity_id = mob_entity.living_entity.entity.entity_id;

        // 1. "Take" selectors out of the mutexes
        let mut target_selector = {
            let mut guard = mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *guard)
        };
        let mut goals_selector = {
            let mut guard = mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *guard)
        };

        // 2. Perform AI logic
        if (age + entity_id) % 2 != 0 && age > 1 {
            target_selector.tick_goals(self, false);
            goals_selector.tick_goals(self, false);
        } else {
            target_selector.tick(self);
            goals_selector.tick(self);
        }

        // 3. "Put back" selectors
        {
            *mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = target_selector;
            *mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = goals_selector;
        };

        // 4. Repeat for Navigator
        let mut navigator = {
            let mut guard = mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *guard)
        };

        navigator.tick(&mob_entity.living_entity);

        {
            *mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = navigator;
        };

        // Controllers are synchronous, so we can just use normal blocks
        {
            let mut look_control = mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            look_control.tick(self);
        };

        {
            let mut move_control = mob_entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            move_control.tick(self);
        };

        mob_entity.living_entity.tick(caller, server);
        self.post_tick();

        // --- Packet logic remains the same ---
        let entity = &mob_entity.living_entity.entity;
        let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let head_yaw = (entity.head_yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;

        let last_yaw = mob_entity.last_sent_yaw.load(Relaxed);
        let last_pitch = mob_entity.last_sent_pitch.load(Relaxed);
        let last_head_yaw = mob_entity.last_sent_head_yaw.load(Relaxed);

        let chunk_pos = entity.chunk_pos.load();
        if yaw.abs_diff(last_yaw) >= 1 || pitch.abs_diff(last_pitch) >= 1 {
            let world = entity.world.load();
            world.broadcast_to_chunk(
                chunk_pos,
                &CUpdateEntityRot::new(
                    entity.entity_id.into(),
                    yaw,
                    pitch,
                    entity.on_ground.load(Relaxed),
                ),
            );
            mob_entity.last_sent_yaw.store(yaw, Relaxed);
            mob_entity.last_sent_pitch.store(pitch, Relaxed);
        }

        if head_yaw.abs_diff(last_head_yaw) >= 1 {
            let world = entity.world.load();

            world.broadcast_to_chunk(chunk_pos, &CHeadRot::new(entity.entity_id.into(), head_yaw));
            mob_entity.last_sent_head_yaw.store(head_yaw, Relaxed);
        }
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn can_hit(&self) -> bool {
        true
    }

    fn damage_with_context(
        &self,
        caller: &dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        // pre_damage hook: allows mobs to dodge/cancel damage (e.g. enderman projectile dodge)
        if !self.pre_damage(damage_type, source) {
            return false;
        }
        // Mob-specific damage modifier (e.g. shulker armor when closed).
        let amount = self.modify_incoming_damage(amount, damage_type);
        let damaged = self.get_mob_entity().living_entity.damage_with_context(
            caller,
            amount,
            damage_type,
            position,
            source,
            cause,
        );
        if damaged {
            self.on_damage(damage_type, source);
        }
        damaged
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.mob_interact(player, item_stack)
    }

    fn on_player_collision(&self, player: &Arc<Player>) {
        self.mob_player_collision(player);
    }

    fn get_entity(&self) -> &Entity {
        &self.get_mob_entity().living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.get_mob_entity().living_entity)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_in_love(&self) -> bool {
        self.get_mob_entity().is_in_love()
    }

    fn is_breeding_ready(&self) -> bool {
        self.get_mob_entity().is_breeding_ready()
    }

    fn reset_love(&self) {
        self.get_mob_entity().reset_love_ticks();
    }

    fn set_breeding_cooldown(&self, ticks: i32) {
        self.get_mob_entity()
            .breeding_cooldown
            .store(ticks, Relaxed);
    }

    fn is_panicking(&self) -> bool {
        self.get_path_aware_entity()
            .is_some_and(PathAwareEntity::is_panicking)
    }

    fn get_job_site_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        <T as Mob>::get_job_site(self)
    }

    fn get_home_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        <T as Mob>::get_home(self)
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.get_mob_entity().write_mob_nbt(nbt);
        if let Some(ageable) = self.as_ageable() {
            ageable.write_ageable_nbt(nbt);
        }
        if let Some(animal) = self.as_animal() {
            animal.write_animal_nbt(nbt);
        }
        if let Some(tamable) = self.as_tamable() {
            tamable.write_tamable_nbt(nbt);
        }
        self.mob_write_nbt(nbt);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.get_mob_entity().read_mob_nbt(nbt);
        if let Some(ageable) = self.as_ageable() {
            ageable.read_ageable_nbt(nbt);
        }
        if let Some(animal) = self.as_animal() {
            animal.read_animal_nbt(nbt);
        }
        if let Some(tamable) = self.as_tamable() {
            tamable.read_tamable_nbt(nbt);
        }
        self.mob_read_nbt(nbt);
    }

    fn get_gravity(&self) -> f64 {
        self.get_mob_gravity()
    }

    fn get_y_velocity_drag(&self) -> Option<f64> {
        self.get_mob_y_velocity_drag()
    }

    fn get_experience_reward(&self, _killer: Option<&dyn EntityBase>) -> u32 {
        if self
            .get_entity()
            .age
            .load(std::sync::atomic::Ordering::Relaxed)
            < 0
        {
            return 0;
        }
        // TODO: apply enchantment processing like in vanilla
        Mob::get_base_experience_reward(self)
    }

    fn get_base_experience_reward(&self) -> u32 {
        Mob::get_base_experience_reward(self)
    }
}

#[expect(dead_code)]
const DEFAULT_PATHFINDING_FAVOR: f32 = 0.0;

pub trait PathAwareEntity: Mob + Send + Sync {
    fn get_pathfinding_favor(&self, _block_pos: BlockPos, _world: Arc<World>) -> f32 {
        0.0
    }

    // TODO: missing SpawnReason attribute
    fn can_spawn(&self, world: Arc<World>) -> bool {
        self.get_pathfinding_favor(
            self.get_mob_entity().living_entity.entity.block_pos.load(),
            world,
        ) >= 0.0
    }

    fn is_navigation(&self) -> bool {
        let navigator = self
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !navigator.is_idle()
    }

    // TODO: implement
    fn is_panicking(&self) -> bool {
        false
    }

    fn should_follow_leash(&self) -> bool {
        true
    }

    fn on_short_leash_tick(&self) {
        // TODO: implement
    }

    fn before_leash_tick(&self) {
        // TODO: implement
    }

    fn get_follow_leash_speed(&self) -> f32 {
        1.0
    }
}

pub trait RangedAttackMob: Mob + Send + Sync {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, power: f32);
}
