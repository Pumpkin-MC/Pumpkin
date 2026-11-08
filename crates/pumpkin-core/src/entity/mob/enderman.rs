use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::attributes::Modifier;
use crate::entity::attributes::ModifierOperation;
use pumpkin_data::{BlockStateId, attributes::Attributes};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    enchantment_provider::EnchantmentProvider,
    entity::EntityType,
    game_event::GameEvent,
    item::Item,
    sound::{Sound, SoundCategory},
    tag,
    tag::Taggable,
    world::WorldEvent,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::block::BlockStateCodec;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::{
        goal::{
            chase_player::ChasePlayerGoal, look_around::RandomLookAroundGoal,
            look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
            pick_up_block::PickUpBlockGoal, place_block::PlaceBlockGoal,
            reset_universal_anger::ResetUniversalAngerGoal, revenge::RevengeGoal, swim::SwimGoal,
            teleport_towards_player::TeleportTowardsPlayerGoal, wander_around::WanderAroundGoal,
        },
        pathfinder::node::PathType,
    },
    mob::{
        Mob, MobEntity,
        neutral::{NeutralData, NeutralMob},
    },
    player::Player,
    projectile::splash_potion::SplashPotionEntity,
};
use crate::world::World;
use crate::world::game_event::GameEventContext;

const SPEED_BOOST: f64 = 0.15;
const ENDERMAN_SPEED_BOOST_ID: &str = "minecraft:attacking";

pub const MAX_TELEPORT_RADIUS: i32 = 32;
pub const MAX_TELEPORT_DIAMETER: i32 = 64;
const MIN_DEAGGRESSION_TIME: i32 = 600;
const TELEPORT_ATTEMPTS: usize = 64;
/// Largest per-axis offset level event 2018 can carry.
const MAX_PACKED_OFFSET: i32 = 127;
const STARE_CONE_SIZE: f64 = 0.025;
const LINE_OF_SIGHT_RANGE: f64 = 128.0;

fn is_projectile_damage(dt: DamageType) -> bool {
    let (names, _) = pumpkin_data::tag::DamageType::MINECRAFT_IS_PROJECTILE;
    names.contains(&dt.message_id)
}

const ENDERMAN_HUNTS: &[Hunt] = &[Hunt::new(
    3,
    TargetMatch::Type(&EntityType::ENDERMITE),
    true,
)];

pub struct EndermanEntity {
    pub mob_entity: MobEntity,
    neutral_data: NeutralData,
    carried_block: AtomicCell<Option<BlockStateId>>,
    /// Synced CREEPY flag, the open-mouth pose. Anger lives in `NeutralMob`.
    creepy: AtomicBool,
    provoked: AtomicBool,
    speed_boosted: AtomicBool,
    target_change_time: AtomicI32,
}

impl EndermanEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let entity = Self {
            mob_entity,
            neutral_data: NeutralData::default(),
            carried_block: AtomicCell::new(None),
            creepy: AtomicBool::new(false),
            provoked: AtomicBool::new(false),
            speed_boosted: AtomicBool::new(false),
            target_change_time: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(entity);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        let mut navigator = mob_arc
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_pathfinding_malus(PathType::Water, -1.0);
        drop(navigator);

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(ChasePlayerGoal::new(mob_arc.clone())));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(
                7,
                Box::new(WanderAroundGoal::water_avoiding_with_probability(1.0, 0.0)),
            );
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(10, Box::new(PlaceBlockGoal::new(mob_arc.clone())));
            goal_selector.add_goal(11, Box::new(PickUpBlockGoal::new(mob_arc.clone())));

            target_selector.add_goal(1, Box::new(TeleportTowardsPlayerGoal::new(mob_arc.clone())));
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
            hunting::apply(target_selector, &mob_arc.mob_entity, ENDERMAN_HUNTS);
            target_selector.add_goal(4, ResetUniversalAngerGoal::new(false));
        };

        mob_arc
    }

    /// Picks a random spot within [`MAX_TELEPORT_RADIUS`] blocks and tries to teleport there.
    pub fn teleport_randomly(&self) -> bool {
        if !self.mob_entity.living_entity.is_alive() {
            return false;
        }
        let pos = self.mob_entity.living_entity.entity.pos.load();
        let diameter = f64::from(MAX_TELEPORT_DIAMETER);
        let (x, y, z) = {
            let mut rng = self.get_random();
            (
                pos.x + (rng.random::<f64>() - 0.5) * diameter,
                pos.y + f64::from(rng.random_range(0..MAX_TELEPORT_DIAMETER) - MAX_TELEPORT_RADIUS),
                pos.z + (rng.random::<f64>() - 0.5) * diameter,
            )
        };

        self.teleport_to(x, y, z)
    }

    pub fn teleport_towards(&self, target: &dyn EntityBase) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();

        let dx = pos.x - target_pos.x;
        let dy = entity.get_y(0.5) - target_entity.get_eye_y();
        let dz = pos.z - target_pos.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        let (nx, ny, nz) = if dist < f64::from(1.0E-5f32) {
            (0.0, 0.0, 0.0)
        } else {
            (dx / dist, dy / dist, dz / dist)
        };
        let (x, y, z) = {
            let mut rng = self.get_random();
            (
                pos.x + (rng.random::<f64>() - 0.5) * 8.0 - nx * 16.0,
                pos.y + f64::from(rng.random_range(0..16) - 8) - ny * 16.0,
                pos.z + (rng.random::<f64>() - 0.5) * 8.0 - nz * 16.0,
            )
        };

        self.teleport_to(x, y, z)
    }

    pub fn teleport_to(&self, x: f64, y: f64, z: f64) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        if entity.has_vehicle() {
            return false;
        }
        let origin = entity.pos.load();
        let origin_block_pos = entity.block_pos.load();
        let world = entity.world.load();

        if !self.mob_entity.living_entity.random_teleport(
            self,
            Vector3::new(x, y, z),
            true,
            |block| block.has_tag(&tag::Block::MINECRAFT_ENDERMAN_DOES_NOT_TELEPORT_TO),
        ) {
            return false;
        }
        let new_pos = entity.pos.load();
        self.play_teleport_effects(&world, origin, origin_block_pos, new_pos);
        true
    }

    fn play_teleport_effects(
        &self,
        world: &World,
        origin: Vector3<f64>,
        origin_block_pos: BlockPos,
        new_pos: Vector3<f64>,
    ) {
        let entity = &self.mob_entity.living_entity.entity;
        world.emit_game_event_with_context(
            GameEvent::Teleport.name(),
            origin,
            &GameEventContext {
                source_entity: Some(self),
            },
        );
        if !entity.is_silent() {
            let destination_block_pos = entity.block_pos.load();
            let packed_difference = |from: i32, to: i32| {
                ((to - from).clamp(-MAX_PACKED_OFFSET, MAX_PACKED_OFFSET) + MAX_PACKED_OFFSET)
                    & 0xFF
            };
            world.sync_world_event(
                WorldEvent::ParticlesEndermanTeleport,
                origin_block_pos,
                packed_difference(origin_block_pos.0.x, destination_block_pos.0.x) << 16
                    | packed_difference(origin_block_pos.0.y, destination_block_pos.0.y) << 8
                    | packed_difference(origin_block_pos.0.z, destination_block_pos.0.z),
            );
            for pos in &[origin, new_pos] {
                world.play_sound(Sound::EntityEndermanTeleport, SoundCategory::Hostile, pos);
            }
        }
    }

    fn repeatedly_try_to_teleport(&self) {
        if !self.mob_entity.living_entity.entity.has_vehicle() {
            for _ in 0..TELEPORT_ATTEMPTS {
                if self.teleport_randomly() {
                    return;
                }
            }
        }
    }

    pub fn set_target(&self, target: Option<Arc<dyn EntityBase>>) {
        let is_some = target.is_some();
        let mut mob_target = self
            .mob_entity
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let target_id = target.as_ref().map(|target| target.get_entity().entity_id);
        let current_target_id = mob_target
            .as_ref()
            .map(|target| target.get_entity().entity_id);
        // `set_mob_target` refreshes an unchanged target, which must not restart the timer.
        if current_target_id != target_id {
            let target_change_time = if is_some {
                self.mob_entity
                    .living_entity
                    .entity
                    .tick_count
                    .load(Ordering::Relaxed)
            } else {
                0
            };
            self.target_change_time
                .store(target_change_time, Ordering::Relaxed);
        }
        *mob_target = target;
        drop(mob_target);

        if is_some {
            self.set_creepy(true);
            // Use attribute modifier instead of direct speed arithmetic
            if !self.speed_boosted.swap(true, Ordering::Relaxed) {
                let living = &self.mob_entity.living_entity;
                let modifier = Modifier {
                    id: ENDERMAN_SPEED_BOOST_ID.to_string(),
                    amount: SPEED_BOOST,
                    operation: ModifierOperation::Add,
                };

                living.update_attribute(&Attributes::MOVEMENT_SPEED, |inst| {
                    inst.add_or_replace_modifier(modifier);
                });

                crate::entity::attributes::send_attribute_updates_for_living(
                    living,
                    vec![Attributes::MOVEMENT_SPEED],
                );
            }
        } else {
            self.set_creepy(false);
            self.set_provoked(false);
            if self.speed_boosted.swap(false, Ordering::Relaxed) {
                let living = &self.mob_entity.living_entity;

                living.update_attribute(&Attributes::MOVEMENT_SPEED, |inst| {
                    inst.remove_modifier(ENDERMAN_SPEED_BOOST_ID);
                });

                crate::entity::attributes::send_attribute_updates_for_living(
                    living,
                    vec![Attributes::MOVEMENT_SPEED],
                );
            }
        }
    }

    pub fn set_creepy(&self, creepy: bool) {
        self.creepy.store(creepy, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::enderman::CREEPY, creepy);
    }

    pub fn is_creepy(&self) -> bool {
        self.creepy.load(Ordering::Relaxed)
    }

    pub fn set_provoked(&self, provoked: bool) {
        self.provoked.store(provoked, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::enderman::STARED_AT, provoked);
    }

    pub fn set_carried_block(&self, block_state: Option<BlockStateId>) {
        self.carried_block.store(block_state);
        let value = block_state.map_or(VarInt(0), |id| VarInt(id.as_u16() as i32));
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::enderman::CARRY_STATE, value);
    }

    pub fn get_carried_block(&self) -> Option<BlockStateId> {
        self.carried_block.load()
    }

    pub fn is_player_staring(&self, player: &Player) -> bool {
        let equipment = player.living_entity.entity_equipment.try_lock();
        if let Ok(equipment) = equipment
            && let Some(head_stack) = equipment.equipment.get(&EquipmentSlot::HEAD)
            && !head_stack.is_empty()
            && head_stack
                .item
                .has_tag(&tag::Item::MINECRAFT_GAZE_DISGUISE_EQUIPMENT)
        {
            return false;
        }

        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let gaze_y = entity.get_eye_y();
        let player_entity = player.get_entity();
        let player_pos = player_entity.pos.load();
        let player_eye_y = player_entity.get_eye_y();

        let rotation = player_entity.rotation();
        let look = Vector3::new(
            f64::from(rotation.x),
            f64::from(rotation.y),
            f64::from(rotation.z),
        )
        .normalize();
        let dir = Vector3::new(
            pos.x - player_pos.x,
            gaze_y - player_eye_y,
            pos.z - player_pos.z,
        );
        let dist = dir.length();
        let in_stare_cone = look.dot(&dir.normalize()) > 1.0 - STARE_CONE_SIZE / dist;
        if !in_stare_cone {
            return false;
        }

        let from = Vector3::new(player_pos.x, player_eye_y, player_pos.z);
        let to = Vector3::new(pos.x, gaze_y, pos.z);
        if from.squared_distance_to_vec(&to) > LINE_OF_SIGHT_RANGE * LINE_OF_SIGHT_RANGE {
            return false;
        }
        entity
            .world
            .load()
            .clip(&pumpkin_world::clip::ClipContext::new(
                from,
                to,
                pumpkin_world::clip::ClipBlock::Collider,
                pumpkin_world::clip::ClipFluid::None,
            ))
            .is_miss
    }
}

impl NeutralMob for EndermanEntity {
    fn get_neutral_data(&self) -> &NeutralData {
        &self.neutral_data
    }
}

impl Mob for EndermanEntity {
    fn is_sensitive_to_water(&self) -> bool {
        true
    }

    fn as_neutral(&self) -> Option<&dyn NeutralMob> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(block_state) = self.carried_block.load() {
            nbt.put_compound(
                "carriedBlockState",
                BlockStateCodec::state_to_nbt(block_state),
            );
        }
        self.write_anger_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        let carried = nbt
            .get_compound("carriedBlockState")
            .and_then(BlockStateCodec::state_from_nbt)
            // Older Pumpkin saves stored the raw state id.
            .or_else(|| {
                nbt.get_int("carriedBlockState")
                    .and_then(|id| u16::try_from(id).ok())
                    .and_then(BlockStateId::new)
            });
        self.set_carried_block(carried.filter(|id| !pumpkin_data::block_properties::is_air(*id)));
        self.read_anger_nbt(nbt);
    }

    fn mob_remove_nbt(&self, removed: &[Box<str>]) {
        if removed
            .iter()
            .any(|key| key.as_ref() == "carriedBlockState")
        {
            self.set_carried_block(None);
        }
    }

    fn requires_custom_persistence(&self) -> bool {
        self.get_carried_block().is_some()
    }

    fn on_die(&self, _source: Option<&dyn EntityBase>) {
        let Some(block_state_id) = self.get_carried_block() else {
            return;
        };
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        if !world.level_info.load().game_rules.mob_drops {
            return;
        }

        let (block, state) = pumpkin_data::BlockState::from_id_with_block(block_state_id);
        let Some(loot_table) = block.loot_table.and_then(|key| world.get_loot_table(key)) else {
            return;
        };
        let mut tool = pumpkin_data::item_stack::ItemStack::new(1, &Item::DIAMOND_AXE);
        EnchantmentProvider::ENDERMAN_LOOT_DROP.apply_to_stack(&mut tool, 0.0);
        let params = crate::world::loot::LootContextParameters {
            world: Some(&world),
            block_state: Some(state),
            this_entity: Some(self),
            position: Some(entity.pos.load()),
            world_time: world.level_info.load().day_time as u64,
            tool: Some(tool),
            ..Default::default()
        };
        for stack in crate::world::loot::generate_loot_from_handle(
            &loot_table,
            self.get_random().random(),
            &params,
        ) {
            entity.spawn_at_location(stack);
        }
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.is_creepy() {
            Sound::EntityEndermanScream
        } else {
            Sound::EntityEndermanAmbient
        })
    }

    fn mob_ai_step(&self) {
        self.mob_entity
            .living_entity
            .jumping
            .store(false, Ordering::Relaxed);
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        if !world.is_bright_outside()
            || entity.tick_count.load(Ordering::Relaxed)
                < self.target_change_time.load(Ordering::Relaxed) + MIN_DEAGGRESSION_TIME
        {
            return;
        }
        let brightness =
            world.get_light_level_dependent_magic_value(&entity.get_eye_pos().to_block_pos());
        if brightness > 0.5
            && world.can_see_sky(&entity.block_pos.load())
            && self.get_random().random::<f32>() * 30.0 < (brightness - 0.4) * 2.0
        {
            self.set_target(None);
            self.teleport_randomly();
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn can_randomly_teleport_to(&self, x: f64, y: f64, z: f64) -> bool {
        let below = BlockPos::floored(x, y, z).down();
        !self
            .get_entity()
            .world
            .load()
            .get_fluid(&below)
            .has_tag(&tag::Fluid::MINECRAFT_WATER)
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        self.is_creepy()
            .then(|| crate::entity::mob::MobMetadata::new().screaming(true))
    }

    fn apply_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        self.set_target(target);
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load();
        let raining_at_feet = world.is_raining_at(&entity.block_pos.load());
        let raining_at_head = world.is_raining_at(&entity.bounding_box.load().max_block_pos());
        if entity.touching_water.load(Ordering::SeqCst) || raining_at_feet || raining_at_head {
            caller.damage(caller, 1.0, DamageType::DROWN);
        }
    }

    fn pre_damage_with_context(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        if entity.is_invulnerable_to(&damage_type, cause) {
            return false;
        }
        if is_projectile_damage(damage_type) && !entity.has_vehicle() {
            self.repeatedly_try_to_teleport();
            return false;
        }
        if let Some(potion) = source.and_then(as_splash_potion)
            && !potion.hurts_water_sensitive_entities()
        {
            self.repeatedly_try_to_teleport();
            return false;
        }
        true
    }

    /// Runs after the damage attempt even when it was blocked.
    fn post_damage_with_context(
        &self,
        _damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        if source.and_then(as_splash_potion).is_some() {
            self.repeatedly_try_to_teleport();
            return;
        }
        if cause
            .or(source)
            .is_some_and(|entity| entity.get_living_entity().is_some())
        {
            return;
        }
        if self.get_random().random_range(0..10) != 0 {
            self.teleport_randomly();
        }
    }
}

fn as_splash_potion(entity: &dyn EntityBase) -> Option<&SplashPotionEntity> {
    entity.cast_any().downcast_ref::<SplashPotionEntity>()
}
