use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::slot_ranges::{
    SLOT_RANGE_ALL_NAMES, SLOT_RANGE_SINGLE_SLOT_NAMES, get_slot_range,
};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use std::pin::Pin;

pub const UNKNOWN_SLOT_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SLOT_UNKNOWN,
    translation::java::SLOT_UNKNOWN,
);

pub const ONLY_SINGLE_SLOT_ALLOWED_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SLOT_ONLY_SINGLE_ALLOWED,
    translation::java::SLOT_ONLY_SINGLE_ALLOWED,
);

pub struct SlotArgumentType;

impl ArgumentType for SlotArgumentType {
    type Item = usize;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        reader.read_until_space();
        let name = &reader.string()[start..reader.cursor()];

        match get_slot_range(name) {
            Some([slot]) => Ok(*slot),
            Some(_) => Err(ONLY_SINGLE_SLOT_ALLOWED_ERROR_TYPE
                .create(reader, TextComponent::text(name.to_string()))),
            None => {
                Err(UNKNOWN_SLOT_ERROR_TYPE.create(reader, TextComponent::text(name.to_string())))
            }
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::ItemSlot
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move {
            builder
                .filter_and_suggest(&SLOT_RANGE_SINGLE_SLOT_NAMES)
                .build()
        })
    }

    fn examples(&self) -> Vec<String> {
        examples!("weapon", "container.2", "enderchest.0")
    }
}

impl_copy_get!(SlotArgumentType, usize);

pub struct SlotsArgumentType;

impl ArgumentType for SlotsArgumentType {
    type Item = &'static [usize];

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        reader.read_until_space();
        let name = &reader.string()[start..reader.cursor()];

        get_slot_range(name).ok_or_else(|| {
            let text = name.to_string();
            UNKNOWN_SLOT_ERROR_TYPE.create(reader, TextComponent::text(text))
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::ItemSlots
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move { builder.filter_and_suggest(&SLOT_RANGE_ALL_NAMES).build() })
    }

    fn examples(&self) -> Vec<String> {
        examples!("weapon", "container.2", "enderchest.*")
    }
}

impl_copy_get!(SlotsArgumentType, &'static [usize]);
