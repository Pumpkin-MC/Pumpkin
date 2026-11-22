use std::sync::{Arc, atomic::Ordering::Relaxed};

use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{data_component_impl::EquipmentSlot, game_event::GameEvent, sound::Sound};
use pumpkin_util::math::{cos, sin, vector3::Vector3};
use pumpkin_world::clip::{ClipBlock, ClipContext, ClipFluid};

use super::{CreakingEntity, ai};
use crate::{
    entity::{EntityBase, mob::Mob, player::Player},
    plugin::api::events::entity::entity_target::TargetReason,
};

pub(super) const MAX_GAZE_RANGE: f64 = 128.0;
const ACTIVATION_DISTANCE_SQUARED: f64 = 144.0;
const MAX_PLAYER_STUCK_COUNTER: i32 = 4;
const NORMALIZATION_EPSILON: f64 = 1.0e-5f32 as f64;

impl CreakingEntity {
    fn is_looking_at_me(&self, player: &Player) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        let player_entity = player.get_entity();
        if !Arc::ptr_eq(&world, &player_entity.world.load()) {
            return false;
        }

        let position = entity.pos.load();
        let viewer_position = player_entity.pos.load();
        let eyes = viewer_position.add_raw(0.0, player_entity.get_eye_height(), 0.0);
        let descending = player_entity.is_sneaking();
        let pitch = player_entity.pitch.load().to_radians();
        let yaw = -player_entity.head_yaw.load().to_radians();
        let pitch_cos = cos(pitch);
        let view = Vector3::new(
            f64::from(sin(yaw) * pitch_cos),
            f64::from(-sin(pitch)),
            f64::from(cos(yaw) * pitch_cos),
        )
        .normalize();
        let eye_y = entity.get_eye_y();
        let feet_y = position.y + 0.5 * f64::from(entity.scale.load());
        let mid_y = position.y.midpoint(eye_y);
        [eye_y, feet_y, mid_y].into_iter().any(|y| {
            let point = Vector3::new(position.x, y, position.z);
            let offset = point - eyes;
            let distance_squared = offset.length_squared();
            distance_squared >= NORMALIZATION_EPSILON.powi(2)
                && distance_squared <= MAX_GAZE_RANGE.powi(2)
                && view.dot(&offset.normalize()) > 0.5
                && world
                    .clip(
                        &ClipContext::new(eyes, point, ClipBlock::Visual, ClipFluid::None)
                            .with_visual_viewer(viewer_position.y, descending),
                    )
                    .is_miss()
        })
    }

    fn is_player_disguised(player: &Player) -> bool {
        let equipment = player
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment
            .equipment
            .get(&EquipmentSlot::HEAD)
            .is_some_and(|head| {
                !head.is_empty()
                    && head
                        .item
                        .has_tag(&tag::Item::MINECRAFT_GAZE_DISGUISE_EQUIPMENT)
            })
    }

    fn check_can_move(&self) -> bool {
        let active = self.is_active();
        let position = self.get_entity().pos.load();
        let mut has_potential_target = false;
        for player in ai::nearest_players(self) {
            if !self.can_attack(player.as_ref()) || self.considers_entity_as_ally(player.as_ref()) {
                continue;
            }
            has_potential_target = true;
            if active && Self::is_player_disguised(&player) || !self.is_looking_at_me(&player) {
                continue;
            }
            if active {
                return false;
            }
            if position.squared_distance_to_vec(&player.get_entity().pos.load())
                < ACTIVATION_DISTANCE_SQUARED
                && self.set_mob_target(Some(player), TargetReason::ClosestPlayer)
                && self.mob_entity.stored_target().is_some()
            {
                self.emit_entity_action();
                self.make_sound(Sound::EntityCreakingActivate);
                self.set_is_active(true);
                return false;
            }
        }

        if !has_potential_target
            && active
            && self.set_mob_target(None, TargetReason::ForgotTarget)
            && self.mob_entity.stored_target().is_none()
        {
            self.emit_entity_action();
            self.make_sound(Sound::EntityCreakingDeactivate);
            self.set_is_active(false);
        }
        true
    }

    pub(super) fn update_gaze(&self) {
        let now_can_move = self.check_can_move();
        if now_can_move == self.can_move() {
            return;
        }
        self.emit_entity_action();
        if now_can_move {
            self.make_sound(Sound::EntityCreakingUnfreeze);
        } else {
            self.mob_entity.stop_in_place();
            self.make_sound(Sound::EntityCreakingFreeze);
        }
        self.set_can_move(now_can_move);
    }

    pub fn player_is_stuck_in_you(&self) -> bool {
        let bounds = self.get_entity().bounding_box.load();
        if ai::nearest_players(self)
            .into_iter()
            .any(|player| bounds.contains(player.eye_position()))
        {
            self.player_stuck_counter.fetch_add(1, Relaxed) >= MAX_PLAYER_STUCK_COUNTER
        } else {
            self.player_stuck_counter.store(0, Relaxed);
            false
        }
    }

    pub(super) fn emit_entity_action(&self) {
        self.get_entity()
            .emit_own_game_event(self, GameEvent::EntityAction);
    }
}
