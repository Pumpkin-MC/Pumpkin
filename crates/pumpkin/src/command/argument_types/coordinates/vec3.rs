use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::coordinates::{Coordinates, MIXED_TYPE_ERROR_TYPE};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder, TextCoordinates};
use pumpkin_data::translation;
use pumpkin_util::math::vector3::Vector3;

pub const INCOMPLETE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_POS3D_INCOMPLETE,
    translation::java::ARGUMENT_POS3D_INCOMPLETE,
);
pub const ERROR_NOT_COMPLETE: CommandErrorType<0> = INCOMPLETE_ERROR_TYPE;
pub const ERROR_MIXED_TYPE: CommandErrorType<0> = MIXED_TYPE_ERROR_TYPE;

#[derive(Debug, Default, Clone, Copy)]
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
    #[default]
    Default,
    /// No center correction occurs for this `Vec3ArgumentType` variant.
    Uncorrected,
}

impl Vec3ArgumentType {
    /// Returns whether this argument type centers integers.
    #[must_use]
    pub const fn centers_integers(&self) -> bool {
        matches!(self, Self::Default)
    }

    /// Creates a new `Vec3ArgumentType` with the given center correction.
    #[must_use]
    pub const fn new(centers_integers: bool) -> Self {
        if centers_integers {
            Self::Default
        } else {
            Self::Uncorrected
        }
    }
}

impl ArgumentType for Vec3ArgumentType {
    type Item = Coordinates;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('^') {
            Coordinates::parse_local(reader)
        } else {
            Coordinates::parse_world(reader, self.centers_integers())
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Vec3
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "0 0 0".to_string(),
            "~ ~ ~".to_string(),
            "^ ^ ^".to_string(),
            "^1 ^ ^-5".to_string(),
            "0.1 -0.5 .9".to_string(),
            "~0.5 ~1 ~-5".to_string(),
        ]
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let remainder = builder.remaining();

        let suggestioned_coordinates = if remainder.bytes().next() == Some(b'^') {
            TextCoordinates::Local
        } else {
            TextCoordinates::Global
        };

        builder.suggest_3d_coordinates(suggestioned_coordinates, |value| {
            self.parse(&mut StringReader::new(value)).is_ok()
        })
    }
}

impl Vec3ArgumentType {
    /// Returns a [`CommandContext`]'s parsed three-dimensional vector as a set of [`Coordinates`].
    pub fn get_coordinates(
        context: &CommandContext,
        name: &str,
    ) -> Result<Coordinates, CommandSyntaxError> {
        Ok(*context.get_argument(name)?)
    }

    /// Returns a [`CommandContext`]'s parsed three-dimensional vector and resolves it to a [`Vector3`].
    pub fn get_vector3(
        context: &CommandContext,
        name: &str,
    ) -> Result<Vector3<f64>, CommandSyntaxError> {
        Ok(Self::get_coordinates(context, name)?.resolve(context.source.as_ref()))
    }
}
