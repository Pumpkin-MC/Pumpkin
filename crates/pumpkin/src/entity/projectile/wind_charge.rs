use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::tag;
use pumpkin_util::math::vector3::Vector3;
use std::sync::LazyLock;
use std::{
    f64,
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, Ordering},
    },
};

use crate::{
    entity::{
        Entity, EntityBase,
        living::LivingEntity,
        projectile::{ProjectileHit, ThrownItemEntity},
        projectile_deflection::ProjectileDeflectionType,
    },
    server::Server,
    world::SimpleExplosionDamageCalculator,
};

const DEFAULT_DEFLECT_COOLDOWN: u8 = 5;
const DEFLECTED_ACCELERATION_POWER: f64 = 0.1;
pub const WIND_CHARGE_GRAVITY: f64 = 0.0;

enum WindChargeKind {
    Normal { deflect_cooldown: AtomicU8 },
    Breeze,
}

pub struct WindChargeEntity {
    owner_id: AtomicCell<Option<i32>>,
    acceleration_power: AtomicU64,
    kind: WindChargeKind,
    thrown_item_entity: ThrownItemEntity,
}

pub static WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR: LazyLock<Arc<SimpleExplosionDamageCalculator>> =
    LazyLock::new(|| {
        Arc::new(SimpleExplosionDamageCalculator::new(
            true,
            false,
            Some(1.22),
            Some(&tag::Block::MINECRAFT_BLOCKS_WIND_CHARGE_EXPLOSIONS),
        ))
    });

pub static BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR: LazyLock<
    Arc<SimpleExplosionDamageCalculator>,
> = LazyLock::new(|| {
    Arc::new(SimpleExplosionDamageCalculator::new(
        true,
        false,
        None,
        Some(&tag::Block::MINECRAFT_BLOCKS_WIND_CHARGE_EXPLOSIONS),
    ))
});

impl WindChargeEntity {
    #[must_use]
    pub const fn new_normal(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            owner_id: AtomicCell::new(thrown_item_entity.owner_id),
            acceleration_power: AtomicU64::new(0),
            kind: WindChargeKind::Normal {
                deflect_cooldown: AtomicU8::new(DEFAULT_DEFLECT_COOLDOWN),
            },
            thrown_item_entity,
        }
    }

    #[must_use]
    pub const fn new_breeze(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            owner_id: AtomicCell::new(thrown_item_entity.owner_id),
            acceleration_power: AtomicU64::new(0),
            kind: WindChargeKind::Breeze,
            thrown_item_entity,
        }
    }

    pub const fn deflect_cooldown(&self) -> Option<&AtomicU8> {
        if let WindChargeKind::Normal {
            deflect_cooldown, ..
        } = &self.kind
        {
            Some(deflect_cooldown)
        } else {
            None
        }
    }

    pub fn create_explosion(&self, position: Vector3<f64>) {
        let (power, calculator) = match self.kind {
            WindChargeKind::Normal { .. } => (1.2, WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR.clone()),
            WindChargeKind::Breeze => (3.0, BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR.clone()),
        };
        self.get_entity().world.load().explode_with_calculator(
            position,
            power,
            crate::world::ExplosionInteraction::Trigger,
            Some(calculator),
        );
    }

    pub fn redirect(&self, owner: Option<&dyn EntityBase>) {
        if self
            .deflect_cooldown()
            .is_some_and(|cooldown| cooldown.load(Ordering::Relaxed) > 0)
        {
            return;
        }
        self.owner_id
            .store(owner.map(|owner| owner.get_entity().entity_id));
        if let Some(owner) = owner {
            self.get_entity()
                .set_velocity(owner.get_entity().rotation().to_f64());
        }
        self.acceleration_power
            .store(DEFLECTED_ACCELERATION_POWER.to_bits(), Ordering::Relaxed);
    }

    pub fn deflect(
        &mut self,
        deflection: &ProjectileDeflectionType,
        deflector: Option<&dyn EntityBase>,
    ) -> bool {
        if let Some(cooldown) = self.deflect_cooldown()
            && cooldown.load(Ordering::Relaxed) > 0
        {
            return false;
        }

        deflection.deflect(self, deflector);
        true
    }
}

impl EntityBase for WindChargeEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.owner_id.load()
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = self.get_entity();
        let velocity = entity.velocity.load();
        let accel = f64::from_bits(self.acceleration_power.load(Ordering::Relaxed));
        if velocity.length() > 1e-6 {
            entity.velocity.store(
                velocity
                    .normalize()
                    .multiply(accel, accel, accel)
                    .add(&velocity),
            );
        }

        self.thrown_item_entity
            .process_tick_with_owner(caller, self.get_owner_id());

        if let Some(cooldown) = self.deflect_cooldown() {
            let cooldown_ticks = cooldown.load(Ordering::Relaxed);
            if cooldown_ticks > 0 {
                cooldown.store(cooldown_ticks - 1, Ordering::Relaxed);
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.thrown_item_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let hit_pos = hit.hit_pos();
        if let ProjectileHit::Entity { ref entity, .. } = hit {
            let world = self.get_entity().world.load();
            let owner_id = self.get_owner_id();
            let owner = owner_id.and_then(|id| world.get_entity_by_id(id));

            let _ = entity.damage_with_context(
                entity.as_ref(),
                1.0,
                DamageType::WIND_CHARGE,
                Some(hit_pos),
                Some(self.get_entity()),
                owner.as_deref(),
            );
        }
        self.create_explosion(hit_pos);
    }
}
