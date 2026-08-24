use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties},
};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;
use crate::{
    generation::block_state_provider::BlockStateProvider,
    world::{BlockAccessor, WorldPortalExt},
};

pub struct SimpleBlockFeature {
    pub to_place: BlockStateProvider,
    pub schedule_tick: Option<bool>,
}

impl SimpleBlockFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let state = self.to_place.get(random, pos, chunk, block_registry);
        let (state_to_place, upper_state) = tall_plant_states(state)
            .map_or((state, None), |(lower_state, upper_state)| {
                (lower_state, Some(upper_state))
            });
        let block = Block::from_state_id(state_to_place.id);
        let block_accessor: &dyn BlockAccessor = chunk;
        if !block_registry.can_place_at(block, state_to_place, block_accessor, &pos) {
            return false;
        }

        if let Some(upper_state) = upper_state {
            let upper_pos = pos.up();

            // Vanilla places a double plant only when both halves fit. Check the upper block
            // before writing the lower one so a failed attempt cannot leave an orphaned plant.
            if !chunk.is_air(&upper_pos.0) {
                return false;
            }

            chunk.set_block_state(&pos.0, state_to_place);
            chunk.set_block_state(&upper_pos.0, upper_state);
        } else {
            chunk.set_block_state(&pos.0, state_to_place);
        }

        // TODO: schedule tick when needed
        true
    }
}

/// Resolves the two states used by Minecraft's tall natural plants. Restricting this to the
/// generated property group avoids mistaking stairs or other blocks with an unrelated half field
/// for a two-block plant.
fn tall_plant_states(
    state: &'static BlockState,
) -> Option<(&'static BlockState, &'static BlockState)> {
    let block = Block::from_state_id(state.id);
    if !TallSeagrassLikeProperties::handles_block_id(block.id) {
        return None;
    }

    let mut properties = TallSeagrassLikeProperties::from_state_id(state.id, block);
    properties.r#half = DoubleBlockHalf::Lower;
    let lower_state = BlockState::from_id(properties.to_state_id(block));
    properties.r#half = DoubleBlockHalf::Upper;
    let upper_state = BlockState::from_id(properties.to_state_id(block));

    Some((lower_state, upper_state))
}

#[cfg(test)]
mod tests {
    use super::tall_plant_states;
    use pumpkin_data::{
        Block,
        block_properties::{BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties},
    };

    #[test]
    fn tall_plants_resolve_to_a_lower_and_upper_pair() {
        let (lower, upper) = tall_plant_states(Block::SUNFLOWER.default_state)
            .expect("sunflowers use the tall-plant property group");

        let lower_properties =
            TallSeagrassLikeProperties::from_state_id(lower.id, &Block::SUNFLOWER);
        let upper_properties =
            TallSeagrassLikeProperties::from_state_id(upper.id, &Block::SUNFLOWER);

        assert_eq!(lower_properties.r#half, DoubleBlockHalf::Lower);
        assert_eq!(upper_properties.r#half, DoubleBlockHalf::Upper);
    }

    #[test]
    fn unrelated_half_blocks_are_not_treated_as_tall_plants() {
        assert!(tall_plant_states(Block::OAK_STAIRS.default_state).is_none());
    }
}
