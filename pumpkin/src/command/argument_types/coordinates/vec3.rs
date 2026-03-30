use pumpkin_data::translation;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::coordinates::Coordinates;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;

pub const INCOMPLETE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(translation::ARGUMENT_POS3D_INCOMPLETE);

/// An argument type for a 3-dimensional vector.
pub enum Vec3ArgumentType {
    /// The default `Vec3ArgumentType` variant.
    ///
    /// To represent some position in the world,
    /// you'll almost always want to use this.
    ///
    /// For each coordinate, if it does not use the decimal (`.`) sign
    /// (the coordinate is integral) and it is not relative,
    /// a `+0.5` offset is added to it.
    ///
    Default,
    /// No center correction occurs for this `Vec3ArgumentType` variant.
    Uncorrected
}

impl ArgumentType for Vec3ArgumentType {
    type Item = Coordinates;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        todo!()
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Vec3
    }

    fn examples(&self) -> Vec<String> {
        examples!("1 1 1", "3 ~34 ~-2", "40 50 60", "^ ^4 ^3")
    }
}
