use pumpkin_data::sound::Sound;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    tag::{self},
};
use pumpkin_util::math::position::BlockPos;

use crate::{
    block::{
        BlockBehaviour, BlockMetadata, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetRedstonePowerArgs, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
        OnScheduledTickArgs, OnStateReplacedArgs,
    },
    entity::EntityBase,
    world::World,
};

use super::{PressurePlate, detection_box_at};

/// This is for Normal Pressure plates, so not Gold or Iron
pub struct PressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::StonePressurePlateLikeProperties;

impl BlockMetadata for PressurePlateBlock {
    fn ids() -> Box<[BlockId]> {
        let mut combined = Vec::new();
        combined.extend_from_slice(tag::Block::MINECRAFT_WOODEN_PRESSURE_PLATES.1);
        combined.extend_from_slice(tag::Block::MINECRAFT_STONE_PRESSURE_PLATES.1);
        combined.iter().map(|v| BlockId::new_or_air(*v)).collect()
    }
}

impl BlockBehaviour for PressurePlateBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        self.on_entity_collision_pp(args);
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let output = self.get_redstone_output(args.block, state.id);
        if output > 0 {
            let (block, state) = args.world.get_block_and_state(args.position);
            Self.update_plate_state(args.world, args.position, block, state, output);
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        self.on_state_replaced_pp(args);
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        self.get_redstone_output(args.block, args.state.id)
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.direction == BlockDirection::Up {
            return self.get_redstone_output(args.block, args.state.id);
        }
        0
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down
            && !Self::can_pressure_plate_place_at(args.world, args.position)
        {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_pressure_plate_place_at(args.block_accessor, args.position)
    }
}

impl PressurePlate for PressurePlateBlock {
    fn get_redstone_output(&self, _block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state);
        if props.powered { 15 } else { 0 }
    }

    fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8 {
        let aabb = detection_box_at(pos);
        let require_living = block == &Block::STONE_PRESSURE_PLATE
            || block == &Block::POLISHED_BLACKSTONE_PRESSURE_PLATE;
        let has_entity = world.get_entities_at_box(&aabb).iter().any(|e| {
            !e.get_entity().is_spectator() && (!require_living || e.get_living_entity().is_some())
        }) || world
            .get_players_at_box(&aabb)
            .iter()
            .any(|p| !p.get_entity().is_spectator());
        if has_entity { 15 } else { 0 }
    }

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId {
        let mut props = PressurePlateProps::from_state_id(state.id);
        props.powered = output > 0;
        props.to_state_id(block)
    }

    fn click_sounds(&self, block: &Block) -> (Sound, Sound) {
        if block.has_tag(&tag::Block::MINECRAFT_WOODEN_PRESSURE_PLATES) {
            (
                Sound::BlockWoodenPressurePlateClickOn,
                Sound::BlockWoodenPressurePlateClickOff,
            )
        } else {
            (
                Sound::BlockStonePressurePlateClickOn,
                Sound::BlockStonePressurePlateClickOff,
            )
        }
    }
}
