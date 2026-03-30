use crate::command::context::command_source::CommandSource;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::{Axis, Vector3};

pub mod vec3;

/// Represents a single world coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorldCoordinate {
    Absolute(f64),
    Relative(f64),
}

impl WorldCoordinate {
    /// Returns whether this coordinate is relative.
    #[must_use]
    pub const fn is_relative(&self) -> bool {
        matches!(self, Self::Relative(_))
    }

    /// Returns the physical coordinate value this [`WorldCoordinate`] represents, given
    /// an absolute coordinate origin.
    #[must_use]
    pub const fn resolve(&self, origin: f64) -> f64 {
        match self {
            Self::Absolute(absolute) => *absolute,
            Self::Relative(relative) => origin + *relative,
        }
    }
}

/// An object represents some command coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coordinates {
    /// Normal coordinates (each coordinate can be *absolute* or *relative*.)
    World(Vector3<WorldCoordinate>),
    /// Local coordinates (can be different depending on the command source.)
    Local { left: f64, up: f64, forward: f64 },
}

impl Coordinates {
    /// Returns whether a coordinate of these [`Coordinates`] is relative.
    #[must_use]
    pub const fn is_relative(&self, axis: Axis) -> bool {
        match self {
            Self::World(vector) => vector.get_axis(axis).is_relative(),
            Self::Local { .. } => false,
        }
    }

    /// Returns the physical position that these [`Coordinates`] represent.
    #[must_use]
    pub fn position(&self, source: &CommandSource) -> Vector3<f64> {
        match self {
            Self::World(vector) => {
                let pos = source.position;
                Vector3::new(
                    vector.x.resolve(pos.x),
                    vector.y.resolve(pos.y),
                    vector.z.resolve(pos.z),
                )
            }
            Self::Local { left, up, forward } => {
                convert_local_coordinates(*left, *up, *forward, source.rotation)
            }
        }
    }
}

/// Converts a set of local coordinates to their physical [`Vector3`] form.
///
/// # Arguments
/// * `left` - The left component of the coordinates.
/// * `up` - The up component of the coordinates.
/// * `forward` - The forward component of the coordinates.
/// * `rotation` - The rotation to use to calculate the physical coordinates.
///   Both coordinates must be in *degrees*.
///
/// # Returns
/// The physical position represented by the local coordinates.
#[must_use]
pub(super) fn convert_local_coordinates(
    left: f64,
    up: f64,
    forward: f64,
    rotation: Vector2<f32>,
) -> Vector3<f64> {
    let y = (rotation.y + 90.0).to_radians() as f64;
    let y_cos = y.cos();
    let y_sin = y.sin();
    let x = (-rotation.x).to_radians() as f64;
    let x_cos = x.cos();
    let x_sin = x.sin();
    let x_up = (-rotation.x + 90.0).to_radians() as f64;
    let x_up_cos = x_up.cos();
    let x_up_sin = x_up.sin();

    let forward_vector = Vector3::new(y_cos * x_cos, x_sin, y_sin * x_cos);
    let up_vector = Vector3::new(y_cos * x_up_cos, x_up_sin, y_sin * x_up_cos);
    let left_vector = forward_vector.cross(&up_vector) * -1.0;

    Vector3::new(
        forward_vector.x * forward + up_vector.x * up + left_vector.x * left,
        forward_vector.y * forward + up_vector.y * up + left_vector.y * left,
        forward_vector.z * forward + up_vector.z * up + left_vector.z * left,
    )
}
