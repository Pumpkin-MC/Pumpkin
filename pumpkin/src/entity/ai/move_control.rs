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
pub struct MoveControlOutput {
    pub movement: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    /// Optional direct velocity override for controls that emulate vanilla bespoke movement.
    pub velocity: Option<Vector3<f64>>,
}

/// Movement control strategy. `Ground` preserves existing navigator behaviour;
/// `Flying` matches vanilla `FlyingMoveControl` (bees, parrots, allays, vexes).
pub enum MoveControl {
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

impl Default for MoveControl {
    fn default() -> Self {
        Self::Ground
    }
}

/// Minimum distance squared before considering "arrived"
const MIN_DIST_SQ: f64 = 2.5e-7;

impl MoveControl {
    /// Create a new flying move control (vanilla `FlyingMoveControl`).
    pub fn flying(max_turn: i32, hovers_in_place: bool) -> Self {
        Self::Flying {
            max_turn,
            hovers_in_place,
            wanted: None,
            operation: MoveOperation::Wait,
        }
    }

    /// Create a fish move control.
    pub fn fish(max_turn_y: i32) -> Self {
        Self::Fish {
            max_turn_y,
            wanted: None,
            operation: MoveOperation::Wait,
        }
    }

    pub fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed: f64) {
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

    pub fn has_wanted(&self) -> bool {
        match self {
            Self::Ground => false,
            Self::Flying { operation, .. } | Self::Fish { operation, .. } => {
                *operation == MoveOperation::MoveTo
            }
        }
    }

    /// Returns true if this is a flying move control.
    pub fn is_flying(&self) -> bool {
        matches!(self, Self::Flying { .. })
    }

    pub fn tick(
        &mut self,
        pos: Vector3<f64>,
        velocity: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
        touching_water: bool,
        movement_speed: f64,
    ) -> Option<MoveControlOutput> {
        match self {
            Self::Ground => None,
            Self::Flying {
                max_turn,
                hovers_in_place,
                wanted,
                operation,
            } => {
                if *operation == MoveOperation::MoveTo {
                    *operation = MoveOperation::Wait;
                    if let Some(w) = wanted.take() {
                        let xd = w.x - pos.x;
                        let yd = w.y - pos.y;
                        let zd = w.z - pos.z;
                        let dd = xd * xd + yd * yd + zd * zd;

                        if dd < MIN_DIST_SQ {
                            // Already at target
                            return Some(MoveControlOutput {
                                movement: Vector3::new(0.0, 0.0, 0.0),
                                yaw,
                                pitch,
                                velocity: None,
                            });
                        }

                        let desired_yaw = (zd.atan2(xd) as f32).to_degrees() - 90.0;
                        let new_yaw = rotlerp(yaw, desired_yaw, 90.0);

                        // Approximate vanilla FLYING_SPEED (0.6) as movement_speed * 2.0
                        let speed = if on_ground {
                            w.speed * movement_speed
                        } else {
                            w.speed * movement_speed * 2.0
                        };

                        let horizontal_dist = (xd * xd + zd * zd).sqrt();
                        let new_pitch = if yd.abs() > 1.0e-5 || horizontal_dist > 1.0e-5 {
                            let desired_pitch = -(yd.atan2(horizontal_dist) as f32).to_degrees();
                            rotlerp(pitch, desired_pitch, *max_turn as f32)
                        } else {
                            pitch
                        };

                        let vertical_speed = if yd > 0.0 { speed } else { -speed };
                        let movement = Vector3::new(0.0, vertical_speed, speed);
                        return Some(MoveControlOutput {
                            movement,
                            yaw: new_yaw,
                            pitch: new_pitch,
                            velocity: None,
                        });
                    }

                    Some(MoveControlOutput {
                        movement: Vector3::new(0.0, 0.0, 0.0),
                        yaw,
                        pitch,
                        velocity: None,
                    })
                } else {
                    let _ = hovers_in_place;
                    Some(MoveControlOutput {
                        movement: Vector3::new(0.0, 0.0, 0.0),
                        yaw,
                        pitch,
                        velocity: None,
                    })
                }
            }
            Self::Fish {
                max_turn_y,
                wanted,
                operation,
            } => {
                // Vanilla fish gently float upward while their eyes are in water.
                let mut next_velocity = velocity;
                if touching_water {
                    next_velocity.y += 0.005;
                }

                if *operation == MoveOperation::MoveTo {
                    *operation = MoveOperation::Wait;

                    if let Some(w) = wanted.take() {
                        let xd = w.x - pos.x;
                        let yd = w.y - pos.y;
                        let zd = w.z - pos.z;
                        let dd = xd * xd + yd * yd + zd * zd;

                        if dd < MIN_DIST_SQ {
                            return Some(MoveControlOutput {
                                movement: Vector3::default(),
                                yaw,
                                pitch,
                                velocity: Some(next_velocity),
                            });
                        }

                        let desired_yaw = (zd.atan2(xd) as f32).to_degrees() - 90.0;
                        let new_yaw = rotlerp(yaw, desired_yaw, *max_turn_y as f32);

                        let target_speed = (w.speed * movement_speed).max(0.0);
                        let horizontal_speed =
                            (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
                        let speed = lerp(horizontal_speed, target_speed, 0.125);
                        let distance = dd.sqrt();

                        if distance > 1.0e-7 {
                            next_velocity.x += (xd / distance * speed - velocity.x) * 0.125;
                            next_velocity.y += (yd / distance * speed - velocity.y) * 0.125;
                            next_velocity.z += (zd / distance * speed - velocity.z) * 0.125;
                        }

                        let horizontal_dist = (xd * xd + zd * zd).sqrt();
                        let new_pitch = if yd.abs() > 1.0e-5 || horizontal_dist > 1.0e-5 {
                            let desired_pitch = (-(yd.atan2(horizontal_dist) as f32).to_degrees())
                                .clamp(-85.0, 85.0);
                            rotlerp(pitch, desired_pitch, 5.0)
                        } else {
                            pitch
                        };

                        return Some(MoveControlOutput {
                            movement: Vector3::default(),
                            yaw: new_yaw,
                            pitch: new_pitch,
                            velocity: Some(next_velocity),
                        });
                    }
                }

                Some(MoveControlOutput {
                    movement: Vector3::default(),
                    yaw,
                    pitch,
                    velocity: Some(next_velocity),
                })
            }
        }
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
