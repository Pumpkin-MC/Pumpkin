use super::MobEntity;
use crate::entity::EntityBase;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector2::Vector2;
use rand::RngExt;
use std::sync::atomic::Ordering::Relaxed;

/// Tick boundaries (both inclusive) when monsters do not burn in sunlight (26.1).
///
/// Sourced from `data/minecraft/timeline/day.json` — `monsters_burn` keyframes:
/// `value=false` at tick 12542 (dusk), `value=true` at tick 23460 (dawn).
///
/// TODO: Replace with `EnvironmentAttributes::MONSTERS_BURN` lookup once the
/// `EnvironmentAttributeSystem` is implemented in `pumpkin-data`.
const NIGHT_START: i64 = 12542;
const NIGHT_END: i64 = 23459;

impl MobEntity {
    pub async fn tick_sun_burn(&self) {
        if !self
            .living_entity
            .entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_BURN_IN_DAYLIGHT)
        {
            return;
        }
        if !self.is_sun_burn_tick().await {
            return;
        }
        self.apply_sun_burn();
    }

    async fn is_sun_burn_tick(&self) -> bool {
        let entity = &self.living_entity.entity;

        let world_arc = entity.world.load();
        let world = world_arc.as_ref();

        // Night boundary from data/minecraft/timeline/day.json — monsters_burn keyframes:
        // value=false at tick 12542 (dusk), value=true at tick 23460 (dawn).
        // TODO: read directly from EnvironmentAttributes::MONSTERS_BURN once implemented.

        let day_time = world.get_time_of_day().await % 24000;
        if (NIGHT_START..=NIGHT_END).contains(&day_time) {
            return false;
        }

        // Vanilla: getLightLevelDependentMagicValue() — sky light at eye pos, scaled 0–1.
        let eye_block_pos = entity.get_eye_pos();
        let brightness = world
            .level
            .light_engine
            .get_sky_light_level(&world.level, &eye_block_pos.to_block_pos())
            as f32
            / 15.0;

        if brightness <= 0.5 {
            return false;
        }

        let is_in_non_burnable = entity.touching_water.load(Relaxed)
            || world.weather.lock().await.raining
            || entity.is_in_powder_snow()
            || entity.was_in_powder_snow.load(Relaxed);

        if is_in_non_burnable {
            return false;
        }

        let pos = entity.pos.load();
        let top_y = world.get_top_block(Vector2::new(pos.x as i32, pos.z as i32));
        if (entity.get_eye_y() as i32) < top_y {
            return false;
        }

        let mut rng = rand::rng();
        rng.random::<f32>() * 30.0 < (brightness - 0.4) * 2.0
    }

    fn apply_sun_burn(&self) {
        let entity = &self.living_entity.entity;
        entity.set_on_fire_for(8.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_window_matches_day_timeline_keyframes() {
        // monsters_burn: value=false at tick 12542 (dusk), value=true at 23460 (dawn).
        let night = NIGHT_START..=NIGHT_END;
        assert!(night.contains(&12542));
        assert!(night.contains(&23459));
        assert!(!night.contains(&12541));
        assert!(!night.contains(&23460));
    }
}
