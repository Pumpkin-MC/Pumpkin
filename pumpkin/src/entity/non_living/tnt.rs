use std::f64::consts::TAU;
use std::ops::Sub;
use std::sync::Arc;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_protocol::client::play::{MetaDataType, Metadata};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::entity::{Entity, EntityBase};
use pumpkin_world::entity::non_living::tnt::Tnt;
use crate::entity::player::Player;
use crate::entity::WorldEntityExt;
use crate::server::Server;
use crate::world::World;

#[async_trait]
impl WorldEntityExt for Tnt {
    async fn tick(&mut self, server: &Server, world: &World) {
        let fuse = self.fuse_ticks.sub(1);
        if fuse == 0 {
            // TODO: I don't think this is correct
            world.remove_entity(&Entity::Tnt(self.clone())).await;
            world.explode(server, self.pos(), self.explosion_power).await;
        }
    }

    async fn damage(&self, _amount: f32, _damage_type: DamageType, _world: &World) -> bool {
        false
    }

    async fn init_data_tracker(&mut self, world: &World) {
        // TODO: Yes, this is the wrong function, but we need to send this after spawning the entity.
        let pos: f64 = rand::random::<f64>() * TAU;
        
        self.set_velocity(Vector3::new(-pos.sin() * 0.02, 0.2, -pos.cos() * 0.02));
        self.send_meta_data(&[
            Metadata::new(
                8,
                MetaDataType::Integer,
                VarInt(self.fuse_ticks as i32),
            ),
            Metadata::new(
                9,  
                MetaDataType::BlockState,
                VarInt(i32::from(Block::TNT.default_state_id)),
            ),
        ], world).await;
    }

    async fn on_player_collision(&self, player: Arc<Player>, world: &World) {
        // Do nothing
    }
}