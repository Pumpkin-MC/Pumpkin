use pumpkin_protocol::java::client::play::SuggestionProviders;
use pumpkin_util::identifier::Identifier;

use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

/// An [`Identifier`] out of a registry, shown to the client as `client`.
///
/// Commands whose ids come from a registry the client can't list (structures,
/// template pools, placed features) give `names` and have the client ask the
/// server for them; the rest leave suggestions to the node.
pub struct RegistryIdArgumentType {
    client: JavaClientArgumentType,
    names: Option<fn() -> Vec<String>>,
    examples: &'static [&'static str],
}

impl RegistryIdArgumentType {
    #[must_use]
    pub const fn new(client: JavaClientArgumentType) -> Self {
        Self {
            client,
            names: None,
            examples: &["foo", "foo:bar"],
        }
    }

    /// The ids come from `names`, which the client asks the server for.
    #[must_use]
    pub const fn with_names(
        client: JavaClientArgumentType,
        names: fn() -> Vec<String>,
        examples: &'static [&'static str],
    ) -> Self {
        Self {
            client,
            names: Some(names),
            examples,
        }
    }

    /// `ResourceKeyArgument`: `<registry>/<id>` resolved by the client.
    #[must_use]
    pub const fn resource_key(registry: Identifier) -> Self {
        Self::new(JavaClientArgumentType::ResourceKey {
            identifier: registry,
        })
    }

    /// `ResourceArgument`: an entry of `registry` (without tags).
    #[must_use]
    pub const fn resource(registry: Identifier) -> Self {
        Self::new(JavaClientArgumentType::Resource {
            identifier: registry,
        })
    }

    /// Structures, as `/place structure` takes them.
    #[must_use]
    pub const fn structure() -> Self {
        Self::with_names(
            JavaClientArgumentType::ResourceKey {
                identifier: Identifier::vanilla_static("worldgen/structure"),
            },
            || {
                pumpkin_data::structures::StructureKeys::all_names()
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect()
            },
            &["minecraft:village_plains", "minecraft:ancient_city"],
        )
    }

    /// Placed features, as `/place feature` takes them.
    #[must_use]
    pub const fn placed_feature() -> Self {
        Self::with_names(
            JavaClientArgumentType::Feature,
            || {
                pumpkin_data::placed_feature::PlacedFeature::all_names()
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect()
            },
            &["minecraft:acacia", "minecraft:amethyst_geode"],
        )
    }
}

impl<S: crate::source::CommandSource> ArgumentType<S> for RegistryIdArgumentType {
    type Item = Identifier;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        Identifier::from_reader(reader)
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        self.names.map_or_else(Suggestions::empty, |names| {
            builder.filter_and_suggest_iter(names()).build()
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        self.client.clone()
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        self.names.map(|_| SuggestionProviders::AskServer)
    }

    fn examples(&self) -> Vec<String> {
        self.examples.iter().map(|e| (*e).to_owned()).collect()
    }
}

impl RegistryIdArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<Identifier, CommandSyntaxError> {
        Ok(context.get_argument::<Identifier>(name)?.clone())
    }
}
