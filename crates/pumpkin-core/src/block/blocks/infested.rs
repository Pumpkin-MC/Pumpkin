use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;

use crate::block::BlockBehaviour;
use crate::block::BrokenArgs;
use crate::entity::Entity;

#[pumpkin_block_from_tag("c:cobblestones/infested")]
pub struct InfestedBlock;

// Vanilla `InfestedBlock.BLOCK_BY_HOST_BLOCK`.
const HOST_TO_INFESTED: &[(&Block, &Block)] = &[
    (&Block::STONE, &Block::INFESTED_STONE),
    (&Block::COBBLESTONE, &Block::INFESTED_COBBLESTONE),
    (&Block::STONE_BRICKS, &Block::INFESTED_STONE_BRICKS),
    (
        &Block::MOSSY_STONE_BRICKS,
        &Block::INFESTED_MOSSY_STONE_BRICKS,
    ),
    (
        &Block::CRACKED_STONE_BRICKS,
        &Block::INFESTED_CRACKED_STONE_BRICKS,
    ),
    (
        &Block::CHISELED_STONE_BRICKS,
        &Block::INFESTED_CHISELED_STONE_BRICKS,
    ),
    (&Block::DEEPSLATE, &Block::INFESTED_DEEPSLATE),
];

impl InfestedBlock {
    fn infested_by_host(host: &Block) -> Option<&'static Block> {
        HOST_TO_INFESTED
            .iter()
            .find(|(candidate, _)| candidate.id == host.id)
            .map(|(_, infested)| *infested)
    }

    // Vanilla `InfestedBlock.isCompatibleHostBlock`.
    #[must_use]
    pub fn is_compatible_host_block(block_state: &BlockState) -> bool {
        Self::infested_by_host(block_state.id.to_block()).is_some()
    }

    // Vanilla `InfestedBlock.infestedStateByHost`.
    #[must_use]
    pub fn infested_state_by_host(host_state: &BlockState) -> Option<BlockStateId> {
        let infested = Self::infested_by_host(host_state.id.to_block())?;
        Some(copy_properties(host_state, infested))
    }

    // Vanilla `InfestedBlock.hostStateByInfested`.
    #[must_use]
    pub fn host_state_by_infested(infested_state: &BlockState) -> Option<BlockStateId> {
        let infested_id = infested_state.id.to_block_id();
        let host = HOST_TO_INFESTED
            .iter()
            .find(|(_, infested)| infested.id == infested_id)?
            .0;
        Some(copy_properties(infested_state, host))
    }
}

// Vanilla `InfestedBlock.getNewStateWithProperties`.
fn copy_properties(from: &BlockState, to: &'static Block) -> BlockStateId {
    from.id
        .to_block()
        .properties(from.id)
        .map_or(to.default_state.id, |properties| {
            to.from_properties(&properties.to_props()).to_state_id(to)
        })
}

impl BlockBehaviour for InfestedBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        {
            // TODO: ugly fix, use onStacksDropped
            if args.player.gamemode.load() == GameMode::Creative {
                return;
            }
            let entity = Entity::new(
                args.world.clone(),
                args.position.0.to_f64(),
                &EntityType::SILVERFISH,
            );

            args.world.spawn_entity(Arc::new(entity));
        }
    }
}
