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

fn get_redstone_strength(
    hit_pos: &Vector3<f64>,
    block_pos: &pumpkin_util::math::position::BlockPos,
) -> u8 {
    let dx = (hit_pos.x - (f64::from(block_pos.0.x) + 0.5)).abs();
    let dy = (hit_pos.y - (f64::from(block_pos.0.y) + 0.5)).abs();
    let dz = (hit_pos.z - (f64::from(block_pos.0.z) + 0.5)).abs();

    let distance = if dy >= dx && dy >= dz {
        dx.max(dz)
    } else if dz >= dx && dz >= dy {
        dx.max(dy)
    } else {
        dy.max(dz)
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
        let power = get_redstone_strength(args.hit_pos, args.position);
        if let Some(owner_id) = args.projectile.get_owner_id()
            && let Some(player) = args.world.get_player_by_id(owner_id)
        {
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::TargetHit as i32,
                1,
            );
            if power == 15 {
                let player_pos = player.position();
                let dx = player_pos.x - args.hit_pos.x;
                let dz = player_pos.z - args.hit_pos.z;
                if dx.hypot(dz) >= 30.0 {
                    player.trigger_advancement(
                        crate::entity::player::advancement::trigger::AdvancementTrigger::Bullseye,
                    );
                }
            }
        }
        if !args
            .world
            .is_block_tick_scheduled(args.position, args.block)
        {
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
