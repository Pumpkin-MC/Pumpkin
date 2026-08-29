use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;

pub struct RaidOmenMobEffect;

impl MobEffect for RaidOmenMobEffect {
    fn should_apply_effect_tick(&self, duration: i32, _amplifier: u8) -> bool {
        duration == 1
    }

    fn apply_effect_tick(&self, living: &LivingEntity, _amplifier: u8) {
        let world = living.entity.world.load();
        if let Some(entity) = world.get_entity_by_id(living.entity.entity_id)
            && let Some(player) = entity.get_player()
            && !player.is_spectator()
        {
            let raid_pos = player
                .get_raid_omen_position()
                .unwrap_or_else(|| living.entity.block_pos.load());
            let mut raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            raids.create_or_extend_raid(raid_pos, &world);
            player.clear_raid_omen_position();
        }
    }
}
