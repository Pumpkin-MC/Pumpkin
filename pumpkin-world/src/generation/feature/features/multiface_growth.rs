use pumpkin_data::block_properties::{BlockProperties, GlowLichenLikeProperties};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

/// `MultifaceBlock.canAttachTo`: vanilla checks `isFaceFull` against the neighbour's
/// block-support shape and collision shape. This codebase has no per-direction shape
/// query available to `pumpkin-world` (only the coarser `BlockState::is_side_solid`,
/// itself `isFaceSturdy()` in Java), so that's used here as a known simplification —
/// the same substitution `pumpkin/src/block/blocks/abstract_multiface.rs` documents
/// for the runtime (non-worldgen) multiface framework.
const fn can_attach_to(
    neighbour_state: &BlockState,
    direction_towards_neighbour: BlockDirection,
) -> bool {
    neighbour_state.is_side_solid(direction_towards_neighbour.opposite())
}

pub struct MultifaceGrowthFeature;

impl MultifaceGrowthFeature {
    /// `MultifaceGrowthFeature.place`/`placeGrowthIfPossible`
    /// (`net/minecraft/world/level/levelgen/feature/MultifaceGrowthFeature.java`), reduced to
    /// a single origin position (the placed-feature decorator already re-invokes this per
    /// generated position, matching vanilla's per-call `getShuffledDirections` loop over one
    /// origin at a time).
    ///
    /// Vanilla requires `origin` to already be air or water (`isAirOrWater`) before placing
    /// anything, then only attaches to a neighbour actually listed in the feature's
    /// `canBePlacedOn` set (`neighbourState.is(config.canBePlacedOn)`) — never "any non-air
    /// block". This codebase has no per-feature `canBePlacedOn` list wired through yet (see
    /// `pumpkin_data::configured_feature::ConfiguredFeature::GlowLichen`, a unit variant with
    /// no config payload), so `can_attach_to`'s sturdy-face check is used as the nearest
    /// available substitute. The previous implementation both skipped the origin
    /// air/water check and placed `Block::GLOW_LICHEN.default_state` unconditionally — a
    /// state with every face flag false, i.e. no attachment face at all, which the client
    /// renders as an invisible/floating block regardless of which position it occupies.
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let origin_state = GenerationCache::get_block_state(chunk, &pos.0);
        if !(chunk.is_air(&pos.0) || origin_state.to_block_id() == Block::WATER.id) {
            return false;
        }

        for direction in BlockDirection::all() {
            let neighbor = pos.offset(direction.to_offset());
            let neighbor_state = GenerationCache::get_block_state(chunk, &neighbor.0);
            if can_attach_to(BlockState::from_id(neighbor_state), direction) {
                let mut props = GlowLichenLikeProperties::default(&Block::GLOW_LICHEN);
                match direction {
                    BlockDirection::Down => props.r#down = true,
                    BlockDirection::Up => props.r#up = true,
                    BlockDirection::North => props.r#north = true,
                    BlockDirection::South => props.r#south = true,
                    BlockDirection::West => props.r#west = true,
                    BlockDirection::East => props.r#east = true,
                }
                chunk.set_block_state(
                    &pos.0,
                    BlockState::from_id(props.to_state_id(&Block::GLOW_LICHEN)),
                );
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::can_attach_to;
    use pumpkin_data::{Block, BlockDirection, BlockState};

    #[test]
    fn attaches_to_a_sturdy_face() {
        let stone = BlockState::from_id(Block::STONE.default_state.id);
        // Growth sits above the stone block, attaching to stone's up-facing side.
        assert!(can_attach_to(stone, BlockDirection::Down));
    }

    #[test]
    fn does_not_attach_to_air() {
        let air = BlockState::from_id(Block::AIR.default_state.id);
        assert!(!can_attach_to(air, BlockDirection::Down));
    }

    #[test]
    fn does_not_attach_to_a_non_full_block() {
        // Bamboo has no full/sturdy face on any side - growth must not be able to
        // attach to it, unlike the old "any non-air neighbour" check.
        let bamboo = BlockState::from_id(Block::BAMBOO.default_state.id);
        for direction in BlockDirection::all() {
            assert!(!can_attach_to(bamboo, direction));
        }
    }
}
