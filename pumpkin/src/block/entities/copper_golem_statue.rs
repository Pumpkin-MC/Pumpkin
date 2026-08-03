use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::entity::Entity;
use crate::entity::passive::copper_golem::CopperGolemEntity;
use crate::world::World;

use super::BlockEntity;

pub struct CopperGolemStatueBlockEntity {
    pub position: BlockPos,
}

impl BlockEntity for CopperGolemStatueBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(_nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self { position }
    }

    fn write_nbt<'a>(
        &'a self,
        _nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CopperGolemStatueBlockEntity {
    pub const ID: &'static str = "minecraft:copper_golem_statue";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self { position }
    }

    /// `CopperGolemStatueBlockEntity.createStatue`: vanilla additionally persists the
    /// golem's custom name into the block entity's `CUSTOM_NAME` component. Deferred here --
    /// `CopperGolemEntity` has no custom-name field wired up yet, so there is nothing to
    /// carry over. Kept as a named call site so that wiring shows up as a one-line change.
    pub const fn create_statue(&self) {}

    /// `CopperGolemStatueBlockEntity.removeStatue` + `initCopperGolem`: spawns a fresh
    /// (`UNAFFECTED`) copper golem at this position, facing the direction the statue block
    /// was facing, and removes the statue block.
    ///
    /// Callers must first check `WeatheringCopperGolemStatueBlock.useItemOn`'s condition
    /// (an axe used on a statue whose weathering stage is `UNAFFECTED`) -- this method
    /// performs the conversion unconditionally once called. No block currently calls it:
    /// `minecraft:copper_golem_statue` has no registered `BlockBehaviour` in Pumpkin yet, so
    /// wiring the axe-interaction trigger itself is deferred to that follow-up.
    pub async fn remove_statue(&self, world: &Arc<World>, facing: HorizontalFacing) {
        let pos = self.position;
        let center = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y),
            f64::from(pos.0.z) + 0.5,
        );

        let entity = Entity::new(world.clone(), center, &EntityType::COPPER_GOLEM);
        let yaw = horizontal_facing_to_yaw(facing);
        entity.yaw.store(yaw);
        entity.head_yaw.store(yaw);

        let golem = CopperGolemEntity::new(entity);
        world.spawn_entity(golem).await;
        world
            .set_block_state(&pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL)
            .await;
    }
}

/// Inverse of `Entity::get_horizontal_facing`'s `floor(yaw / 90 + 0.5) & 3` mapping.
const fn horizontal_facing_to_yaw(facing: HorizontalFacing) -> f32 {
    match facing {
        HorizontalFacing::South => 0.0,
        HorizontalFacing::West => 90.0,
        HorizontalFacing::North => 180.0,
        HorizontalFacing::East => 270.0,
    }
}
