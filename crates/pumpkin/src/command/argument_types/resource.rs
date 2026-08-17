use crate::command::argument_types::FromStringReader;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{CommandErrorType, DISPATCHER_PARSE_EXCEPTION};
use crate::command::node::attached::AttachedNode;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::entity::EntityType;
use pumpkin_data::translation;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;
use std::any::Any;
use std::iter::Iterator;
use std::pin::Pin;

pub static ENTITY_TYPE_REGISTRY: &Identifier = &Identifier::vanilla_static("entity_type");
static ERROR_UNKNOWN_RESOURCE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
);

static ERROR_INVALID_RESOURCE_TYPE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_INVALID_TYPE,
    translation::java::ARGUMENT_RESOURCE_INVALID_TYPE,
);

static ERROR_NOT_SUMMONABLE_ENTITY: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ENTITY_NOT_SUMMONABLE,
    translation::java::ENTITY_NOT_SUMMONABLE,
);

pub static ENTITY_TYPE_ARGUMENT: ResourceArgument =
    ResourceArgument(ENTITY_TYPE_REGISTRY, &|id: Identifier| {
        EntityType::from_name(id.path()).map(|value| value as &'static (dyn Any + Send + Sync))
    });

#[derive(Clone)]
pub struct ResourceArgument(
    pub &'static Identifier,
    pub &'static (dyn Fn(Identifier) -> Option<&'static (dyn Any + Send + Sync)> + Send + Sync),
);

impl ArgumentType for ResourceArgument {
    type Item = &'static (dyn Any + Send + Sync);

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        self.1(identifier.clone()).ok_or_else(|| {
            ERROR_UNKNOWN_RESOURCE.create(
                reader,
                TextComponent::text(identifier.path().to_string()),
                TextComponent::text(self.0.to_string()),
            )
        })
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        suggestions_builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        if self.0 == ENTITY_TYPE_REGISTRY {
            Box::pin(async move {
                let entity_types = EntityType::ALL
                    .iter()
                    .map(|entity_type| format!("minecraft:{}", entity_type.resource_name));
                suggestions_builder
                    .filter_and_suggest_iter(entity_types)
                    .build()
            })
        } else {
            Box::pin(async move { Suggestions::empty() })
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Resource {
            identifier: self.0.clone(),
        }
    }
}

impl ResourceArgument {
    pub fn get_resource<T: 'static>(
        context: &CommandContext,
        name: &str,
        registry_key: &Identifier,
    ) -> Result<&'static T, CommandSyntaxError> {
        let missing_argument = DISPATCHER_PARSE_EXCEPTION.create_without_context(
            TextComponent::text(format!("Could not find argument with name '{name}'")),
        );
        let node = context
            .tree
            .iter()
            .find_map(|node| {
                if let AttachedNode::Argument(cur) = node
                    && cur.meta.name == name
                {
                    Some(cur)
                } else {
                    None
                }
            })
            .ok_or(missing_argument.clone())?;
        let invalid_argument =
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(format!(
                "argument with name '{name}' isn't a ResourceArgument"
            )));
        let result_argument = node
            .meta
            .argument_type
            .as_any()
            .downcast_ref::<Self>()
            .ok_or(invalid_argument)?;
        let registry_name = result_argument.0;
        let identifier = context
            .arguments
            .get(name)
            .ok_or(missing_argument)?
            .range
            .substring_slice(context.input.as_str())
            .to_string();
        let err = ERROR_INVALID_RESOURCE_TYPE.create_without_context(
            TextComponent::text(identifier),
            TextComponent::text(registry_name.to_string()),
            TextComponent::text(registry_key.to_string()),
        );
        if registry_name == registry_key {
            context
                .get_argument::<&'static (dyn Any + Send + Sync)>(name)?
                .downcast_ref::<T>()
                .ok_or(err)
        } else {
            Err(err)
        }
    }

    pub fn get_entity_type(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static EntityType, CommandSyntaxError> {
        Self::get_resource(context, name, ENTITY_TYPE_REGISTRY)
    }

    pub fn get_summonable_entity_type(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static EntityType, CommandSyntaxError> {
        let val: &'static EntityType = Self::get_resource(context, name, ENTITY_TYPE_REGISTRY)?;
        if val.summonable {
            Ok(val)
        } else {
            Err(ERROR_NOT_SUMMONABLE_ENTITY
                .create_without_context(TextComponent::text(val.resource_name)))
        }
    }
}
