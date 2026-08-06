use crate::block::registry::BlockActionResult;
use crate::block::{BlockFuture, UseWithItemArgs};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::world::game_event::{GameEventContext, emit_game_event};
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

#[pumpkin_block("minecraft:pumpkin")]
pub struct PumpkinBlock;

impl crate::block::BlockBehaviour for PumpkinBlock {
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if !args.item_stack.lock().await.is_shears() {
                return BlockActionResult::Pass;
            }

            // vanilla PumpkinBlock#useItemOn: carve towards the clicked face, unless that
            // face is the top/bottom (axis Y), in which case carve towards the player's back.
            let facing = args
                .hit
                .face
                .to_horizontal_facing()
                .unwrap_or_else(|| args.player.living_entity.entity.get_horizontal_facing());

            let mut props = WallTorchLikeProperties::default(&Block::CARVED_PUMPKIN);
            props.facing = facing;
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(&Block::CARVED_PUMPKIN),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            args.world.play_block_sound(
                Sound::BlockPumpkinCarve,
                SoundCategory::Blocks,
                *args.position,
            );

            let offset = facing.to_offset();
            let spawn_pos = Vector3::new(
                f64::from(args.position.0.x) + 0.5 + f64::from(offset.x) * 0.65,
                f64::from(args.position.0.y) + 0.1,
                f64::from(args.position.0.z) + 0.5 + f64::from(offset.z) * 0.65,
            );
            let velocity = Vector3::new(
                0.05 * f64::from(offset.x) + rand::random::<f64>() * 0.02,
                0.05,
                0.05 * f64::from(offset.z) + rand::random::<f64>() * 0.02,
            );
            let entity = Entity::new(args.world.clone(), spawn_pos, &EntityType::ITEM);
            let item_entity = Arc::new(ItemEntity::new_with_velocity(
                entity,
                ItemStack::new(4, &Item::PUMPKIN_SEEDS),
                velocity,
                10,
            ));
            args.world.spawn_entity(item_entity).await;

            args.player
                .damage_item_in_slot(args.equipment_slot, 1)
                .await;

            emit_game_event(
                args.world,
                GameEvent::Shear,
                args.position.to_f64(),
                GameEventContext::of_entity(args.player.clone()),
            )
            .await;

            BlockActionResult::Consume
        })
    }
}
