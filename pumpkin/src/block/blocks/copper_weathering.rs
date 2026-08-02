use crate::world::World;
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

/// Copper oxidation transition table entry: (from, to, `current_level`)
/// `current_level` is 0 for unoxidized, 1 for exposed, 2 for weathered, 3 for oxidized
pub type CopperOxidationStage = (&'static Block, &'static Block, u8);

/// Base chance per random tick for copper to oxidize: ~5.69%
const BASE_OXIDATION_CHANCE: f32 = 0.056_888_89;

/// Try to oxidize a copper block to its next oxidation level.
/// Uses vanilla's degradation algorithm with neighbor checking.
///
/// # Arguments
/// * `world` - The world to check neighbors in
/// * `position` - The position of the block to potentially oxidize
/// * `current_block` - The current block state
/// * `oxidation_stages` - Array of 4 blocks [unoxidized, exposed, weathered, oxidized]
///   representing the full oxidation family
/// * `state_converter` - Function to convert the target block and preserve properties
///
/// Returns true if the block was oxidized, false otherwise
pub async fn try_oxidize_copper(
    world: &Arc<World>,
    position: &BlockPos,
    current_block: &Block,
    oxidation_stages: &[&'static Block; 4],
    state_converter: impl FnOnce(&'static Block) -> pumpkin_data::BlockStateId,
) {
    use rand::RngExt;

    // First roll: only ~5.69% chance to even attempt oxidation
    if rand::rng().random::<f32>() >= BASE_OXIDATION_CHANCE {
        return;
    }

    // Find current level and next block
    let (next_block, current_level) = match find_oxidation_level(oxidation_stages, current_block) {
        Some((level, idx)) if idx < 3 => (oxidation_stages[idx + 1], level),
        _ => return, // Already fully oxidized or not in the family
    };

    // Scan neighbors in 4-block Manhattan distance to calculate oxidation chance
    let (same_level_count, higher_level_count) =
        count_neighbor_oxidation_levels(world, position, current_level, oxidation_stages);

    // Calculate weighted probability: ((higher + 1) / (higher + same + 1))^2 * multiplier
    let ratio =
        (higher_level_count + 1) as f32 / (higher_level_count + same_level_count + 1) as f32;
    // Multiplier is 0.75 for UNAFFECTED (level 0), 1.0 for others
    let multiplier = if current_level == 0 { 0.75 } else { 1.0 };
    let final_chance = ratio * ratio * multiplier;

    if rand::rng().random::<f32>() >= final_chance {
        return;
    }

    // Apply oxidation with converted state
    let new_state_id = state_converter(next_block);
    world
        .set_block_state(position, new_state_id, BlockFlags::NOTIFY_LISTENERS)
        .await;
}

/// Get the oxidation level for a block if it's in the given family.
/// Returns Some((level, `index_in_family`)) where level is 0-3 and index is position in stages array
fn find_oxidation_level(
    oxidation_stages: &[&'static Block; 4],
    block: &Block,
) -> Option<(u8, usize)> {
    oxidation_stages
        .iter()
        .enumerate()
        .find(|(_, stage)| **stage == block)
        .map(|(idx, _)| (idx as u8, idx))
}

/// Count copper blocks at same and higher oxidation levels within 4-block Manhattan distance.
/// Returns (same, higher) counts, or (0, 0) if a lower-level neighbor was found (blocking oxidation).
fn count_neighbor_oxidation_levels(
    world: &Arc<World>,
    center: &BlockPos,
    current_level: u8,
    oxidation_stages: &[&'static Block; 4],
) -> (i32, i32) {
    use std::cmp::Ordering;

    let mut same_level_count = 0i32;
    let mut higher_level_count = 0i32;

    // Iterate in a 4-block Manhattan distance
    for dx in -4i32..=4 {
        for dy in -4i32..=4 {
            for dz in -4i32..=4 {
                let manhattan_dist = dx.abs() + dy.abs() + dz.abs();
                if manhattan_dist > 4 || manhattan_dist == 0 {
                    continue;
                }

                let neighbor_pos = BlockPos(pumpkin_util::math::vector3::Vector3::new(
                    center.0.x + dx,
                    center.0.y + dy,
                    center.0.z + dz,
                ));

                let neighbor_block = world.get_block(&neighbor_pos);

                if let Some((neighbor_level, _)) =
                    find_oxidation_level(oxidation_stages, neighbor_block)
                {
                    match neighbor_level.cmp(&current_level) {
                        Ordering::Less => {
                            // Found a neighbor at lower oxidation level - block oxidation entirely
                            return (0, 0);
                        }
                        Ordering::Greater => {
                            higher_level_count += 1;
                        }
                        Ordering::Equal => {
                            same_level_count += 1;
                        }
                    }
                }
            }
        }
    }

    (same_level_count, higher_level_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;

    #[test]
    fn find_oxidation_level_works() {
        let stages = [
            &Block::COPPER_BULB,
            &Block::EXPOSED_COPPER_BULB,
            &Block::WEATHERED_COPPER_BULB,
            &Block::OXIDIZED_COPPER_BULB,
        ];

        // Test each stage
        assert_eq!(
            find_oxidation_level(&stages, &Block::COPPER_BULB),
            Some((0, 0))
        );
        assert_eq!(
            find_oxidation_level(&stages, &Block::EXPOSED_COPPER_BULB),
            Some((1, 1))
        );
        assert_eq!(
            find_oxidation_level(&stages, &Block::WEATHERED_COPPER_BULB),
            Some((2, 2))
        );
        assert_eq!(
            find_oxidation_level(&stages, &Block::OXIDIZED_COPPER_BULB),
            Some((3, 3))
        );

        // Test non-matching block
        assert_eq!(
            find_oxidation_level(&stages, &Block::WAXED_COPPER_BULB),
            None
        );
    }
}
