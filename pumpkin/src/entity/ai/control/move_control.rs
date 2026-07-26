use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use pumpkin_data::{
    Block,
    attributes::Attributes,
    tag::{self, Taggable},
};
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    #[default]
    Wait,
    MoveTo,
    Strafe,
    Jumping,
}

pub struct MoveControl {
    pub wanted_x: f64,
    pub wanted_y: f64,
    pub wanted_z: f64,
    pub speed_modifier: f64,
    pub strafe_forwards: f32,
    pub strafe_right: f32,
    pub operation: Operation,
}

impl Default for MoveControl {
    fn default() -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            speed_modifier: 0.0,
            strafe_forwards: 0.0,
            strafe_right: 0.0,
            operation: Operation::Wait,
        }
    }
}

impl Control for MoveControl {}

impl MoveControlTrait for MoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let living_entity = &mob_entity.living_entity;
        let entity = &living_entity.entity;
        if self.operation == Operation::Strafe {
            // TODO: is_walkable check
            let attr = living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
            living_entity.set_speed(self.speed_modifier * attr);
            // Strafe overrides axes after set_speed
            living_entity.movement_input.store(Vector3::new(
                self.strafe_right as f64,
                0.0,
                self.strafe_forwards as f64,
            ));
            self.operation = Operation::Wait;
        } else if self.operation == Operation::MoveTo {
            self.operation = Operation::Wait;
            let pos = entity.pos.load();
            let xd = self.wanted_x - pos.x;
            let zd = self.wanted_z - pos.z;
            let yd = self.wanted_y - pos.y;
            let dd = xd * xd + yd * yd + zd * zd;

            if dd < 2.5000003E-7 {
                living_entity.clear_speed();
                return;
            }

            let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
            let yaw = self.change_angle(entity.yaw.load(), y_rot_d, 90.0);
            entity.yaw.store(yaw);
            // Pumpkin has no separate BodyRotationControl yet; keep the body aligned
            // with navigation while LookControl retains independent head tracking.
            entity.body_yaw.store(yaw);

            // Vanilla: setSpeed(speedModifier * MOVEMENT_SPEED)
            let attr = living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
            living_entity.set_speed(self.speed_modifier * attr);

            // `MoveControl` is also the fallback jump path when a direct navigation
            // attempt reaches a solid obstacle before A* can produce a step-up node.
            // Vanilla uses the target height and the current block's collision shape.
            let step_height = living_entity.get_attribute_value(&Attributes::STEP_HEIGHT);
            // MoveControl compares the squared horizontal distance directly with
            // `max(1.0, bbWidth)`, rather than squaring that limit again.
            let max_jump_distance_sq = 1.0f64.max(entity.entity_dimension.load().width as f64);
            let block_pos = entity.block_pos.load();
            let world = entity.world.load();
            let state = world.get_block_state(&block_pos);
            let block = Block::from_state_id(state.id);
            let collision_requires_jump = !state.collision_shapes.is_empty()
                && !block.has_tag(&tag::Block::MINECRAFT_DOORS)
                && !block.has_tag(&tag::Block::MINECRAFT_FENCES)
                && state
                    .get_block_collision_shapes()
                    .any(|shape| pos.y < shape.max.y + f64::from(block_pos.0.y));
            let target_requires_jump = yd > step_height && xd * xd + zd * zd < max_jump_distance_sq;

            if target_requires_jump || collision_requires_jump {
                living_entity.jumping.store(true, Ordering::SeqCst);
                self.operation = Operation::Jumping;
            } else {
                living_entity.jumping.store(false, Ordering::SeqCst);
            }
        } else if self.operation == Operation::Jumping {
            let attr = living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
            living_entity.set_speed(self.speed_modifier * attr);

            // Vanilla JumpControl consumes the jump request in the same AI tick.
            // Keep the move controller in JUMPING for its speed handling, but do
            // not leave LivingEntity.jumping set for the whole arc.
            living_entity.jumping.store(false, Ordering::SeqCst);

            if entity.on_ground.load(Ordering::Relaxed) {
                self.operation = Operation::Wait;
            }
        } else {
            // Vanilla clears forward input while idle. Active navigation dispatches a
            // fresh wanted position before this controller runs every tick.
            living_entity.clear_speed();
        }
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        Self::set_wanted_position(self, x, y, z, speed_modifier);
    }

    fn stop(&mut self) {
        Self::stop(self);
    }
}

impl MoveControl {
    #[must_use]
    pub fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    #[must_use]
    pub const fn get_speed_modifier(&self) -> f64 {
        self.speed_modifier
    }

    pub fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        if self.operation != Operation::Jumping {
            self.operation = Operation::MoveTo;
        }
    }

    pub fn stop(&mut self) {
        self.operation = Operation::Wait;
        self.strafe_forwards = 0.0;
        self.strafe_right = 0.0;
        self.speed_modifier = 0.0;
    }

    pub const fn strafe(&mut self, forwards: f32, right: f32) {
        self.operation = Operation::Strafe;
        self.strafe_forwards = forwards;
        self.strafe_right = right;
        self.speed_modifier = 0.25;
    }
}
