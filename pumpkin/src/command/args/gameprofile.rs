use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use uuid::Uuid;

use crate::{
    command::{
        CommandSender,
        args::{ConsumeResult, SuggestResult},
        dispatcher::CommandError,
        tree::RawArgs,
    },
    net::{GameProfile, offline_uuid},
    server::Server,
};

use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};
use crate::command::args::ArgumentConsumer;

/// Select one or multiple game profiles.
///
/// This behaves like a simplified vanilla `game_profile` argument:
/// selectors are resolved against online players, while plain names/UUIDs
/// can also be resolved from JSON-backed lists for offline targets.
pub struct GameProfilesArgumentConsumer;

impl GetClientSideArgParser for GameProfilesArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::GameProfile
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for GameProfilesArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let Some(s) = s_opt else {
            return Box::pin(async move { None });
        };

        let sync_result: Option<Vec<GameProfile>> = match s {
            "@s" => sender.as_player().map(|p| vec![p.gameprofile.clone()]),
            "@n" | "@p" => sender.as_player().map(|p| vec![p.gameprofile.clone()]),
            _ => None,
        };

        if let Some(profiles) = sync_result {
            return Box::pin(async move { Some(Arg::GameProfiles(profiles)) });
        }

        Box::pin(async move {
            let profiles = match s {
                "@r" => server
                    .get_random_player()
                    .map_or_else(|| Some(vec![]), |p| Some(vec![p.gameprofile.clone()])),
                "@a" | "@e" => Some(
                    server
                        .get_all_players()
                        .into_iter()
                        .map(|p| p.gameprofile.clone())
                        .collect(),
                ),
                value => resolve_single_profile(server, value)
                    .await
                    .map(|profile| vec![profile]),
            };

            profiles.map(Arg::GameProfiles)
        })
    }

    fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        server: &'a Server,
        _input: &'a str,
    ) -> SuggestResult<'a> {
        Box::pin(async move {
            let mut suggestions = vec![
                CommandSuggestion::new("@s".to_string(), None),
                CommandSuggestion::new("@p".to_string(), None),
                CommandSuggestion::new("@r".to_string(), None),
                CommandSuggestion::new("@a".to_string(), None),
                CommandSuggestion::new("@e".to_string(), None),
            ];

            let mut names = Vec::new();
            for player in server.get_all_players() {
                push_name_if_missing(&mut names, player.gameprofile.name.clone());
            }

            {
                let ops = server.data.operator_config.read().await;
                for op in &ops.ops {
                    push_name_if_missing(&mut names, op.name.clone());
                }
            }

            {
                let banned_players = server.data.banned_player_list.read().await;
                for banned in &banned_players.banned_players {
                    push_name_if_missing(&mut names, banned.name.clone());
                }
            }

            {
                let whitelist = server.data.whitelist_config.read().await;
                for entry in &whitelist.whitelist {
                    push_name_if_missing(&mut names, entry.name.clone());
                }
            }

            suggestions.extend(
                names
                    .into_iter()
                    .map(|name| CommandSuggestion::new(name, None)),
            );

            Ok(Some(suggestions))
        })
    }
}

impl DefaultNameArgConsumer for GameProfilesArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "targets"
    }
}

impl<'a> FindArg<'a> for GameProfilesArgumentConsumer {
    type Data = &'a [GameProfile];

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::GameProfiles(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

fn push_name_if_missing(names: &mut Vec<String>, name: String) {
    if names.iter().any(|known| known.eq_ignore_ascii_case(&name)) {
        return;
    }
    names.push(name);
}

async fn resolve_single_profile(server: &Server, value: &str) -> Option<GameProfile> {
    if let Some(player) = server.get_player_by_name(value) {
        return Some(player.gameprofile.clone());
    }

    if let Ok(uuid) = Uuid::parse_str(value) {
        if let Some(player) = server.get_player_by_uuid(uuid) {
            return Some(player.gameprofile.clone());
        }

        if let Some(profile) = resolve_known_profile_by_uuid(server, uuid).await {
            return Some(profile);
        }

        return None;
    }

    if let Some(profile) = resolve_known_profile_by_name(server, value).await {
        return Some(profile);
    }

    if !server.basic_config.online_mode {
        if let Ok(uuid) = offline_uuid(value) {
            return Some(GameProfile {
                id: uuid,
                name: value.to_string(),
                properties: vec![],
                profile_actions: None,
            });
        }
    }

    None
}

async fn resolve_known_profile_by_name(server: &Server, name: &str) -> Option<GameProfile> {
    {
        let ops = server.data.operator_config.read().await;
        if let Some(op) = ops.ops.iter().find(|op| op.name.eq_ignore_ascii_case(name)) {
            return Some(GameProfile {
                id: op.uuid,
                name: op.name.clone(),
                properties: vec![],
                profile_actions: None,
            });
        }
    }

    {
        let banned_players = server.data.banned_player_list.read().await;
        if let Some(entry) = banned_players
            .banned_players
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
        {
            return Some(GameProfile {
                id: entry.uuid,
                name: entry.name.clone(),
                properties: vec![],
                profile_actions: None,
            });
        }
    }

    {
        let whitelist = server.data.whitelist_config.read().await;
        if let Some(entry) = whitelist
            .whitelist
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
        {
            return Some(GameProfile {
                id: entry.uuid,
                name: entry.name.clone(),
                properties: vec![],
                profile_actions: None,
            });
        }
    }

    None
}

async fn resolve_known_profile_by_uuid(server: &Server, uuid: Uuid) -> Option<GameProfile> {
    {
        let ops = server.data.operator_config.read().await;
        if let Some(op) = ops.ops.iter().find(|op| op.uuid == uuid) {
            return Some(GameProfile {
                id: op.uuid,
                name: op.name.clone(),
                properties: vec![],
                profile_actions: None,
            });
        }
    }

    {
        let banned_players = server.data.banned_player_list.read().await;
        if let Some(entry) = banned_players
            .banned_players
            .iter()
            .find(|entry| entry.uuid == uuid)
        {
            return Some(GameProfile {
                id: entry.uuid,
                name: entry.name.clone(),
                properties: vec![],
                profile_actions: None,
            });
        }
    }

    {
        let whitelist = server.data.whitelist_config.read().await;
        if let Some(entry) = whitelist.whitelist.iter().find(|entry| entry.uuid == uuid) {
            return Some(GameProfile {
                id: entry.uuid,
                name: entry.name.clone(),
                properties: vec![],
                profile_actions: None,
            });
        }
    }

    None
}
