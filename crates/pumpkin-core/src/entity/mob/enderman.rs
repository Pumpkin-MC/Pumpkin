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
    item::Item,
    sound::{Sound, SoundCategory},
    tag,
    tag::Taggable,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
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
};

const SPEED_BOOST: f64 = 0.15;
const ENDERMAN_SPEED_BOOST_ID: &str = "minecraft:attacking";

pub const ENDERMAN_EYE_HEIGHT: f64 = 2.55;
pub const ENDERMAN_BODY_Y_OFFSET: f64 = 1.45;
pub const PLAYER_EYE_HEIGHT: f64 = 1.62;

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
    daylight_deaggression: AtomicBool,
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
            daylight_deaggression: AtomicBool::new(false),
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
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
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

    pub fn teleport_randomly(&self) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let (x, y, z) = {
            let mut rng = self.get_random();
            (
                pos.x + (rng.random_range(0.0..1.0) - 0.5) * 64.0,
                pos.y + (rng.random_range(0i32..64) - 32) as f64,
                pos.z + (rng.random_range(0.0..1.0) - 0.5) * 64.0,
            )
        };

        self.teleport_to(x, y, z)
    }

    pub fn teleport_towards(&self, target: &dyn EntityBase) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();

        let dx = pos.x - target_pos.x;
        let dy = (pos.y + ENDERMAN_BODY_Y_OFFSET)
            - (target_pos.y + target.get_entity().get_eye_height());
        let dz = pos.z - target_pos.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < 1e-6 {
            return false;
        }

        let nx = dx / dist;
        let ny = dy / dist;
        let nz = dz / dist;
        let (x, y, z) = {
            let mut rng = self.get_random();
            (
                pos.x + (rng.random_range(0.0..1.0) - 0.5) * 8.0 - nx * 16.0,
                pos.y + (rng.random_range(0i32..16) - 8) as f64 - ny * 16.0,
                pos.z + (rng.random_range(0.0..1.0) - 0.5) * 8.0 - nz * 16.0,
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

        let min_y = f64::from(world.dimension.min_y);
        let max_y = f64::from(world.dimension.min_y + world.dimension.height - 1);
        let mut target_y = y.clamp(min_y, max_y);

        let block_x = x.floor() as i32;
        let mut block_y = target_y.floor() as i32;
        let block_z = z.floor() as i32;
        let ground_pos = loop {
            let below_pos = BlockPos::new(block_x, block_y - 1, block_z);
            let below_state = world.get_block_state(&below_pos);
            if below_state.is_solid() {
                break Some(below_pos);
            }
            if block_y <= world.dimension.min_y {
                break None;
            }
            block_y -= 1;
            target_y = block_y as f64;
        };
        let Some(ground_pos) = ground_pos else {
            return false;
        };

        let ground_block =
            pumpkin_data::Block::from_state_id(world.get_block_state_id(&ground_pos));
        if ground_block.has_tag(&tag::Block::MINECRAFT_ENDERMAN_DOES_NOT_TELEPORT_TO) {
            return false;
        }
        if world
            .get_fluid(&ground_pos)
            .has_tag(&tag::Fluid::MINECRAFT_WATER)
        {
            return false;
        }

        let bb = BoundingBox::new_from_pos(x, target_y, z, &entity.entity_dimension.load());
        if BlockPos::iterate(bb.min_block_pos(), bb.max_block_pos()).any(|pos| {
            pumpkin_data::Block::from_state_id(world.get_block_state_id(&pos))
                .has_tag(&tag::Block::MINECRAFT_ENDERMAN_DOES_NOT_TELEPORT_TO)
        }) {
            return false;
        }
        if !world.is_space_empty(bb) || world.contains_any_liquid(bb) {
            return false;
        }

        let new_pos = Vector3::new(x, target_y, z);

        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_teleport::EntityTeleportEvent::new(
                    entity.entity_id,
                    origin,
                    new_pos,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return false;
            }
        }

        entity.teleport(new_pos, None, None, &world);
        world.emit_game_event(pumpkin_data::game_event::GameEvent::Teleport.name(), origin);
        for pos in &[origin, new_pos] {
            world.play_sound(Sound::EntityEndermanTeleport, SoundCategory::Hostile, pos);
        }
        let destination_block_pos = entity.block_pos.load();
        let difference = BlockPos::new(
            (destination_block_pos.0.x - origin_block_pos.0.x).clamp(-127, 127),
            (destination_block_pos.0.y - origin_block_pos.0.y).clamp(-127, 127),
            (destination_block_pos.0.z - origin_block_pos.0.z).clamp(-127, 127),
        );
        world.sync_world_event(
            pumpkin_data::world::WorldEvent::ParticlesEndermanTeleport,
            origin_block_pos,
            (((difference.0.x + 127) & 0xff) << 16)
                | (((difference.0.y + 127) & 0xff) << 8)
                | ((difference.0.z + 127) & 0xff),
        );

        self.mob_entity.steered().navigator().stop();

        true
    }

    pub fn set_target(&self, target: Option<Arc<dyn EntityBase>>) {
        let is_some = target.is_some();
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
        let mut mob_target = self
            .mob_entity
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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

    pub(crate) fn take_daylight_deaggression(&self) -> bool {
        self.daylight_deaggression.swap(false, Ordering::Relaxed)
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
        let enderman_pos = entity.pos.load();
        let enderman_eye_y = enderman_pos.y + ENDERMAN_EYE_HEIGHT;

        let player_entity = player.get_entity();
        let player_pos = player_entity.pos.load();
        let player_eye_y = player_pos.y + PLAYER_EYE_HEIGHT;

        let pitch = player_entity.pitch.load().to_radians();
        let yaw = -player_entity.yaw.load().to_radians();

        let cos_pitch = pitch.cos();
        let look_dir = Vector3::new(
            (yaw.sin() * cos_pitch) as f64,
            (-pitch.sin()) as f64,
            (yaw.cos() * cos_pitch) as f64,
        );

        let dx = enderman_pos.x - player_pos.x;
        let dy = enderman_eye_y - player_eye_y;
        let dz = enderman_pos.z - player_pos.z;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        if distance < 0.1 {
            return false;
        }

        let dir_x = dx / distance;
        let dir_y = dy / distance;
        let dir_z = dz / distance;

        let dot = look_dir.x * dir_x + look_dir.y * dir_y + look_dir.z * dir_z;

        if dot <= 1.0 - 0.025 / distance {
            return false;
        }

        let enderman_eye_pos = Vector3::new(enderman_pos.x, enderman_eye_y, enderman_pos.z);
        let player_eye_pos = Vector3::new(player_pos.x, player_eye_y, player_pos.z);
        let world = entity.world.load();
        world
            .clip(&pumpkin_world::clip::ClipContext::new(
                enderman_eye_pos,
                player_eye_pos,
                pumpkin_world::clip::ClipBlock::Visual,
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
            nbt.put_int("carriedBlockState", block_state.as_u16() as i32);
        }
        self.write_anger_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(value) = nbt.get_int("carriedBlockState") {
            self.set_carried_block(
                BlockStateId::new(value as u16).filter(|id| *id != BlockStateId::AIR),
            );
        }
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

    fn on_die(&self, source: Option<&dyn EntityBase>) {
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
            killer_entity: source,
            direct_killer_entity: source,
            position: Some(entity.pos.load()),
            world_time: world.level_info.load().day_time as u64,
            tool: Some(tool),
            killed_by_player: Some(source.is_some_and(|source| source.get_player().is_some())),
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
        let ticks_since_target_change = entity
            .tick_count
            .load(Ordering::Relaxed)
            .wrapping_sub(self.target_change_time.load(Ordering::Relaxed))
            as u32;
        if ticks_since_target_change < 600 {
            return;
        }

        let world = entity.world.load();
        if !world.is_bright_outside() {
            return;
        }
        let pos = entity.block_pos.load();
        let brightness = world.get_light_level_dependent_magic_value(&pos);
        if brightness <= 0.5 || !world.can_see_sky(&pos) {
            return;
        }

        if self.get_random().random::<f32>() * 30.0 < (brightness - 0.4) * 2.0 {
            self.daylight_deaggression.store(true, Ordering::Relaxed);
            self.set_target(None);
            self.teleport_randomly();
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
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

    fn pre_damage(&self, damage_type: DamageType, _source: Option<&dyn EntityBase>) -> bool {
        let is_projectile = is_projectile_damage(damage_type);
        if is_projectile
            && !self.mob_entity.living_entity.entity.has_vehicle()
            && self.repeatedly_try_to_teleport()
        {
            return false;
        }
        true
    }

    fn pre_damage_with_context(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self
            .mob_entity
            .living_entity
            .entity
            .is_invulnerable_to(&damage_type, cause)
        {
            return false;
        }
        if let Some(potion) = source.and_then(|source| {
            source
                .cast_any()
                .downcast_ref::<crate::entity::projectile::splash_potion::SplashPotionEntity>()
        }) {
            return potion.hurts_water_sensitive_entities();
        }
        self.pre_damage(damage_type, source)
    }

    fn post_damage_with_context(
        &self,
        _damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        if let Some(potion) = source.and_then(|source| {
            source
                .cast_any()
                .downcast_ref::<crate::entity::projectile::splash_potion::SplashPotionEntity>()
        }) {
            if potion.hurts_water_sensitive_entities() && self.mob_entity.living_entity.is_alive() {
                self.repeatedly_try_to_teleport();
            }
            return;
        }
        if cause
            .or(source)
            .is_some_and(|entity| entity.get_living_entity().is_some())
        {
            return;
        }
        let should_teleport = self.get_random().random_range(0..10) != 0;
        if should_teleport {
            self.teleport_randomly();
        }
    }
}

impl EndermanEntity {
    fn repeatedly_try_to_teleport(&self) -> bool {
        !self.mob_entity.living_entity.entity.has_vehicle()
            && (0..64).any(|_| self.teleport_randomly())
    }
}
