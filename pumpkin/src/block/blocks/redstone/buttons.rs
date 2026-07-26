use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::AttachFace;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

type ButtonLikeProperties = pumpkin_data::block_properties::LeverLikeProperties;

use crate::block::BlockFuture;
use crate::block::CanPlaceAtArgs;
use crate::block::EmitsRedstonePowerArgs;
use crate::block::GetRedstonePowerArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnScheduledTickArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::blocks::abstract_wall_mounting::WallMountedBlock;
use crate::block::blocks::redstone::lever::LeverLikePropertiesExt;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs};
use crate::entity::player::Player;
use crate::world::World;

/// Vanilla `BlockSetType.buttonClickOn/Off` per button material.
fn button_click_sound(block: &Block, pressed: bool) -> Sound {
    if block.has_tag(&tag::Block::MINECRAFT_STONE_BUTTONS) {
        if pressed {
            Sound::BlockStoneButtonClickOn
        } else {
            Sound::BlockStoneButtonClickOff
        }
    } else if block == &Block::BAMBOO_BUTTON {
        if pressed {
            Sound::BlockBambooWoodButtonClickOn
        } else {
            Sound::BlockBambooWoodButtonClickOff
        }
    } else if block == &Block::CHERRY_BUTTON {
        if pressed {
            Sound::BlockCherryWoodButtonClickOn
        } else {
            Sound::BlockCherryWoodButtonClickOff
        }
    } else if block == &Block::CRIMSON_BUTTON || block == &Block::WARPED_BUTTON {
        if pressed {
            Sound::BlockNetherWoodButtonClickOn
        } else {
            Sound::BlockNetherWoodButtonClickOff
        }
    } else if pressed {
        Sound::BlockWoodenButtonClickOn
    } else {
        Sound::BlockWoodenButtonClickOff
    }
}

/// Vanilla `AbstractArrow` occupancy test for `ButtonBlock.checkPressed`,
/// approximated with the button's full block space.
fn arrow_resting_at(world: &World, block_pos: &BlockPos) -> bool {
    let mut arrows = Vec::new();
    world.extend_entities_in_box_where(
        &mut arrows,
        1,
        BoundingBox::new(
            block_pos.to_f64(),
            block_pos.to_f64().add_raw(1.0, 1.0, 1.0),
        ),
        |entity| {
            let entity_type = entity.get_entity().entity_type;
            entity_type == &EntityType::ARROW
                || entity_type == &EntityType::SPECTRAL_ARROW
                || entity_type == &EntityType::TRIDENT
        },
    );
    !arrows.is_empty()
}

/// Vanilla `ButtonBlock.press` for a non-player cause (arrows).
pub async fn press_button_by_arrow(world: &Arc<World>, block_pos: &BlockPos) {
    let (block, state) = world.get_block_and_state_id(block_pos);
    if !block.has_tag(&tag::Block::MINECRAFT_BUTTONS)
        || block.has_tag(&tag::Block::MINECRAFT_STONE_BUTTONS)
    {
        return;
    }
    let mut button_props = ButtonLikeProperties::from_state_id(state, block);
    if button_props.powered {
        return;
    }
    button_props.powered = true;
    world
        .set_block_state(
            block_pos,
            button_props.to_state_id(block),
            BlockFlags::NOTIFY_ALL,
        )
        .await;
    world.schedule_block_tick(block, *block_pos, 30, TickPriority::Normal);
    world.play_sound_fine(
        button_click_sound(block, true),
        SoundCategory::Blocks,
        &block_pos.to_centered_f64(),
        1.0,
        1.0,
    );
    world
        .emit_vibration(
            crate::world::vibrations::Vibration::BlockActivate,
            block_pos.to_centered_f64(),
        )
        .await;
    ButtonBlock::update_neighbors(world, block_pos, &button_props).await;
}

async fn click_button(world: &Arc<World>, block_pos: &BlockPos, player: &Player) {
    let (block, state) = world.get_block_and_state_id(block_pos);

    let mut button_props = ButtonLikeProperties::from_state_id(state, block);
    if !button_props.powered {
        button_props.powered = true;
        world
            .set_block_state(
                block_pos,
                button_props.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        // Vanilla ButtonBlock.ticksToStayPressed: stone-type 20, wooden 30.
        let delay = if block.has_tag(&tag::Block::MINECRAFT_STONE_BUTTONS) {
            20
        } else {
            30
        };
        world.schedule_block_tick(block, *block_pos, delay, TickPriority::Normal);
        // The pressing client predicts its own click-on sound.
        world.play_sound_raw_expect(
            player,
            button_click_sound(block, true) as u16,
            SoundCategory::Blocks,
            &block_pos.to_centered_f64(),
            1.0,
            1.0,
        );
        world
            .emit_vibration(
                crate::world::vibrations::Vibration::BlockActivate,
                block_pos.to_centered_f64(),
            )
            .await;
        ButtonBlock::update_neighbors(world, block_pos, &button_props).await;
    }
}

#[pumpkin_block_from_tag("minecraft:buttons")]
pub struct ButtonBlock;

impl BlockBehaviour for ButtonBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            click_button(args.world, args.position, args.player).await;

            BlockActionResult::Success
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let mut props = ButtonLikeProperties::from_state_id(state.id, args.block);
            if !props.powered {
                return;
            }

            // Vanilla checkPressed: wooden-type buttons stay pressed while an
            // arrow rests inside the button's block space.
            if !args.block.has_tag(&tag::Block::MINECRAFT_STONE_BUTTONS)
                && arrow_resting_at(args.world, args.position)
            {
                args.world.schedule_block_tick(
                    args.block,
                    *args.position,
                    30,
                    TickPriority::Normal,
                );
                return;
            }

            props.powered = false;
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            // Vanilla plays click-off to everyone on release.
            args.world.play_sound_fine(
                button_click_sound(args.block, false),
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
                1.0,
                1.0,
            );
            args.world
                .emit_vibration(
                    crate::world::vibrations::Vibration::BlockDeactivate,
                    args.position.to_centered_f64(),
                )
                .await;
            Self::update_neighbors(args.world, args.position, &props).await;
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let button_props = ButtonLikeProperties::from_state_id(args.state.id, args.block);
            if button_props.powered { 15 } else { 0 }
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let button_props = ButtonLikeProperties::from_state_id(args.state.id, args.block);
            if button_props.powered && button_props.get_direction() == args.direction {
                15
            } else {
                0
            }
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !args.moved {
                let button_props =
                    ButtonLikeProperties::from_state_id(args.old_state_id, args.block);
                if button_props.powered {
                    Self::update_neighbors(args.world, args.position, &button_props).await;
                }
            }
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                ButtonLikeProperties::from_state_id(args.block.default_state.id, args.block);
            (props.face, props.facing) =
                WallMountedBlock::get_placement_face(self, args.player, args.direction);

            props.to_state_id(args.block)
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        // Use the provided direction, or fallback to the current state's direction if missing
        let direction = args
            .direction
            .unwrap_or_else(|| self.get_direction(args.state.id, args.block));

        WallMountedBlock::can_place_at(self, args.block_accessor, args.position, direction)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { WallMountedBlock::get_state_for_neighbor_update(self, args).await })
    }
}

impl WallMountedBlock for ButtonBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = ButtonLikeProperties::from_state_id(state_id, block);
        match props.face {
            AttachFace::Floor => BlockDirection::Up,
            AttachFace::Ceiling => BlockDirection::Down,
            AttachFace::Wall => props.facing.to_block_direction(),
        }
    }
}

impl ButtonBlock {
    async fn update_neighbors(
        world: &Arc<World>,
        block_pos: &BlockPos,
        props: &ButtonLikeProperties,
    ) {
        let direction = props.get_direction().opposite();
        world.update_neighbors(block_pos, None).await;
        world
            .update_neighbors(&block_pos.offset(direction.to_offset()), None)
            .await;
    }
}
