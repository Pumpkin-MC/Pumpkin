use std::sync::Arc;

use pumpkin_data::dimension::Dimension;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, slime::SlimeEntity},
};
use crate::world::World;

pub struct MagmaCubeEntity {
    pub slime: Arc<SlimeEntity>,
}

impl MagmaCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let slime = SlimeEntity::new(entity);
        Arc::new(Self { slime })
    }

    /// Vanilla magma cube spawn rules.
    ///
    /// Magma cubes only spawn naturally in the nether, on solid ground.
    /// No light check is needed because the nether is naturally dark.
    /// See `SpawnPlacements#checkMagmaCubeSpawnRules` in Mojang mappings.
    pub fn check_spawn_rules(world: &World, _pos: &BlockPos) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }
        world.dimension == Dimension::THE_NETHER
    }
}

impl NBTStorage for MagmaCubeEntity {}

impl Mob for MagmaCubeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.slime.get_mob_entity()
    }
}
