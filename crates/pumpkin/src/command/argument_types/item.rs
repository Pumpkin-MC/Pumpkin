use pumpkin_data::data_component::DataComponent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::snbt::SnbtParser;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_UNKNOWN_ITEM: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ITEM_ID_INVALID,
    translation::java::ARGUMENT_ITEM_ID_INVALID,
);

#[derive(Clone, Copy)]
pub struct ItemStackArgumentType;

impl ArgumentType for ItemStackArgumentType {
    type Item = ItemStack;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        while let Some(c) = reader.peek() {
            if c.is_alphanumeric() || c == '_' || c == ':' || c == '/' || c == '.' || c == '-' {
                reader.skip();
            } else {
                break;
            }
        }
        let raw_id = &reader.string()[start..reader.cursor()];
        let item = Item::from_registry_key(raw_id).ok_or_else(|| {
            let full_name = if raw_id.contains(':') {
                raw_id.to_string()
            } else {
                format!("minecraft:{raw_id}")
            };
            ERROR_UNKNOWN_ITEM.create(reader, TextComponent::text(full_name))
        })?;

        let mut stack = ItemStack::new(1, item);

        // Optional components [...]
        if reader.peek() == Some('[') {
            reader.skip();
            let mut patch = Vec::new();
            while reader.can_read_char() && reader.peek() != Some(']') {
                let key_start = reader.cursor();
                while let Some(c) = reader.peek() {
                    if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' || c == '-' {
                        reader.skip();
                    } else {
                        break;
                    }
                }
                let key_str = reader.string()[key_start..reader.cursor()]
                    .trim()
                    .to_string();

                reader.skip_whitespace();
                if reader.peek() == Some('=') {
                    reader.skip();
                    reader.skip_whitespace();
                    let nbt_tag = SnbtParser::parse_for_commands(reader)?;
                    if let (Some(data_comp), Some(comp_impl)) = (
                        DataComponent::try_from_name(&key_str),
                        pumpkin_data::data_component_impl::read_data(
                            DataComponent::try_from_name(&key_str)
                                .unwrap_or(DataComponent::CustomData),
                            &nbt_tag,
                        ),
                    ) {
                        patch.push((data_comp, Some(comp_impl)));
                    }
                }
                reader.skip_whitespace();
                if reader.peek() == Some(',') {
                    reader.skip();
                    reader.skip_whitespace();
                }
            }
            if reader.peek() == Some(']') {
                reader.skip();
            }
            if !patch.is_empty() {
                stack.patch = patch;
            }
        }

        Ok(stack)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ItemStack
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

impl ItemStackArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<ItemStack, CommandSyntaxError> {
        context.get_argument::<ItemStack>(name).cloned()
    }
}
