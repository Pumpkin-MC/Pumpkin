use crate::entity::ArcEntityBaseFuture;
use crate::entity::projectile::{ProjectileHit, ThrownItemEntityCondition};
use crate::world::explosion::ExplosionInteraction;
use crate::world::explosion_damage_calculator::ExplosionDamageCalculator;
use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity,
        projectile::ThrownItemEntity, projectile_deflection::ProjectileDeflectionType,
    },
    server::Server,
};
use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::vector3::Vector3;
use std::sync::LazyLock;
use std::sync::atomic::Ordering::Relaxed;
use std::{
    f64,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

const DEFAULT_DEFLECT_COOLDOWN: u8 = 5;

static EXPLOSION_DAMAGE_CALCULATOR: LazyLock<ExplosionDamageCalculator> =
    LazyLock::new(|| ExplosionDamageCalculator::Simple {
        explodes_blocks: true,
        damages_entities: false,
        knockback_multiplier: None,
        // Block tag: #blocks_wind_charge_explosions
        immune_blocks: Some([Block::BARRIER, Block::BEDROCK].iter().collect()),
    });

/// An entity for a wind charge.
pub struct WindChargeEntity {
    thrown: ThrownItemEntity,
    kind: WindChargeKind,
}

enum WindChargeKind {
    /// Represents a wind charge spawned by a player or dispenser.
    /// This wind charge also has a deflect cooldown counter.
    Normal { deflect_cooldown: AtomicU8 },
    /// Represents a wind charge spawned by a breeze.
    Breeze,
}

impl WindChargeEntity {
    fn new(entity: Entity, kind: WindChargeKind, condition: &ThrownItemEntityCondition) -> Self {
        Self {
            thrown: ThrownItemEntity::new(entity, condition),
            kind,
        }
    }

    /// Creates a normal wind charge (spawned by a player or dispenser.)
    #[must_use]
    pub fn new_normal(entity: Entity, condition: &ThrownItemEntityCondition) -> Self {
        Self::new(
            entity,
            WindChargeKind::Normal {
                deflect_cooldown: AtomicU8::new(DEFAULT_DEFLECT_COOLDOWN),
            },
            condition,
        )
    }

    /// Creates a breeze wind charge (spawned by a breeze.)
    #[must_use]
    pub fn new_breeze(entity: Entity, condition: &ThrownItemEntityCondition) -> Self {
        Self::new(entity, WindChargeKind::Breeze, condition)
    }

    pub const fn get_thrown_item_entity(&self) -> &ThrownItemEntity {
        &self.thrown
    }

    pub async fn explode(&self, position: Vector3<f64>) {
        self.get_entity()
            .world
            .load()
            .explode(position, self.explosion_radius())
            .await;
    }

    pub fn deflect(
        &mut self,
        deflection: &ProjectileDeflectionType,
        deflector: Option<&dyn EntityBase>,
        _from_attack: bool,
    ) -> bool {
        deflection.deflect(self, deflector);

        /* TODO: Does this need to be implemented?
        if self.get_entity().world().is_client() {
            self.set_owner();
            self.on_Deflected(from_attack);
        }
         */
        true
    }

    pub const fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    const fn explosion_radius(&self) -> f32 {
        match self.kind {
            WindChargeKind::Normal { .. } => 1.2,
            WindChargeKind::Breeze => 3.0,
        }
    }

    const fn explosion_sound(&self) -> Sound {
        match self.kind {
            WindChargeKind::Normal { .. } => Sound::EntityWindChargeWindBurst,
            WindChargeKind::Breeze => Sound::EntityBreezeWindBurst,
        }
    }
}

impl EntityBase for WindChargeEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // If the wind charge is too high up, immediately explode it.
            if self.get_entity().block_pos.load().0.y
                >= self.get_entity().world.load().get_top_y() + 30
            {
                if let Some(entity) = self
                    .get_entity()
                    .world
                    .load()
                    .get_entity_by_id(self.get_entity().entity_id)
                {
                    entity.clone().explode(entity.get_entity().pos.load()).await;
                }
            } else {
                self.thrown.process_tick(caller, server).await;
            }

            if let WindChargeKind::Normal { deflect_cooldown } = &self.kind {
                let loaded = deflect_cooldown.load(Relaxed);
                if loaded > 0 {
                    deflect_cooldown.store(loaded - 1, Relaxed);
                }
            }
        })
    }

    fn explode(self: Arc<Self>, position: Vector3<f64>) -> ArcEntityBaseFuture<()> {
        Box::pin(async move {
            self.get_entity()
                .world
                .load()
                .explode_with(
                    Some(self.clone()),
                    None,
                    Some(EXPLOSION_DAMAGE_CALCULATOR.clone()),
                    self.explosion_radius(),
                    position,
                    false,
                    ExplosionInteraction::Trigger,
                )
                .await;
            self.get_entity().remove().await;
        })
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn get_thrown_item_entity(&self) -> Option<&ThrownItemEntity> {
        Some(&self.thrown)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_gravity(&self) -> f64 {
        0.0
    }

    fn on_hit(self: Arc<Self>, hit: ProjectileHit) -> EntityBaseFuture<'static, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();

            // Always send particle status regardless of what was hit
            world
                .send_entity_status(
                    self.get_entity(),
                    EntityStatus::PlayDeathSoundOrAddProjectileHitParticles,
                )
                .await;

            match hit {
                ProjectileHit::Block { hit_pos, face, .. } => {
                    let vec = face.to_offset().to_f64() * 0.25;
                    self.clone().explode(hit_pos.to_f64() + vec).await;
                    self.get_entity().remove().await;
                }

                ProjectileHit::Entity {
                    entity: ref target, ..
                } => {
                    let mut owner = self.thrown.owner_id.and_then(|i| world.get_player_by_id(i));

                    if let Some(owner) = &mut owner {
                        owner
                            .living_entity
                            .last_attacking_id
                            .store(target.get_entity().entity_id, Relaxed);
                    }

                    target
                        .damage_with_context(
                            target.as_ref(),
                            1.0,
                            DamageType::WIND_CHARGE,
                            None,
                            owner.as_ref().map(|o| o.as_ref() as &dyn EntityBase),
                            Some(target.as_ref()),
                        )
                        .await;

                    let pos = self.get_entity().pos.load();
                    self.clone().explode(pos).await;
                    self.get_entity().remove().await;
                }
            }
        })
    }
}

impl NBTStorage for WindChargeEntity {}
