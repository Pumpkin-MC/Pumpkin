use crate::block::blocks::bubble_column;
use crate::entity::projectile::owner::ProjectileOwner;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

use crate::entity::projectile::ProjectileHit;
use crate::entity::projectile::abstract_arrow::{self, AbstractArrow, ArrowFlight};
use crate::entity::projectile_deflection::{ProjectileDeflection, deflect};
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
use pumpkin_data::sound::Sound;
use pumpkin_protocol::java::client::play::{CGameEvent, GameEvent, Metadata};
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
    pub owner: ProjectileOwner,
    pub item_stack: RwLock<ItemStack>,
    pub base_damage: AtomicU64,
    pub pickup: ArrowPickup,
    pub is_critical: AtomicBool,
    pub pierce_level: AtomicU8,
    pub punch_level: AtomicU8,
    pub flight: ArrowFlight,
    pub weapon: RwLock<Option<ItemStack>>,
}

impl ArrowEntity {
    const ARROW_BASE_DAMAGE: f64 = 2.0;
    const WATER_INERTIA: f64 = 0.6;
    /// Seconds a flame arrow burns.
    const FLAME_SECONDS: f32 = 100.0;
    /// Seconds a burning arrow sets its target on fire.
    const IGNITE_SECONDS: f32 = 5.0;

    pub fn new(entity: Entity, owner: Option<&Entity>) -> Self {
        let item_stack = ItemStack::new(1, Self::default_item(entity.entity_type));
        Self::new_with_item(entity, owner, &item_stack, ArrowPickup::Disallowed)
    }

    pub fn new_with_item(
        entity: Entity,
        owner: Option<&Entity>,
        item_stack: &ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        Self {
            entity,
            owner: ProjectileOwner::new(owner),
            item_stack: RwLock::new(item_stack.copy_with_count(1)),
            base_damage: AtomicU64::new(Self::ARROW_BASE_DAMAGE.to_bits()),
            pickup,
            is_critical: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            punch_level: AtomicU8::new(0),
            flight: ArrowFlight::new(),
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
        entity.set_pos(owner_pos);
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
            owner: ProjectileOwner::new(Some(shooter)),
            item_stack: RwLock::new(item_stack.copy_with_count(1)),
            base_damage: AtomicU64::new(Self::ARROW_BASE_DAMAGE.to_bits()),
            pickup,
            is_critical: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            punch_level: AtomicU8::new(0),
            flight: ArrowFlight::new(),
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
        if let Some(item) = item_stack.copy_with_count(1).to_nbt() {
            nbt.put_compound("item", item);
        }
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
        if self.is_no_physics() {
            flags |= 0x02;
        }
        flags
    }

    pub fn set_critical(&self, critical: bool) {
        self.is_critical.store(critical, Ordering::Relaxed);
        let flags = self.get_flags();
        self.entity
            .set_synced_data(pumpkin_data::tracked_data::abstract_arrow::ID_FLAGS, flags);
    }

    pub fn set_no_physics(&self, no_physics: bool) {
        self.entity.no_physics.store(no_physics, Ordering::Relaxed);
        let flags = self.get_flags();
        self.entity
            .set_synced_data(pumpkin_data::tracked_data::abstract_arrow::ID_FLAGS, flags);
    }

    #[must_use]
    pub fn is_no_physics(&self) -> bool {
        self.entity.no_physics.load(Ordering::Relaxed)
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
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::abstract_arrow::PIERCE_LEVEL,
            level,
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
        if flame {
            self.entity.set_on_fire_for(Self::FLAME_SECONDS);
        } else {
            self.entity.extinguish();
        }
    }

    #[must_use]
    pub fn is_on_fire(&self) -> bool {
        self.entity.is_on_fire()
    }
}

impl EntityBase for ArrowEntity {
    fn on_above_bubble_column(&self, drag_down: bool, pos: &BlockPos) {
        if !self.flight.is_in_ground() {
            let entity = self.get_entity();
            bubble_column::push_projectile(entity, drag_down, true);
            bubble_column::send_bubble_column_particles(&entity.world.load(), pos);
        }
    }

    fn on_inside_bubble_column(&self, drag_down: bool) {
        if !self.flight.is_in_ground() {
            bubble_column::push_projectile(self.get_entity(), drag_down, false);
        }
    }

    fn should_bounce_on_world_border(&self) -> bool {
        true
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.owner)
    }

    fn write_custom_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let item_stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::write_item_stack_nbt(&item_stack, nbt);
        nbt.put_bool("crit", self.is_critical.load(Ordering::Relaxed));
        nbt.put_double("damage", self.get_base_damage());
        self.flight.write_nbt(nbt);
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
        self.flight.read_nbt(nbt);
        if let Some(pierce) = nbt.get_byte("PierceLevel") {
            self.pierce_level
                .store(pierce.max(0) as u8, Ordering::Relaxed);
        }
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        let flags = self.get_flags();
        let pierce = self.pierce_level.load(Ordering::Relaxed);
        let in_ground = self.flight.is_in_ground();

        if entity.entity_type.id == EntityType::SPECTRAL_ARROW.id {
            entity.set_synced_data(pumpkin_data::tracked_data::spectral_arrow::ID_FLAGS, flags);
            entity.set_synced_data(
                pumpkin_data::tracked_data::spectral_arrow::PIERCE_LEVEL,
                pierce,
            );
            entity.set_synced_data(
                pumpkin_data::tracked_data::spectral_arrow::IN_GROUND,
                in_ground,
            );
        } else {
            let item_stack = self
                .item_stack
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let color = Self::get_effect_color(&item_stack);
            entity.set_synced_data(pumpkin_data::tracked_data::arrow::ID_FLAGS, flags);
            entity.set_synced_data(pumpkin_data::tracked_data::arrow::PIERCE_LEVEL, pierce);
            entity.set_synced_data(pumpkin_data::tracked_data::arrow::IN_GROUND, in_ground);
            entity.set_synced_data(pumpkin_data::tracked_data::arrow::ID_EFFECT_COLOR, color);
        }

        if self.is_on_fire() {
            entity.set_on_fire(true);
        }
    }

    fn java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        let entity = self.get_entity();
        let flags = self.get_flags();
        let pierce = self.pierce_level.load(Ordering::Relaxed);
        let in_ground = self.flight.is_in_ground();
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

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        abstract_arrow::tick(self, caller, server);
    }

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

        match hit {
            ProjectileHit::Block { pos, .. } => self.on_hit_block(&pos),
            ProjectileHit::Entity {
                entity: target,
                hit_pos,
                ..
            } => self.on_hit_entity(&target, hit_pos),
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    #[allow(dead_code, clippy::unused_self)]
    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn receives_player_touch(&self) -> bool {
        true
    }

    fn on_player_collision(&self, player: &Arc<Player>) {
        if !(self.flight.is_in_ground() || self.is_no_physics())
            || self.flight.shake_time.load(Ordering::Relaxed) > 0
        {
            return;
        }

        // Check pickup rules
        match self.pickup {
            ArrowPickup::Disallowed => return,
            ArrowPickup::CreativeOnly if !player.is_creative() => return,
            _ => {}
        }

        if let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id)
            && let Some(server) = player.world().server.upgrade()
        {
            let mut event = crate::plugin::api::events::player::player_pickup_arrow::PlayerPickupArrowEvent::new(
                player_arc,
                self.entity.entity_id,
            );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }

        // Pickup callbacks may teleport or respawn the player.
        if player.living_entity.health.load() <= 0.0
            || player.is_spectator()
            || player.world().uuid != self.entity.world.load().uuid
            || !crate::world::pickup_area(player).intersects(&self.entity.bounding_box.load())
        {
            return;
        }

        // Try to insert an arrow into the player's inventory
        let item_stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut stack = Self::pickup_item_stack(&item_stack);
        if player.is_creative() || player.inventory.insert_stack_anywhere(&mut stack) {
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::PickedUp,
                stack.item.id as i32,
                1,
            );
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
    fn on_hit_block(&self, pos: &BlockPos) {
        let entity = self.get_entity();
        let world = entity.world.load();
        if world.get_block(pos) == &pumpkin_data::Block::TARGET
            && let Some(player) = self.owner.get_player(&world)
        {
            player.trigger_advancement(
                crate::entity::player::advancement::trigger::AdvancementTrigger::Bullseye,
            );
        }

        abstract_arrow::stick_in_block(self, Sound::EntityArrowHit);
        self.set_critical(false);
        self.set_pierce_level(0);
    }

    fn on_hit_entity(&self, target: &Arc<dyn EntityBase>, hit_pos: Vector3<f64>) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let velocity = entity.velocity.load();
        let mut damage = (velocity.length() * self.get_base_damage())
            .clamp(0.0, f64::from(i32::MAX))
            .ceil() as i32;

        let pierce = self.pierce_level.load(Ordering::Relaxed);
        if pierce > 0 {
            let mut pierced = self
                .flight
                .pierced
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if pierced.len() > usize::from(pierce) {
                drop(pierced);
                entity.remove();
                return;
            }
            pierced.push(target.get_entity().entity_id);
        }

        if self.is_critical.load(Ordering::Relaxed) {
            let bonus = rand::random_range(0..damage / 2 + 2);
            damage = damage.saturating_add(bonus);
        }

        let target_entity = target.get_entity();
        let fire_ticks = target_entity.fire_ticks.load(Ordering::Relaxed);
        if self.is_on_fire() {
            target_entity.set_on_fire_for(Self::IGNITE_SECONDS);
        }

        let owner = self.owner.get(&world);
        let hurt = target.damage_with_context(
            target.as_ref(),
            damage as f32,
            DamageType::ARROW,
            Some(hit_pos),
            Some(self),
            // An ownerless arrow is its own cause.
            Some(owner.as_deref().unwrap_or(self)),
        );

        if hurt {
            if let Some(living) = target.get_living_entity() {
                if pierce == 0 {
                    living.set_arrow_count(living.get_arrow_count() + 1);
                }
                self.do_knockback(living);
                self.do_post_hurt_effects(living);
                if let Some(hit_player) = target.get_player()
                    && let Some(shooter) = self.owner.get_player(&world)
                    && !entity.silent.load(Ordering::Relaxed)
                    && hit_player.entity_id() != shooter.entity_id()
                {
                    shooter
                        .try_send_client_packet(&CGameEvent::new(GameEvent::ArrowHitPlayer, 0.0));
                }
            }
            self.play_entity_sound(
                Sound::EntityArrowHit,
                1.0,
                abstract_arrow::hit_sound_pitch(),
            );
            if pierce == 0 {
                entity.remove();
            }
        } else if target.projectile_receives_side_effects_on_hit(false) {
            target_entity
                .fire_ticks
                .store(fire_ticks, Ordering::Relaxed);
            deflect(
                self,
                ProjectileDeflection::Reverse,
                Some(target.as_ref()),
                self.owner.uuid(),
                false,
                Vector3::new(0.2, 0.2, 0.2),
            );
            if entity.velocity.load().length_squared() < 1.0e-7 {
                if self.pickup == ArrowPickup::Allowed {
                    let item_stack = self
                        .item_stack
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    entity.spawn_at_location(Self::pickup_item_stack(&item_stack));
                }
                entity.remove();
            }
        }
    }

    /// Effects for a target the arrow hurt: the tipped potion, or glowing from a spectral arrow.
    fn do_post_hurt_effects(&self, living: &LivingEntity) {
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
            None,
        );
        if self.entity.entity_type.id == EntityType::SPECTRAL_ARROW.id {
            living.add_effect(Self::spectral_glowing_effect());
        }
    }

    /// Punch pushes the target along the arrow's flight, less for knockback resistant mobs.
    fn do_knockback(&self, living: &LivingEntity) {
        let punch = self.punch_level.load(Ordering::Relaxed);
        if punch == 0 {
            return;
        }
        let resistance = (1.0
            - living
                .get_attribute_value(&pumpkin_data::attributes::Attributes::KNOCKBACK_RESISTANCE))
        .max(0.0);
        let velocity = self.entity.velocity.load();
        let push = Vector3::new(velocity.x, 0.0, velocity.z).normalize()
            * (f64::from(punch) * 0.6 * resistance);
        if push.length_squared() > 0.0 {
            let target = &living.entity;
            target
                .velocity
                .store(target.velocity.load() + Vector3::new(push.x, 0.1, push.z));
            target.velocity_dirty.store(true, Ordering::Relaxed);
        }
    }
}

impl AbstractArrow for ArrowEntity {
    fn flight(&self) -> &ArrowFlight {
        &self.flight
    }

    fn owner(&self) -> &ProjectileOwner {
        &self.owner
    }

    fn water_inertia(&self) -> f64 {
        Self::WATER_INERTIA
    }

    fn pierce_level(&self) -> u8 {
        self.pierce_level.load(Ordering::Relaxed)
    }

    fn set_in_ground(&self, in_ground: bool) {
        self.flight.in_ground.store(in_ground, Ordering::Relaxed);
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::abstract_arrow::IN_GROUND,
            in_ground,
        );
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
