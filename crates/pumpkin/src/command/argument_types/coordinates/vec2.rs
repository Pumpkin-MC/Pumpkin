use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::coordinates::{Coordinates, WorldCoordinate};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder, TextCoordinates};
use pumpkin_data::translation;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

pub const INCOMPLETE_2D_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_POS2D_INCOMPLETE,
    translation::java::ARGUMENT_POS2D_INCOMPLETE,
);

#[derive(Debug, Default)]
pub enum Vec2ArgumentType {
    #[default]
    Default,
    Uncorrected,
}

impl Vec2ArgumentType {
    #[must_use]
    pub const fn centers_integers(&self) -> bool {
        matches!(self, Self::Default)
    }

    #[must_use]
    pub const fn new(centers_integers: bool) -> Self {
        if centers_integers {
            Self::Default
        } else {
            Self::Uncorrected
        }
    }
}

impl ArgumentType for Vec2ArgumentType {
    type Item = Coordinates;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        if !reader.can_read_char() {
            return Err(INCOMPLETE_2D_ERROR_TYPE.create(reader));
        }

        let x = WorldCoordinate::parse(reader, self.centers_integers())?;
        if reader.can_read_char() && reader.peek() == Some(' ') {
            reader.skip();
            let z = WorldCoordinate::parse(reader, self.centers_integers())?;
            Ok(Coordinates::World(Vector3::new(
                x,
                WorldCoordinate::Relative(0.0),
                z,
            )))
        } else {
            reader.set_cursor(start);
            Err(INCOMPLETE_2D_ERROR_TYPE.create(reader))
        }
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::Vec2
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "0 0".to_string(),
            "~ ~".to_string(),
            "0.1 -0.5".to_string(),
            "~1 ~-2".to_string(),
        ]
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let remainder = builder.remaining();

        let suggested_coordinates = if remainder.bytes().next() == Some(b'^') {
            TextCoordinates::Local
        } else {
            TextCoordinates::Global
        };

        builder.suggest_2d_coordinates(suggested_coordinates, |value| {
            self.parse(&mut StringReader::new(value)).is_ok()
        })
    }
}

impl Vec2ArgumentType {
    pub fn get_coordinates(
        context: &CommandContext,
        name: &str,
    ) -> Result<Coordinates, CommandSyntaxError> {
        Ok(*context.get_argument(name)?)
    }

    pub fn get_vector2(
        context: &CommandContext,
        name: &str,
    ) -> Result<Vector2<f64>, CommandSyntaxError> {
        let resolved = Self::get_coordinates(context, name)?.resolve(context.source.as_ref());
        Ok(Vector2::new(resolved.x, resolved.z))
    }
}
