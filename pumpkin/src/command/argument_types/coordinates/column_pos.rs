use std::pin::Pin;

use pumpkin_data::translation;
use pumpkin_util::math::position::{BlockPos, ColumnPos};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use crate::command::suggestion::suggestions::TextCoordinates;
use crate::command::{
    argument_types::{
        argument_type::{ArgumentType, JavaClientArgumentType},
        coordinates::{Coordinates, WorldCoordinate},
    },
    context::command_context::CommandContext,
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

pub const INCOMPLETE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_POS2D_INCOMPLETE,
    translation::java::ARGUMENT_POS2D_INCOMPLETE,
);

pub struct ColumnPosArgumentType;

impl ArgumentType for ColumnPosArgumentType {
    type Item = Coordinates;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if !reader.can_read_char() {
            return Err(INCOMPLETE_ERROR_TYPE.create(reader));
        }

        let start = reader.cursor();
        let x = WorldCoordinate::parse_integer(reader)?;
        if reader.peek() == Some(' ') {
            reader.skip();
            let z = WorldCoordinate::parse_integer(reader)?;
            Ok(Coordinates::World(Vector3 {
                x,
                y: WorldCoordinate::Relative(0.0),
                z,
            }))
        } else {
            reader.set_cursor(start);
            Err(INCOMPLETE_ERROR_TYPE.create(reader))
        }
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move {
            let remainder = builder.remaining();

            let suggested_coordinates = if remainder.bytes().next() == Some(b'^') {
                TextCoordinates::Local
            } else {
                TextCoordinates::Global
            };

            builder.suggest_2d_coordinates(suggested_coordinates, |value| {
                self.parse(&mut StringReader::new(value)).is_ok()
            })
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::ColumnPos
    }

    fn examples(&self) -> Vec<String> {
        examples!("0 0", "~ ~", "~-1 ~2")
    }
}

impl ColumnPosArgumentType {
    /// Returns a [`CommandContext`]'s parsed coordinate argument in the form of a [`ColumnPos`].
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed `Coordinates` with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The `ColumnPos` containing the position represented by the parsed argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved.
    pub fn get_column_pos(
        context: &CommandContext,
        name: &str,
    ) -> Result<ColumnPos, CommandSyntaxError> {
        let block_pos = BlockPos::floored_v(
            context
                .get_argument::<Coordinates>(name)?
                .resolve(context.source.as_ref()),
        );
        Ok(ColumnPos(Vector2::new(block_pos.0.x, block_pos.0.z)))
    }
}
