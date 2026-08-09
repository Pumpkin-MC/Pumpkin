use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, slime::SlimeEntity},
};
use crate::world::World;

/// Wraps `SlimeEntity`, mirroring how `MagmaCube.java` extends `AbstractCubeMob` the same
/// way `Slime.java` does.
///
/// `SlimeEntity` branches internally on `EntityType::MAGMA_CUBE` for the handful of
/// overrides that differ (sounds, jump delay, squish decay, attack damage/armor scaling,
/// fire-render suppression in `Entity::tick`).
///
/// Not ported: `MagmaCube.jumpInLiquid` (extra upward velocity while jumping in lava) and
/// `MagmaCube.jumpFromGround`'s size-scaled jump boost. Both hook into `LivingEntity`'s
/// liquid-jump/jump-from-ground physics, which this codebase does not implement for any
/// mob (jumping only ever comes from the `JUMP_STRENGTH` attribute via `LivingEntity::jump`).
/// Porting them would mean building that physics hook first, which is out of scope here.
pub struct MagmaCubeEntity {
    pub slime: Arc<SlimeEntity>,
}

impl MagmaCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let slime = SlimeEntity::new(entity);
        Arc::new(Self { slime })
    }
}

impl NBTStorage for MagmaCubeEntity {}

impl Mob for MagmaCubeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.slime.get_mob_entity()
    }

    fn light_level_dependent_magic_value(&self, _world: &World) -> f32 {
        1.0
    }

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        self.slime.mob_init_data_tracker()
    }

    fn mob_tick<'a>(
        &'a self,
        caller: &'a Arc<dyn crate::entity::EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        self.slime.mob_tick(caller)
    }

    fn post_tick(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        self.slime.post_tick()
    }

    fn mob_player_collision<'a>(
        &'a self,
        player: &'a Arc<crate::entity::player::Player>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        self.slime.mob_player_collision(player)
    }
}
