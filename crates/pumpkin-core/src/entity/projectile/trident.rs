use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use bytes::BufMut;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::IntangibleProjectileImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;

use super::ProjectileHit;
use super::abstract_arrow::{self, AbstractArrow, ArrowFlight, get_entity_hit_result};
use super::arrow::ArrowPickup;
use crate::block::blocks::bubble_column;
use crate::enchantment::EnchantmentHelper;
use crate::entity::projectile::owner::ProjectileOwner;
use crate::entity::projectile_deflection::{ProjectileDeflection, deflect};
use crate::entity::{Entity, EntityBase, living::LivingEntity, player::Player};
use crate::server::Server;

pub struct TridentEntity {
    pub entity: Entity,
    pub owner: ProjectileOwner,
    pub item_stack: RwLock<ItemStack>,
    pub pickup: AtomicCell<ArrowPickup>,
    pub flight: ArrowFlight,
    /// Set once the trident hit an entity or lay in the ground for a moment, it can't hurt
    /// anything after that.
    dealt_damage: AtomicBool,
    is_critical: AtomicBool,
    pierce_level: AtomicU8,
    loyalty: AtomicU8,
    foil: AtomicBool,
    return_ticks: AtomicU32,
    base_damage: AtomicCell<f64>,
}

impl TridentEntity {
    const BASE_DAMAGE: f64 = 8.0;
    const WATER_INERTIA: f64 = 0.99;
    /// Ticks in the ground after which the trident counts as having dealt its damage.
    const DEALT_DAMAGE_GROUND_TICKS: u32 = 4;
    const FLAG_CRIT: u8 = 0x01;
    const FLAG_NOPHYSICS: u8 = 0x02;

    #[must_use]
    pub fn loyalty_from_item(item_stack: &ItemStack) -> u8 {
        EnchantmentHelper::modify_trident_return_acceleration(item_stack, 0.0).clamp(0.0, 127.0)
            as u8
    }

    fn sanitize_pickup_stack(mut stack: ItemStack, pickup: &mut ArrowPickup) -> ItemStack {
        if stack.is_empty() {
            stack = ItemStack::new(1, &Item::TRIDENT);
        } else {
            stack = stack.copy_with_count(1);
        }
        if stack
            .get_data_component::<IntangibleProjectileImpl>()
            .is_some()
        {
            stack
                .patch
                .retain(|(id, _)| *id != DataComponent::IntangibleProjectile);
            *pickup = ArrowPickup::CreativeOnly;
        }
        stack
    }

    pub fn new(entity: Entity, owner: Option<&Entity>) -> Self {
        let pickup = if owner.is_some_and(|o| o.entity_type == &EntityType::PLAYER) {
            ArrowPickup::Allowed
        } else {
            ArrowPickup::Disallowed
        };
        Self::new_with_item(entity, owner, ItemStack::new(1, &Item::TRIDENT), pickup)
    }

    pub fn new_with_item(
        entity: Entity,
        owner: Option<&Entity>,
        item_stack: ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        entity
            .data
            .store(owner.map_or(0, |o| o.entity_id), Ordering::Relaxed);
        Self::with_item(entity, ProjectileOwner::new(owner), item_stack, pickup)
    }

    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        item_stack: ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        let mut owner_pos = shooter.pos.load();
        owner_pos.y += f64::from(shooter.entity_dimension.load().eye_height) - 0.1;
        entity.set_pos(owner_pos);
        entity.data.store(shooter.entity_id, Ordering::Relaxed);

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

        Self::with_item(
            entity,
            ProjectileOwner::new(Some(shooter)),
            item_stack,
            pickup,
        )
    }

    fn with_item(
        entity: Entity,
        owner: ProjectileOwner,
        item_stack: ItemStack,
        mut pickup: ArrowPickup,
    ) -> Self {
        let clean_stack = Self::sanitize_pickup_stack(item_stack, &mut pickup);
        let loyalty = Self::loyalty_from_item(&clean_stack);
        let foil = clean_stack.has_foil();
        Self {
            entity,
            owner,
            item_stack: RwLock::new(clean_stack),
            pickup: AtomicCell::new(pickup),
            flight: ArrowFlight::new(),
            dealt_damage: AtomicBool::new(false),
            is_critical: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            loyalty: AtomicU8::new(loyalty),
            foil: AtomicBool::new(foil),
            return_ticks: AtomicU32::new(0),
            base_damage: AtomicCell::new(Self::BASE_DAMAGE),
        }
    }

    fn item(&self) -> ItemStack {
        self.item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .copy_with_count(1)
    }

    /// Applies projectile-spawned enchantment effects matching vanilla `Projectile::applyOnProjectileSpawned`.
    pub fn apply_on_projectile_spawned(&self, pickup_item_stack: &ItemStack) {
        super::apply_on_projectile_spawned(self.get_entity(), pickup_item_stack, None, None);
    }

    /// Vanilla `Projectile.shootFromRotation`: aims along `(pitch, yaw)` and adds shooter movement.
    pub fn shoot_from_rotation(
        &self,
        shooter: &Entity,
        pitch: f32,
        yaw: f32,
        roll: f32,
        speed: f32,
        divergence: f32,
    ) {
        self.set_velocity_from_rotation(pitch, yaw, roll, speed, divergence);
        let shooter_mov = shooter.known_movement();
        let added = Vector3::new(
            shooter_mov.x,
            if shooter.on_ground.load(Ordering::Relaxed) {
                0.0
            } else {
                shooter_mov.y
            },
            shooter_mov.z,
        );
        self.entity
            .velocity
            .store(self.entity.velocity.load().add(&added));
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

        let dir = Vector3::new(x, y, z);
        let normalized = if dir.length_squared() > 1.0e-12 {
            dir.normalize()
        } else {
            Vector3::default()
        };
        let velocity = normalized
            .add_raw(
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
            )
            .multiply(power, power, power);

        self.entity.velocity.store(velocity);
        self.flight.life.store(0, Ordering::Relaxed);
        let len = velocity.horizontal_length();
        self.entity.set_rotation(
            velocity.x.atan2(velocity.z).to_degrees() as f32,
            velocity.y.atan2(len).to_degrees() as f32,
        );
    }

    fn get_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.is_critical.load(Ordering::Relaxed) {
            flags |= Self::FLAG_CRIT;
        }
        if self.entity.no_physics.load(Ordering::Relaxed) {
            flags |= Self::FLAG_NOPHYSICS;
        }
        flags
    }

    pub fn set_no_physics(&self, no_physics: bool) {
        if self.entity.no_physics.swap(no_physics, Ordering::Relaxed) != no_physics {
            self.entity
                .set_synced_data(tracked_data::trident::ID_FLAGS, self.get_flags());
        }
    }

    pub fn set_critical(&self, critical: bool) {
        if self.is_critical.swap(critical, Ordering::Relaxed) != critical {
            self.entity
                .set_synced_data(tracked_data::trident::ID_FLAGS, self.get_flags());
        }
    }

    fn refresh_item_metadata(&self, sync: bool) {
        let stack = self.item();
        let loyalty = Self::loyalty_from_item(&stack);
        let foil = stack.has_foil();
        let old_loyalty = self.loyalty.swap(loyalty, Ordering::Relaxed);
        let old_foil = self.foil.swap(foil, Ordering::Relaxed);
        if sync {
            if old_loyalty != loyalty {
                self.entity
                    .set_synced_data(tracked_data::trident::ID_LOYALTY, loyalty);
            }
            if old_foil != foil {
                self.entity
                    .set_synced_data(tracked_data::trident::ID_FOIL, foil);
            }
        }
    }

    /// Flies back to the owner once it has done its job, or drops when the owner is gone.
    /// Returns false when the trident is gone.
    fn tick_loyalty(&self) -> bool {
        let entity = &self.entity;
        if self.flight.in_ground_time.load(Ordering::Relaxed) > Self::DEALT_DAMAGE_GROUND_TICKS {
            self.dealt_damage.store(true, Ordering::Relaxed);
        }
        let loyalty = self.loyalty.load(Ordering::Relaxed);
        if loyalty == 0
            || !(self.dealt_damage.load(Ordering::Relaxed)
                || entity.no_physics.load(Ordering::Relaxed))
        {
            return true;
        }
        let world = entity.world.load();
        let Some(owner) = self.owner.get(&world) else {
            return true;
        };
        let owner_entity = owner.get_entity();
        if !owner_entity.is_alive() || owner.is_spectator() {
            if self.pickup.load() == ArrowPickup::Allowed {
                entity.spawn_at_location(self.item());
            }
            entity.remove();
            return false;
        }
        let eye = owner.get_eye_pos();
        let position = entity.pos.load();
        if owner.get_player().is_none() {
            let max_dist = f64::from(owner_entity.width()) + 1.0;
            if position.squared_distance_to_vec(&eye) < max_dist * max_dist {
                entity.remove();
                return false;
            }
        }

        if self.flight.is_in_ground() {
            self.set_in_ground(false);
        }
        self.set_no_physics(true);
        let towards = eye - position;
        let loyalty = f64::from(loyalty);
        entity.set_pos(position.add_raw(0.0, towards.y * 0.015 * loyalty, 0.0));
        let dir = if towards.length_squared() > 1.0e-12 {
            towards.normalize()
        } else {
            Vector3::default()
        };
        entity
            .velocity
            .store(entity.velocity.load() * 0.95 + dir * (0.05 * loyalty));
        if self.return_ticks.fetch_add(1, Ordering::Relaxed) == 0 {
            self.play_entity_sound(Sound::ItemTridentReturn, 10.0, 1.0);
        }
        true
    }

    fn on_hit_block(&self, pos: &BlockPos, hit_pos: Vector3<f64>, world_border: bool) {
        let world = self.entity.world.load();
        if !world_border {
            let weapon_stack = self.item();
            let owner_player = self.owner.get_player(&world);
            EnchantmentHelper::on_trident_hit_block(
                &world,
                &self.entity,
                owner_player.as_ref(),
                pos,
                hit_pos,
                &weapon_stack,
            );
        }
        abstract_arrow::stick_in_block(self, Sound::ItemTridentHitGround);
        self.set_critical(false);
    }

    fn on_hit_entity(&self, target: &Arc<dyn EntityBase>, hit_pos: Vector3<f64>) {
        let world = self.entity.world.load();
        let weapon_stack = self.item();
        let damage = EnchantmentHelper::modify_damage_against(
            &weapon_stack,
            self.base_damage.load(),
            Some(target.get_entity().entity_type),
        );

        let owner = self.owner.get(&world);
        self.dealt_damage.store(true, Ordering::Relaxed);
        let hurt = target.damage_with_context(
            target.as_ref(),
            damage as f32,
            DamageType::TRIDENT,
            Some(hit_pos),
            Some(self),
            Some(owner.as_deref().unwrap_or(self)),
        );
        if hurt {
            EnchantmentHelper::on_trident_post_attack(
                &world,
                &self.entity,
                owner.as_deref(),
                target.as_ref(),
                &weapon_stack,
            );
        }
        if target.projectile_receives_side_effects_on_hit(hurt) {
            self.play_entity_sound(Sound::ItemTridentHit, 1.0, 1.0);
            deflect(
                self,
                ProjectileDeflection::Reverse,
                Some(target.as_ref()),
                self.owner.uuid(),
                false,
                Vector3::new(0.02, 0.2, 0.02),
            );
        }
    }

    fn try_pickup(&self, player: &Player) -> bool {
        let owned = self.owner.matches(player.get_entity());
        let allowed = match self.pickup.load() {
            ArrowPickup::Disallowed => false,
            ArrowPickup::CreativeOnly => player.is_creative(),
            ArrowPickup::Allowed => {
                let mut stack = self.item();
                player.inventory.insert_stack_anywhere(&mut stack)
            }
        };
        allowed
            || (self.entity.no_physics.load(Ordering::Relaxed)
                && owned
                && player.inventory.insert_stack_anywhere(&mut self.item()))
    }
}

impl AbstractArrow for TridentEntity {
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
        if self.flight.in_ground.swap(in_ground, Ordering::Relaxed) != in_ground {
            self.entity
                .set_synced_data(tracked_data::trident::IN_GROUND, in_ground);
        }
    }

    fn tick_despawn(&self) {
        // A loyal trident that can be picked up waits for its owner forever.
        if self.pickup.load() != ArrowPickup::Allowed || self.loyalty.load(Ordering::Relaxed) == 0 {
            abstract_arrow::count_down_despawn(self);
        }
    }

    fn find_hit_entities(
        &self,
        from: Vector3<f64>,
        to: Vector3<f64>,
    ) -> Vec<(Arc<dyn EntityBase>, Vector3<f64>)> {
        if self.dealt_damage.load(Ordering::Relaxed) {
            return Vec::new();
        }
        let world = self.entity.world.load();
        let owner_is_player = self.owner.get_player(&world).is_some();
        let pvp_enabled = world.level_info.load().game_rules.pvp;
        get_entity_hit_result(&self.entity, from, to, |other| {
            if owner_is_player && other.get_player().is_some() && !pvp_enabled {
                return false;
            }
            abstract_arrow::can_hit_entity(self, other)
        })
        .into_iter()
        .collect()
    }
}

impl EntityBase for TridentEntity {
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

    fn is_pushed_by_fluids(&self) -> bool {
        !self.flight.is_in_ground() && !self.entity.no_physics.load(Ordering::Relaxed)
    }

    fn should_bounce_on_world_border(&self) -> bool {
        true
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.owner)
    }

    fn init_data_tracker(&self) {
        self.entity
            .set_synced_data(tracked_data::trident::ID_FLAGS, self.get_flags());
        self.entity.set_synced_data(
            tracked_data::trident::PIERCE_LEVEL,
            self.pierce_level.load(Ordering::Relaxed),
        );
        self.entity
            .set_synced_data(tracked_data::trident::IN_GROUND, self.flight.is_in_ground());
        self.entity.set_synced_data(
            tracked_data::trident::ID_LOYALTY,
            self.loyalty.load(Ordering::Relaxed),
        );
        self.entity.set_synced_data(
            tracked_data::trident::ID_FOIL,
            self.foil.load(Ordering::Relaxed),
        );
        if self.entity.is_on_fire() {
            self.entity.set_on_fire(true);
        }
    }

    fn java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        let shared_flags = self.entity.flags.load(Ordering::Relaxed);
        let mut buf = Vec::new();
        if shared_flags != 0 {
            let _ = Metadata::new(tracked_data::trident::DATA_SHARED_FLAGS_ID, shared_flags)
                .write(&mut buf, &version);
        }
        let _ = Metadata::new(tracked_data::trident::ID_FLAGS, self.get_flags())
            .write(&mut buf, &version);
        let _ = Metadata::new(
            tracked_data::trident::PIERCE_LEVEL,
            self.pierce_level.load(Ordering::Relaxed),
        )
        .write(&mut buf, &version);
        let _ = Metadata::new(tracked_data::trident::IN_GROUND, self.flight.is_in_ground())
            .write(&mut buf, &version);
        let _ = Metadata::new(
            tracked_data::trident::ID_LOYALTY,
            self.loyalty.load(Ordering::Relaxed),
        )
        .write(&mut buf, &version);
        let _ = Metadata::new(
            tracked_data::trident::ID_FOIL,
            self.foil.load(Ordering::Relaxed),
        )
        .write(&mut buf, &version);

        (!buf.is_empty()).then(|| {
            buf.put_u8(255);
            buf.into_boxed_slice()
        })
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.flight.write_nbt(nbt);
        if let Some(item) = self.item().to_nbt() {
            nbt.put_compound("item", item);
        }
        nbt.put_bool("crit", self.is_critical.load(Ordering::Relaxed));
        nbt.put_double("damage", self.base_damage.load());
        nbt.put_byte(
            "PierceLevel",
            self.pierce_level.load(Ordering::Relaxed) as i8,
        );
        nbt.put_byte("pickup", self.pickup.load().to_byte() as i8);
        nbt.put_bool("DealtDamage", self.dealt_damage.load(Ordering::Relaxed));
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.flight.read_nbt(nbt);
        let mut pickup = nbt
            .get_byte("pickup")
            .map_or(ArrowPickup::Disallowed, |b| ArrowPickup::from_byte(b as u8));
        if let Some(item) = nbt
            .get_compound("item")
            .and_then(ItemStack::read_item_stack)
        {
            let clean = Self::sanitize_pickup_stack(item, &mut pickup);
            *self
                .item_stack
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = clean;
        }
        self.pickup.store(pickup);
        if let Some(crit) = nbt.get_bool("crit") {
            self.is_critical.store(crit, Ordering::Relaxed);
        }
        if let Some(damage) = nbt.get_double("damage") {
            self.base_damage.store(damage);
        }
        if let Some(pierce) = nbt.get_byte("PierceLevel") {
            self.pierce_level
                .store(pierce.max(0) as u8, Ordering::Relaxed);
        }
        self.dealt_damage.store(
            nbt.get_bool("DealtDamage").unwrap_or(false),
            Ordering::Relaxed,
        );
        self.refresh_item_metadata(true);
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        if self.tick_loyalty() {
            abstract_arrow::tick(self, caller, server);
            if !self.entity.is_removed()
                && !self.flight.is_in_ground()
                && self.entity.touching_water.load(Ordering::Relaxed)
            {
                let pos = self.entity.pos.load();
                let vel = self.entity.velocity.load();
                let world = self.entity.world.load();
                for i in 0..4 {
                    let factor = 0.25 * f64::from(i);
                    world.spawn_particle(
                        Vector3::new(
                            pos.x - vel.x * factor,
                            pos.y - vel.y * factor,
                            pos.z - vel.z * factor,
                        ),
                        Vector3::new(vel.x as f32, vel.y as f32, vel.z as f32),
                        0.0,
                        1,
                        Particle::Bubble,
                    );
                }
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let (event_pos, hit_entity_id) = match &hit {
            ProjectileHit::Block { hit_pos, .. } => (*hit_pos, None),
            ProjectileHit::Entity {
                entity, hit_pos, ..
            } => (*hit_pos, Some(entity.get_entity().entity_id)),
        };
        let mut hit_event =
            crate::plugin::api::events::entity::projectile_hit::ProjectileHitEvent::new(
                self.entity.entity_id,
                event_pos,
                hit_entity_id,
            );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut hit_event);
        }

        match hit {
            ProjectileHit::Block {
                pos,
                hit_pos,
                world_border,
                ..
            } => self.on_hit_block(&pos, hit_pos, world_border),
            ProjectileHit::Entity {
                entity: target,
                hit_pos,
                ..
            } => self.on_hit_entity(&target, hit_pos),
        }
    }

    fn receives_player_touch(&self) -> bool {
        true
    }

    fn on_player_collision(&self, player: &Arc<Player>) {
        let world = self.entity.world.load();
        if !(self.owner.matches(player.get_entity()) || self.owner.get(&world).is_none()) {
            return;
        }
        if !(self.flight.is_in_ground() || self.entity.no_physics.load(Ordering::Relaxed))
            || self.flight.shake_time.load(Ordering::Relaxed) > 0
        {
            return;
        }
        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::player::player_pickup_arrow::PlayerPickupArrowEvent::new(
                    player.clone(),
                    self.entity.entity_id,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        if self.try_pickup(player) {
            let stack = self.item();
            player.increment_stat(StatisticCategory::PickedUp, i32::from(stack.item.id), 1);
            player.living_entity.pickup(&self.entity, 1);
            player.sync_inventory_to_client();
            self.entity.remove();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use pumpkin_data::Enchantment;
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{
        DataComponentImpl, EnchantmentGlintOverrideImpl, EnchantmentsImpl, IntangibleProjectileImpl,
    };
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    use super::{ArrowPickup, TridentEntity};
    use crate::enchantment::EnchantmentHelper;
    use crate::entity::player::advancement::predicate::CriterionConditions;

    fn enchanted_trident(enchantment: &'static Enchantment, level: i32) -> ItemStack {
        let mut stack = ItemStack::new(1, &Item::TRIDENT);
        stack.patch.push((
            DataComponent::Enchantments,
            Some(
                EnchantmentsImpl {
                    enchantment: Cow::Owned(vec![(enchantment, level)]),
                }
                .to_dyn(),
            ),
        ));
        stack
    }

    #[test]
    fn impaling_adds_2_5_per_level_only_to_sensitive_mobs() {
        let stack = enchanted_trident(&Enchantment::IMPALING, 5);
        let drowned_dmg =
            EnchantmentHelper::modify_damage_against(&stack, 8.0, Some(&EntityType::DROWNED));
        let guardian_dmg =
            EnchantmentHelper::modify_damage_against(&stack, 8.0, Some(&EntityType::GUARDIAN));
        let zombie_dmg =
            EnchantmentHelper::modify_damage_against(&stack, 8.0, Some(&EntityType::ZOMBIE));

        // Java Edition Impaling V: +12.5 against GUARDIAN, NOT DROWNED (undead, not aquatic tag).
        assert!((guardian_dmg - 20.5).abs() < 1.0e-6);
        assert!((drowned_dmg - 8.0).abs() < 1.0e-6);
        assert!((zombie_dmg - 8.0).abs() < 1.0e-6);
    }

    #[test]
    fn loyalty_and_foil_derived_from_item_components() {
        let plain = ItemStack::new(1, &Item::TRIDENT);
        assert_eq!(TridentEntity::loyalty_from_item(&plain), 0);
        assert!(!plain.has_foil());

        let loyalty_iii = enchanted_trident(&Enchantment::LOYALTY, 3);
        assert_eq!(TridentEntity::loyalty_from_item(&loyalty_iii), 3);
        assert!(loyalty_iii.has_foil());

        let mut suppressed = loyalty_iii;
        suppressed.set_data_component(EnchantmentGlintOverrideImpl { glint: false });
        assert!(!suppressed.has_foil());
    }

    #[test]
    fn intangible_component_forces_creative_only_and_is_stripped() {
        let mut stack = enchanted_trident(&Enchantment::CHANNELING, 1);
        stack.set_data_component(IntangibleProjectileImpl);
        let mut pickup = ArrowPickup::Allowed;

        let sanitized = TridentEntity::sanitize_pickup_stack(stack, &mut pickup);
        assert_eq!(pickup, ArrowPickup::CreativeOnly);
        assert!(
            sanitized
                .get_data_component::<IntangibleProjectileImpl>()
                .is_none()
        );
    }

    #[test]
    fn throw_trident_and_shoot_arrow_advancements_do_not_cross_trigger() {
        let trident_cond = CriterionConditions::from(
            pumpkin_data::advancement::Advancement::ADVENTURE_THROW_TRIDENT.criteria_definitions[0]
                .conditions,
        );
        let arrow_cond = CriterionConditions::from(
            pumpkin_data::advancement::Advancement::ADVENTURE_SHOOT_ARROW.criteria_definitions[0]
                .conditions,
        );

        let CriterionConditions::PlayerHurtEntity(trident_pred) = trident_cond else {
            panic!("expected PlayerHurtEntity predicate for throw_trident");
        };
        let CriterionConditions::PlayerHurtEntity(arrow_pred) = arrow_cond else {
            panic!("expected PlayerHurtEntity predicate for shoot_arrow");
        };

        assert!(trident_pred.matches_with_context(false, true, false, Some(8.0), Some("trident")));
        assert!(!trident_pred.matches_with_context(false, true, false, Some(8.0), Some("arrow")));
        assert!(arrow_pred.matches_with_context(false, true, false, Some(8.0), Some("arrow")));
        assert!(arrow_pred.matches_with_context(
            false,
            true,
            false,
            Some(8.0),
            Some("spectral_arrow")
        ));
        assert!(!arrow_pred.matches_with_context(false, true, false, Some(8.0), Some("trident")));
    }
}
