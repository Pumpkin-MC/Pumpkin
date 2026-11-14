use crate::block::blocks::bubble_column;
use crate::entity::projectile::owner::ProjectileOwner;
use pumpkin_util::math::position::BlockPos;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

use super::ProjectileHit;
use super::abstract_arrow::{self, AbstractArrow, ArrowFlight, get_entity_hit_result};
use super::arrow::ArrowPickup;
use crate::enchantment::EnchantmentHelper;
use crate::entity::projectile_deflection::{ProjectileDeflection, deflect};

pub struct TridentEntity {
    pub entity: Entity,
    pub owner: ProjectileOwner,
    pub item_stack: Arc<Mutex<ItemStack>>,
    pub pickup: ArrowPickup,
    pub flight: ArrowFlight,
    /// Set once the trident hit an entity or lay in the ground for a moment, it can't hurt
    /// anything after that.
    dealt_damage: AtomicBool,
    loyalty: AtomicU8,
    return_ticks: AtomicU32,
}

impl TridentEntity {
    const BASE_DAMAGE: f32 = 8.0;
    const WATER_INERTIA: f64 = 0.99;
    /// Ticks in the ground after which the trident counts as having dealt its damage.
    const DEALT_DAMAGE_GROUND_TICKS: u32 = 4;

    pub fn new(entity: Entity, owner: Option<&Entity>) -> Self {
        Self::with_item(
            entity,
            ProjectileOwner::new(owner),
            ItemStack::new(1, &Item::TRIDENT),
            ArrowPickup::Disallowed,
        )
    }

    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        item_stack: ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        let mut owner_pos = shooter.pos.load();
        owner_pos.y = owner_pos.y + f64::from(shooter.entity_dimension.load().eye_height) - 0.1;
        entity.set_pos(owner_pos);
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
        pickup: ArrowPickup,
    ) -> Self {
        let loyalty = Self::loyalty_of(&item_stack);
        Self {
            entity,
            owner,
            item_stack: Arc::new(Mutex::new(item_stack)),
            pickup,
            flight: ArrowFlight::new(),
            dealt_damage: AtomicBool::new(false),
            loyalty: AtomicU8::new(loyalty),
            return_ticks: AtomicU32::new(0),
        }
    }

    fn loyalty_of(item_stack: &ItemStack) -> u8 {
        EnchantmentHelper::modify_trident_return_acceleration(item_stack, 0.0).clamp(0.0, 127.0)
            as u8
    }

    fn item(&self) -> ItemStack {
        self.item_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Applies projectile-spawned enchantment effects matching vanilla `Projectile::applyOnProjectileSpawned`.
    pub fn apply_on_projectile_spawned(&self, pickup_item_stack: &ItemStack) {
        super::apply_on_projectile_spawned(self.get_entity(), pickup_item_stack, None, None);
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

    fn set_no_physics(&self, no_physics: bool) {
        self.entity.no_physics.store(no_physics, Ordering::Relaxed);
        self.entity.set_synced_data(
            tracked_data::trident::ID_FLAGS,
            if no_physics { 0x02u8 } else { 0 },
        );
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
            if self.pickup == ArrowPickup::Allowed {
                entity.spawn_at_location(self.item());
            }
            entity.remove();
            return false;
        }
        let eye = owner_entity
            .pos
            .load()
            .add_raw(0.0, owner_entity.get_eye_height(), 0.0);
        let position = entity.pos.load();
        if owner.get_player().is_none()
            && position.squared_distance_to_vec(&eye).sqrt() < f64::from(owner_entity.width()) + 1.0
        {
            entity.remove();
            return false;
        }

        self.set_no_physics(true);
        let towards = eye - position;
        let loyalty = f64::from(loyalty);
        entity.set_pos(position.add_raw(0.0, towards.y * 0.015 * loyalty, 0.0));
        entity
            .velocity
            .store(entity.velocity.load() * 0.95 + towards.normalize() * (0.05 * loyalty));
        if self.return_ticks.fetch_add(1, Ordering::Relaxed) == 0 {
            self.play_entity_sound(Sound::ItemTridentReturn, 10.0, 1.0);
        }
        true
    }

    fn on_hit_entity(&self, target: &Arc<dyn EntityBase>, hit_pos: Vector3<f64>) {
        let world = self.entity.world.load();
        let mut damage = f64::from(Self::BASE_DAMAGE);
        if let Some(enchantments) = self
            .item()
            .get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
        {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_damage_against(
                    *level,
                    &mut damage,
                    Some(target.get_entity().entity_type),
                );
            }
        }

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
        let allowed = match self.pickup {
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

    fn set_in_ground(&self, in_ground: bool) {
        self.flight.in_ground.store(in_ground, Ordering::Relaxed);
        self.entity
            .set_synced_data(tracked_data::trident::IN_GROUND, in_ground);
    }

    fn tick_despawn(&self) {
        // A loyal trident that can be picked up waits for its owner forever.
        if self.pickup != ArrowPickup::Allowed || self.loyalty.load(Ordering::Relaxed) == 0 {
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
        get_entity_hit_result(&self.entity, from, to, |other| {
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

    fn should_bounce_on_world_border(&self) -> bool {
        true
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.owner)
    }

    fn init_data_tracker(&self) {
        let item = self.item();
        self.entity.set_synced_data(
            tracked_data::trident::ID_LOYALTY,
            self.loyalty.load(Ordering::Relaxed),
        );
        self.entity.set_synced_data(
            tracked_data::trident::ID_FOIL,
            item.get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
                .is_some_and(|enchantments| !enchantments.enchantment.is_empty()),
        );
        self.entity
            .set_synced_data(tracked_data::trident::IN_GROUND, self.flight.is_in_ground());
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.flight.write_nbt(nbt);
        let mut item = NbtCompound::new();
        self.item().write_item_stack(&mut item);
        nbt.put_compound("item", item);
        nbt.put_byte("pickup", self.pickup.to_byte() as i8);
        nbt.put_bool("DealtDamage", self.dealt_damage.load(Ordering::Relaxed));
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.flight.read_nbt(nbt);
        if let Some(item) = nbt
            .get_compound("item")
            .and_then(ItemStack::read_item_stack)
        {
            self.loyalty
                .store(Self::loyalty_of(&item), Ordering::Relaxed);
            *self
                .item_stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = item;
        }
        self.dealt_damage.store(
            nbt.get_bool("DealtDamage").unwrap_or(false),
            Ordering::Relaxed,
        );
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        if self.tick_loyalty() {
            abstract_arrow::tick(self, caller, server);
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
        match hit {
            ProjectileHit::Block { .. } => {
                abstract_arrow::stick_in_block(self, Sound::ItemTridentHitGround);
            }
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
        if self.try_pickup(player) {
            let stack = self.item();
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::PickedUp,
                stack.item.id as i32,
                1,
            );
            player.living_entity.pickup(&self.entity, 1);
            self.entity.remove();
        }
    }
}
