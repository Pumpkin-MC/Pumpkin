use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::potion::Effect;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_inventory::build_equipment_slots;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::{
    AtomicBool, AtomicU8,
    Ordering::{Relaxed, SeqCst},
};
use std::{collections::HashMap, sync::atomic::AtomicI32};
use tracing::warn;

use super::{Entity, NBTStorage};
use super::{EntityBase, NBTStorageInit};
use crate::block::OnLandedUponArgs;
use crate::entity::{EntityBaseFuture, NbtFuture};
use crate::server::Server;
use crate::world::loot::{LootContextParameters, LootTableExt};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DeathMessageType;
use pumpkin_data::data_component_impl::{DeathProtectionImpl, EquipmentSlot, FoodImpl};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::{Block, translation};
use pumpkin_data::{damage::DamageType, sound::Sound};
use pumpkin_inventory::entity_equipment::EntityEquipment;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    Animation, CEntityAnimation, CHurtAnimation, CSetPlayerInventory, CTakeItemEntity,
};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::{CDamageEvent, CSetEquipment, Metadata},
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::plugin::player::player_item_break::PlayerItemBreakEvent;
use crate::plugin::player::player_item_consume::PlayerItemConsumeEvent;
use crate::plugin::player::player_item_damage::PlayerItemDamageEvent;

/// Represents a living entity within the game world.
///
/// This struct encapsulates the core properties and behaviors of living entities, including players, mobs, and other creatures.
pub struct LivingEntity {
    /// The underlying entity object, providing basic entity information and functionality.
    pub entity: Entity,
    /// Tracks the remaining time until the entity can regenerate health.
    pub hurt_cooldown: AtomicI32,
    /// Stores the amount of damage the entity last received.
    pub last_damage_taken: AtomicCell<f32>,
    /// The current health level of the entity.
    pub health: AtomicCell<f32>,
    pub item_use_time: AtomicI32,
    pub item_in_use: Mutex<Option<ItemStack>>,
    pub death_time: AtomicU8,
    /// Indicates whether the entity is dead. (`on_death` called)
    pub dead: AtomicBool,
    /// The distance the entity has been falling.
    pub fall_distance: AtomicCell<f32>,
    pub active_effects: Mutex<HashMap<&'static StatusEffect, Effect>>,
    pub entity_equipment: Arc<Mutex<EntityEquipment>>,
    pub movement_input: AtomicCell<Vector3<f64>>,
    pub equipment_slots: Arc<HashMap<usize, EquipmentSlot>>,

    pub movement_speed: AtomicCell<f64>,

    pub jumping: AtomicBool,

    pub jumping_cooldown: AtomicU8,

    pub climbing: AtomicBool,

    /// The position where the entity was last climbing, used for death messages
    pub climbing_pos: AtomicCell<Option<BlockPos>>,

    water_movement_speed_multiplier: f32,
    livings_flags: AtomicU8,
}

impl LivingEntity {
    const USING_ITEM_FLAG: u8 = 1;
    const OFF_HAND_ACTIVE_FLAG: u8 = 2;
    #[expect(dead_code)]
    const USING_RIPTIDE_FLAG: u8 = 4;

    const PREVENT_AREA_FALL_DAMAGE_BLOCKS: [&'static Block; 4] = [
        &Block::COBWEB,
        &Block::LADDER,
        &Block::POWDER_SNOW,
        &Block::SLIME_BLOCK,
    ];
    const FALL_DAMAGE_SAFE_DISTANCE: f64 = 1.3;

    pub fn new(entity: Entity) -> Self {
        let water_movement_speed_multiplier = if entity.entity_type == &EntityType::POLAR_BEAR {
            0.98
        } else if entity.entity_type == &EntityType::SKELETON_HORSE {
            0.96
        } else {
            0.8
        };
        // TODO: Extract default MOVEMENT_SPEED Entity Attribute
        let default_movement_speed = 0.25;
        let health = entity.entity_type.max_health.unwrap_or(20.0);
        Self {
            entity,
            hurt_cooldown: AtomicI32::new(0),
            last_damage_taken: AtomicCell::new(0.0),
            health: AtomicCell::new(health),
            fall_distance: AtomicCell::new(0.0),
            death_time: AtomicU8::new(0),
            dead: AtomicBool::new(false),
            item_use_time: AtomicI32::new(0),
            item_in_use: Mutex::new(None),
            livings_flags: AtomicU8::new(0),
            active_effects: Mutex::new(HashMap::new()),
            entity_equipment: Arc::new(Mutex::new(EntityEquipment::new())),
            equipment_slots: Arc::new(build_equipment_slots()),
            jumping: AtomicBool::new(false),
            jumping_cooldown: AtomicU8::new(0),
            climbing: AtomicBool::new(false),
            climbing_pos: AtomicCell::new(None),
            movement_input: AtomicCell::new(Vector3::default()),
            movement_speed: AtomicCell::new(default_movement_speed),
            water_movement_speed_multiplier,
        }
    }

    pub async fn send_equipment_changes(&self, equipment: &[(EquipmentSlot, ItemStack)]) {
        let equipment: Vec<(i8, ItemStackSerializer)> = equipment
            .iter()
            .map(|(slot, stack)| {
                (
                    slot.discriminant(),
                    ItemStackSerializer::from(stack.clone()),
                )
            })
            .collect();
        self.entity
            .world
            .load()
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
            .load()
            .broadcast_packet_all(&CTakeItemEntity::new(
                item.entity_id.into(),
                self.entity.entity_id.into(),
                stack_amount.try_into().unwrap(),
            ))
            .await;
    }

    /// Sends the Hand animation to all others, used when Eating for example
    pub async fn set_active_hand(&self, hand: Hand, stack: ItemStack) {
        self.item_use_time
            .store(stack.get_max_use_time(), Ordering::Relaxed);
        *self.item_in_use.lock().await = Some(stack);
        self.set_living_flag(Self::USING_ITEM_FLAG, true).await;
        self.set_living_flag(Self::OFF_HAND_ACTIVE_FLAG, hand == Hand::Left)
            .await;
    }

    async fn set_living_flag(&self, flag: u8, value: bool) {
        let index = flag;
        let mut b = self.livings_flags.load(Ordering::Relaxed);
        if value {
            b |= index;
        } else {
            b &= !index;
        }
        self.livings_flags.store(b, Ordering::Relaxed);
        self.entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_LIVING_FLAGS,
                MetaDataType::Byte,
                b,
            )])
            .await;
    }

    pub async fn clear_active_hand(&self) {
        *self.item_in_use.lock().await = None;
        self.item_use_time.store(0, Ordering::Relaxed);

        self.set_living_flag(Self::USING_ITEM_FLAG, false).await;
    }

    pub async fn heal(&self, additional_health: f32) {
        assert!(additional_health > 0.0);
        self.set_health(self.health.load() + additional_health)
            .await;
    }

    pub async fn set_health(&self, health: f32) {
        self.health.store(health.max(0.0));
        // tell everyone entities health changed
        self.entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_HEALTH,
                MetaDataType::Float,
                health,
            )])
            .await;
    }

    pub const fn entity_id(&self) -> i32 {
        self.entity.entity_id
    }

    pub async fn add_effect(&self, effect: Effect) {
        self.active_effects
            .lock()
            .await
            .insert(effect.effect_type, effect);
        // TODO broadcast metadata
    }

    pub async fn remove_effect(&self, effect_type: &'static StatusEffect) -> bool {
        let succeeded = self
            .active_effects
            .lock()
            .await
            .remove(&effect_type)
            .is_some();
        self.entity
            .world
            .load()
            .send_remove_mob_effect(&self.entity, effect_type)
            .await;
        succeeded
    }

    pub async fn has_effect(&self, effect: &'static StatusEffect) -> bool {
        let effects = self.active_effects.lock().await;
        effects.contains_key(&effect)
    }

    pub async fn get_effect(&self, effect: &'static StatusEffect) -> Option<Effect> {
        let effects = self.active_effects.lock().await;
        effects.get(&effect).cloned()
    }

    pub async fn is_in_fall_damage_resetting(&self) -> (bool, &Block) {
        let block_pos = self.entity.block_pos.load();
        let block = self.entity.world.load().get_block(&block_pos).await;
        (
            block.has_tag(&tag::Block::MINECRAFT_FALL_DAMAGE_RESETTING),
            block,
        )
    }

    // Check if the entity is in water
    pub async fn is_in_water(&self) -> bool {
        let block_pos = self.entity.block_pos.load();
        self.entity.world.load().get_block(&block_pos).await == &Block::WATER
    }

    // Check if the entity is in powder snow
    pub async fn is_in_powder_snow(&self) -> bool {
        let block_pos = self.entity.block_pos.load();
        self.entity.world.load().get_block(&block_pos).await == &Block::POWDER_SNOW
    }

    pub async fn should_prevent_fall_damage(&self) -> bool {
        let (prevents, block) = self.is_in_fall_damage_resetting().await;

        if block == &Block::SCAFFOLDING && !self.entity.sneaking.load(Ordering::Relaxed) {
            return false;
        }

        if block == &Block::WATER {
            return true;
        }

        if self.entity.entity_type == &EntityType::PLAYER {
            if block == &Block::END_GATEWAY || block == &Block::END_PORTAL {
                return true;
            }

            if block == &Block::NETHER_PORTAL {
                let world = self.entity.world.load();
                let level_info = world.level_info.load();

                return level_info.game_rules.players_nether_portal_default_delay == 0;
            }
        }

        prevents
    }

    pub async fn should_prevent_fall_damage_in_area(&self) -> bool {
        let world = self.entity.world.load();
        let block_pos = self.entity.block_pos.load().down();
        let entity_pos = self.entity.pos.load();

        let min = BlockPos(Vector3::new(
            block_pos.0.x - 1,
            block_pos.0.y,
            block_pos.0.z - 1,
        ));
        let max = BlockPos(Vector3::new(
            block_pos.0.x + 1,
            block_pos.0.y,
            block_pos.0.z + 1,
        ));
        let pos_iter = BlockPos::iterate(min, max);

        // FIXME: it seems the java server checks all blocks around with a raycast and check if miss or hit,
        // then added to a collision checker to handle in the tick handler
        for pos in pos_iter {
            let block = world.get_block(&pos).await;

            if Self::PREVENT_AREA_FALL_DAMAGE_BLOCKS.contains(&block) {
                let block_center = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y) + 0.5,
                    f64::from(pos.0.z) + 0.5,
                );
                let distance = entity_pos.squared_distance_to_vec(&block_center);

                return distance.sqrt()
                    <= Self::FALL_DAMAGE_SAFE_DISTANCE * Self::FALL_DAMAGE_SAFE_DISTANCE;
            }
        }

        false
    }

    pub fn is_immune_to_fall_damage(&self) -> bool {
        self.entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_FALL_DAMAGE_IMMUNE)
    }

    async fn get_effective_gravity(&self, caller: &Arc<dyn EntityBase>) -> f64 {
        let final_gravity = caller.get_gravity();

        if self.entity.velocity.load().y <= 0.0
            && self.has_effect(&StatusEffect::SLOW_FALLING).await
        {
            final_gravity.min(0.01)
        } else {
            final_gravity
        }
    }

    pub async fn swing_hand(&self) {
        // TODO: radius
        self.entity
            .world
            .load()
            .broadcast_packet_all(&CEntityAnimation::new(
                self.entity_id().into(),
                Animation::SwingMainArm,
            ))
            .await;
    }

    async fn tick_movement(&self, server: &Server, caller: Arc<dyn EntityBase>) {
        if self.jumping_cooldown.load(Relaxed) != 0 {
            self.jumping_cooldown.fetch_sub(1, Relaxed);
        }

        let should_swim_in_fluids = if let Some(player) = caller.get_player() {
            !player.is_flying().await
        } else {
            true
        };

        self.entity.check_zero_velo();

        let mut movement_input = self.movement_input.load();

        movement_input.x *= 0.98;

        movement_input.z *= 0.98;

        self.movement_input.store(movement_input);

        // TODO: Tick AI

        if self.jumping.load(SeqCst) && should_swim_in_fluids {
            let in_lava = self.entity.touching_lava.load(SeqCst);

            let in_water = self.entity.touching_water.load(SeqCst);

            let fluid_height = if in_lava {
                self.entity.lava_height.load()
            } else {
                self.entity.water_height.load()
            };

            let swim_height = self.get_swim_height();

            let on_ground = self.entity.on_ground.load(SeqCst);

            if (in_water || in_lava) && (!on_ground || fluid_height > swim_height) {
                // Swim upward

                let mut velo = self.entity.velocity.load();

                velo.y += 0.04;

                self.entity.velocity.store(velo);
            } else if (on_ground || in_water && fluid_height <= swim_height)
                && self.jumping_cooldown.load(SeqCst) == 0
            {
                self.jump().await;

                self.jumping_cooldown.store(10, SeqCst);
            }
        } else {
            self.jumping_cooldown.store(0, SeqCst);
        }

        if self.has_effect(&StatusEffect::SLOW_FALLING).await
            || self.has_effect(&StatusEffect::LEVITATION).await
        {
            self.fall_distance.store(0.0);
        }

        let touching_water = self.entity.touching_water.load(SeqCst);

        // Strider is the only entity that has canWalkOnFluid = false

        if (touching_water || self.entity.touching_lava.load(SeqCst))
            && should_swim_in_fluids
            && self.entity.entity_type != &EntityType::STRIDER
        {
            self.travel_in_fluid(caller.clone(), touching_water).await;
        } else {
            // TODO: Gliding

            self.travel_in_air(caller.clone()).await;
        }

        //self.entity.tick_block_underneath(&caller);

        let suffocating = self.entity.tick_block_collisions(&caller, server).await;

        if suffocating {
            self.damage(&*caller, 1.0, DamageType::IN_WALL).await;
        }
    }

    async fn travel_in_air(&self, caller: Arc<dyn EntityBase>) {
        // applyMovementInput

        let (speed, friction) = if self.entity.on_ground.load(SeqCst) {
            // getVelocityAffectingPos

            let slipperiness = f64::from(
                self.entity
                    .get_block_with_y_offset(0.500_001)
                    .await
                    .1
                    .slipperiness,
            );

            let speed = self.movement_speed.load() * 0.216_000_02
                / (slipperiness * slipperiness * slipperiness);

            (speed, slipperiness * 0.91)
        } else {
            let speed = if let Some(player) = caller.get_player() {
                player.get_off_ground_speed().await
            } else {
                // TODO: If the passenger is a player, ogs = movement_speed * 0.1

                0.02
            };

            (speed, 0.91)
        };

        self.entity
            .update_velocity_from_input(self.movement_input.load(), speed);

        self.apply_climbing_speed();

        self.make_move(caller.clone()).await;

        let mut velo = self.entity.velocity.load();

        // TODO: Add powdered snow

        if (self.entity.horizontal_collision.load(SeqCst) || self.jumping.load(SeqCst))
            && (self.climbing.load(Relaxed))
        {
            velo.y = 0.2;
        }

        let levitation = self.get_effect(&StatusEffect::LEVITATION).await;

        if let Some(lev) = levitation {
            velo.y += 0.05f64.mul_add(f64::from(lev.amplifier + 1), -velo.y) * 0.2;
        } else {
            velo.y -= self.get_effective_gravity(&caller).await;

            // TODO: If world is not loaded: replace effective gravity with:

            // if below world's bottom y then -0.1, else 0.0
        }

        // If entity has no drag: store velo and return

        velo.x *= friction;

        velo.z *= friction;

        velo.y *= if caller.is_flutterer() {
            friction
        } else {
            0.98
        };

        self.entity.velocity.store(velo);
    }

    async fn travel_in_fluid(&self, caller: Arc<dyn EntityBase>, water: bool) {
        let movement_input = self.movement_input.load();

        let y0 = self.entity.pos.load().y;

        let falling = self.entity.velocity.load().y <= 0.0;

        let gravity = self.get_effective_gravity(&caller).await;

        if water {
            let mut friction = if self.entity.sprinting.load(Relaxed) {
                0.9
            } else {
                f64::from(self.water_movement_speed_multiplier)
            };

            let mut speed = 0.02;

            let mut water_movement_efficiency = 0.0; // TODO: Entity attribute

            if water_movement_efficiency > 0.0 {
                if !self.entity.on_ground.load(SeqCst) {
                    water_movement_efficiency *= 0.5;
                }

                friction += (0.546_000_06 - friction) * water_movement_efficiency;

                speed += (self.movement_speed.load() - speed) * water_movement_efficiency;
            }

            if self.has_effect(&StatusEffect::DOLPHINS_GRACE).await {
                friction = 0.96;
            }

            self.entity
                .update_velocity_from_input(movement_input, speed);

            self.make_move(caller).await;

            let mut velo = self.entity.velocity.load();

            if self.entity.horizontal_collision.load(SeqCst) && self.climbing.load(Relaxed) {
                velo.y = 0.2;
            }

            velo = velo.multiply(friction, 0.8, friction);

            self.apply_fluid_moving_speed(&mut velo.y, gravity, falling);

            self.entity.velocity.store(velo);
        } else {
            self.entity.update_velocity_from_input(movement_input, 0.02);

            self.make_move(caller).await;

            let mut velo = self.entity.velocity.load();

            if self.entity.lava_height.load() <= self.get_swim_height() {
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

        if self.entity.horizontal_collision.load(SeqCst)
            && !self
                .entity
                .world
                .load()
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

    async fn make_move(&self, caller: Arc<dyn EntityBase>) {
        self.entity
            .move_entity(caller, self.entity.velocity.load())
            .await;

        self.check_climbing();
    }

    fn check_climbing(&self) {
        // If spectator: return false

        // TODO
        // let mut pos = self.entity.block_pos.load();

        // let world = self.entity.world.read().await;

        // let (block, state) = world.get_block_and_state(&pos).await;

        // let name = block.properties(state.id).map(|props| props.name());

        // if let Some(name) = name {
        //     if name == "LadderLikeProperties"
        //         || name == "ScaffoldingLikeProperties"
        //         || name == "CaveVinesLikeProperties"
        //         || name == "CaveVinesPlantLikeProperties"
        //     {
        //         self.climbing.store(true, Relaxed);

        //         self.climbing_pos.store(Some(pos));

        //         return;
        //     }

        //     if name == "OakTrapdoorLikeProperties" {
        //         let trapdoor = OakTrapdoorLikeProperties::from_state_id(state.id, &block);

        //         pos.0.y -= 1;

        //         let (down_block, down_state) = world.get_block_and_state(&pos).await;

        //         let is_ladder = down_block
        //             .properties(down_state.id)
        //             .is_some_and(|down_props| down_props.name() == "LadderLikeProperties");

        //         if is_ladder {
        //             let ladder = LadderLikeProperties::from_state_id(down_state.id, &down_block);

        //             if trapdoor.r#facing == ladder.r#facing {
        //                 self.climbing.store(true, Relaxed);

        //                 self.climbing_pos.store(Some(pos));

        //                 return;
        //             }
        //         }
        //     }
        // }

        self.climbing.store(false, Relaxed);

        if self.entity.on_ground.load(SeqCst) {
            self.climbing_pos.store(None);
        }
    }

    fn apply_climbing_speed(&self) {
        if self.climbing.load(Relaxed) {
            self.fall_distance.store(0.0);

            let mut velo = self.entity.velocity.load();

            let pos = 0.15;

            let neg = -0.15;

            if velo.x < neg {
                velo.x = neg;
            } else if velo.x > pos {
                velo.x = pos;
            }

            if velo.z < neg {
                velo.z = neg;
            } else if velo.z > pos {
                velo.z = pos;
            }

            velo.y = velo.y.max(neg);

            // TODO
            // if velo.y < 0.0
            //     && self.entity.entity_type == &EntityType::PLAYER
            //     && self.entity.sneaking.load(Relaxed)
            // {
            //     let block = self
            //         .entity
            //         .world
            //         .read()
            //         .await
            //         .get_block(&self.entity.block_pos.load())
            //         .await;

            //     if let Some(props) = block.properties(block.default_state.id) {
            //         if props.name() == "ScaffoldingLikeProperties" {
            //             velo.y = 0.0;
            //         }
            //     }
            // }

            self.entity.velocity.store(velo);
        }
    }

    pub fn get_swim_height(&self) -> f64 {
        let eye_height = self.entity.get_eye_height();

        if self.entity.entity_type == &EntityType::BREEZE {
            eye_height
        } else if eye_height < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    async fn jump(&self) {
        let jump = self.get_jump_velocity(1.0).await;

        if jump <= 1.0e-5 {
            return;
        }

        let mut velo = self.entity.velocity.load();

        velo.y = jump.max(velo.y);

        if self.entity.sprinting.load(Relaxed) {
            let yaw = f64::from(self.entity.yaw.load()).to_radians();

            velo.x -= yaw.sin() * 0.2;

            velo.y += yaw.cos() * 0.2;
        }

        self.entity.velocity.store(velo);

        self.entity.velocity_dirty.store(true, SeqCst);
    }

    async fn get_jump_velocity(&self, mut strength: f64) -> f64 {
        strength *= 0.42; // TODO: Read from Entity Attribute JUMP_STRENGTH (default 0.42)

        strength *= f64::from(self.entity.get_jump_velocity_multiplier().await);

        if let Some(effect) = self.get_effect(&StatusEffect::JUMP_BOOST).await {
            strength += 0.1 * f64::from(effect.amplifier + 1);
        }

        strength
    }

    pub async fn fall(
        &self,
        caller: Arc<dyn EntityBase>,
        height_difference: f64,
        ground: bool,
        dont_damage: bool,
    ) {
        if ground {
            let fall_distance = self.fall_distance.swap(0.0);
            if fall_distance <= 0.0
                || dont_damage
                || self.should_prevent_fall_damage().await
                || self.should_prevent_fall_damage_in_area().await
                || self.is_immune_to_fall_damage()
            {
                return;
            }
            let world = self.entity.world.load();
            let block = world
                .get_block(&self.entity.get_pos_with_y_offset(0.2).await.0)
                .await;
            let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);
            if let Some(pumpkin_block) = pumpkin_block {
                pumpkin_block
                    .on_landed_upon(OnLandedUponArgs {
                        world: &world,
                        fall_distance,
                        entity: caller.as_ref(),
                    })
                    .await;
            } else {
                self.handle_fall_damage(&*caller, fall_distance, 1.0).await;
            }
        } else if height_difference < 0.0 {
            let new_fall_distance = if !self.should_prevent_fall_damage().await
                && !self.should_prevent_fall_damage_in_area().await
            {
                let distance = self.fall_distance.load();
                distance - (height_difference as f32)
            } else {
                0f32
            };
            self.fall_distance.store(new_fall_distance);
        }
    }

    pub async fn handle_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_per_distance: f32,
    ) {
        if self.is_immune_to_fall_damage() {
            return;
        }

        // TODO: use attributes
        let safe_fall_distance = 3.0;
        let unsafe_fall_distance = fall_distance + 1.0E-6 - safe_fall_distance;

        let damage = (unsafe_fall_distance * damage_per_distance).floor();
        if damage > 0.0 {
            let check_damage = self.damage(caller, damage, DamageType::FALL).await; // Fall
            if check_damage {
                self.entity
                    .play_sound(Self::get_fall_sound(fall_distance as i32))
                    .await;
            }
        }
    }

    const fn get_fall_sound(distance: i32) -> Sound {
        if distance > 4 {
            Sound::EntityGenericBigFall
        } else {
            Sound::EntityGenericSmallFall
        }
    }

    pub async fn get_death_message(
        dyn_self: &dyn EntityBase,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> TextComponent {
        match damage_type.death_message_type {
            DeathMessageType::Default => {
                if let Some(cause) = cause
                    && source.is_some()
                {
                    TextComponent::translate(
                        format!("death.attack.{}.player", damage_type.message_id),
                        [
                            dyn_self.get_display_name().await,
                            cause.get_display_name().await,
                        ],
                    )
                } else {
                    TextComponent::translate(
                        format!("death.attack.{}", damage_type.message_id),
                        [dyn_self.get_display_name().await],
                    )
                }
            }
            DeathMessageType::FallVariants => {
                //TODO
                TextComponent::translate(
                    translation::DEATH_FELL_ACCIDENT_GENERIC,
                    [dyn_self.get_display_name().await],
                )
            }
            DeathMessageType::IntentionalGameDesign => TextComponent::text("[")
                .add_child(TextComponent::translate(
                    format!("death.attack.{}.message", damage_type.message_id),
                    [dyn_self.get_display_name().await],
                ))
                .add_child(TextComponent::text("]")),
        }
    }

    pub async fn on_death(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let world = self.entity.world.load();
        let dyn_self = world
            .get_entity_by_id(self.entity.entity_id)
            .expect("Entity not found in world");
        if self
            .dead
            .compare_exchange(false, true, Relaxed, Relaxed)
            .is_ok()
        {
            // Immediately clear all movement/velocity so the dead entity stops being
            // simulated by physics and doesn't accumulate additional fall_distance.
            self.entity.velocity.store(Vector3::default());
            self.entity.velocity_dirty.store(true, SeqCst);
            self.movement_input.store(Vector3::default());
            self.jumping.store(false, Relaxed);
            // Plays the death sound
            world
                .send_entity_status(
                    &self.entity,
                    EntityStatus::PlayDeathSoundOrAddProjectileHitParticles,
                )
                .await;
            let params = LootContextParameters {
                killed_by_player: cause.map(|c| c.get_entity().entity_type == &EntityType::PLAYER),
                ..Default::default()
            };

            self.drop_loot(params).await;
            self.entity.pose.store(EntityPose::Dying);

            let block_pos = self.entity.block_pos.load();

            for slot in self.equipment_slots.values() {
                let item = {
                    let lock = self.entity_equipment.lock().await;
                    let equipment = lock.get(slot);
                    let mut item_lock = equipment.lock().await;
                    mem::replace(&mut *item_lock, ItemStack::EMPTY.clone())
                };
                world.drop_stack(&block_pos, item).await;
            }

            let show_death_messages = { world.level_info.load().game_rules.show_death_messages };
            if self.entity.entity_type == &EntityType::PLAYER && show_death_messages {
                //TODO: KillCredit
                let death_message =
                    Self::get_death_message(&*dyn_self, damage_type, source, cause).await;
                if let Some(server) = world.server.upgrade() {
                    for player in server.get_all_players() {
                        player.send_system_message(&death_message).await;
                    }
                }
            }
        }
    }

    async fn drop_loot(&self, params: LootContextParameters) {
        if let Some(loot_table) = &self.get_entity().entity_type.loot_table {
            let pos = self.entity.block_pos.load();
            for stack in loot_table.get_loot(params) {
                self.entity.world.load().drop_stack(&pos, stack).await;
            }
        }
    }

    async fn tick_effects(&self) {
        let mut effects_to_remove = Vec::new();

        {
            let mut effects = self.active_effects.lock().await;
            for effect in effects.values_mut() {
                if effect.duration == 0 {
                    effects_to_remove.push(effect.effect_type);
                }
                effect.duration -= 1;
            }
        }

        for effect_type in effects_to_remove {
            self.remove_effect(effect_type).await;
        }
    }

    async fn try_use_death_protector(&self, caller: &dyn EntityBase) -> bool {
        for hand in Hand::all() {
            let stack = self.get_stack_in_hand(caller, hand).await;
            let mut stack = stack.lock().await;
            // TODO: effects...
            if stack.get_data_component::<DeathProtectionImpl>().is_some() {
                stack.clear();
                self.set_health(1.0).await;
                self.entity
                    .world
                    .load()
                    .send_entity_status(&self.entity, EntityStatus::UseTotemOfUndying)
                    .await;
                return true;
            }
        }

        false
    }

    async fn damage_armor_items(&self, caller: &dyn EntityBase, damage_amount: f32) {
        let armor_damage = (damage_amount / 4.0).floor().max(1.0) as i32;
        let mut equipment_updates = Vec::new();

        for (slot_index, slot) in self.equipment_slots.iter() {
            if !slot.is_armor_slot() {
                continue;
            }

            let equipment = self.entity_equipment.lock().await.get(slot);
            let mut damage = armor_damage;
            let mut broken_item: Option<ItemStack> = None;

            if let Some(player) = caller.get_player() {
                let snapshot = {
                    let stack = equipment.lock().await;
                    if stack.is_empty() {
                        continue;
                    }
                    stack.clone()
                };
                if let Some(server) = player.world().server.upgrade()
                    && let Some(player_arc) = player.as_arc()
                {
                    let event = PlayerItemDamageEvent::new(player_arc, snapshot, damage);
                    let event = server.plugin_manager.fire(event).await;
                    if event.cancelled {
                        continue;
                    }
                    damage = event.damage;
                }
                if damage <= 0 {
                    continue;
                }
            }

            let updated_stack = {
                let mut stack = equipment.lock().await;
                if stack.is_empty() {
                    None
                } else {
                    let before = stack.clone();
                    if stack.damage_item_with_context(damage, true) {
                        if stack.is_empty() {
                            broken_item = Some(before);
                        }
                        Some(stack.clone())
                    } else {
                        None
                    }
                }
            };

            if let Some(updated_stack) = updated_stack {
                equipment_updates.push((slot.clone(), updated_stack.clone()));
                if let Some(player) = caller.get_player() {
                    player
                        .enqueue_slot_set_packet(&CSetPlayerInventory::new(
                            (*slot_index as i32).into(),
                            &ItemStackSerializer::from(updated_stack),
                        ))
                        .await;
                }
                if let (Some(player), Some(broken_item)) = (caller.get_player(), broken_item) {
                    if let Some(server) = player.world().server.upgrade()
                        && let Some(player_arc) = player.as_arc()
                    {
                        let event = PlayerItemBreakEvent::new(player_arc, broken_item);
                        server
                            .plugin_manager
                            .fire::<PlayerItemBreakEvent>(event)
                            .await;
                    }
                }
            }
        }

        if !equipment_updates.is_empty() {
            self.send_equipment_changes(&equipment_updates).await;
        }
    }

    pub async fn held_item(&self, caller: &dyn EntityBase) -> Arc<Mutex<ItemStack>> {
        if let Some(player) = caller.get_player() {
            return player.inventory.held_item();
        }
        // TODO: this is wrong
        let slot = self
            .equipment_slots
            .get(&PlayerInventory::OFF_HAND_SLOT)
            .unwrap();
        self.entity_equipment.lock().await.get(slot)
    }

    pub async fn get_stack_in_hand(
        &self,
        caller: &dyn EntityBase,
        hand: Hand,
    ) -> Arc<Mutex<ItemStack>> {
        match hand {
            Hand::Left => self.off_hand_item().await,
            Hand::Right => self.held_item(caller).await,
        }
    }

    /// getOffHandStack in source
    pub async fn off_hand_item(&self) -> Arc<Mutex<ItemStack>> {
        let slot = self
            .equipment_slots
            .get(&PlayerInventory::OFF_HAND_SLOT)
            .unwrap();
        self.entity_equipment.lock().await.get(slot)
    }

    pub fn can_take_damage(&self) -> bool {
        !self.entity.invulnerable.load(Ordering::Relaxed) && self.is_part_of_game()
    }

    pub fn is_part_of_game(&self) -> bool {
        !self.is_spectator() && self.entity.is_alive()
    }

    pub async fn reset_state(&self) {
        self.entity.reset_state().await;

        // Restore to maximum health for this entity type
        let max_health = self.entity.entity_type.max_health.unwrap_or(20.0);
        self.set_health(max_health).await;

        // Give a short grace period of invulnerability after respawn
        self.hurt_cooldown.store(20, Relaxed);
        self.last_damage_taken.store(0f32);

        self.entity.portal_cooldown.store(0, Relaxed);
        *self.entity.portal_manager.lock().await = None;

        // Clear fall/fire state
        self.fall_distance.store(0f32);
        self.death_time.store(0, Relaxed);
        self.entity.extinguish();
        self.entity.fire_ticks.store(0, Relaxed);

        // Clear velocity and movement input to remove persisted momentum
        self.entity.velocity.store(Vector3::default());
        self.entity.velocity_dirty.store(true, SeqCst);
        self.movement_input.store(Vector3::default());
        self.jumping.store(false, Relaxed);

        // If this LivingEntity corresponds to a Player, reset their hunger manager
        let world = self.entity.world.load();
        if let Some(player) = world.get_player_by_id(self.entity.entity_id) {
            player.hunger_manager.restart();
        }

        self.dead.store(false, Relaxed);
    }

    pub fn is_player(&self) -> bool {
        let world = self.entity.world.load();
        world.get_player_by_id(self.entity.entity_id).is_some()
    }

    pub fn get_movement(&self) -> Vector3<f64> {
        self.entity.movement.load()
    }
}

impl NBTStorage for LivingEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;
            nbt.put("Health", NbtTag::Float(self.health.load()));
            // Avoid persisting a lethal fall distance when the entity is dead to prevent death loops
            let fall_distance = if self.dead.load(Relaxed) {
                0.0
            } else {
                self.fall_distance.load()
            };
            nbt.put("fall_distance", NbtTag::Float(fall_distance));
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
                    nbt.put("active_effects", NbtTag::List(effects_list));
                }
            }
            //TODO: write equipment
            // todo more...
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.read_nbt_non_mut(nbt).await;
            self.health.store(nbt.get_float("Health").unwrap_or(0.0));
            // Load fall distance, but if this entity is currently marked dead ensure we don't restore
            // a lethal fall distance that would immediately re-kill on spawn.
            let fd = nbt.get_float("fall_distance").unwrap_or(0.0);
            if self.dead.load(Relaxed) {
                self.fall_distance.store(0.0);
            } else {
                self.fall_distance.store(fd);
            }
            {
                let mut active_effects = self.active_effects.lock().await;
                let nbt_effects = nbt.get_list("active_effects");
                if let Some(nbt_effects) = nbt_effects {
                    for effect in nbt_effects {
                        if let NbtTag::Compound(effect_nbt) = effect {
                            let effect = Effect::create_from_nbt(&mut effect_nbt.clone()).await;
                            if effect.is_none() {
                                warn!("Unable to read effect from nbt");
                                continue;
                            }
                            let mut effect = effect.unwrap();
                            effect.blend = true; // TODO: change, is taken from effect give command
                            active_effects.insert(effect.effect_type, effect);
                        }
                    }
                }
            }
        })
        // todo more...
    }
}

impl EntityBase for LivingEntity {
    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Check invulnerability before applying damage
            if self.entity.is_invulnerable_to(&damage_type) {
                return false;
            }

            if self.health.load() <= 0.0 || self.dead.load(Relaxed) {
                return false; // Dying or dead
            }

            if amount < 0.0 {
                return false;
            }

            if (damage_type == DamageType::IN_FIRE || damage_type == DamageType::ON_FIRE)
                && self.has_effect(&StatusEffect::FIRE_RESISTANCE).await
            {
                return false; // Fire resistance
            }

            let world = self.entity.world.load();
            let mut effective_amount = amount;
            if let Some(server) = world.server.upgrade() {
                let event = crate::plugin::entity::entity_damage::EntityDamageEvent::new(
                    self.entity.entity_uuid,
                    amount,
                    damage_type,
                );
                let event = server
                    .plugin_manager
                    .fire::<crate::plugin::entity::entity_damage::EntityDamageEvent>(event)
                    .await;
                if event.cancelled {
                    return false;
                }
                if event.damage <= 0.0 {
                    return false;
                }
                effective_amount = event.damage;
            }

            // These damage types bypass the hurt cooldown and death protection
            let bypasses_cooldown_protection =
                damage_type == DamageType::GENERIC_KILL || damage_type == DamageType::OUT_OF_WORLD;

            let last_damage = self.last_damage_taken.load();
            let play_sound;
            let mut damage_amount =
                if self.hurt_cooldown.load(Relaxed) > 10 && !bypasses_cooldown_protection {
                    if effective_amount <= last_damage {
                        return false;
                    }
                    play_sound = false;
                    effective_amount - self.last_damage_taken.load()
                } else {
                    self.hurt_cooldown.store(20, Relaxed);
                    play_sound = true;
                    effective_amount
                };
            self.last_damage_taken.store(effective_amount);
            damage_amount = damage_amount.max(0.0);

            let config = &world.server.upgrade().unwrap().advanced_config.pvp;

            if config.hurt_animation {
                let entity_id = VarInt(self.entity.entity_id);
                let hurt_yaw = source.map_or(0.0, |source| {
                    let src = source.get_entity().pos.load();
                    let tgt = self.entity.pos.load();
                    (src.z - tgt.z).atan2(src.x - tgt.x).to_degrees() as f32
                        - self.entity.yaw.load()
                });
                world
                    .broadcast_packet_all(&CHurtAnimation::new(entity_id, hurt_yaw))
                    .await;
            }

            world
                .broadcast_packet_all(&CDamageEvent::new(
                    self.entity.entity_id.into(),
                    damage_type.id.into(),
                    source.map(|e| e.get_entity().entity_id.into()),
                    cause.map(|e| e.get_entity().entity_id.into()),
                    position,
                ))
                .await;

            if play_sound {
                world
                    .play_sound(
                        Sound::EntityGenericHurt,
                        SoundCategory::Players,
                        &self.entity.pos.load(),
                    )
                    .await;

                if let Some(source) = source {
                    let source_pos = source.get_entity().pos.load();
                    let target_pos = self.entity.pos.load();
                    let dx = source_pos.x - target_pos.x;
                    let dz = source_pos.z - target_pos.z;
                    self.entity.apply_knockback(0.4, dx, dz);
                    self.entity.send_velocity().await;
                }
            }

            let new_health = self.health.load() - damage_amount;
            if damage_amount > 0.0 {
                //self.on_actually_hurt(damage_amount, damage_type).await;
                self.set_health(new_health).await;
            }

            // Check if the entity died and isn't protected by a death protection mechanic (ex. totem of undying)
            if new_health <= 0.0
                && (bypasses_cooldown_protection || !self.try_use_death_protector(caller).await)
            {
                self.on_death(damage_type, source, cause).await;
                if let Some(server) = world.server.upgrade() {
                    let event = crate::plugin::entity::entity_death::EntityDeathEvent::new(
                        self.entity.entity_uuid,
                        damage_type,
                    );
                    server
                        .plugin_manager
                        .fire::<crate::plugin::entity::entity_death::EntityDeathEvent>(event)
                        .await;
                }
            }

            if damage_amount > 0.0 {
                self.damage_armor_items(caller, damage_amount).await;
            }

            true
        })
    }

    fn tick_in_void<'a>(&'a self, dyn_self: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            dyn_self
                .damage(dyn_self, 4.0, DamageType::OUT_OF_WORLD)
                .await;
        })
    }

    fn get_gravity(&self) -> f64 {
        const GRAVITY: f64 = 0.08;
        GRAVITY
    }

    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.entity.tick(caller.clone(), server).await;
            // Only tick movement if the entity is alive. This prevents a dead "corpse"
            // from continuing to be simulated (accumulating fall_distance/velocity).
            if !self.dead.load(Relaxed) && self.health.load() > 0.0 {
                self.tick_movement(server, caller.clone()).await;
            }
            // TODO
            if caller.get_player().is_none() {
                self.entity.send_pos_rot().await;
            }
            self.tick_effects().await;
            // Current active item
            {
                let item_in_use = self.item_in_use.lock().await.clone();
                if let Some(item) = item_in_use.as_ref()
                    && self.item_use_time.fetch_sub(1, Ordering::Relaxed) <= 0
                {
                    let mut item_to_consume = item.clone();
                    let hand = if self.livings_flags.load(Relaxed) & Self::OFF_HAND_ACTIVE_FLAG != 0
                    {
                        Hand::Left
                    } else {
                        Hand::Right
                    };

                    let mut cancelled = false;
                    if let Some(player) = caller.get_player() {
                        if let Some(server) = player.world().server.upgrade()
                            && let Some(player_arc) = player.as_arc()
                        {
                            let event =
                                PlayerItemConsumeEvent::new(player_arc, item_to_consume.clone(), hand);
                            let event = server.plugin_manager.fire(event).await;
                            if event.cancelled {
                                cancelled = true;
                            } else {
                                item_to_consume = event.item_stack;
                            }
                        }
                    }

                    if !cancelled {
                        if let Some(food) = item_to_consume.get_data_component::<FoodImpl>()
                            && let Some(player) = caller.get_player()
                        {
                            player
                                .hunger_manager
                                .eat(player, food.nutrition as u8, food.saturation)
                                .await;
                        }
                        if let Some(player) = caller.get_player() {
                            let stack = if hand == Hand::Left {
                                player.inventory.off_hand_item().await
                            } else {
                                player.inventory.held_item()
                            };
                            let mut stack = stack.lock().await;
                            *stack = item_to_consume.clone();
                            stack.decrement_unless_creative(player.gamemode.load(), 1);
                        }
                    }

                    self.clear_active_hand().await;
                }
            }

            if self.hurt_cooldown.load(Relaxed) > 0 {
                self.hurt_cooldown.fetch_sub(1, Relaxed);
            }
            if self.health.load() <= 0.0 {
                let time = self.death_time.fetch_add(1, Relaxed);
                if time >= 20 && self.entity.is_alive() {
                    // Spawn Death particles
                    self.entity
                        .world
                        .load()
                        .send_entity_status(&self.entity, EntityStatus::AddDeathParticles)
                        .await;
                    self.entity.remove().await;
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(self)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}
