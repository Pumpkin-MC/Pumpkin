use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumeResult, ConsumedArgs, DefaultNameArgConsumer, FindArg,
        GetClientSideArgParser, SuggestResult,
    },
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

const STRUCTURES: &[&str] = &[
    "minecraft:pillager_outpost",
    "minecraft:mineshaft",
    "minecraft:mineshaft_mesa",
    "minecraft:mansion",
    "minecraft:woodland_mansion",
    "minecraft:jungle_pyramid",
    "minecraft:jungle_temple",
    "minecraft:desert_pyramid",
    "minecraft:igloo",
    "minecraft:shipwreck",
    "minecraft:shipwreck_beached",
    "minecraft:swamp_hut",
    "minecraft:stronghold",
    "minecraft:monument",
    "minecraft:ocean_monument",
    "minecraft:ocean_ruin_cold",
    "minecraft:ocean_ruin_warm",
    "minecraft:fortress",
    "minecraft:nether_fossil",
    "minecraft:end_city",
    "minecraft:buried_treasure",
    "minecraft:bastion_remnant",
    "minecraft:village_plains",
    "minecraft:village_desert",
    "minecraft:village_savanna",
    "minecraft:village_snowy",
    "minecraft:village_taiga",
    "minecraft:ruined_portal",
    "minecraft:ruined_portal_desert",
    "minecraft:ruined_portal_jungle",
    "minecraft:ruined_portal_swamp",
    "minecraft:ruined_portal_mountain",
    "minecraft:ruined_portal_ocean",
    "minecraft:ruined_portal_nether",
    "minecraft:ancient_city",
    "minecraft:trail_ruins",
    "minecraft:trial_chambers",
];

const STRUCTURE_TAGS: &[&str] = &[
    "#minecraft:village",
    "#minecraft:mineshaft",
    "#minecraft:shipwreck",
    "#minecraft:ruined_portal",
    "#minecraft:ocean_ruin",
    "#minecraft:cats_spawn_in",
];

pub struct StructureArgumentConsumer;

impl GetClientSideArgParser for StructureArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::ResourceOrTagKey {
            identifier: "minecraft:worldgen/structure",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for StructureArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let name_opt = args.pop().map(|arg| arg.value);
        let result = name_opt.and_then(|name| {
            let is_valid = if name.starts_with('#') {
                STRUCTURE_TAGS.iter().any(|&tag| {
                    tag.eq_ignore_ascii_case(name)
                        || tag
                            .strip_prefix("#minecraft:")
                            .unwrap_or(tag)
                            .eq_ignore_ascii_case(name.strip_prefix('#').unwrap())
                })
            } else {
                STRUCTURES.iter().any(|&struct_name| {
                    struct_name.eq_ignore_ascii_case(name)
                        || struct_name
                            .strip_prefix("minecraft:")
                            .unwrap_or(struct_name)
                            .eq_ignore_ascii_case(name)
                })
            };
            is_valid.then(|| Arg::ResourceLocation(name))
        });
        Box::pin(async move { result })
    }

    fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        input: &'a str,
    ) -> SuggestResult<'a> {
        Box::pin(async move {
            let last_word_start = input.char_indices().rfind(|(_, c)| c.is_whitespace());
            let typed_word = match last_word_start {
                Some((idx, _)) => &input[idx + 1..],
                None => input,
            };
            let input_lower = typed_word.to_lowercase();
            let mut matches = Vec::new();
            for s in STRUCTURES.iter().chain(STRUCTURE_TAGS.iter()) {
                let s_lower = s.to_lowercase();
                let mut match_found = s_lower.starts_with(&input_lower);

                if !match_found
                    && let Some(stripped) = s_lower.strip_prefix("minecraft:")
                    && stripped.starts_with(&input_lower)
                {
                    match_found = true;
                }

                if !match_found && s_lower.starts_with('#') {
                    let tag_name = s_lower.strip_prefix('#').unwrap();
                    let user_tag_name = input_lower.strip_prefix('#').unwrap_or(&input_lower);
                    if tag_name.starts_with(user_tag_name) {
                        match_found = true;
                    } else if let Some(stripped_tag) = tag_name.strip_prefix("minecraft:")
                        && stripped_tag.starts_with(user_tag_name)
                    {
                        match_found = true;
                    }
                }

                if match_found {
                    matches.push(CommandSuggestion::new(s.to_string(), None));
                }
            }
            Ok(Some(matches))
        })
    }
}

impl DefaultNameArgConsumer for StructureArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "structure"
    }
}

impl<'a> FindArg<'a> for StructureArgumentConsumer {
    type Data = &'a str;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::ResourceLocation(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
