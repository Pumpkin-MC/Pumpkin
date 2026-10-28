use pumpkin_data::{
    Block, BlockId, BlockState, BlockStateId, Rotation,
    block_properties::{Axis, PaleOakWoodLikeProperties},
    tag,
};

use crate::block::{BlockBehaviour, BlockMetadata, OnPlaceArgs};

type LogProperties = PaleOakWoodLikeProperties;

pub struct LogBlock;

impl BlockMetadata for LogBlock {
    fn ids() -> Box<[BlockId]> {
        let mut ids: Vec<BlockId> = tag::Block::MINECRAFT_LOGS
            .1
            .iter()
            .copied()
            .map(BlockId::new_or_air)
            .collect();
        ids.extend_from_slice(&[
            BlockId::BASALT,
            BlockId::POLISHED_BASALT,
            BlockId::QUARTZ_PILLAR,
            BlockId::PURPUR_PILLAR,
            BlockId::BONE_BLOCK,
            BlockId::DEEPSLATE,
            BlockId::OCHRE_FROGLIGHT,
            BlockId::VERDANT_FROGLIGHT,
            BlockId::PEARLESCENT_FROGLIGHT,
            BlockId::MUDDY_MANGROVE_ROOTS,
            BlockId::BAMBOO_BLOCK,
            BlockId::STRIPPED_BAMBOO_BLOCK,
        ]);
        ids.into_boxed_slice()
    }
}

impl BlockBehaviour for LogBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut log_props = LogProperties::default(args.block);
        log_props.axis = args.direction.to_axis();

        log_props.to_state_id(args.block)
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let mut log_props = LogProperties::from_state_id(state_id);
        if matches!(
            rotation,
            Rotation::Clockwise90 | Rotation::CounterClockwise90
        ) {
            log_props.axis = match log_props.axis {
                Axis::X => Axis::Z,
                Axis::Z => Axis::X,
                Axis::Y => Axis::Y,
            };
        }
        BlockState::from_id(log_props.to_state_id(block))
    }
}
