use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering::Relaxed};
use std::{collections::HashMap, sync::atomic::AtomicI32};

use super::EntityBase;
use super::{Entity, EntityId, NBTStorage, effect::Effect};
use super::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::advanced_config;
use pumpkin_data::entity::{EffectType, EntityStatus};
use pumpkin_data::{block_properties::{
    BlockProperties
    LadderLikeProperties,
    OakTrapdoorLikeProperties,
}, damage::DamageType, sound::Sound, entity::EntityType};
use pumpkin_inventory::entity_equipment::EntityEquipment;
use pumpkin_inventory::equipment_slot::EquipmentSlot;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::client::play::{CHurtAnimation, CTakeItemEntity};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::{
    client::play::{CDamageEvent, CSetEquipment, MetaDataType, Metadata},
    codec::item_stack_seralizer::ItemStackSerializer,
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

const GRAVITY: f64 = 0.08;

/// Represents a living entity within the game world.
///
/// This struct encapsulates the core properties and behaviors of living entities, including players, mobs, and other creatures.
pub struct LivingEntity {
    /// The underlying entity object, providing basic entity information and functionality.
    pub entity: Entity,
    /// Tracks the remaining time until the entity can regenerate health.
    pub time_until_regen: AtomicI32,
    /// Stores the amount of damage the entity last received.
    pub last_damage_taken: AtomicCell<f32>,
    /// The current health level of the entity.
    pub health: AtomicCell<f32>,
    pub death_time: AtomicU8,
    /// The distance the entity has been falling.
    pub fall_distance: AtomicCell<f32>,
    pub active_effects: Mutex<HashMap<EffectType, Effect>>,
    pub entity_equipment: Arc<Mutex<EntityEquipment>>,
    pub movement_input: AtomicCell<Vector3<f64>>,
    pub movement_speed: AtomicCell<f64>,
    pub jumping: AtomicBool,
    pub jumping_cooldown: AtomicU8,
    pub climbing: AtomicBool,
    /// The position where the entity was last climbing, used for death messages
    pub climbing_pos: AtomicCell<Option<BlockPos>>,
    water_movement_speed_multiplier: f32,
}
impl LivingEntity {
    pub fn new(entity: Entity) -> Self {
        let water_movement_speed_multiplier = if entity.entity_type == EntityType::POLAR_BEAR {
            0.98
        } else if entity.entity_type == EntityType::SKELETON_HORSE {
            0.96
        } else {
            0.8
        };
        Self {
            entity,
            time_until_regen: AtomicI32::new(0),
            last_damage_taken: AtomicCell::new(0.0),
            health: AtomicCell::new(20.0),
            fall_distance: AtomicCell::new(0.0),
            death_time: AtomicU8::new(0),
            active_effects: Mutex::new(HashMap::new()),
            entity_equipment: Arc::new(Mutex::new(EntityEquipment::new())),
            jumping: AtomicBool::new(false),
            jumping_cooldown: AtomicU8::new(0),
            climbing: AtomicBool::new(false),
            climbing_pos: AtomicCell::new(None),
            water_movement_speed_multiplier,
        }
    }

    pub async fn send_equipment_changes(&self, equipment: &[(EquipmentSlot, ItemStack)]) {
        let equipment: Vec<(i8, ItemStackSerializer)> = equipment
            .iter()
            .map(|(slot, stack)| (slot.discriminant(), ItemStackSerializer::from(*stack)))
            .collect();
        self.entity
            .world
            .read()
            .await
            .broadcast_packet_except(
                &[self.entity.entity_uuid],
                &CSetEquipment::new(self.entity_id().into(), equipment),
            )
            .await;
    }

    /// Picks up and Item entity or XP Orb
    pub async fn pickup(&self, item: &Entity, stack_amount: u32) {
        // TODO: Only nearby
        self.entity
            .world
            .read()
            .await
            .broadcast_packet_all(&CTakeItemEntity::new(
                item.entity_id.into(),
                self.entity.entity_id.into(),
                stack_amount.try_into().unwrap(),
            ))
            .await;
    }

    pub async fn heal(&self, additional_health: f32) {
        assert!(additional_health > 0.0);
        self.set_health(self.health.load() + additional_health)
            .await;
    }

    pub async fn set_health(&self, health: f32) {
        self.health.store(health);
        // tell everyone entities health changed
        self.entity
            .send_meta_data(&[Metadata::new(9, MetaDataType::Float, health)])
            .await;
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity.entity_id
    }

    pub async fn damage_with_context(
        &self,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&Entity>,
        cause: Option<&Entity>,
    ) -> bool {
        // Check invulnerability before applying damage
        if self.entity.is_invulnerable_to(&damage_type) {
            return false;
        }

        self.entity
            .world
            .read()
            .await
            .broadcast_packet_all(&CDamageEvent::new(
                self.entity.entity_id.into(),
                damage_type.id.into(),
                source.map(|e| e.entity_id.into()),
                cause.map(|e| e.entity_id.into()),
                position,
            ))
            .await;

        let new_health = (self.health.load() - amount).max(0.0);

        if new_health == 0.0 {
            self.kill().await;
        } else {
            self.set_health(new_health).await;
        }

        true
    }

    pub async fn add_effect(&self, effect: Effect) {
        let mut effects = self.active_effects.lock().await;
        effects.insert(effect.r#type, effect);
        // TODO broadcast metadata
    }

    pub async fn remove_effect(&self, effect_type: EffectType) {
        let mut effects = self.active_effects.lock().await;
        effects.remove(&effect_type);
        self.entity
            .world
            .read()
            .await
            .send_remove_mob_effect(&self.entity, effect_type)
            .await;
    }

    pub async fn has_effect(&self, effect: EffectType) -> bool {
        let effects = self.active_effects.lock().await;
        effects.contains_key(&effect)
    }

    pub async fn get_effect(&self, effect: EffectType) -> Option<Effect> {
        let effects = self.active_effects.lock().await;
        effects.get(&effect).cloned()
    }

    /// Returns if the entity was damaged or not
    pub fn check_damage(&self, amount: f32) -> bool {
        let regen = self.time_until_regen.load(Relaxed);

        let last_damage = self.last_damage_taken.load();
        // TODO: check if bypasses iframe
        if regen > 10 {
            if amount <= last_damage {
                return false;
            }
        } else {
            self.time_until_regen.store(20, Relaxed);
        }

        self.last_damage_taken.store(amount);
        amount > 0.0
    }

    // Check if the entity is in water
    /*
    pub async fn is_in_water(&self) -> bool {
        let world = self.entity.world.read().await;
        let block_pos = self.entity.block_pos.load();
        world.get_block(&block_pos).await == Block::WATER
    }
    */

    pub async fn update_fall_distance(
        &self,
        height_difference: f64,
        ground: bool,
        dont_damage: bool,
    ) {
        if ground {
            let fall_distance = self.fall_distance.swap(0.0);
            if fall_distance <= 0.0 || dont_damage {
                return;
            }

            let safe_fall_distance = 3.0;
            let mut damage = fall_distance - safe_fall_distance;
            damage = (damage).ceil();

            // TODO: Play block fall sound
            let check_damage = self.damage(damage, DamageType::FALL).await; // Fall
            if check_damage {
                self.entity
                    .play_sound(Self::get_fall_sound(fall_distance as i32))
                    .await;
            }
        } else if height_difference < 0.0 {
            let distance = self.fall_distance.load();
            self.fall_distance
                .store(distance - (height_difference as f32));
        }
    }

    fn get_fall_sound(distance: i32) -> Sound {
        if distance > 4 {
            Sound::EntityGenericBigFall
        } else {
            Sound::EntityGenericSmallFall
        }
    }

    /// Kills the Entity
    ///
    /// This is similar to `kill` but Spawn Particles, Animation and plays death sound
    pub async fn kill(&self) {
        self.set_health(0.0).await;

        // Plays the death sound
        self.entity
            .world
            .read()
            .await
            .send_entity_status(
                &self.entity,
                EntityStatus::PlayDeathSoundOrAddProjectileHitParticles,
            )
            .await;
    }

    async fn tick_effects(&self) {
        let mut effects_to_remove = Vec::new();

        {
            let mut effects = self.active_effects.lock().await;
            for effect in effects.values_mut() {
                if effect.duration == 0 {
                    effects_to_remove.push(effect.r#type);
                }
                effect.duration -= 1;
            }
        }

        for effect_type in effects_to_remove {
            self.remove_effect(effect_type).await;
        }
    }

    pub async fn base_tick(&self) {
        self.tick_effects().await;
        if self.time_until_regen.load(Relaxed) > 0 {
            self.time_until_regen.fetch_sub(1, Relaxed);
        }
        if self.health.load() <= 0.0 {
            let time = self.death_time.fetch_add(1, Relaxed);
            if time == 20 {
                // Spawn Death particles
                self.entity
                    .world
                    .read()
                    .await
                    .send_entity_status(&self.entity, EntityStatus::AddDeathParticles)
                    .await;
                self.entity.remove().await;
            }
        }
    }

    async fn get_effective_gravity(&self) -> f64 {
        // TODO: If hasNoGravity() { 0.0 }
        if self.entity.velocity.load().y <= 0.0 && self.has_effect(EffectType::SlowFalling).await {
            0.01
        } else {
            GRAVITY
        }
    }

    async fn tick_movement(&self, server: &Server, caller: Arc<dyn EntityBase>, player: Option<Arc<Player>>) {
        if self.jumping_cooldown.load(Relaxed) != 0 {
            self.jumping_cooldown.fetch_sub(1, Relaxed);
        }
        let movement_speed = self.movement_speed.load();
        let off_ground_speed = if let Some(p) = player {
            player.get_off_ground_speed()
        } else {
            // TODO: If the passenger is a player, ogs = movement_speed * 0.1
            0.02
        };
        let should_swim_in_fluids = if let Some(p) = player {
            if p.abilities.flying {
                false
            } else {
                true
            }
        } else {
            true
        };
        let can_walk_on_fluid = if self.entity.entity_type == EntityType::STRIDER {
            true
        } else {
            false
        };

        self.entity.check_zero_velo();
        let mut movement_input = self.movement_input.load();
        movement_input.x *= 0.98;
        movement_input.z *= 0.98;
        self.movement_input.store(movement_input);
        // TODO: Tick AI

        if self.jumping.load(Relaxed) && should_swim_in_fluids {
            let in_lava = self.entity.touching_lava.load(Relaxed);
            let in_water = self.entity.touching_water.load(Relaxed);
            let fluid_height = if in_lava {
                self.entity.lava_height.load()
            } else {
                self.entity.water_height.load()
            };
            let swim_height = self.get_swim_height();
            let on_ground = self.entity.on_ground.load(Relaxed);
            if (in_water || in_lava) && (!on_ground || fluid_height > swim_height) {
                self.swim_upward();
            } else if (on_ground || in_water && fluid_height <= swim_height) && self.jumping_cooldown.load(Relaxed) == 0 {
                self.jump();
                self.jumping_cooldown.store(10, Relaxed);
            }
        } else {
            self.jumping_cooldown.store(0, Relaxed);
        }

        if self.has_effect(EffectType::SlowFalling).await || self.has_effect(EffectType::Levitation).await {
            self.fall_distance.store(0.0);
        }

        let touching_water = self.entity.touching_water.load(Relaxed);
        if (touching_water || self.entity.touching_lava.load(Relaxed)) && should_swim_in_fluids && !can_walk_on_fluid {
            self.travel_in_fluid(touching_water, movement_speed);
        } else {
            // TODO: Gliding
            self.travel_in_air(
                movement_speed,
                off_ground_speed
            );
        }
        self.entity.tick_block_underneath(&caller).await;
        let suffocating = self.entity.check_block_collisions(caller, server).await;
        if suffocating {
            self.damage(1.0, DamageType::IN_WALL).await;
        }
    }

    // movement_speed and off_ground_speed are different for a Player
    async fn travel_in_air(&self, movement_speed: f64, off_ground_speed: f64) {
        // applyMovementInput
        let (speed, friction) = if self.entity.on_ground.load(Relaxed) {
            // getVelocityAffectingPos
            let pos_affecting_velo = self.entity.get_pos_with_y_offset(0.500001).await;
            let world = self.entity.world.read().await;
            let slipperiness = world.get_block(&pos_affecting_velo).await.slipperiness as f64;
            let speed = movement_speed * 0.216 / (slipperiness * slipperiness * slipperiness);
            (speed, slipperiness * 0.91)
        } else {
            let speed = off_ground_speed;
            (speed, 0.91)
        };
        self.entity.update_velocity_from_input(self.movement_input.load(), speed);
        self.apply_climbing_speed();

        self.make_move().await;

        let mut velo = self.entity.velocity.load();
        // TODO: Add powdered snow
        if (self.entity.horizontal_collision.load(Relaxed) || self.jumping.load(Relaxed)) && (self.climbing.load(Relaxed)) {
            velo.y = 0.2;
        }
        let levitation = self.get_effect(EffectType::Levitation).await;
        if let Some(lev) = levitation {
            velo.y += 0.05 * (lev.amplifier + 1) as f64;
        } else {
            velo.y -= self.get_effective_gravity().await;
            // TODO: If world is not loaded: replace effective gravity with:
            // if below world's bottom y, 0.1, else 0.0
        }
        // If entity has no drag: store velo and return

        velo.x *= friction;
        velo.z *= friction;
        // If flutterer: multiply y by friction instead of 0.98
        velo.y *= 0.98;
        self.entity.velocity.store(velo);
    }

    // movement_speed is different for Player
    async fn travel_in_fluid(&self, water: bool, movement_speed: f64) {
        let movement_input = self.movement_input.load();
        let y0 = self.entity.pos.load().y;
        let falling = self.entity.velocity.load().y <= 0.0;
        let gravity = self.get_effective_gravity().await;
        if water {
            let mut friction = if self.entity.sprinting.load(Relaxed) {
                0.9
            } else {
                self.water_movement_speed_multiplier as f64
            };
            let mut speed = 0.02;
            let water_movement_efficiency = 0.0; // TODO: Entity attribute
            if water_movement_efficiency > 0.0 {
                if !self.entity.on_ground.load(Relaxed) {
                    water_movement_efficiency *= 0.5;
                }
                friction += (0.54600006 - friction) * water_movement_efficiency;
                speed += (movement_speed - speed) * water_movement_efficiency;
            }
            if self.has_effect(EffectType::DolphinsGrace).await {
                friction = 0.96;
            }

            self.entity.update_velocity_from_input(self.movement_input.load(), speed);
            self.make_move().await;

            let mut velo = self.entity.velocity.load();
            if self.entity.horizontal_collision.load(Relaxed) && self.climbing.load(Relaxed) {
                velo.y = 0.2;
            }
            velo = velo.multiply(friction, 0.8, friction);
            self.apply_fluid_moving_speed(&mut velo.y, gravity, falling);
            self.entity.velocity.store(velo);

        } else {
            self.entity.update_velocity_from_input(self.movement_input.load(), 0.02);
            self.make_move().await;

            let mut velo = self.entity.velocity.load();
            if self.lava_height.load() <= self.get_swim_height() {
                velo.x *= 0.5;
                velo.z *= 0.5;
                velo.y *= 0.8;
                self.apply_fluid_moving_speed(&mut velo.y, gravity, falling);
            } else {
                velo = velo * 0.5;
            }
            if gravity != 0.0 {
                velo.y -= gravity / 4.0; // Negative gravity = buoyancy
            }
            self.entity.velocity.store(velo);
        }
        let mut velo = self.entity.velocity.load();
        velo.y += 0.6 - self.entity.pos.load().y + y0;
        if self.entity.horizontal_collision.load(Relaxed) && !self
            .entity
            .world
            .read()
            .await
            .check_fluid_collision(self.entity.bounding_box.load().shift(velo))
            .await
        {
            velo.y = 0.3;
            self.entity.velocity.store(velo);
        }
    }

    fn apply_fluid_moving_speed(&self, dy: &mut f64, gravity: f64, falling: bool) {
        if gravity != 0.0 && !self.entity.sprinting.load(Relaxed) {
            if falling && (*dy - 0.005).abs() >= 0.003 && (*dy - gravity / 16.0).abs() < 0.003 {
                *dy = -0.003;
            } else {
                *dy -= gravity / 16.0;
            }
        }
    }

    async fn make_move(&self) {
        self.entity.move_entity(self.entity.velocity.load()).await;
        self.check_climbing().await;
    }

    async fn check_climbing(&self) {
        // If spectator: return false
        let mut pos = self.entity.block_pos.load();
        let world = self.entity.world.read().await;

        let (block, state) = world.get_block_and_block_state(&pos).await;
        let props = block.properties(state.id);
        let name = props.name();
        let climbable = name == "LadderLikeProperties"
            || name == "ScaffoldingLikeProperties"
            || name == "CaveVinesLikeProperties"
            || name == "CaveVinesPlantLikeProperties";
        if climbable {
            self.climbing.store(true, Relaxed);
            self.climbing_pos.store(Some(pos));
            return;
        }
        if name == "OakTrapdoorLikeProperties" {
            let trapdoor = OakTrapdoorLikeProperties::from_state_id(state.id);
            pos.0.y -= 1;
            let (down_block, down_state) = world.get_block_and_block_state(&pos).await;
            let down_props = down_block.properties(down_state.id).to_props();
            if down_props.name() == "LadderLikeProperties" {
                let ladder = LadderLikeProperties::from_state_id(state.id);
                if trapdoor.r#facing == ladder.r#facing {
                    self.climbing.store(true, Relaxed);
                    self.climbing_pos.store(Some(pos));
                    return;
                }
            }
        }
        self.climbing.store(false, Relaxed);
        if self.entity.on_ground.load(Relaxed) {
            self.climbing_pos.store(None);
        }
    }

    fn apply_climbing_speed(&self) {
        if self.climbing.load(Relaxed) {
            self.fall_distance.store(0.0);
            let mut velo = self.entity.velocity.load();

            let f = 0.15;
            if velo.x < -0.15 {
                velo.x = -0.15;
            } else if velo.x > 0.15 {
                velo.x = 0.15;
            }
            if velo.z < -0.15 {
                velo.z = -0.15;
            } else if velo.z > 0.15 {
                velo.z = 0.15;
            }
            velo.y = velo.y.max(-0.15);

            /* This only applies to players
            if velo.y < 0.0 {
                let (block, state) = self.entity.world.read().await.get_block_and_block_state(&self.block_pos.load());
                let props = block.properties(state.id);
                if self.entity.sneaking.load(Relaxed) && Block::has_properties::<block_properties::ScaffoldingLikeProperties>::() {
                    velo.y = 0.0;
                }
            }
            */
        }
    }

    pub fn get_swim_height(&self) -> f64 {
        let eye_height = self.entity.standing_eye_height;
        if self.entity.entity_type = EntityType::BREEZE {
            eye_height as f64
        } else {
            if eye_height < 0.4 {
                0.0
            } else {
                0.4
            }
        }
    }
    fn swim_upward(&self) {
        let mut velo = self.entity.velocity.load();
        velo.y = 0.04;
        self.entity.velocity.store(velo);
    }
    async fn jump(&self) {
        let jump = self.get_jump_velocity(1.0).await;
        if jump <= 1.0e-5 {
            return;
        }
        let mut velo = self.entity.velocity.load();
        velo.y = jump.max(velo.y);
        if self.entity.sprinting.load(Relaxed) {
            let yaw = self.entity.yaw.load() * std::f32::consts::PI / 180.0;
            velo.x += -yaw.sin() * 0.2;
            velo.y += yaw.cos() * 0.2;
        }
        self.entity.velocity.store(velo);
        // Todo? VelocityDirty = true
    }
    async fn get_jump_velocity(&self, mut strength: f64) -> f64 {
        strength *= 1.0; // TODO: Entity Attribute jump strength
        let block_multiplier = {
            // TODO: getJumpVelocityMultiplier for blocks
            let block_pos = self.entity.block_pos.load();
            let f = 1.0; // ^.multiplier
            let pos_affecting_velo = self.entity.get_pos_with_y_offset(0.500001);
            let g = 1.0; // ^.multiplier
            if f == 1.0 {
                g
            } else {
                f
            }
        };
        strength *= block_multiplier;
        if let Some(effect) = self.get_effect(EffectType::JumpBoost).await {
            strength += 0.1 * (effect.amplifier + 1) as f64;
        }
        strength
    }
    // TODO: Caching getBlockStateAtPos
}

#[async_trait]
impl EntityBase for LivingEntity {
    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
        // Following vanilla order of operations
        self.entity.tick(caller, server).await;
        self.base_tick().await;

        self.tick_movement(server, caller, None).await;
    }

    async fn damage(&self, amount: f32, damage_type: DamageType) -> bool {
        let world = self.entity.world.read().await;
        if !self.check_damage(amount) {
            return false;
        }
        let config = &advanced_config().pvp;

        if !self
            .damage_with_context(amount, damage_type, None, None, None)
            .await
        {
            return false;
        }

        if config.hurt_animation {
            let entity_id = VarInt(self.entity.entity_id);
            world
                .broadcast_packet_all(&CHurtAnimation::new(entity_id, self.entity.yaw.load()))
                .await;
        }
        true
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(self)
    }
}

#[async_trait]
impl NBTStorage for LivingEntity {
    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.entity.write_nbt(nbt).await;
        nbt.put("Health", NbtTag::Float(self.health.load()));
        nbt.put("fall_distance", NbtTag::Float(self.fall_distance.load()));
        {
            let effects = self.active_effects.lock().await;
            if !effects.is_empty() {
                // Iterate effects and create Box<[NbtTag]>
                let mut effects_list = Vec::with_capacity(effects.len());
                for effect in effects.values() {
                    let mut effect_nbt = pumpkin_nbt::compound::NbtCompound::new();
                    effect.write_nbt(&mut effect_nbt).await;
                    effects_list.push(NbtTag::Compound(effect_nbt));
                }
                nbt.put(
                    "active_effects",
                    NbtTag::List(effects_list.into_boxed_slice()),
                );
            }
        }
        //TODO: write equipment
        // todo more...
    }

    async fn read_nbt(&mut self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.entity.read_nbt(nbt).await;
        self.health.store(nbt.get_float("Health").unwrap_or(0.0));
        self.fall_distance
            .store(nbt.get_float("fall_distance").unwrap_or(0.0));
        {
            let mut active_effects = self.active_effects.lock().await;
            let nbt_effects = nbt.get_list("active_effects");
            if let Some(nbt_effects) = nbt_effects {
                for effect in nbt_effects {
                    if let NbtTag::Compound(effect_nbt) = effect {
                        let effect = Effect::create_from_nbt(&mut effect_nbt.clone()).await;
                        if effect.is_none() {
                            log::warn!("Unable to read effect from nbt");
                            continue;
                        }
                        let mut effect = effect.unwrap();
                        effect.blend = true; // TODO: change, is taken from effect give command
                        active_effects.insert(effect.r#type, effect);
                    }
                }
            }
        }
        // todo more...
    }
}
