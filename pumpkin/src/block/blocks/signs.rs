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
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::block::entities::sign::DyeColor;
use pumpkin_world::block::entities::sign::SignBlockEntity;

use crate::block::pumpkin_block::NormalUseArgs;
use crate::block::pumpkin_block::OnStateReplacedArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::block::pumpkin_block::PlayerPlacedArgs;
use crate::block::pumpkin_block::UseWithItemArgs;
use crate::block::pumpkin_block::{BlockMetadata, OnPlaceArgs, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::world::World;

type SignProperties = pumpkin_data::block_properties::OakSignLikeProperties;

pub struct SignBlock;

impl BlockMetadata for SignBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:signs").unwrap()
    }
}

//TODO: Add support for Wall Signs
//TODO: Add support for Hanging Signs
//TODO: add support for click commands
//TODO: Check if other people are already editing
#[async_trait]
impl PumpkinBlock for SignBlock {
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
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() {
                if sign_entity.is_waxed.load(Ordering::Relaxed) {
                    args.world
                        .play_block_sound(
                            pumpkin_data::sound::Sound::BlockSignWaxedInteractFail,
                            pumpkin_data::sound::SoundCategory::Blocks,
                            *args.position,
                        )
                        .await;
                    return BlockActionResult::Success;
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

                return BlockActionResult::Success;
            }
        }
        BlockActionResult::Continue
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let Some(block_entity) = args.world.get_block_entity(args.position).await else {
            return BlockActionResult::PassToDefault;
        };
        let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() else {
            return BlockActionResult::PassToDefault;
        };

        if sign_entity.is_waxed.load(Ordering::Relaxed) {
            return BlockActionResult::PassToDefault;
        }

        let mut item = args.item_stack.lock().await;

        if item.item.id == pumpkin_data::item::Item::HONEYCOMB.id {
            sign_entity.is_waxed.store(true, Ordering::Relaxed);

            if !args.player.has_infinite_materials() {
                item.decrement(1);
            }
            drop(item);

            args.world.update_block_entity(block_entity).await;
            args.world
                .sync_world_event(WorldEvent::BlockWaxed, *args.position, 0)
                .await;

            return BlockActionResult::Success;
        }

        let is_facing_front_text =
            is_facing_front_text(args.world, args.position, args.block, args.player).await;

        let text = if is_facing_front_text {
            &sign_entity.front_text
        } else {
            &sign_entity.back_text
        };

        if item.item.id == pumpkin_data::item::Item::GLOW_INK_SAC.id {
            let changed = !text.has_glowing_text.swap(true, Ordering::Relaxed);

            if !changed {
                return BlockActionResult::PassToDefault;
            }

            if !args.player.has_infinite_materials() {
                item.decrement(1);
            }
            drop(item);

            args.world.update_block_entity(block_entity).await;
            args.world
                .play_block_sound(
                    pumpkin_data::sound::Sound::ItemGlowInkSacUse,
                    pumpkin_data::sound::SoundCategory::Blocks,
                    *args.position,
                )
                .await;
            return BlockActionResult::Success;
        }

        if item.item.id == pumpkin_data::item::Item::INK_SAC.id {
            let changed = text.has_glowing_text.swap(false, Ordering::Relaxed);

            if !changed {
                return BlockActionResult::PassToDefault;
            }

            if !args.player.has_infinite_materials() {
                item.decrement(1);
            }
            drop(item);

            args.world.update_block_entity(block_entity).await;
            args.world
                .play_block_sound(
                    pumpkin_data::sound::Sound::ItemInkSacUse,
                    pumpkin_data::sound::SoundCategory::Blocks,
                    *args.position,
                )
                .await;
            return BlockActionResult::Success;
        }

        if let Some(color_name) = item.item.registry_key.strip_suffix("_dye") {
            let dye_color = DyeColor::from(color_name.to_string());

            text.set_color(dye_color);

            if !args.player.has_infinite_materials() {
                item.decrement(1);
            }
            drop(item);

            args.world.update_block_entity(block_entity).await;
            args.world
                .play_block_sound(
                    pumpkin_data::sound::Sound::ItemDyeUse,
                    pumpkin_data::sound::SoundCategory::Blocks,
                    *args.position,
                )
                .await;
            return BlockActionResult::Success;
        }

        BlockActionResult::PassToDefault
    }
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
