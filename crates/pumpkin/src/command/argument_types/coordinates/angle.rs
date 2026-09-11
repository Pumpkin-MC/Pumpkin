use pumpkin_data::translation;
use pumpkin_util::math::wrap_degrees;

use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;

pub const ERROR_NOT_COMPLETE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_ANGLE_INCOMPLETE,
    translation::java::ARGUMENT_ANGLE_INCOMPLETE,
);

pub const ERROR_INVALID_ANGLE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_ANGLE_INVALID,
    translation::java::ARGUMENT_ANGLE_INVALID,
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
    pub angle: f32,
    pub is_relative: bool,
}

impl Angle {
    #[must_use]
    pub const fn new(angle: f32, is_relative: bool) -> Self {
        Self { angle, is_relative }
    }

    #[must_use]
    pub fn get_angle(&self, source: &CommandSource) -> f32 {
        let base = if self.is_relative {
            source.rotation.y
        } else {
            0.0
        };
        wrap_degrees(base + self.angle)
    }
}

pub struct AngleArgumentType;

impl ArgumentType for AngleArgumentType {
    type Item = Angle;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if !reader.can_read_char() {
            return Err(ERROR_NOT_COMPLETE.create(reader));
        }

        let mut is_relative = false;
        if reader.peek() == Some('~') {
            is_relative = true;
            reader.skip();
        }

        let angle = if reader.can_read_char() && reader.peek() != Some(' ') {
            reader.read_float()?
        } else if is_relative {
            0.0
        } else {
            return Err(ERROR_NOT_COMPLETE.create(reader));
        };

        if angle.is_nan() || angle.is_infinite() {
            return Err(ERROR_INVALID_ANGLE.create(reader));
        }

        Ok(Angle::new(angle, is_relative))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Angle
    }

    fn examples(&self) -> Vec<String> {
        examples!("0", "~", "~-5")
    }
}

impl AngleArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<Angle, CommandSyntaxError> {
        context.get_argument::<Angle>(name).copied()
    }
}
