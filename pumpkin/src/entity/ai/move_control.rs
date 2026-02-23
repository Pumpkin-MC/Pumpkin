use pumpkin_util::math::{vector3::Vector3, wrap_degrees};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOperation {
    Wait,
    MoveTo,
}

#[derive(Debug, Clone, Copy)]
pub struct WantedPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub speed: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveControlInput {
    pub pos: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub touching_water: bool,
    pub movement_speed: f64,
    pub flying_speed: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveControlOutput {
    pub movement: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    /// Optional direct velocity override for controls that emulate vanilla bespoke movement.
    pub velocity: Option<Vector3<f64>>,
    /// Optional movement speed override for controls that mirror vanilla speed updates.
    pub movement_speed: Option<f64>,
}

/// Movement control strategy. `Ground` preserves existing navigator behaviour;
/// `Flying` matches vanilla `FlyingMoveControl` (bees, parrots, allays, vexes).
#[derive(Default)]
pub enum MoveControl {
    #[default]
    Ground,
    Flying {
        /// Max pitch turn per tick (e.g. 20 for bees)
        max_turn: i32,
        hovers_in_place: bool,
        wanted: Option<WantedPosition>,
        operation: MoveOperation,
    },
    /// Fish movement controller, mirroring vanilla fish steering behavior:
    /// smooth yaw turn, gentle buoyancy in water, and velocity steering.
    Fish {
        max_turn_y: i32,
        wanted: Option<WantedPosition>,
        operation: MoveOperation,
    },
}

/// Minimum distance squared before considering "arrived"
const MIN_DIST_SQ: f64 = 2.5e-7;

impl MoveControl {
    /// Create a new flying move control (vanilla `FlyingMoveControl`).
    #[must_use]
    pub const fn flying(max_turn: i32, hovers_in_place: bool) -> Self {
        Self::Flying {
            max_turn,
            hovers_in_place,
            wanted: None,
            operation: MoveOperation::Wait,
        }
    }

    /// Create a fish move control.
    #[must_use]
    pub const fn fish(max_turn_y: i32) -> Self {
        Self::Fish {
            max_turn_y,
            wanted: None,
            operation: MoveOperation::Wait,
        }
    }

    pub const fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed: f64) {
        match self {
            Self::Flying {
                wanted, operation, ..
            }
            | Self::Fish {
                wanted, operation, ..
            } => {
                *wanted = Some(WantedPosition { x, y, z, speed });
                *operation = MoveOperation::MoveTo;
            }
            Self::Ground => {}
        }
    }

    #[must_use]
    pub fn has_wanted(&self) -> bool {
        match self {
            Self::Ground => false,
            Self::Flying { operation, .. } | Self::Fish { operation, .. } => {
                *operation == MoveOperation::MoveTo
            }
        }
    }

    /// Returns true if this is a flying move control.
    #[must_use]
    pub const fn is_flying(&self) -> bool {
        matches!(self, Self::Flying { .. })
    }

    pub fn tick(&mut self, input: MoveControlInput) -> Option<MoveControlOutput> {
        match self {
            Self::Ground => None,
            Self::Flying {
                max_turn,
                hovers_in_place,
                wanted,
                operation,
            } => Some(Self::tick_flying(
                *max_turn,
                *hovers_in_place,
                wanted,
                operation,
                input,
            )),
            Self::Fish {
                max_turn_y,
                wanted,
                operation,
            } => Some(Self::tick_fish(*max_turn_y, wanted, operation, input)),
        }
    }

    fn tick_flying(
        max_turn: i32,
        hovers_in_place: bool,
        wanted: &mut Option<WantedPosition>,
        operation: &mut MoveOperation,
        input: MoveControlInput,
    ) -> MoveControlOutput {
        if *operation != MoveOperation::MoveTo {
            return flying_idle_output(hovers_in_place, input.yaw, input.pitch);
        }
        *operation = MoveOperation::Wait;

        let Some(wanted_position) = wanted.take() else {
            return flying_idle_output(hovers_in_place, input.yaw, input.pitch);
        };

        let xd = wanted_position.x - input.pos.x;
        let yd = wanted_position.y - input.pos.y;
        let zd = wanted_position.z - input.pos.z;
        let distance_sq = xd * xd + yd * yd + zd * zd;
        if distance_sq < MIN_DIST_SQ {
            return idle_output(input.yaw, input.pitch);
        }

        let desired_yaw = (zd.atan2(xd) as f32).to_degrees() - 90.0;
        let new_yaw = rotlerp(input.yaw, desired_yaw, 90.0);

        let speed = if input.on_ground {
            wanted_position.speed * input.movement_speed
        } else {
            wanted_position.speed * input.flying_speed
        };
        let horizontal_dist = xd.hypot(zd);
        let new_pitch = if yd.abs() > 1.0e-5 || horizontal_dist > 1.0e-5 {
            let desired_pitch = -(yd.atan2(horizontal_dist) as f32).to_degrees();
            rotlerp(input.pitch, desired_pitch, max_turn as f32)
        } else {
            input.pitch
        };

        let vertical_speed = if yd > 0.0 { speed } else { -speed };
        MoveControlOutput {
            movement: Vector3::new(0.0, vertical_speed, speed),
            yaw: new_yaw,
            pitch: new_pitch,
            velocity: None,
            movement_speed: Some(speed),
        }
    }

    fn tick_fish(
        max_turn_y: i32,
        wanted: &mut Option<WantedPosition>,
        operation: &mut MoveOperation,
        input: MoveControlInput,
    ) -> MoveControlOutput {
        // Vanilla fish gently float upward while their eyes are in water.
        let mut next_velocity = input.velocity;
        if input.touching_water {
            next_velocity.y += 0.005;
        }

        if *operation == MoveOperation::MoveTo {
            *operation = MoveOperation::Wait;

            if let Some(wanted_position) = wanted.take() {
                let xd = wanted_position.x - input.pos.x;
                let yd = wanted_position.y - input.pos.y;
                let zd = wanted_position.z - input.pos.z;
                let distance_sq = xd * xd + yd * yd + zd * zd;

                if distance_sq < MIN_DIST_SQ {
                    return fish_idle_output(input.yaw, input.pitch, next_velocity);
                }

                let desired_yaw = (zd.atan2(xd) as f32).to_degrees() - 90.0;
                let new_yaw = rotlerp(input.yaw, desired_yaw, max_turn_y as f32);

                let target_speed = (wanted_position.speed * input.movement_speed).max(0.0);
                let horizontal_speed = input.velocity.x.hypot(input.velocity.z);
                let speed = lerp(horizontal_speed, target_speed, 0.125);
                let distance = distance_sq.sqrt();

                if distance > 1.0e-7 {
                    next_velocity.x += (xd / distance * speed - input.velocity.x) * 0.125;
                    next_velocity.y += (yd / distance * speed - input.velocity.y) * 0.125;
                    next_velocity.z += (zd / distance * speed - input.velocity.z) * 0.125;
                }

                let horizontal_dist = xd.hypot(zd);
                let new_pitch = if yd.abs() > 1.0e-5 || horizontal_dist > 1.0e-5 {
                    let desired_pitch =
                        (-(yd.atan2(horizontal_dist) as f32).to_degrees()).clamp(-85.0, 85.0);
                    rotlerp(input.pitch, desired_pitch, 5.0)
                } else {
                    input.pitch
                };

                return MoveControlOutput {
                    movement: Vector3::default(),
                    yaw: new_yaw,
                    pitch: new_pitch,
                    velocity: Some(next_velocity),
                    movement_speed: None,
                };
            }
        }

        fish_idle_output(input.yaw, input.pitch, next_velocity)
    }
}

const fn idle_output(yaw: f32, pitch: f32) -> MoveControlOutput {
    MoveControlOutput {
        movement: Vector3::new(0.0, 0.0, 0.0),
        yaw,
        pitch,
        velocity: None,
        movement_speed: None,
    }
}

const fn flying_idle_output(hovers_in_place: bool, yaw: f32, pitch: f32) -> MoveControlOutput {
    if !hovers_in_place {
        // Vanilla toggles noGravity here; Pumpkin applies gravity via `get_gravity()` in physics.
    }
    idle_output(yaw, pitch)
}

fn fish_idle_output(yaw: f32, pitch: f32, velocity: Vector3<f64>) -> MoveControlOutput {
    MoveControlOutput {
        movement: Vector3::default(),
        yaw,
        pitch,
        velocity: Some(velocity),
        movement_speed: None,
    }
}

fn lerp(current: f64, target: f64, factor: f64) -> f64 {
    current + (target - current) * factor
}

fn rotlerp(current: f32, target: f32, max: f32) -> f32 {
    let diff = wrap_degrees(target - current).clamp(-max, max);
    let mut result = current + diff;
    if result < 0.0 {
        result += 360.0;
    } else if result > 360.0 {
        result -= 360.0;
    }
    result
}
