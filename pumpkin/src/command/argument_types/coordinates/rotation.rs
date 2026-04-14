use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::coordinates::{Coordinates, WorldCoordinate};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::translation;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

pub const NOT_COMPLETE_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ROTATION_INCOMPLETE);

pub struct RotationArgumentType;

impl ArgumentType for RotationArgumentType {
    type Item = Coordinates;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let i = reader.cursor();
        if reader.can_read_char() {
            let y = Coordinates::parse_world_single(i, reader, false)?;
            if reader.peek() == Some('_') {
                reader.skip();
                let x = Coordinates::parse_world_single(i, reader, false)?;
                Ok(Coordinates::World(Vector3::new(
                    x,
                    y,
                    WorldCoordinate::Relative(0.0),
                )))
            } else {
                Err(Self::syntax_error(reader))
            }
        } else {
            Err(Self::syntax_error(reader))
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Rotation
    }

    fn examples(&self) -> Vec<String> {
        examples!("1 1", "~5 4", "~-6 ~-6")
    }
}

impl RotationArgumentType {
    fn syntax_error(reader: &StringReader) -> CommandSyntaxError {
        NOT_COMPLETE_ERROR_TYPE.create(reader)
    }

    /// Returns the [`Coordinates`] representing the rotation of a parsed rotation argument.
    ///
    /// Use [`Coordinates::rotation`] to resolve the coordinates to a rotation [`Vector2`].
    ///
    /// If the rotation is successfully provided in an `Ok`:
    /// - The returned *x*-coordinate of the coordinates is the absolute/relative **pitch**.
    /// - The returned *y*-coordinate of the coordinates is the absolute/relative **yaw**.
    /// - The returned *z*-coordinate of the coordinates is always 0.
    pub fn get_rotation(
        context: &CommandContext,
        name: &str,
    ) -> Result<Coordinates, CommandSyntaxError> {
        context
            .get_argument::<Coordinates>(name).copied()
    }
}
