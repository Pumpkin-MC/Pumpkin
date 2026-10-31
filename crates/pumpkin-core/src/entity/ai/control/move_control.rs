use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::mob::Mob;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3, wrap_degrees};
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
            let speed = self.speed_modifier as f32
                * living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED) as f32;
            let scale = speed / self.strafe_forwards.hypot(self.strafe_right).max(1.0);
            let forwards = self.strafe_forwards * scale;
            let right = self.strafe_right * scale;
            let (sin, cos) = entity.yaw.load().to_radians().sin_cos();
            let pos = entity.pos.load();
            let next = BlockPos::floored_v(Vector3::new(
                pos.x + f64::from(forwards * cos - right * sin),
                f64::from(entity.block_pos.load().0.y),
                pos.z + f64::from(right * cos + forwards * sin),
            ));
            if mob_entity
                .steered()
                .navigator()
                .get_path_type(living_entity, next)
                != PathType::Walkable
            {
                self.strafe_forwards = 1.0;
                self.strafe_right = 0.0;
            }
            mob_entity.set_speed(speed);
            let input = living_entity.movement_input.load();
            living_entity.movement_input.store(Vector3::new(
                f64::from(self.strafe_right),
                input.y,
                f64::from(self.strafe_forwards),
            ));
            self.operation = Operation::Wait;
        } else if self.operation == Operation::MoveTo {
            self.operation = Operation::Wait;
            let pos = entity.pos.load();
            let xd = self.wanted_x - pos.x;
            let zd = self.wanted_z - pos.z;
            let yd = self.wanted_y - pos.y;
            let dd = xd * xd + yd * yd + zd * zd;

            if dd < f64::from(Self::MIN_SPEED_SQR) {
                let input = living_entity.movement_input.load();
                living_entity
                    .movement_input
                    .store(Vector3::new(input.x, input.y, 0.0));
                return;
            }

            let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
            entity
                .yaw
                .store(Self::rotlerp(entity.yaw.load(), y_rot_d, Self::MAX_TURN));

            let movement_speed = living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
            let speed = self.speed_modifier * movement_speed;
            mob_entity.set_speed(speed as f32);

            let step_height = living_entity.get_attribute_value(&Attributes::STEP_HEIGHT);
            let block_pos = entity.block_pos.load();
            let world = entity.world.load();
            let (block, state) = world.get_block_and_state(&block_pos);
            let inside_collision = state
                .get_block_collision_shapes_at(&block_pos)
                .map(|shape| shape.max.y)
                .reduce(f64::max)
                .is_some_and(|height| pos.y < f64::from(block_pos.0.y) + height)
                && !block.has_tag(&tag::Block::MINECRAFT_DOORS)
                && !block.has_tag(&tag::Block::MINECRAFT_FENCES);
            if yd > step_height
                && xd * xd + zd * zd < 1.0f64.max(entity.entity_dimension.load().width as f64)
                || inside_collision
            {
                mob_entity
                    .jump_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .jump();
                self.operation = Operation::Jumping;
            }
        } else if self.operation == Operation::Jumping {
            let movement_speed = living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
            let speed = self.speed_modifier * movement_speed;
            mob_entity.set_speed(speed as f32);

            if entity.on_ground.load(Ordering::Relaxed)
                || entity.touching_water.load(Ordering::Relaxed)
                || entity.touching_lava.load(Ordering::Relaxed)
            {
                self.operation = Operation::Wait;
            }
        } else {
            let input = living_entity.movement_input.load();
            living_entity
                .movement_input
                .store(Vector3::new(input.x, input.y, 0.0));
        }
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        if self.operation != Operation::Jumping {
            self.operation = Operation::MoveTo;
        }
    }

    fn strafe(&mut self, forwards: f32, right: f32) {
        self.operation = Operation::Strafe;
        self.strafe_forwards = forwards;
        self.strafe_right = right;
        self.speed_modifier = 0.25;
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }
}

impl MoveControl {
    pub const MIN_SPEED_SQR: f32 = 2.5000003E-7;
    pub const MAX_TURN: f32 = 90.0;

    #[must_use]
    pub fn rotlerp(start: f32, end: f32, max_change: f32) -> f32 {
        let result = start + wrap_degrees(end - start).clamp(-max_change, max_change);
        if result < 0.0 {
            result + 360.0
        } else if result > 360.0 {
            result - 360.0
        } else {
            result
        }
    }

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

    pub const fn strafe(&mut self, forwards: f32, right: f32) {
        self.operation = Operation::Strafe;
        self.strafe_forwards = forwards;
        self.strafe_right = right;
        self.speed_modifier = 0.25;
    }
}
