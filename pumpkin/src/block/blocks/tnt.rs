use pumpkin_data::item::Item;
use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, ExplodeArgs, OnNeighborUpdateArgs, PlacedArgs, UseWithItemArgs,
};
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::tnt::TNTEntity;
use crate::plugin::block::tnt_prime::TNTPrimeEvent;
use crate::world::World;
use pumpkin_data::entity::EntityType;
use pumpkin_data::Block;
use pumpkin_data::sound::SoundCategory;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use super::redstone::block_receives_redstone_power;

#[pumpkin_block("minecraft:tnt")]
pub struct TNTBlock;

impl TNTBlock {
    pub async fn prime(
        world: &Arc<World>,
        location: &BlockPos,
        cause: String,
        player: Option<Arc<Player>>,
    ) -> bool {
        if let Some(server) = world.server.upgrade() {
            let event = TNTPrimeEvent::new(
                player.clone(),
                &Block::TNT,
                *location,
                world.uuid,
                cause,
            );
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                return false;
            }
        }
        let entity = Entity::new(world.clone(), location.to_f64(), &EntityType::TNT);
        let pos = entity.pos.load();
        let tnt = Arc::new(TNTEntity::new(entity, DEFAULT_POWER, DEFAULT_FUSE));
        world.spawn_entity(tnt).await;
        world
            .play_sound(
                pumpkin_data::sound::Sound::EntityTntPrimed,
                SoundCategory::Blocks,
                &pos,
            )
            .await;
        world
            .set_block_state(location, 0, BlockFlags::NOTIFY_ALL)
            .await;
        true
    }
}

const DEFAULT_FUSE: u32 = 80;
const DEFAULT_POWER: f32 = 4.0;

impl BlockBehaviour for TNTBlock {
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item = args.item_stack.lock().await.item;
            if item != &Item::FLINT_AND_STEEL && item != &Item::FIRE_CHARGE {
                return BlockActionResult::Pass;
            }
            let world = args.player.world();
            let cause = if item == &Item::FIRE_CHARGE {
                "FIRE_CHARGE"
            } else {
                "FLINT_AND_STEEL"
            };
            let player = args.player.as_arc();
            let _ = Self::prime(&world, args.position, cause.to_string(), player).await;

            BlockActionResult::Consume
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if block_receives_redstone_power(args.world, args.position).await {
                let _ = Self::prime(
                    args.world,
                    args.position,
                    "REDSTONE".to_string(),
                    None,
                )
                .await;
            }
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if block_receives_redstone_power(args.world, args.position).await {
                let _ = Self::prime(
                    args.world,
                    args.position,
                    "REDSTONE".to_string(),
                    None,
                )
                .await;
            }
        })
    }

    fn explode<'a>(&'a self, args: ExplodeArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(server) = args.world.server.upgrade() {
                let event = TNTPrimeEvent::new(
                    None,
                    &Block::TNT,
                    *args.position,
                    args.world.uuid,
                    "EXPLOSION".to_string(),
                );
                let event = server.plugin_manager.fire(event).await;
                if event.cancelled {
                    return;
                }
            }
            let entity = Entity::new(args.world.clone(), args.position.to_f64(), &EntityType::TNT);
            let angle = rand::random::<f64>() * std::f64::consts::TAU;
            entity
                .set_velocity(Vector3::new(-angle.sin() * 0.02, 0.2, -angle.cos() * 0.02))
                .await;
            let fuse = rand::rng().random_range(0..DEFAULT_FUSE / 4) + DEFAULT_FUSE / 8;
            let tnt = Arc::new(TNTEntity::new(entity, DEFAULT_POWER, fuse));
            args.world.spawn_entity(tnt).await;
        })
    }

    fn should_drop_items_on_explosion(&self) -> bool {
        false
    }
}
