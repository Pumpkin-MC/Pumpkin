//! Sculk growth rules: sensor / shrieker placement above sculk.

use pumpkin_data::Block;
use pumpkin_data::BlockId;
use pumpkin_data::BlockState;
use pumpkin_data::block_properties::SculkSensorLikeProperties;
use pumpkin_data::block_properties::SculkShriekerLikeProperties;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::RandomImpl;

use super::SculkLevel;

/// Radius of the nearby-growth scan: -4..4 on XZ, 0..2 on Y.
const GROWTH_CHECK_RADIUS: i32 = 4;
const GROWTH_CHECK_HEIGHT: i32 = 2;
/// Maximum nearby growths allowed before placement is blocked.
const MAX_NEARBY_GROWTHS: i32 = 2;

/// Rules governing sculk sensor / shrieker placement.
pub struct GrowthRules;

impl GrowthRules {
    /// Checks whether a growth can be placed above `pos`: the block above
    /// must be air or water and nearby growths must be few.
    pub fn can_place_growth(level: &dyn SculkLevel, pos: BlockPos) -> bool {
        let above = pos.up();
        let Some(above_state) = level.sculk_get(above) else {
            return false;
        };
        let above_id = above_state.to_block_id();
        // The block above must be air, or a water block holding water.
        if !(above_state.to_state().is_air()
            || (above_id == BlockId::WATER && level.sculk_is_water(above)))
        {
            return false;
        }
        // Count nearby growths (sensor + shrieker).
        let mut growth_count = 0i32;
        for dx in -GROWTH_CHECK_RADIUS..=GROWTH_CHECK_RADIUS {
            for dz in -GROWTH_CHECK_RADIUS..=GROWTH_CHECK_RADIUS {
                for dy in 0..=GROWTH_CHECK_HEIGHT {
                    let check_pos = pos.offset(Vector3::new(dx, dy, dz));
                    if let Some(s) = level.sculk_get(check_pos) {
                        let id = s.to_block_id();
                        // Only sculk sensors and shriekers count
                        // (exact block match — calibrated sensors don't).
                        if id == BlockId::SCULK_SENSOR || id == BlockId::SCULK_SHRIEKER {
                            growth_count += 1;
                            if growth_count > MAX_NEARBY_GROWTHS {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    /// Returns a sculk sensor (10/11 chance) or a sculk shrieker
    /// (1/11 chance), waterlogged when placed in water.
    pub fn random_growth_state(
        level: &dyn SculkLevel,
        pos: BlockPos,
        random: &mut RandomGenerator,
        is_world_gen: bool,
    ) -> &'static BlockState {
        let base_state = if random.next_bounded_i32(11) == 0 {
            // Shrieker (1/11 chance).
            let mut props = SculkShriekerLikeProperties::default(&Block::SCULK_SHRIEKER);
            props.r#can_summon = is_world_gen;
            BlockState::from_id(props.to_state_id(&Block::SCULK_SHRIEKER))
        } else {
            Block::SCULK_SENSOR.default_state
        };
        // Apply waterlogging if the position holds water.
        if level.sculk_is_water(pos) {
            let block_id = base_state.id.to_block_id();
            if block_id == BlockId::SCULK_SHRIEKER {
                let mut props = SculkShriekerLikeProperties::from_state_id(base_state.id);
                props.r#waterlogged = true;
                return BlockState::from_id(props.to_state_id(&Block::SCULK_SHRIEKER));
            } else if block_id == BlockId::SCULK_SENSOR {
                let mut props = SculkSensorLikeProperties::from_state_id(base_state.id);
                props.r#waterlogged = true;
                return BlockState::from_id(props.to_state_id(&Block::SCULK_SENSOR));
            }
        }
        base_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::feature::features::sculk::test_utils::MockSculkLevel;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn shrieker_state_default() {
        let props = SculkShriekerLikeProperties::default(&Block::SCULK_SHRIEKER);
        assert!(!props.r#can_summon);
    }

    #[test]
    fn can_place_growth_accepts_air_like_states() {
        // Cavity and void air count as air here, not just the plain
        // `minecraft:air` block.
        let mut level = MockSculkLevel::new();
        let pos = BlockPos::new(0, 60, 0);
        level.set_id(pos.up(), Block::CAVE_AIR.default_state.id);
        assert!(GrowthRules::can_place_growth(&level, pos));
    }

    #[test]
    fn can_place_growth_accepts_water_above() {
        let mut level = MockSculkLevel::new();
        let pos = BlockPos::new(0, 60, 0);
        level.set_id(pos.up(), Block::WATER.default_state.id);
        assert!(GrowthRules::can_place_growth(&level, pos));
    }

    #[test]
    fn density_ignores_calibrated_sensors() {
        // Only SCULK_SENSOR / SCULK_SHRIEKER count; three nearby
        // calibrated sensors must not block placement.
        let mut level = MockSculkLevel::new();
        let pos = BlockPos::new(0, 60, 0);
        level.set_id(pos.up(), Block::AIR.default_state.id);
        for dx in 0..3 {
            level.set_id(
                pos.offset(Vector3::new(dx, 0, 0)),
                Block::CALIBRATED_SCULK_SENSOR.default_state.id,
            );
        }
        assert!(GrowthRules::can_place_growth(&level, pos));
    }

    #[test]
    fn density_blocks_after_two_sensors_or_shriekers() {
        let mut level = MockSculkLevel::new();
        let pos = BlockPos::new(0, 60, 0);
        level.set_id(pos.up(), Block::AIR.default_state.id);
        for dx in 0..2 {
            level.set_id(
                pos.offset(Vector3::new(dx, 0, 0)),
                Block::SCULK_SENSOR.default_state.id,
            );
        }
        assert!(GrowthRules::can_place_growth(&level, pos));
        level.set_id(
            pos.offset(Vector3::new(2, 0, 0)),
            Block::SCULK_SHRIEKER.default_state.id,
        );
        assert!(!GrowthRules::can_place_growth(&level, pos));
    }
}
