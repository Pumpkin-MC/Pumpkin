use std::string::ToString;
use serde::Serialize;
use pumpkin_data::{translation, Advancement};
use pumpkin_data::registry::Registry;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;


#[derive(Debug,Clone)]
pub struct ResourceKey<'a> {
    registry_name: &'a str,
    identifier: ResourceLocation,
}

impl <'a> ResourceKey<'a> {
    pub fn new(registry_name: &'a str, identifier: ResourceLocation) -> Self {
        Self { registry_name, identifier }
    }

    pub fn cast(&self, registry: &'a str) -> Option<&ResourceKey> {
        if self.identifier == registry {
            Some(self)
        } else {
            None
        }
    }
}

pub const ADVANCEMENT_REGISTRY: &str = "advancement";

pub const ERROR_INVALID_ADVANCEMENT : CommandErrorType<1> =
    CommandErrorType::new(translation::ADVANCEMENT_ADVANCEMENTNOTFOUND);

pub struct ResourceKeyArgument<'a>(&'a str);

impl <'a> ArgumentType for ResourceKeyArgument<'a> {
    type Item = ResourceKey<'a>;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = reader.read_unquoted_string()?;
        Ok(ResourceKey::new(self.0,identifier))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::ResourceKey {identifier: self.0}
    }
}

impl<'a> ResourceKeyArgument<'a> {
    pub fn get_advancement(context: &CommandContext,name:&str) -> Result<&'static Advancement, CommandSyntaxError> {
        let identifier : &ResourceKey = Self::get_registry_key(context,name,ADVANCEMENT_REGISTRY,ERROR_INVALID_ADVANCEMENT)?;
        Advancement::from_name(identifier.identifier.as_str())
            .ok_or_else(|| ERROR_INVALID_ADVANCEMENT.create_without_context(TextComponent::text(identifier.identifier)))
    }

    pub fn get_registry_key(context: &CommandContext, name: &str, registry: &str, error: CommandErrorType<1>) -> Result<&'a ResourceKey<'a>, CommandSyntaxError> {
        let argument = context.get_argument::<ResourceKey>(name)?;
        argument.cast(registry).ok_or_else(|| error.create_without_context(TextComponent::text(argument.identifier)))
    }
}