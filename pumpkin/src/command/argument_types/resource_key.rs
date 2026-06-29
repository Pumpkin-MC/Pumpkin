use crate::command::argument_types::FromStringReader;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::{Advancement, translation};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::resource_key::ResourceKey;
use pumpkin_util::text::TextComponent;
use std::pin::Pin;
use std::string::ToString;

pub static ADVANCEMENT_REGISTRY: Identifier = Identifier::vanilla_static("advancement");

pub const ERROR_INVALID_ADVANCEMENT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ADVANCEMENT_ADVANCEMENTNOTFOUND,
    translation::java::ADVANCEMENT_ADVANCEMENTNOTFOUND,
);

/// Represents an argument type used to get a resource key from an identifier.
///
/// if you want an [`Advancement`] put the [`ADVANCEMENT_REGISTRY`]
/// and get it with the [`ResourceKeyArgument::get_advancement`] when it has been parsed
///
/// TODO Recipe
///
/// if you juste want the [`ResourceKey`] use the [`ResourceKeyArgument::get_registry_key`] function
pub struct ResourceKeyArgument(pub Identifier);

pub static ERROR_INVALID: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_ID_INVALID,
    translation::java::ARGUMENT_ID_INVALID,
);

impl ArgumentType for ResourceKeyArgument {
    type Item = ResourceKey;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        Ok(ResourceKey::new(self.0.clone(), identifier))
    }

    fn list_suggestions(
        &self,
        context: &CommandContext,
        suggestions_builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        if self.0 == ADVANCEMENT_REGISTRY {
            let advancements = context.server().advancement_manager.get_advancements();
            Box::pin(async move {
                suggestions_builder
                    .filter_and_suggest_iter(advancements.iter().map(ToString::to_string))
                    .build()
            })
        } else {
            Box::pin(async move { Suggestions::empty() })
        }
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::ResourceKey {
            identifier: self.0.clone(),
        }
    }
}

impl ResourceKeyArgument {
    /// Returns a [`CommandContext`]'s parsed resource key argument as an [`Advancement`].
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed [`ResourceKey`] with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The `Advancement` containing the advancement get from the resource key argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved or
    /// that the key correspond to an unknown advancement.
    pub fn get_advancement(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static Advancement, CommandSyntaxError> {
        let resource_key: &ResourceKey = Self::get_registry_key(
            context,
            name,
            &ADVANCEMENT_REGISTRY,
            &ERROR_INVALID_ADVANCEMENT,
        )?;
        Advancement::from_name(resource_key.identifier.path()).ok_or_else(|| {
            ERROR_INVALID_ADVANCEMENT.create_without_context(TextComponent::text(
                resource_key.identifier.path().to_string(),
            ))
        })
    }

    /// Returns a [`CommandContext`]'s parsed resource key argument in the form of a [`ResourceKey`].
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed [`ResourceKey`] with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The `ResourceKey` containing the key get from the argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved
    pub fn get_registry_key<'a>(
        context: &'a CommandContext,
        name: &str,
        registry: &Identifier,
        error: &'static CommandErrorType<1>,
    ) -> Result<&'a ResourceKey, CommandSyntaxError> {
        let argument = context.get_argument::<ResourceKey>(name)?;
        argument.cast(registry).ok_or_else(|| {
            error
                .create_without_context(TextComponent::text(argument.identifier.path().to_string()))
        })
    }
}
