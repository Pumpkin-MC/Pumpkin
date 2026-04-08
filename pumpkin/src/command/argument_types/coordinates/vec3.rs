use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::coordinates::Coordinates;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::translation;

pub const INCOMPLETE_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_POS3D_INCOMPLETE);

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
    Uncorrected,
}

impl Vec3ArgumentType {
    /// Returns whether this argument type centers integers.\
    #[must_use]
    pub const fn centers_integers(&self) -> bool {
        matches!(self, Self::Default)
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

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Vec3
    }

    fn examples(&self) -> Vec<String> {
        examples!("1 1 1", "3 ~34 ~-2", "40 50 60", "^ ^4 ^3")
    }
}

#[cfg(test)]
mod test {
    use crate::command::argument_types::argument_type::ArgumentType;
    use crate::command::argument_types::coordinates::vec3::{
        INCOMPLETE_ERROR_TYPE, Vec3ArgumentType,
    };
    use crate::command::argument_types::coordinates::{
        Coordinates, MIXED_TYPE_ERROR_TYPE, WorldCoordinate,
    };
    use crate::command::string_reader::StringReader;
    use pumpkin_util::math::vector3::Vector3;

    macro_rules! world_coordinate {
        ($( ($variant:ident, $value:expr) ),+) => {
            Coordinates::World(Vector3::new(
                $( WorldCoordinate::$variant($value), )+
            ))
        };
    }

    #[test]
    fn parse_test() {
        let mut reader = StringReader::new("0 0 0");

        // The default type centers both the X and Z coordinates.
        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Default,
            world_coordinate!((Absolute, 0.5), (Absolute, 0.0), (Absolute, 0.5))
        );
        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Uncorrected,
            world_coordinate!((Absolute, 0.0), (Absolute, 0.0), (Absolute, 0.0))
        );

        let mut reader = StringReader::new("~ ~4 8");

        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Default,
            world_coordinate!(
                (Relative, 0.0),
                (Relative, 4.0),
                // Only the Z coordinate is centered.
                (Absolute, 8.5)
            )
        );
        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Uncorrected,
            world_coordinate!((Relative, 0.0), (Relative, 4.0), (Absolute, 8.0))
        );

        let mut reader = StringReader::new("~-1 ~-2.5 ~-3");

        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Default,
            world_coordinate!((Relative, -1.0), (Relative, -2.5), (Relative, -3.0))
        );
        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Uncorrected,
            world_coordinate!((Relative, -1.0), (Relative, -2.5), (Relative, -3.0))
        );

        let mut reader = StringReader::new("-3 4 5");

        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Default,
            world_coordinate!((Absolute, -2.5), (Absolute, 4.0), (Absolute, 5.5))
        );

        let mut reader = StringReader::new("1000 2000");

        assert_parse_err_reset!(reader, Vec3ArgumentType::Default, &INCOMPLETE_ERROR_TYPE);
    }

    #[test]
    fn parse_local_coordinates() {
        let mut reader = StringReader::new("^1 ^10 ^30");

        assert_parse_ok_reset!(
            reader,
            Vec3ArgumentType::Default,
            Coordinates::Local {
                left: 1.0,
                up: 10.0,
                forward: 30.0
            }
        );

        // We can't mix world & local coordinates.
        let mut reader = StringReader::new("^1 ^10 ~20");

        assert_parse_err_reset!(reader, Vec3ArgumentType::Default, &MIXED_TYPE_ERROR_TYPE);

        let mut reader = StringReader::new("^-3 ^-7 10");

        assert_parse_err_reset!(reader, Vec3ArgumentType::Default, &MIXED_TYPE_ERROR_TYPE);

        let mut reader = StringReader::new("^1 ^2");

        assert_parse_err_reset!(reader, Vec3ArgumentType::Default, &INCOMPLETE_ERROR_TYPE);
    }
}
