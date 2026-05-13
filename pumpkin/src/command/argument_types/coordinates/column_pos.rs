use std::pin::Pin;

use pumpkin_data::translation;
use pumpkin_util::math::vector3::Vector3;

use crate::command::{argument_types::{argument_type::{ArgumentType, JavaClientArgumentType}, coordinates::{Coordinates, WorldCoordinate}}, context::command_context::CommandContext, errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType}, string_reader::StringReader, suggestion::suggestions::{Suggestions, SuggestionsBuilder}};

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
                z
            }))
        } else {
            reader.set_cursor(start);
            Err(INCOMPLETE_ERROR_TYPE.create(reader))
        }
    }

    fn list_suggestions(
        &self,
        context: &CommandContext,
        suggestions_builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::ColumnPos
    }

    fn examples(&self) -> Vec<String> {
        examples!("0 0", "~ ~", "^ ^", "^0 ^1", "~-1 ~2")
    }
}