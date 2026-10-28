use pumpkin_data::Block;
use pumpkin_data::block_properties::TargetProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, EmitsRedstonePowerArgs, GetRedstonePowerArgs, OnProjectileHitArgs,
    OnScheduledTickArgs, PlacedArgs,
};

fn get_redstone_strength(hit_pos: &Vector3<f64>) -> u8 {
    let dist_x = ((hit_pos.x - hit_pos.x.floor()) - 0.5).abs();
    let dist_y = ((hit_pos.y - hit_pos.y.floor()) - 0.5).abs();
    let dist_z = ((hit_pos.z - hit_pos.z.floor()) - 0.5).abs();

    // The hit face is on the block boundary where distance from center is greatest (~0.5).
    // The signal strength is determined by the distance from center on the other two axes.
    let distance = if dist_x >= dist_y && dist_x >= dist_z {
        dist_y.max(dist_z)
    } else if dist_y >= dist_z {
        dist_x.max(dist_z)
    } else {
        dist_x.max(dist_y)
    };

    let norm = ((0.5 - distance) / 0.5).clamp(0.0, 1.0);
    let power = (15.0 * norm).ceil() as u8;
    power.max(1)
}

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

impl BlockBehaviour for TargetBlock {
    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = TargetProperties::from_state_id(args.state.id);
        props.r#power
    }

    fn on_projectile_hit(&self, args: OnProjectileHitArgs<'_>) {
        if let Some(owner_id) = args.projectile.get_owner_id()
            && let Some(player) = args.world.get_player_by_id(owner_id)
        {
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::TargetHit as i32,
                1,
            );
        }
        if !args
            .world
            .is_block_tick_scheduled(args.position, args.block)
        {
            let power = get_redstone_strength(args.hit_pos);
            let mut props = TargetProperties::from_state_id(args.state.id);
            props.r#power = power;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            let entity_type = args.projectile.get_entity().entity_type;
            let delay = if entity_type == &EntityType::ARROW
                || entity_type == &EntityType::SPECTRAL_ARROW
                || entity_type == &EntityType::TRIDENT
            {
                20
            } else {
                8
            };
            args.world
                .schedule_block_tick(args.block, *args.position, delay, TickPriority::Normal);
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = TargetProperties::from_state_id(state_id);
        if props.r#power != 0 {
            props.r#power = 0;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if Block::from_state_id(args.old_state_id) != args.block {
            let mut props = TargetProperties::from_state_id(args.state_id);
            if props.r#power != 0
                && !args
                    .world
                    .is_block_tick_scheduled(args.position, args.block)
            {
                props.r#power = 0;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_redstone_strength_matches_vanilla_formula() {
        // Center hit gives power 15
        assert_eq!(get_redstone_strength(&Vector3::new(0.5, 0.5, 0.0)), 15);
        // Distance 0.1 from center gives power 12
        assert_eq!(get_redstone_strength(&Vector3::new(0.4, 0.5, 0.0)), 12);
        // Distance 0.25 from center gives power 8
        assert_eq!(get_redstone_strength(&Vector3::new(0.25, 0.5, 0.0)), 8);
        // Distance 0.45 from center gives power 2
        assert_eq!(get_redstone_strength(&Vector3::new(0.05, 0.5, 0.0)), 2);
        // Edge hit gives minimum power 1
        assert_eq!(get_redstone_strength(&Vector3::new(0.0, 0.5, 0.0)), 1);
    }
}
