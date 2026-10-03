use pumpkin_data::block_properties::TargetProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, EmitsRedstonePowerArgs, GetRedstonePowerArgs, OnPlaceArgs, OnProjectileHitArgs,
    OnScheduledTickArgs, OnStateReplacedArgs,
};

fn get_redstone_strength(hit_pos: &Vector3<f64>) -> u8 {
    let dist_x = ((hit_pos.x - hit_pos.x.floor()) - 0.5).abs();
    let dist_y = ((hit_pos.y - hit_pos.y.floor()) - 0.5).abs();
    let dist_z = ((hit_pos.z - hit_pos.z.floor()) - 0.5).abs();

    let distance = if dist_x >= dist_y && dist_x >= dist_z {
        dist_y.max(dist_z)
    } else if dist_y >= dist_z {
        dist_x.max(dist_z)
    } else {
        dist_x.max(dist_y)
    };

    let norm = ((0.5 - distance) / 0.5).clamp(0.0, 1.0);
    let power = (15.0 * norm).ceil() as u8;
    power.clamp(1, 15)
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

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
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

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            let props = TargetProperties::from_state_id(args.old_state_id);
            if props.r#power > 0 {
                args.world.update_neighbors(args.position, None);
            }
        }
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> pumpkin_data::BlockStateId {
        let mut props = TargetProperties::default(args.block);
        props.r#power = 0;
        props.to_state_id(args.block)
    }
}
