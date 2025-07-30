use std::sync::Arc;
use std::sync::atomic::Ordering;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::EnumVariants;
use pumpkin_data::block_properties::Integer0To15;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_data::world::WorldEvent;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::block::entities::BlockEntity;
use pumpkin_world::block::entities::sign::DyeColor;
use pumpkin_world::block::entities::sign::SignBlockEntity;
use pumpkin_world::block::entities::sign::Text;
use uuid::Uuid;

use crate::block::BlockBehaviour;
use crate::block::NormalUseArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::block::PlayerPlacedArgs;
use crate::block::UseWithItemArgs;
use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::world::World;

type SignProperties = pumpkin_data::block_properties::OakSignLikeProperties;

#[pumpkin_block_from_tag("minecraft:signs")]
pub struct SignBlock;

//TODO: Add support for Wall Signs
//TODO: Add support for Hanging Signs
//TODO: add support for click commands
#[async_trait]
impl BlockBehaviour for SignBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> u16 {
        let mut sign_props = SignProperties::default(args.block);
        sign_props.waterlogged = args.replacing.water_source();
        sign_props.rotation = args.player.get_entity().get_flipped_rotation_16();
        sign_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(SignBlockEntity::empty(*args.position)))
            .await;
    }

    async fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        match &args.player.client {
            crate::net::ClientPlatform::Java(java) => {
                java.send_sign_packet(*args.position, true).await;
            }
            crate::net::ClientPlatform::Bedrock(_bedrock) => todo!(),
        }
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.position).await;
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let Some(block_entity) = args.world.get_block_entity(args.position).await else {
            return BlockActionResult::Pass;
        };
        let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() else {
            return BlockActionResult::Pass;
        };

        if sign_entity.is_waxed.load(Ordering::Relaxed) {
            args.world
                .play_block_sound(
                    pumpkin_data::sound::Sound::BlockSignWaxedInteractFail,
                    pumpkin_data::sound::SoundCategory::Blocks,
                    *args.position,
                )
                .await;
            return BlockActionResult::SuccessServer;
        }

        let mut currently_editing = sign_entity.currently_editing_player.lock().await;
        if !try_claim_sign(
            &mut currently_editing,
            &args.player.gameprofile.id,
            args.world,
            args.position,
        )
        .await
        {
            return BlockActionResult::Pass;
        }

        let is_facing_front_text =
            is_facing_front_text(args.world, args.position, args.block, args.player).await;
        match &args.player.client {
            crate::net::ClientPlatform::Java(java) => {
                java.send_sign_packet(*args.position, is_facing_front_text)
                    .await;
            }
            crate::net::ClientPlatform::Bedrock(_bedrock) => todo!(),
        }

        BlockActionResult::SuccessServer
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let Some(block_entity) = args.world.get_block_entity(args.position).await else {
            return BlockActionResult::Pass;
        };
        let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() else {
            return BlockActionResult::Pass;
        };

        if sign_entity.is_waxed.load(Ordering::Relaxed) {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        let mut currently_editing = sign_entity.currently_editing_player.lock().await;
        if !try_claim_sign(
            &mut currently_editing,
            &args.player.gameprofile.id,
            args.world,
            args.position,
        )
        .await
        {
            // I don't think that makes sense, since it will also just return in normal_use, but vanilla does it like this
            return BlockActionResult::PassToDefaultBlockAction;
        }

        let text = if is_facing_front_text(args.world, args.position, args.block, args.player).await
        {
            &sign_entity.front_text
        } else {
            &sign_entity.back_text
        };

        let mut item = args.item_stack.lock().await;

        let result = match item.item.id {
            id if id == pumpkin_data::item::Item::HONEYCOMB.id => {
                apply_wax_to_sign(&args, &block_entity, sign_entity).await
            }
            id if id == pumpkin_data::item::Item::GLOW_INK_SAC.id => {
                apply_glow_ink_to_sign(&args, &block_entity, text).await
            }
            id if id == pumpkin_data::item::Item::INK_SAC.id => {
                apply_ink_to_sign(&args, &block_entity, text).await
            }
            _ => {
                if let Some(color_name) = item.item.registry_key.strip_suffix("_dye") {
                    apply_dye_to_sign(&args, &block_entity, text, color_name).await
                } else {
                    BlockActionResult::PassToDefaultBlockAction
                }
            }
        };

        if result == BlockActionResult::Success {
            if !args.player.has_infinite_materials() {
                item.decrement(1);
            }
            *currently_editing = None;
        }

        result
    }
}

async fn apply_wax_to_sign(
    args: &UseWithItemArgs<'_>,
    block_entity: &Arc<dyn BlockEntity>,
    sign_entity: &SignBlockEntity,
) -> BlockActionResult {
    sign_entity.is_waxed.store(true, Ordering::Relaxed);

    args.world.update_block_entity(block_entity).await;
    args.world
        .sync_world_event(WorldEvent::BlockWaxed, *args.position, 0)
        .await;

    BlockActionResult::Success
}

async fn apply_glow_ink_to_sign(
    args: &UseWithItemArgs<'_>,
    block_entity: &Arc<dyn BlockEntity>,
    text: &Text,
) -> BlockActionResult {
    let changed = !text.has_glowing_text.swap(true, Ordering::Relaxed);

    if !changed {
        return BlockActionResult::PassToDefaultBlockAction;
    }

    args.world.update_block_entity(block_entity).await;
    args.world
        .play_block_sound(
            pumpkin_data::sound::Sound::ItemGlowInkSacUse,
            pumpkin_data::sound::SoundCategory::Blocks,
            *args.position,
        )
        .await;
    BlockActionResult::Success
}

async fn apply_ink_to_sign(
    args: &UseWithItemArgs<'_>,
    block_entity: &Arc<dyn BlockEntity>,
    text: &Text,
) -> BlockActionResult {
    let changed = text.has_glowing_text.swap(false, Ordering::Relaxed);

    if !changed {
        return BlockActionResult::PassToDefaultBlockAction;
    }

    args.world.update_block_entity(block_entity).await;
    args.world
        .play_block_sound(
            pumpkin_data::sound::Sound::ItemInkSacUse,
            pumpkin_data::sound::SoundCategory::Blocks,
            *args.position,
        )
        .await;
    BlockActionResult::Success
}

async fn apply_dye_to_sign(
    args: &UseWithItemArgs<'_>,
    block_entity: &Arc<dyn BlockEntity>,
    text: &Text,
    color_name: &str,
) -> BlockActionResult {
    let dye_color = DyeColor::from(color_name);

    text.set_color(dye_color);

    args.world.update_block_entity(block_entity).await;
    args.world
        .play_block_sound(
            pumpkin_data::sound::Sound::ItemDyeUse,
            pumpkin_data::sound::SoundCategory::Blocks,
            *args.position,
        )
        .await;
    BlockActionResult::Success
}

async fn is_facing_front_text(
    world: &World,
    location: &BlockPos,
    block: &Block,
    player: &Player,
) -> bool {
    let state_id = world.get_block_state_id(location).await;
    let sign_properties = SignProperties::from_state_id(state_id, block);
    let rotation = get_yaw_from_rotation_16(sign_properties.rotation);
    let bounding_box = Vector3::new(0.5, 0.5, 0.5);

    let d = player.eye_position().x - (f64::from(location.0.x) + bounding_box.x);
    let d1 = player.eye_position().z - (f64::from(location.0.z) + bounding_box.z);

    let f = (d1.atan2(d).to_degrees() as f32) - 90.0;

    let diff = (f - rotation + 180.0).rem_euclid(360.0) - 180.0;
    diff.abs() <= 90.0
}

fn get_yaw_from_rotation_16(rotation: Integer0To15) -> f32 {
    let index = rotation.to_index();

    f32::from(index) * 22.5
}

async fn try_claim_sign(
    currently_editing: &mut Option<Uuid>,
    uuid: &Uuid,
    world: &World,
    position: &BlockPos,
) -> bool {
    if let Some(editing_player_id) = *currently_editing {
        if editing_player_id != *uuid {
            if let Some(editing_player) = world.get_player_by_uuid(editing_player_id).await {
                if editing_player.can_interact_with_block_at(position, 4.0) {
                    return false;
                }
            }
        }
    }
    *currently_editing = Some(*uuid);
    true
}
