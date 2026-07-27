use std::sync::Arc;

use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;

use crate::entity::water_animal::{SquidAi, WaterAnimalAir};
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity},
};

/// Glow squid.
///
/// `GlowSquid` extends `Squid` (GlowSquid.java:26), so vanilla `Squid` registers only `SquidRandomMovementGoal` and `SquidFleeGoal`
/// (Squid.java:64-68) and never uses the navigator: movement is an impulse
/// vector applied in `aiStep` (Squid.java:119-171) with `travel` just applying
/// the current delta (Squid.java:203-206). Out of water it sinks and flops
/// instead of walking (Squid.java:162-170), and as an `AgeableWaterCreature` it
/// drowns on land (AgeableWaterCreature.java:41-51).
pub struct GlowSquidEntity {
    pub mob_entity: MobEntity,
    ai: SquidAi,
    air: WaterAnimalAir,
}

impl GlowSquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // No goals and no navigator setup: vanilla squid movement is entirely
        // the custom jet AI in `SquidAi`, driven from `mob_tick` below.
        Arc::new(Self {
            mob_entity: MobEntity::new(entity),
            ai: SquidAi::new(),
            air: WaterAnimalAir::new(),
        })
    }
}

impl NBTStorage for GlowSquidEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.get_mob_entity().living_entity.write_nbt(nbt).await;
            self.air.write_nbt(nbt);
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.get_mob_entity()
                .living_entity
                .read_nbt_non_mut(nbt)
                .await;
            self.air.read_nbt(nbt);
        })
    }
}

impl Mob for GlowSquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.air.tick(&self.mob_entity, caller).await;
            self.ai.tick(&self.mob_entity).await;
        })
    }

    /// `Squid.hurtServer` (Squid.java:174-180): record the attacker for the
    /// flee goal and burst ink.
    fn on_damage<'a>(
        &'a self,
        _damage_type: pumpkin_data::damage::DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.ai.on_hurt(
                &self.mob_entity,
                source,
                Particle::GlowSquidInk,
                Sound::EntityGlowSquidSquirt,
                self.mob_entity
                    .living_entity
                    .entity
                    .age
                    .load(std::sync::atomic::Ordering::Relaxed)
                    < 0,
            );
        })
    }
}
