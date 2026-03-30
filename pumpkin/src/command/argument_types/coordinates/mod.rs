use pumpkin_data::translation;
use crate::command::context::command_source::CommandSource;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::{Axis, Vector3};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{CommandErrorType, READER_EXPECTED_DOUBLE, READER_EXPECTED_INT};
use crate::command::string_reader::StringReader;

pub mod vec3;

pub const MIXED_TYPE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(translation::ARGUMENT_POS_MIXED);

/// Represents a single world coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorldCoordinate {
    Absolute(f64),
    Relative(f64),
}

impl WorldCoordinate {

    /// Creates a new `WorldCoordinate`.
    pub const fn new(is_relative: bool, value: f64) -> Self {
        if is_relative {
            WorldCoordinate::Relative(value)
        } else {
            WorldCoordinate::Absolute(value)
        }
    }

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

    /// Checks if a `StringReader` is about to describle a relative coordinate.
    ///
    /// # Arguments
    /// * `reader` - The `StringReader` to check.
    ///
    /// # Returns
    /// - `true` if a `~` (tilde) can be found. It is also skipped by this method.
    /// - `false` if no `~` can be found.
    pub fn consume_relative_start(reader: &mut StringReader) -> bool {
        if reader.peek() == Some('~') {
            reader.skip();
            true
        } else {
            false
        }
    }

    /// Tries to parse a [`WorldCoordinate`] from a single number, expecting an `f64`.
    ///
    /// # Arguments
    /// * `reader` - The `StringReader` to parse the coordinate from.
    /// * `correct_center` - Whether to correct integral coordinates by adding `+0.5` to them
    ///   (as mentioned by [`Vec3ArgumentType::Default`]).
    ///
    /// # Returns
    /// - The `WorldCoordinate` if it was correctly parsed, wrapped in an `Ok`.
    /// - A [`CommandSyntaxError`] describing an error if it could not be correctly parsed,
    ///   wrapped in an `Err`.
    ///
    /// [`Vec3ArgumentType::Default`]: vec3::Vec3ArgumentType::Default
    pub fn parse_world_f64(&self, reader: &mut StringReader, correct_center: bool) -> Result<WorldCoordinate, CommandSyntaxError> {
        if reader.peek() == Some('^') {
            Err(MIXED_TYPE_ERROR_TYPE.create(reader))
        } else if !reader.can_read_char() {
            Err(READER_EXPECTED_DOUBLE.create(reader))
        } else {
            let is_relative = Self::consume_relative_start(reader);
            let i = reader.cursor();
            let mut value = if reader.can_read_char() && reader.peek() != Some(' ') {
                reader.read_double()?
            } else {
                0.0
            };
            let slice = &reader.string()[i..reader.cursor()];
            if is_relative && slice.is_empty() {
                Ok(Self::Relative(0.0))
            } else {
                if !slice.contains('.') && !is_relative && correct_center {
                    value += 0.5;
                }
                Ok(Self::new(is_relative, value))
            }
        }
    }

    /// Tries to parse a [`WorldCoordinate`] from a single number, expecting an integral position.
    ///
    /// # Arguments
    /// * `reader` - The `StringReader` to parse the coordinate from.
    ///
    /// # Returns
    /// - The `WorldCoordinate` if it was correctly parsed, wrapped in an `Ok`.
    /// - A [`CommandSyntaxError`] describing an error if it could not be correctly parsed,
    ///   wrapped in an `Err`.
    ///
    /// [`Vec3ArgumentType::Default`]: vec3::Vec3ArgumentType::Default
    pub fn parse_world_i32(&self, reader: &mut StringReader) -> Result<WorldCoordinate, CommandSyntaxError> {
        if reader.peek() == Some('^') {
            Err(MIXED_TYPE_ERROR_TYPE.create(reader))
        } else if !reader.can_read_char() {
            Err(READER_EXPECTED_INT.create(reader))
        } else {
            let is_relative = Self::consume_relative_start(reader);
            let value = if reader.can_read_char() && reader.peek() != Some(' ') {
                if is_relative {
                    reader.read_double()?
                } else {
                    reader.read_int()? as f64
                }
            } else {
                0.0
            };
            Ok(Self::new(is_relative, value))
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
