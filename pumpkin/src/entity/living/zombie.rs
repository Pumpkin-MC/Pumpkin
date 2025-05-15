use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_protocol::client::play::{MetaDataType, Metadata};
use pumpkin_world::entity::living::zombie::{Drowned, Zombie};

use crate::{
    entity::{WorldEntityExt, player::Player},
    server::Server,
    world::World,
};

macro_rules! gen_trait {
    ($s: ident, $drowned: literal) => {
        #[async_trait]
        impl WorldEntityExt for $s {
            async fn on_player_collision(&self, _player: Arc<Player>, _world: &World) {
                // TODO
            }

            async fn tick(&mut self, _server: &Server, _world: &World) {
                // TODO
            }

            async fn init_data_tracker(&mut self, world: &World) {
                let baby_metadata = Metadata::new(16, MetaDataType::Boolean, self.0.is_baby);
                let drowned_metadata = Metadata::new(18, MetaDataType::Boolean, $drowned);

                self.send_meta_data(&[baby_metadata, drowned_metadata], world)
                    .await;
            }

            async fn damage(&self, _amount: f32, _damage_type: DamageType, _world: &World) -> bool {
                // TODO
                false
            }
        }
    };
}

gen_trait!(Zombie, false);
gen_trait!(Drowned, true);
