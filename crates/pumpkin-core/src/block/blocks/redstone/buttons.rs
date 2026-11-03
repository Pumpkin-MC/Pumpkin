use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::AttachFace;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

type ButtonLikeProperties = pumpkin_data::block_properties::LeverLikeProperties;

use crate::block::blocks::abstract_wall_mounting::WallMountedBlock;
use crate::block::blocks::redstone::lever::LeverLikePropertiesExt;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    GetStateForNeighborUpdateArgs, NormalUseArgs, OnEntityCollisionArgs, OnPlaceArgs,
    OnScheduledTickArgs, OnStateReplacedArgs,
};
use crate::world::World;

fn is_stone_button(block: &Block) -> bool {
    block == &Block::STONE_BUTTON || block == &Block::POLISHED_BLACKSTONE_BUTTON
}

fn is_abstract_arrow(entity_type: &EntityType) -> bool {
    entity_type == &EntityType::ARROW
        || entity_type == &EntityType::SPECTRAL_ARROW
        || entity_type == &EntityType::TRIDENT
}

fn get_sound(block: &Block, on: bool) -> Sound {
    if is_stone_button(block) {
        if on {
            Sound::BlockStoneButtonClickOn
        } else {
            Sound::BlockStoneButtonClickOff
        }
    } else if on {
        Sound::BlockWoodenButtonClickOn
    } else {
        Sound::BlockWoodenButtonClickOff
    }
}

#[pumpkin_block_from_tag("minecraft:buttons")]
pub struct ButtonBlock;

impl BlockBehaviour for ButtonBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if Self::press(args.world, args.position) {
            BlockActionResult::Success
        } else {
            BlockActionResult::Consume
        }
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if !is_stone_button(args.block) {
            let entity_type = args.entity.get_entity().entity_type;
            if is_abstract_arrow(entity_type) {
                Self::press(args.world, args.position);
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let mut props = ButtonLikeProperties::from_state_id(state.id);
        if props.powered {
            let stay_pressed = if is_stone_button(args.block) {
                false
            } else {
                let aabb = BoundingBox::from_block(args.position);
                args.world
                    .get_entities_at_box(&aabb)
                    .iter()
                    .any(|e| is_abstract_arrow(e.get_entity().entity_type))
            };

            if stay_pressed {
                args.world.schedule_block_tick(
                    args.block,
                    *args.position,
                    30,
                    TickPriority::Normal,
                );
            } else {
                props.powered = false;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
                Self::update_neighbors(args.world, args.position, props);
                args.world.play_block_sound(
                    get_sound(args.block, false),
                    SoundCategory::Blocks,
                    *args.position,
                );
                args.world.emit_game_event(
                    GameEvent::BlockDeactivate.name(),
                    args.position.to_centered_f64(),
                );
            }
        }
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let button_props = ButtonLikeProperties::from_state_id(args.state.id);
        if button_props.powered { 15 } else { 0 }
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let button_props = ButtonLikeProperties::from_state_id(args.state.id);
        if button_props.powered && button_props.get_direction() == args.direction {
            15
        } else {
            0
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            let button_props = ButtonLikeProperties::from_state_id(args.old_state_id);
            if button_props.powered {
                Self::update_neighbors(args.world, args.position, button_props);
            }
        }
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = ButtonLikeProperties::from_state_id(args.block.default_state.id);
        (props.face, props.facing) = WallMountedBlock::get_placement_face_for_args(self, &args);

        props.to_state_id(args.block)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        WallMountedBlock::can_place_at_args(self, &args)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        WallMountedBlock::get_state_for_neighbor_update(self, args)
    }
}

impl WallMountedBlock for ButtonBlock {
    fn get_direction(&self, state_id: BlockStateId, _block: &Block) -> BlockDirection {
        let props = ButtonLikeProperties::from_state_id(state_id);
        match props.face {
            AttachFace::Floor => BlockDirection::Up,
            AttachFace::Ceiling => BlockDirection::Down,
            AttachFace::Wall => props.facing.to_block_direction(),
        }
    }
}

impl ButtonBlock {
    /// Presses the button, unless it is already pressed. Returns whether it was
    /// pressed, so callers can tell the two cases apart the way vanilla's
    /// `ButtonBlock::useWithoutItem` does.
    pub(crate) fn press(world: &Arc<World>, block_pos: &BlockPos) -> bool {
        let (block, state) = world.get_block_and_state_id(block_pos);

        let mut button_props = ButtonLikeProperties::from_state_id(state);
        if button_props.powered {
            return false;
        }

        button_props.powered = true;
        world.set_block_state(
            block_pos,
            button_props.to_state_id(block),
            BlockFlags::NOTIFY_ALL,
        );
        let delay = if is_stone_button(block) { 20 } else { 30 };
        world.schedule_block_tick(block, *block_pos, delay, TickPriority::Normal);
        Self::update_neighbors(world, block_pos, button_props);
        world.play_block_sound(get_sound(block, true), SoundCategory::Blocks, *block_pos);
        world.emit_game_event(GameEvent::BlockActivate.name(), block_pos.to_centered_f64());

        true
    }

    fn update_neighbors(world: &Arc<World>, block_pos: &BlockPos, props: ButtonLikeProperties) {
        let direction = props.get_direction().opposite();
        world.update_neighbors(block_pos, None);
        world.update_neighbors(&block_pos.offset(direction.to_offset()), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stone_button_classification_matches_vanilla() {
        assert!(is_stone_button(&Block::STONE_BUTTON));
        assert!(is_stone_button(&Block::POLISHED_BLACKSTONE_BUTTON));
        assert!(!is_stone_button(&Block::OAK_BUTTON));
        assert!(!is_stone_button(&Block::SPRUCE_BUTTON));
        assert!(!is_stone_button(&Block::BIRCH_BUTTON));
        assert!(!is_stone_button(&Block::JUNGLE_BUTTON));
        assert!(!is_stone_button(&Block::ACACIA_BUTTON));
        assert!(!is_stone_button(&Block::DARK_OAK_BUTTON));
        assert!(!is_stone_button(&Block::MANGROVE_BUTTON));
        assert!(!is_stone_button(&Block::CHERRY_BUTTON));
        assert!(!is_stone_button(&Block::BAMBOO_BUTTON));
        assert!(!is_stone_button(&Block::CRIMSON_BUTTON));
        assert!(!is_stone_button(&Block::WARPED_BUTTON));
    }
}
