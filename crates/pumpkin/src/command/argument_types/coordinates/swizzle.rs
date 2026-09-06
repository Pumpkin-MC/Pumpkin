use pumpkin_data::translation;
use pumpkin_util::math::vector3::Vector3;

use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;

pub const ERROR_INVALID_SWIZZLE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_SWIZZLE_INVALID,
    translation::java::ARGUMENTS_SWIZZLE_INVALID,
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Swizzle {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl Swizzle {
    #[must_use]
    pub const fn align(&self, pos: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            if self.x { pos.x.floor() } else { pos.x },
            if self.y { pos.y.floor() } else { pos.y },
            if self.z { pos.z.floor() } else { pos.z },
        )
    }
}

pub struct SwizzleArgumentType;

impl ArgumentType for SwizzleArgumentType {
    type Item = Swizzle;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let mut swizzle = Swizzle::default();
        let mut count = 0;

        while reader.can_read_char() && reader.peek() != Some(' ') {
            let c = reader.read().unwrap();
            match c {
                'x' => {
                    if swizzle.x {
                        return Err(ERROR_INVALID_SWIZZLE.create(reader));
                    }
                    swizzle.x = true;
                }
                'y' => {
                    if swizzle.y {
                        return Err(ERROR_INVALID_SWIZZLE.create(reader));
                    }
                    swizzle.y = true;
                }
                'z' => {
                    if swizzle.z {
                        return Err(ERROR_INVALID_SWIZZLE.create(reader));
                    }
                    swizzle.z = true;
                }
                _ => return Err(ERROR_INVALID_SWIZZLE.create(reader)),
            }
            count += 1;
        }

        if count == 0 {
            return Err(ERROR_INVALID_SWIZZLE.create(reader));
        }

        Ok(swizzle)
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::Swizzle
    }

    fn examples(&self) -> Vec<String> {
        vec!["xyz".to_string(), "x".to_string()]
    }
}

impl SwizzleArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<Swizzle, CommandSyntaxError> {
        Ok(*context.get_argument::<Swizzle>(name)?)
    }
}
