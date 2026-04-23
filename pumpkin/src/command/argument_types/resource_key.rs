use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::{translation, Advancement};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::resource_key::ResourceKey;
use pumpkin_util::text::TextComponent;
use std::string::ToString;
use crate::command::argument_types::FromStringReader;

pub static ADVANCEMENT_REGISTRY: Identifier = Identifier::vanilla_static("advancement");

pub const ERROR_INVALID_ADVANCEMENT : CommandErrorType<1> =
    CommandErrorType::new(translation::ADVANCEMENT_ADVANCEMENTNOTFOUND);

pub struct ResourceKeyArgument(pub Identifier);

pub static ERROR_INVALID : CommandErrorType<0> = CommandErrorType::new("argument.id.invalid");

impl ArgumentType for ResourceKeyArgument {
    type Item = ResourceKey;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        Ok(ResourceKey::new(self.0.clone(),identifier))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::ResourceKey {identifier: self.0.path()}
    }
}

impl ResourceKeyArgument {
    pub fn get_advancement(context: &CommandContext,name:&str) -> Result<&'static Advancement, CommandSyntaxError> {
        let resource_key: &ResourceKey = Self::get_registry_key(context, name, &ADVANCEMENT_REGISTRY, &ERROR_INVALID_ADVANCEMENT)?;
        Advancement::from_name(resource_key.identifier.path())
            .ok_or_else(|| ERROR_INVALID_ADVANCEMENT.create_without_context(TextComponent::text(resource_key.identifier.path().to_string())))
    }

    pub fn get_registry_key<'a>(context: &'a CommandContext, name: &str, registry: &Identifier, error: &'static CommandErrorType<1>) -> Result<&'a ResourceKey, CommandSyntaxError> {
        let argument = context.get_argument::<ResourceKey>(name)?;
        argument.cast(registry).ok_or_else(|| error.create_without_context(TextComponent::text(argument.identifier.path().to_string())))
    }
}