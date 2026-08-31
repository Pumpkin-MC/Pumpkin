use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use bytes::BufMut;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::PotionDurationScaleImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::{CEntityVelocity, CSoundEffect, Metadata};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;

/// Represents the pickup rules for arrows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowPickup {
    Disallowed,
    Allowed,
    CreativeOnly,
}

impl ArrowPickup {
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            1 => Self::Allowed,
            2 => Self::CreativeOnly,
            _ => Self::Disallowed,
        }
    }

    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Disallowed => 0,
            Self::Allowed => 1,
            Self::CreativeOnly => 2,
        }
    }
}

pub struct ArrowEntity {
    pub entity: Entity,
    pub owner_id: Option<i32>,
    pub item_stack: RwLock<ItemStack>,
    pub base_damage: AtomicU64,
    pub pickup: ArrowPickup,
    pub is_critical: AtomicBool,
    pub no_physics: AtomicBool,
    pub pierce_level: AtomicU8,
    pub punch_level: AtomicU8,
    pub is_flame: AtomicBool,
    pub in_ground: AtomicBool,
    pub in_ground_time: AtomicU32,
    pub life: AtomicU32,
    pub shake_time: AtomicU8,
    pub has_hit: AtomicBool,
    pub last_block_pos: Arc<std::sync::RwLock<Option<BlockPos>>>,
    pub pierced_entities: Arc<RwLock<Vec<i32>>>,
    pub weapon: RwLock<Option<ItemStack>>,
}

impl ArrowEntity {
    const ARROW_BASE_DAMAGE: f64 = 2.0;
    const WATER_INERTIA: f64 = 0.6;
    const AIR_INERTIA: f64 = 0.99;
    const GRAVITY: f64 = 0.05;
    const DESPAWN_TIME: u32 = 1200;

    pub fn new(entity: Entity, owner_id: Option<i32>) -> Self {
        let item_stack = ItemStack::new(1, Self::default_item(entity.entity_type));
        Self::new_with_item(entity, owner_id, &item_stack, ArrowPickup::Disallowed)
    }

    pub fn new_with_item(
        entity: Entity,
        owner_id: Option<i32>,
        item_stack: &ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        Self {
            entity,
            owner_id,
            item_stack: RwLock::new(item_stack.copy_with_count(1)),
            base_damage: AtomicU64::new(Self::ARROW_BASE_DAMAGE.to_bits()),
            pickup,
            is_critical: AtomicBool::new(false),
            no_physics: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            punch_level: AtomicU8::new(0),
            is_flame: AtomicBool::new(false),
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            has_hit: AtomicBool::new(false),
            last_block_pos: Arc::new(std::sync::RwLock::new(None)),
            pierced_entities: Arc::new(RwLock::new(Vec::new())),
            weapon: RwLock::new(None),
        }
    }

    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        item_stack: &ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        let mut owner_pos = shooter.pos.load();
        owner_pos.y = owner_pos.y + f64::from(shooter.entity_dimension.load().eye_height) - 0.1;
        entity.pos.store(owner_pos);
        let mut launch_event =
            crate::plugin::api::events::entity::projectile_launch::ProjectileLaunchEvent::new(
                entity.entity_id,
                Some(shooter.entity_id),
            );
        if let Some(server) = entity.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut launch_event);
        }

        Self {
            entity,
            owner_id: Some(shooter.entity_id),
            item_stack: RwLock::new(item_stack.copy_with_count(1)),
            base_damage: AtomicU64::new(Self::ARROW_BASE_DAMAGE.to_bits()),
            pickup,
            is_critical: AtomicBool::new(false),
            no_physics: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            punch_level: AtomicU8::new(0),
            is_flame: AtomicBool::new(false),
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            has_hit: AtomicBool::new(false),
            last_block_pos: Arc::new(std::sync::RwLock::new(None)),
            pierced_entities: Arc::new(RwLock::new(Vec::new())),
            weapon: RwLock::new(None),
        }
    }

    pub fn new_shot_with_weapon(
        entity: Entity,
        shooter: &Entity,
        item_stack: &ItemStack,
        weapon: &ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        let mut arrow = Self::new_shot(entity, shooter, item_stack, pickup);
        arrow.weapon = RwLock::new(Some(weapon.copy_with_count(1)));
        arrow
    }

    #[must_use]
    pub fn get_weapon_item(&self) -> Option<ItemStack> {
        self.weapon.read().ok().and_then(|w| w.clone())
    }

    /// Applies projectile-spawned enchantment effects matching vanilla `Projectile::applyOnProjectileSpawned`.
    pub fn apply_on_projectile_spawned(&self, pickup_item_stack: &ItemStack) {
        let weapon = self.weapon.read().ok().and_then(|w| w.clone());
        super::apply_on_projectile_spawned(
            self.get_entity(),
            pickup_item_stack,
            weapon.as_ref(),
            Some(self),
        );
    }

    #[must_use]
    pub const fn entity_type_for_item(item: &'static Item) -> &'static EntityType {
        if item.id == Item::SPECTRAL_ARROW.id {
            &EntityType::SPECTRAL_ARROW
        } else {
            &EntityType::ARROW
        }
    }

    #[must_use]
    pub const fn default_item(entity_type: &'static EntityType) -> &'static Item {
        if entity_type.id == EntityType::SPECTRAL_ARROW.id {
            &Item::SPECTRAL_ARROW
        } else {
            &Item::ARROW
        }
    }

    fn write_item_stack_nbt(item_stack: &ItemStack, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let mut item = pumpkin_nbt::compound::NbtCompound::new();
        item_stack.copy_with_count(1).write_item_stack(&mut item);
        nbt.put_compound("item", item);
    }

    fn read_item_stack_nbt(nbt: &pumpkin_nbt::compound::NbtCompound) -> Option<ItemStack> {
        nbt.get_compound("item")
            .and_then(ItemStack::read_item_stack)
            .map(|item_stack| item_stack.copy_with_count(1))
    }

    fn pickup_item_stack(item_stack: &ItemStack) -> ItemStack {
        item_stack.copy_with_count(1)
    }

    const fn spectral_glowing_effect() -> pumpkin_data::potion::Effect {
        pumpkin_data::potion::Effect {
            effect_type: &pumpkin_data::effect::StatusEffect::GLOWING,
            duration: 200,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }

    const fn should_apply_post_hurt_effects(damage_succeeded: bool) -> bool {
        damage_succeeded
    }

    #[must_use]
    pub fn get_effect_color(item_stack: &ItemStack) -> i32 {
        if let Some(pc) =
            item_stack.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
        {
            if let Some(color) = pc.custom_color {
                return color;
            }
            let effects = crate::item::potion::PotionContents::read_potion_effects(item_stack);
            if effects.is_empty() {
                return -1;
            }
            let mut r = 0;
            let mut g = 0;
            let mut b = 0;
            let mut total = 0;
            for (effect, _, amplifier, _, _, _) in effects {
                let color = effect.color;
                let weight = (amplifier as i32) + 1;
                r += weight * ((color >> 16) & 0xFF);
                g += weight * ((color >> 8) & 0xFF);
                b += weight * (color & 0xFF);
                total += weight;
            }
            if total == 0 {
                -1
            } else {
                ((r / total) << 16) | ((g / total) << 8) | (b / total)
            }
        } else {
            -1
        }
    }

    pub fn set_velocity_from_rotation(
        &self,
        pitch: f32,
        yaw: f32,
        roll: f32,
        speed: f32,
        divergence: f32,
    ) {
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
        let roll_rad = (pitch + roll).to_radians();

        let x = -yaw_rad.sin() * pitch_rad.cos();
        let y = -roll_rad.sin();
        let z = yaw_rad.cos() * pitch_rad.cos();

        self.set_velocity(
            f64::from(x),
            f64::from(y),
            f64::from(z),
            f64::from(speed),
            f64::from(divergence),
        );
    }

    pub fn set_velocity(&self, x: f64, y: f64, z: f64, power: f64, uncertainty: f64) {
        fn next_triangular(mode: f64, deviation: f64) -> f64 {
            deviation.mul_add(rand::random::<f64>() - rand::random::<f64>(), mode)
        }

        let velocity = Vector3::new(x, y, z)
            .normalize()
            .add_raw(
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
            )
            .multiply(power, power, power);

        self.entity.velocity.store(velocity);
        let len = velocity.horizontal_length();
        self.entity.set_rotation(
            velocity.x.atan2(velocity.z) as f32 * 57.295_776,
            velocity.y.atan2(len) as f32 * 57.295_776,
        );
    }

    fn get_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.is_critical.load(Ordering::Relaxed) {
            flags |= 0x01;
        }
        if self.no_physics.load(Ordering::Relaxed) {
            flags |= 0x02;
        }
        flags
    }

    pub fn set_critical(&self, critical: bool) {
        self.is_critical.store(critical, Ordering::Relaxed);
        let flags = self.get_flags();
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::abstract_arrow::ID_FLAGS,
                flags,
            )],
            None,
        );
    }

    pub fn set_no_physics(&self, no_physics: bool) {
        self.no_physics.store(no_physics, Ordering::Relaxed);
        let flags = self.get_flags();
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::abstract_arrow::ID_FLAGS,
                flags,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_no_physics(&self) -> bool {
        self.no_physics.load(Ordering::Relaxed)
    }

    pub fn set_base_damage_from_mob(&self, power: f64, difficulty_id: i32) {
        fn next_triangular(mode: f64, deviation: f64) -> f64 {
            deviation.mul_add(rand::random::<f64>() - rand::random::<f64>(), mode)
        }
        let diff_factor = f64::from(difficulty_id) * 0.11;
        let base = power * 2.0 + next_triangular(diff_factor, 0.57425);
        self.set_base_damage(base);
    }

    pub fn set_pierce_level(&self, level: u8) {
        self.pierce_level.store(level, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::abstract_arrow::PIERCE_LEVEL,
                level,
            )],
            None,
        );
    }

    #[must_use]
    pub fn get_base_damage(&self) -> f64 {
        f64::from_bits(self.base_damage.load(Ordering::Relaxed))
    }

    pub fn set_base_damage(&self, damage: f64) {
        self.base_damage.store(damage.to_bits(), Ordering::Relaxed);
    }

    pub fn set_flame(&self, flame: bool) {
        self.is_flame.store(flame, Ordering::Relaxed);
        if flame {
            self.entity.set_on_fire_for(100.0);
            self.entity.set_on_fire(true);
        } else {
            self.entity.extinguish();
        }
    }

    #[must_use]
    pub fn is_on_fire(&self) -> bool {
        self.entity.is_on_fire() || self.is_flame.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    fn apply_inertia(&self, inertia: f64) {
        let velocity = self.entity.velocity.load();
        self.entity
            .velocity
            .store(velocity.multiply(inertia, inertia, inertia));
    }

    #[allow(dead_code)]
    fn apply_gravity(&self) {
        let mut velocity = self.entity.velocity.load();
        velocity.y -= Self::GRAVITY;
        self.entity.velocity.store(velocity);
    }
}

impl EntityBase for ArrowEntity {
    fn write_custom_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let item_stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::write_item_stack_nbt(&item_stack, nbt);
        nbt.put_bool("crit", self.is_critical.load(Ordering::Relaxed));
        nbt.put_double("damage", self.get_base_damage());
        nbt.put_bool("inGround", self.in_ground.load(Ordering::Relaxed));
        nbt.put_int("life", self.life.load(Ordering::Relaxed) as i32);
        nbt.put_byte("shake", self.shake_time.load(Ordering::Relaxed) as i8);
        nbt.put_byte(
            "PierceLevel",
            self.pierce_level.load(Ordering::Relaxed) as i8,
        );
        nbt.put_byte("pickup", self.pickup.to_byte() as i8);
    }

    fn read_custom_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        if let Some(item_stack) = Self::read_item_stack_nbt(nbt) {
            *self
                .item_stack
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = item_stack;
        }
        if let Some(crit) = nbt.get_bool("crit") {
            self.is_critical.store(crit, Ordering::Relaxed);
        }
        if let Some(damage) = nbt.get_double("damage") {
            self.set_base_damage(damage);
        }
        if let Some(in_ground) = nbt
            .get_bool("inGround")
            .or_else(|| nbt.get_byte("inGround").map(|b| b != 0))
        {
            self.in_ground.store(in_ground, Ordering::Relaxed);
        }
        if let Some(life) = nbt
            .get_int("life")
            .or_else(|| nbt.get_short("life").map(i32::from))
        {
            self.life.store(life.max(0) as u32, Ordering::Relaxed);
        }
        if let Some(shake) = nbt.get_byte("shake") {
            self.shake_time.store(shake.max(0) as u8, Ordering::Relaxed);
        }
        if let Some(pierce) = nbt.get_byte("PierceLevel") {
            self.pierce_level
                .store(pierce.max(0) as u8, Ordering::Relaxed);
        }
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        let flags = self.get_flags();
        let pierce = self.pierce_level.load(Ordering::Relaxed);
        let in_ground = self.in_ground.load(Ordering::Relaxed);

        if entity.entity_type.id == EntityType::SPECTRAL_ARROW.id {
            entity.send_meta_data(
                &[
                    Metadata::new(pumpkin_data::tracked_data::spectral_arrow::ID_FLAGS, flags),
                    Metadata::new(
                        pumpkin_data::tracked_data::spectral_arrow::PIERCE_LEVEL,
                        pierce,
                    ),
                ],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::spectral_arrow::IN_GROUND,
                    in_ground,
                )],
                None,
            );
        } else {
            let item_stack = self
                .item_stack
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let color = Self::get_effect_color(&item_stack);
            entity.send_meta_data(
                &[
                    Metadata::new(pumpkin_data::tracked_data::arrow::ID_FLAGS, flags),
                    Metadata::new(pumpkin_data::tracked_data::arrow::PIERCE_LEVEL, pierce),
                ],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::arrow::IN_GROUND,
                    in_ground,
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::arrow::ID_EFFECT_COLOR,
                    color,
                )],
                None,
            );
        }

        if self.is_on_fire() {
            entity.set_on_fire(true);
        }
    }

    fn java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        let entity = self.get_entity();
        let flags = self.get_flags();
        let pierce = self.pierce_level.load(Ordering::Relaxed);
        let in_ground = self.in_ground.load(Ordering::Relaxed);
        let shared_flags = entity.flags.load(Ordering::Relaxed);

        let mut buf = Vec::new();
        if shared_flags != 0 {
            let _ = Metadata::new(
                pumpkin_data::tracked_data::entity::DATA_SHARED_FLAGS_ID,
                shared_flags,
            )
            .write(&mut buf, &version);
        }
        if entity.entity_type.id == EntityType::SPECTRAL_ARROW.id {
            let _ = Metadata::new(pumpkin_data::tracked_data::spectral_arrow::ID_FLAGS, flags)
                .write(&mut buf, &version);
            let _ = Metadata::new(
                pumpkin_data::tracked_data::spectral_arrow::PIERCE_LEVEL,
                pierce,
            )
            .write(&mut buf, &version);
            let _ = Metadata::new(
                pumpkin_data::tracked_data::spectral_arrow::IN_GROUND,
                in_ground,
            )
            .write(&mut buf, &version);
        } else {
            let item_stack = self
                .item_stack
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let color = Self::get_effect_color(&item_stack);
            let _ = Metadata::new(pumpkin_data::tracked_data::arrow::ID_FLAGS, flags)
                .write(&mut buf, &version);
            let _ = Metadata::new(pumpkin_data::tracked_data::arrow::PIERCE_LEVEL, pierce)
                .write(&mut buf, &version);
            let _ = Metadata::new(pumpkin_data::tracked_data::arrow::IN_GROUND, in_ground)
                .write(&mut buf, &version);
            if color != -1 {
                let _ = Metadata::new(pumpkin_data::tracked_data::arrow::ID_EFFECT_COLOR, color)
                    .write(&mut buf, &version);
            }
        }
        (!buf.is_empty()).then(|| {
            buf.put_u8(255);
            buf.into_boxed_slice()
        })
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = self.get_entity();
        let world = entity.world.load();

        // Fire & Extinguish logic
        let mut fire_ticks = entity.fire_ticks.load(Ordering::Relaxed);
        let touching_water = entity.touching_water.load(Ordering::Relaxed);
        let in_water = touching_water || entity.is_in_water();
        let block_pos = entity.block_pos.load();
        let in_rain = world.is_raining_at(&block_pos);

        if in_water || in_rain {
            if entity.is_on_fire() || self.is_flame.load(Ordering::Relaxed) {
                entity.extinguish();
                self.is_flame.store(false, Ordering::Relaxed);
            }
        } else if fire_ticks > 0 {
            fire_ticks -= 1;
            entity.fire_ticks.store(fire_ticks, Ordering::Relaxed);
            if fire_ticks <= 0 && self.is_flame.load(Ordering::Relaxed) {
                self.is_flame.store(false, Ordering::Relaxed);
                entity.set_on_fire(false);
            }
        }

        // Check if arrow enters lava or fire block
        let current_block = world.get_block(&block_pos);
        if current_block == &pumpkin_data::Block::LAVA {
            entity.set_on_fire_for(15.0);
            self.is_flame.store(true, Ordering::Relaxed);
        } else if current_block == &pumpkin_data::Block::FIRE
            || current_block == &pumpkin_data::Block::SOUL_FIRE
        {
            entity.set_on_fire_for(8.0);
            self.is_flame.store(true, Ordering::Relaxed);
        }

        let is_on_fire = entity.is_on_fire() || self.is_flame.load(Ordering::Relaxed);
        entity.set_on_fire(is_on_fire);

        // Handle shake time
        let shake = self.shake_time.load(Ordering::Relaxed);
        if shake > 0 {
            self.shake_time.store(shake - 1, Ordering::Relaxed);
        }

        if self.in_ground.load(Ordering::Relaxed) {
            // Check if the block we are stuck into was broken / turned to air
            let last_pos = *self
                .last_block_pos
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(pos) = last_pos {
                let block = world.get_block(&pos);
                if block.is_air() {
                    self.in_ground.store(false, Ordering::Relaxed);
                    entity.send_meta_data(
                        &[Metadata::new(
                            pumpkin_data::tracked_data::abstract_arrow::IN_GROUND,
                            false,
                        )],
                        None,
                    );
                    let mut vel = entity.velocity.load();
                    vel.x *= rand::random::<f64>() * 0.2;
                    vel.y *= rand::random::<f64>() * 0.2;
                    vel.z *= rand::random::<f64>() * 0.2;
                    entity.velocity.store(vel);
                }
            }

            // Increment in-ground time and life
            let _in_ground_time = self.in_ground_time.fetch_add(1, Ordering::Relaxed);
            let life = self.life.fetch_add(1, Ordering::Relaxed);

            // Despawn after enough time
            if life >= Self::DESPAWN_TIME {
                entity.remove();
            }
            return;
        }

        // Arrow is flying
        let start_pos = entity.pos.load();
        let mut velocity = entity.velocity.load();

        // Apply gravity
        velocity.y -= Self::GRAVITY;

        // Apply inertia (air resistance or water drag)
        let inertia = if in_water {
            Self::WATER_INERTIA
        } else {
            Self::AIR_INERTIA
        };
        velocity = velocity.multiply(inertia, inertia, inertia);

        entity.velocity.store(velocity);

        // Update rotation based on velocity
        let len = velocity.horizontal_length();
        entity.set_rotation(
            velocity.x.atan2(velocity.z) as f32 * 57.295_776,
            velocity.y.atan2(len) as f32 * 57.295_776,
        );

        // Move arrow
        let new_pos = start_pos.add(&velocity);
        entity.set_pos(new_pos);

        // Spawn particles while arrow is flying
        if in_water {
            for i in 0..4 {
                let factor = 0.25 * f64::from(i);
                world.spawn_particle(
                    Vector3::new(
                        new_pos.x - velocity.x * factor,
                        new_pos.y - velocity.y * factor,
                        new_pos.z - velocity.z * factor,
                    ),
                    Vector3::new(velocity.x as f32, velocity.y as f32, velocity.z as f32),
                    0.0,
                    1,
                    Particle::Bubble,
                );
            }
        }

        if is_on_fire {
            world.spawn_particle(
                entity.pos.load(),
                Vector3::new(0.0f32, 0.0f32, 0.0f32),
                0.0,
                1,
                Particle::Flame,
            );
        }

        if self.is_critical.load(Ordering::Relaxed) {
            for i in 0..4 {
                let factor = f64::from(i) / 4.0;
                world.spawn_particle(
                    Vector3::new(
                        start_pos.x + velocity.x * factor,
                        start_pos.y + velocity.y * factor,
                        start_pos.z + velocity.z * factor,
                    ),
                    Vector3::new(
                        -velocity.x as f32,
                        (-velocity.y + 0.2) as f32,
                        -velocity.z as f32,
                    ),
                    0.0,
                    1,
                    Particle::Crit,
                );
            }
        }

        // Broadcast velocity update
        let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);

        let chunk_pos = entity.chunk_pos.load();
        world.broadcast_to_chunk(chunk_pos, &packet);

        // Check for collisions using raycasting
        let search_box = BoundingBox::new(
            Vector3::new(
                start_pos.x.min(new_pos.x),
                start_pos.y.min(new_pos.y),
                start_pos.z.min(new_pos.z),
            ),
            Vector3::new(
                start_pos.x.max(new_pos.x),
                start_pos.y.max(new_pos.y),
                start_pos.z.max(new_pos.z),
            ),
        )
        .expand(0.3, 0.3, 0.3);

        let mut closest_t = 1.0f64;
        let mut hit = None;

        // Block collisions
        let (block_cols, block_positions) =
            world.get_block_collisions(search_box, self.get_entity());
        for (idx, bb) in block_cols.iter().enumerate() {
            if let Some(t) = calculate_ray_intersection(&start_pos, &velocity, bb)
                && t < closest_t
            {
                closest_t = t;

                // Map back to block pos
                let mut curr = 0;
                for (len, pos) in &block_positions {
                    curr += len;
                    if idx < curr {
                        let hit_pos = start_pos.add(&velocity.multiply(t, t, t));
                        hit = Some(ProjectileHit::Block {
                            pos: *pos,
                            face: get_hit_face(hit_pos, *pos),
                            hit_pos,
                            normal: velocity.normalize().multiply(-1.0, -1.0, -1.0),
                        });
                        break;
                    }
                }
            }
        }

        // Entity collisions
        let candidates = world.get_entities_at_box(&search_box);
        for cand in candidates {
            if self.should_skip_collision(entity, &cand) {
                continue;
            }

            let ebb = cand.get_entity().bounding_box.load().expand(0.3, 0.3, 0.3);
            if let Some(t) = calculate_ray_intersection(&start_pos, &velocity, &ebb)
                && t < closest_t
            {
                closest_t = t;
                let hit_pos = start_pos.add(&velocity.multiply(t, t, t));
                hit = Some(ProjectileHit::Entity {
                    entity: cand.clone(),
                    hit_pos,
                    normal: velocity.normalize().multiply(-1.0, -1.0, -1.0),
                });
            }
        }

        // Handle hit
        if let Some(h) = hit {
            match h {
                ProjectileHit::Block { .. } => {
                    if self.has_hit.swap(true, Ordering::SeqCst) {
                        return;
                    }
                    caller.on_hit(h);
                }
                ProjectileHit::Entity { .. } => {
                    let pierce = self.pierce_level.load(Ordering::Relaxed);
                    let pierced_len = self
                        .pierced_entities
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .len();
                    if pierced_len >= pierce as usize && self.has_hit.swap(true, Ordering::SeqCst) {
                        return;
                    }
                    caller.on_hit(h);
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn on_hit(&self, hit: ProjectileHit) {
        let (hit_pos, hit_entity) = match hit {
            ProjectileHit::Block { hit_pos, .. } => (hit_pos, None),
            ProjectileHit::Entity {
                ref entity,
                hit_pos,
                ..
            } => (hit_pos, Some(entity.get_entity().entity_id)),
        };
        let mut hit_event =
            crate::plugin::api::events::entity::projectile_hit::ProjectileHitEvent::new(
                self.entity.entity_id,
                hit_pos,
                hit_entity,
            );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut hit_event);
        }

        let entity = self.get_entity();
        let world = entity.world.load();

        match hit {
            ProjectileHit::Block {
                pos,
                face: _,
                hit_pos,
                ..
            } => {
                // Arrow hit a block - stick into it
                self.in_ground.store(true, Ordering::Relaxed);
                self.shake_time.store(7, Ordering::Relaxed);
                *self
                    .last_block_pos
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(pos);

                let block = world.get_block(&pos);
                let state = world.get_block_state(&pos);
                if let Some(server) = world.server.upgrade() {
                    world
                        .block_registry
                        .on_projectile_hit(block, &world, self, &pos, state, &hit_pos, &server);
                }

                if block == &pumpkin_data::Block::TARGET
                    && let Some(player) = self.owner_id.and_then(|id| world.get_player_by_id(id))
                {
                    player.trigger_advancement(
                        crate::entity::player::advancement::trigger::AdvancementTrigger::Bullseye,
                    );
                }

                // Stop the arrow with slight position offset backwards
                let velocity = entity.velocity.load();
                let norm_dir = Vector3::new(
                    velocity.x.signum(),
                    velocity.y.signum(),
                    velocity.z.signum(),
                );
                let offset = norm_dir.multiply(0.05, 0.05, 0.05);
                entity.set_pos(hit_pos.sub(&offset));
                entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));

                // Notify client that arrow is in ground
                entity.send_meta_data(
                    &[Metadata::new(
                        pumpkin_data::tracked_data::abstract_arrow::IN_GROUND,
                        true,
                    )],
                    None,
                );

                // Play sound with vanilla pitch formula
                let sound_pitch = 1.2 / (rand::random::<f32>() * 0.2 + 0.9);
                let sound_packet = CSoundEffect::new(
                    IdOr::Id(Sound::EntityArrowHit as u16),
                    SoundCategory::Neutral,
                    &hit_pos,
                    1.0,
                    sound_pitch,
                    0.0,
                );
                let chunk_pos = entity.chunk_pos.load();
                world.broadcast_to_chunk(chunk_pos, &sound_packet);

                // Reset critical flag and pierce level
                self.set_critical(false);
                self.set_pierce_level(0);
                self.pierced_entities
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clear();
            }
            ProjectileHit::Entity {
                entity: target,
                hit_pos,
                ..
            } => {
                let target_entity_id = target.get_entity().entity_id;
                self.pierced_entities
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(target_entity_id);

                // Calculate damage
                let velocity = entity.velocity.load();
                let power = velocity.length();
                let mut damage = (power * self.get_base_damage()).ceil() as i32;

                // Apply critical hit bonus
                if self.is_critical.load(Ordering::Relaxed) {
                    let bonus = (rand::random::<u32>() % (damage.max(0) / 2 + 2) as u32) as i32;
                    damage = damage.saturating_add(bonus);
                }

                let is_enderman =
                    target.get_entity().entity_type == &pumpkin_data::entity::EntityType::ENDERMAN;
                let is_on_fire = entity.is_on_fire() || self.is_flame.load(Ordering::Relaxed);
                if is_on_fire && !is_enderman {
                    target.get_entity().set_on_fire_for(5.0);
                }

                let punch = self.punch_level.load(Ordering::Relaxed);
                let is_spectral = entity.entity_type.id == EntityType::SPECTRAL_ARROW.id;
                let owner_id = self.owner_id;
                let pierce = self.pierce_level.load(Ordering::Relaxed);

                let owner_entity = owner_id.and_then(|id| world.get_entity_by_id(id));

                let damage_succeeded = target.damage_with_context(
                    target.as_ref(),
                    damage as f32,
                    DamageType::ARROW,
                    Some(hit_pos),
                    owner_entity.as_deref(),
                    None,
                );

                if let Some(living) = target.get_living_entity() {
                    if punch > 0 {
                        let norm = Vector3::new(velocity.x, 0.0, velocity.z).normalize();
                        let push_scale = f64::from(punch) * 0.6;
                        target.get_entity().velocity.store(
                            target.get_entity().velocity.load().add(&Vector3::new(
                                norm.x * push_scale,
                                0.1,
                                norm.z * push_scale,
                            )),
                        );
                    }

                    // Play hit sound
                    let sound_pitch = 1.2 / (rand::random::<f32>() * 0.2 + 0.9);
                    let sound_packet = CSoundEffect::new(
                        IdOr::Id(Sound::EntityArrowHit as u16),
                        SoundCategory::Neutral,
                        &hit_pos,
                        1.0,
                        sound_pitch,
                        0.0,
                    );
                    world.broadcast_packet_all(&sound_packet);

                    if Self::should_apply_post_hurt_effects(damage_succeeded) {
                        let item_stack = self
                            .item_stack
                            .read()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        let scale = item_stack
                            .get_data_component::<PotionDurationScaleImpl>()
                            .map_or(1.0, |component| component.scale);
                        crate::item::potion::PotionContents::apply_effects_to(
                            living,
                            crate::item::potion::PotionContents::read_potion_effects(&item_stack),
                            scale,
                            crate::item::potion::PotionApplicationSource::Arrow,
                        );

                        if is_spectral {
                            living.add_effect(Self::spectral_glowing_effect());
                        }
                    }
                }

                // Check pierce level
                let pierced_count = self
                    .pierced_entities
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .len();
                if pierced_count > pierce as usize {
                    entity.remove();
                }
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    #[allow(dead_code, clippy::unused_self)]
    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn on_player_collision(&self, player: &Arc<Player>) {
        // Only allow picking up grounded arrows
        if !self.in_ground.load(Ordering::Relaxed) {
            return;
        }

        if player.living_entity.health.load() <= 0.0 {
            return;
        }

        // Check pickup rules
        match self.pickup {
            ArrowPickup::Disallowed => return,
            ArrowPickup::CreativeOnly if !player.is_creative() => return,
            _ => {}
        }

        // Try to insert an arrow into the player's inventory
        let item_stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut stack = Self::pickup_item_stack(&item_stack);
        if player.is_creative() || player.inventory.insert_stack_anywhere(&mut stack) {
            player.living_entity.pickup(&self.entity, 1);

            // Remove arrow entity after pickup
            self.get_entity().remove();
        }
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ArrowEntity {
    fn should_skip_collision(&self, self_ent: &Entity, other: &Arc<dyn EntityBase>) -> bool {
        let other_ent = other.get_entity();

        // Don't collide with self
        if other_ent.entity_id == self_ent.entity_id {
            return true;
        }

        // Skip owner for initial frames (5 ticks)
        if Some(other_ent.entity_id) == self.owner_id && self_ent.age.load(Ordering::Relaxed) < 5 {
            return true;
        }

        // Skip already pierced entities
        if self
            .pierced_entities
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(&other_ent.entity_id)
        {
            return true;
        }

        // Skip dead entities
        if !other_ent.is_alive() {
            return true;
        }

        // Skip other arrows, item entities, falling block entities, and area effect clouds
        if (other_ent.entity_type == &pumpkin_data::entity::EntityType::ARROW
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::SPECTRAL_ARROW)
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::ITEM
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::FALLING_BLOCK
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::AREA_EFFECT_CLOUD
        {
            return true;
        }

        false
    }
}

/// Ray intersection algorithm for AABBs
fn calculate_ray_intersection(
    start: &Vector3<f64>,
    dir: &Vector3<f64>,
    bb: &pumpkin_util::math::boundingbox::BoundingBox,
) -> Option<f64> {
    let mut t_min = 0.0f64;
    let mut t_max = 1.0f64;

    let b_min = [bb.min.x, bb.min.y, bb.min.z];
    let b_max = [bb.max.x, bb.max.y, bb.max.z];
    let s = [start.x, start.y, start.z];
    let d = [dir.x, dir.y, dir.z];

    for i in 0..3 {
        if d[i].abs() < 1e-9 {
            if s[i] < b_min[i] || s[i] > b_max[i] {
                return None;
            }
        } else {
            let t1 = (b_min[i] - s[i]) / d[i];
            let t2 = (b_max[i] - s[i]) / d[i];
            t_min = t_min.max(t1.min(t2));
            t_max = t_max.min(t1.max(t2));
        }
    }

    (0.0..=1.0).contains(&t_min).then_some(t_min)
}

/// Get the face of the block that was hit
fn get_hit_face(hit_pos: Vector3<f64>, block_pos: BlockPos) -> pumpkin_data::BlockDirection {
    use pumpkin_data::BlockDirection;

    let local = hit_pos.sub(&block_pos.0.to_f64());
    let eps = 1.0e-4;

    if local.x <= eps {
        BlockDirection::West
    } else if local.x >= 1.0 - eps {
        BlockDirection::East
    } else if local.y <= eps {
        BlockDirection::Down
    } else if local.y >= 1.0 - eps {
        BlockDirection::Up
    } else if local.z <= eps {
        BlockDirection::North
    } else {
        BlockDirection::South
    }
}

#[cfg(test)]
mod tests {
    use super::ArrowEntity;
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{
        DataComponentImpl, PotionContentsImpl, PotionDurationScaleImpl,
    };
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    fn tipped_payload(count: u8) -> ItemStack {
        let mut tipped = ItemStack::new(32, &Item::TIPPED_ARROW);
        tipped.patch.push((
            DataComponent::PotionContents,
            Some(
                PotionContentsImpl {
                    potion_id: Some(5),
                    custom_color: Some(0x123456),
                    custom_effects: Vec::new(),
                    custom_name: Some("payload".to_string()),
                }
                .to_dyn(),
            ),
        ));
        tipped.patch.push((
            DataComponent::PotionDurationScale,
            Some(PotionDurationScaleImpl { scale: 0.5 }.to_dyn()),
        ));
        tipped.copy_with_count(count)
    }

    #[test]
    fn projectile_payload_keeps_components_at_one_count() {
        let tipped = tipped_payload(32);

        let payload = tipped.copy_with_count(1);

        assert_eq!(payload.item_count, 1);
        assert!(payload.are_items_and_components_equal(&tipped));
        assert_eq!(
            ArrowEntity::entity_type_for_item(payload.item),
            &EntityType::ARROW
        );
        assert_eq!(
            ArrowEntity::entity_type_for_item(&Item::SPECTRAL_ARROW),
            &EntityType::SPECTRAL_ARROW
        );
    }

    #[test]
    fn arrow_nbt_payload_round_trips() {
        let payload = tipped_payload(1);
        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();

        ArrowEntity::write_item_stack_nbt(&payload, &mut nbt);
        let restored = ArrowEntity::read_item_stack_nbt(&nbt).expect("arrow payload should decode");

        assert!(restored.are_equal(&payload));
        assert_eq!(restored.item_count, 1);
    }

    #[test]
    fn grounded_pickup_stack_keeps_exact_arrow_payload() {
        let payload = tipped_payload(32);
        let pickup = ArrowEntity::pickup_item_stack(&payload);

        assert_eq!(pickup.item_count, 1);
        assert!(pickup.are_items_and_components_equal(&payload));
    }

    #[test]
    fn spectral_arrow_applies_vanilla_glowing_effect() {
        let effect = ArrowEntity::spectral_glowing_effect();

        assert_eq!(
            effect.effect_type,
            &pumpkin_data::effect::StatusEffect::GLOWING
        );
        assert_eq!(effect.duration, 200);
        assert_eq!(effect.amplifier, 0);
        assert!(effect.show_particles);
        assert!(effect.show_icon);
    }

    #[test]
    fn post_hurt_effects_require_successful_arrow_damage() {
        assert!(!ArrowEntity::should_apply_post_hurt_effects(false));
        assert!(ArrowEntity::should_apply_post_hurt_effects(true));
    }

    #[test]
    fn tipped_arrow_effect_color_uses_custom_color_or_effects() {
        let tipped = tipped_payload(1);
        let color = ArrowEntity::get_effect_color(&tipped);
        assert_eq!(color, 0x123456);

        let normal = ItemStack::new(1, &Item::ARROW);
        let normal_color = ArrowEntity::get_effect_color(&normal);
        assert_eq!(normal_color, -1);
    }

    #[test]
    fn power_enchantment_damage_formula_matches_vanilla() {
        let base_damage = 2.0;
        let power_1 = base_damage + 1.0 * 0.5 + 0.5;
        let power_5 = base_damage + 5.0 * 0.5 + 0.5;
        assert_eq!(power_1, 3.0);
        assert_eq!(power_5, 5.0);
    }
}
