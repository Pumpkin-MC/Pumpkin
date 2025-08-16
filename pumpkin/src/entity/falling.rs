use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::{damage::DamageType, entity::EntityType};
use pumpkin_protocol::java::client::play::{MetaDataType, Metadata};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::{BlockStateId, world::BlockFlags};
use std::sync::{Arc, atomic::Ordering};
use uuid::Uuid;

use crate::{
    entity::{Entity, EntityBase, NBTStorage, living::LivingEntity},
    server::Server,
    world::World,
};

pub struct FallingEntity {
    entity: Entity,
    block_state_id: BlockStateId,
}

impl FallingEntity {
    pub fn new(entity: Entity, block_state_id: BlockStateId) -> Self {
        Self {
            entity,
            block_state_id,
        }
    }

    /// Replaced the current Block and Spawns a new Falling one
    pub async fn replace_spawn(world: &Arc<World>, position: BlockPos, block_state: BlockStateId) {
        // Replace the original block, TODO: use fluid state
        world
            .set_block_state(
                &position,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;

        let position = position.0.to_f64().add_raw(0.5, 0.0, 0.5);
        let entity = Entity::new(
            Uuid::new_v4(),
            world.clone(),
            position,
            &EntityType::FALLING_BLOCK,
            false,
        );
        entity.data.store(i32::from(block_state), Ordering::Relaxed);
        let entity = Arc::new(Self::new(entity, block_state));
        world.spawn_entity(entity).await;
    }
}

impl NBTStorage for FallingEntity {}

#[async_trait]
impl EntityBase for FallingEntity {
    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
        let entity = &self.entity;
        entity.tick(caller.clone(), server).await;

        let original_velo = entity.velocity.load();
        let mut velo = original_velo;
        velo.y -= self.get_gravity();

        entity.velocity.store(velo);

        entity.move_entity(caller.clone(), velo).await;
        entity.tick_block_collisions(&caller, server).await;
        if entity.on_ground.load(Ordering::Relaxed) {
            entity.velocity.store(velo.multiply(0.7, -0.5, 0.7));
            entity
                .world
                .read()
                .await
                .set_block_state(
                    &self.entity.block_pos.load(),
                    self.block_state_id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            entity.remove().await;
        }

        entity.velocity.store(velo.multiply(0.98, 0.98, 0.98));

        entity.send_pos_rot().await;

        entity.send_velocity().await;
    }

    async fn init_data_tracker(&self) {
        self.entity
            .send_meta_data(&[Metadata::new(
                8,
                MetaDataType::BlockPos,
                self.entity.block_pos.load(),
            )])
            .await;
    }

    async fn damage_with_context(
        &self,
        _dyn_self: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        false
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    async fn kill(&self) {
        self.entity.kill().await;
    }
}
