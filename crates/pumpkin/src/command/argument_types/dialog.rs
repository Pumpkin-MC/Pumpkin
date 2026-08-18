use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::identifier::Identifier;

use crate::command::{
    argument_types::{
        FromStringReader,
        argument_type::{ArgumentType, JavaClientArgumentType},
        nbt::NbtCompoundArgumentType,
    },
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
};

#[derive(Clone, Debug, PartialEq)]
pub enum DialogArg {
    Id(Identifier),
    Nbt(NbtCompound),
}

pub struct DialogArgumentType;

impl ArgumentType for DialogArgumentType {
    type Item = DialogArg;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('{') {
            let compound = NbtCompoundArgumentType.parse(reader)?;
            Ok(DialogArg::Nbt(compound))
        } else {
            let id = Identifier::from_reader(reader)?;
            Ok(DialogArg::Id(id))
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Dialog
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "minecraft:server_links",
            "custom:my_dialog",
            "{type:\"minecraft:notice\",title:\"Hello\"}"
        )
    }
}

impl DialogArgumentType {
    /// Returns the parsed [`DialogArg`] from the name of the argument.
    pub fn get<'a>(
        context: &'a CommandContext,
        name: &'_ str,
    ) -> Result<&'a DialogArg, CommandSyntaxError> {
        context.get_argument(name)
    }
}
