use crate::block::blocks::weathering_copper::{change_over_time, get_weather_state};
use crate::block::entities::copper_golem_statue::CopperGolemStatueBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, GetStateForNeighborUpdateArgs, NormalUseArgs,
    OnPlaceArgs, OnStateReplacedArgs, PathComputationType, RandomTickArgs, UseWithItemArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::{
    CopperGolemPose, CopperGolemStatueLikeProperties, EnumVariants,
};
use pumpkin_data::data_component_impl::BlockStateImpl;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:copper_golem_statues")]
pub struct CopperGolemStatueBlock;

impl CopperGolemStatueBlock {
    fn update_pose(world: &Arc<World>, position: &BlockPos, block: &Block) {
        let mut props =
            CopperGolemStatueLikeProperties::from_state_id(world.get_block_state_id(position));
        props.copper_golem_pose = CopperGolemPose::from_index(
            (props.copper_golem_pose.to_index() + 1) % CopperGolemPose::variant_count(),
        );
        world.play_block_sound(
            Sound::EntityCopperGolemBecomeStatue,
            SoundCategory::Blocks,
            *position,
        );
        world.set_block_state(position, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        world.emit_game_event(GameEvent::BlockChange.name(), position.to_centered_f64());
    }
}

impl BlockBehaviour for CopperGolemStatueBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CopperGolemStatueLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.waterlogged = args.replacing.water_source();
        if let Ok(hand) = Hand::from_packet_id(args.use_item_on.hand.0)
            && let Some(component) = args
                .player
                .inventory()
                .get_stack_in_hand(hand)
                .get_data_component::<BlockStateImpl>()
            && let Some((_, value)) = component
                .properties
                .iter()
                .find(|(name, _)| name == "copper_golem_pose")
            && let Some(pose) = (0..CopperGolemPose::variant_count())
                .map(CopperGolemPose::from_index)
                .find(|pose| pose.to_value() == value.as_ref())
        {
            props.copper_golem_pose = pose;
        }
        props.to_state_id(args.block)
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.item.has_tag(&tag::Item::MINECRAFT_AXES) {
            if args.block != &Block::COPPER_GOLEM_STATUE {
                return BlockActionResult::Pass;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            let Some(statue) = block_entity
                .as_any()
                .downcast_ref::<CopperGolemStatueBlockEntity>()
            else {
                return BlockActionResult::Pass;
            };
            let state = args.world.get_block_state(args.position);
            let golem = statue.remove_statue(args.world, state);
            if !args.world.spawn_entity(golem.clone()) {
                return BlockActionResult::Fail;
            }
            args.player.damage_item_in_slot(args.equipment_slot, 1);
            golem.play_spawn_sound();
            let replacement = if state.is_waterlogged() {
                &Block::WATER
            } else {
                &Block::AIR
            };
            args.world.set_block_state(
                args.position,
                replacement.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            return BlockActionResult::Success;
        }
        if args.item_stack.item == &Item::HONEYCOMB && get_weather_state(args.block).is_some() {
            return BlockActionResult::Pass;
        }
        Self::update_pose(args.world, args.position, args.block);
        BlockActionResult::Success
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::update_pose(args.world, args.position, args.block);
        BlockActionResult::Success
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.state_id.is_waterlogged() {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn is_pathfindable(&self, state: &BlockState, computation_type: PathComputationType) -> bool {
        computation_type == PathComputationType::Water && state.is_waterlogged()
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world
            .update_neighbour_for_output_signal(args.position, args.block);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let props = CopperGolemStatueLikeProperties::from_state_id(args.state.id);
        Some(props.copper_golem_pose.to_index() as u8 + 1)
    }
}
