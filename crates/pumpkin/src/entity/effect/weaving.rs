use std::collections::HashSet;

use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;

pub struct WeavingMobEffect;

impl MobEffect for WeavingMobEffect {
    fn on_mob_death(&self, living: &LivingEntity, _amplifier: u8, _damage_type: &DamageType) {
        let world = living.entity.world.load();

        // Check if mob griefing is enabled or if player
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;
        if !living.is_player() && !mob_griefing {
            return;
        }

        let center_pos = living.entity.block_pos.load();
        let cobweb_count = (rand::random::<u32>() % 2 + 2) as usize; // 2 to 3 cobwebs

        let mut positions_to_transform = HashSet::new();

        // Sample up to 15 random positions in a cube of radius 1
        for _ in 0..15 {
            let dx = (rand::random::<u32>() % 3) as i32 - 1;
            let dy = (rand::random::<u32>() % 3) as i32 - 1;
            let dz = (rand::random::<u32>() % 3) as i32 - 1;

            let target_pos = BlockPos(center_pos.0 + Vector3::new(dx, dy, dz));
            let below_pos = BlockPos(target_pos.0 + Vector3::new(0, -1, 0));

            let target_state = world.get_block_state(&target_pos);
            let below_state = world.get_block_state(&below_pos);

            if target_state.is_air()
                && !below_state.is_air()
                && positions_to_transform.insert(target_pos)
                && positions_to_transform.len() >= cobweb_count
            {
                break;
            }
        }

        for target_pos in positions_to_transform {
            world.set_block_state(
                &target_pos,
                Block::COBWEB.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            world.sync_world_event(WorldEvent::AnimationSpawnCobweb, target_pos, 0);
        }
    }
}
