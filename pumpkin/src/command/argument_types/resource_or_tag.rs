use std::{pin::Pin, string::ToString};

use pumpkin_data::{
    biome::Biome,
    tag::{RegistryKey, get_registry_key_tags},
};
use pumpkin_util::{identifier::Identifier, version::JavaMinecraftVersion};

use crate::command::{
    argument_types::{
        FromStringReader,
        argument_type::{ArgumentType, JavaClientArgumentType},
    },
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

pub static BIOME_REGISTRY: Identifier = Identifier::vanilla_static("worldgen/biome");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOrTag {
    pub identifier: Identifier,
    pub is_tag: bool,
}

impl ResourceOrTag {
    #[must_use]
    pub fn printable(&self) -> String {
        if self.is_tag {
            format!("#{}", self.identifier)
        } else {
            self.identifier.to_string()
        }
    }
}

pub struct ResourceOrTagArgument {
    registry: Identifier,
    registry_key: RegistryKey,
}

impl ResourceOrTagArgument {
    #[must_use]
    pub fn biome() -> Self {
        Self {
            registry: BIOME_REGISTRY.clone(),
            registry_key: RegistryKey::WorldgenBiome,
        }
    }

    pub fn get<'a>(
        context: &'a CommandContext,
        name: &str,
    ) -> Result<&'a ResourceOrTag, CommandSyntaxError> {
        context.get_argument(name)
    }

    fn direct_suggestions() -> Vec<String> {
        Biome::ALL
            .iter()
            .map(|biome| format!("minecraft:{}", biome.registry_id))
            .collect()
    }

    fn tag_suggestions(&self) -> Vec<String> {
        get_registry_key_tags(JavaMinecraftVersion::V_26_2, self.registry_key)
            .into_iter()
            .flat_map(|tags| tags.keys())
            .map(|tag| format!("#{tag}"))
            .collect()
    }
}

impl ArgumentType for ResourceOrTagArgument {
    type Item = ResourceOrTag;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let is_tag = reader.peek() == Some('#');
        if is_tag {
            reader.skip();
        }

        Ok(ResourceOrTag {
            identifier: Identifier::from_reader(reader)?,
            is_tag,
        })
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        suggestions_builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        let direct = Self::direct_suggestions();
        let tags = self.tag_suggestions();
        Box::pin(async move {
            suggestions_builder
                .filter_and_suggest_iter(direct)
                .filter_and_suggest_iter(tags)
                .build()
        })
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::ResourceOrTag {
            identifier: self.registry.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ResourceOrTag, ResourceOrTagArgument};
    use crate::command::{
        argument_types::argument_type::ArgumentType, string_reader::StringReader,
    };
    use pumpkin_util::identifier::Identifier;

    #[test]
    fn parses_direct_resource() {
        let mut reader = StringReader::new("minecraft:plains");
        assert_eq!(
            ResourceOrTagArgument::biome().parse(&mut reader),
            Ok(ResourceOrTag {
                identifier: Identifier::parse("minecraft:plains").unwrap(),
                is_tag: false,
            })
        );
    }

    #[test]
    fn parses_tag() {
        let mut reader = StringReader::new("#minecraft:is_overworld");
        assert_eq!(
            ResourceOrTagArgument::biome().parse(&mut reader),
            Ok(ResourceOrTag {
                identifier: Identifier::parse("minecraft:is_overworld").unwrap(),
                is_tag: true,
            })
        );
    }
}
