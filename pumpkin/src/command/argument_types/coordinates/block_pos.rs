use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::coordinates::Coordinates;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::string_reader::StringReader;

/// An argument type for a 3-dimensional vector representing a block position.
///
/// The returned [`Coordinates`] can be converted to a [`BlockPos`] via one of the
/// following methods:
/// - [`Coordinates::to_block_pos`]: Normal conversion that always succeeds.
/// - [`Coordinates::try_to_loaded_block_pos`]: Converts the coordinates to a *loaded* `BlockPos`.
///   This may not succeed. This is what you want to use most of the time.
/// - [`Coordinates::try_to_loaded_block_pos_in_world`]: Converts the coordinates to a *loaded* `BlockPos`.
///   in the provided world. This may not succeed.
pub struct BlockPosArgumentType;

impl ArgumentType for BlockPosArgumentType {
    type Item = Coordinates;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('^') {
            Coordinates::parse_local(reader)
        } else {
            Coordinates::parse_world_integers(reader)
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Vec3
    }

    fn examples(&self) -> Vec<String> {
        examples!("1 3 5", "-3 ~24 ~-1", "80 80 80", "^ ^9 ^56")
    }
}
