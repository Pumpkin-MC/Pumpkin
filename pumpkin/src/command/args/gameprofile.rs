use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::text::TextComponent;
use uuid::Uuid;

use crate::command::errors::command_syntax_error::{CommandSyntaxError, CommandSyntaxErrorContext};
use crate::command::errors::error_types;
use crate::{
    command::{
        CommandSender,
        args::{ConsumeResult, ConsumeResultWithSyntax, SuggestResult},
        dispatcher::CommandError,
        tree::{RawArg, RawArgs},
    },
    net::authentication::lookup_profile_by_name,
    net::{GameProfile, offline_uuid},
    server::Server,
};

use super::entities::{ensure_player_only_selector, parse_target_selector_with_context};
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};
use crate::command::args::ArgumentConsumer;

#[derive(Clone, Copy)]
pub enum GameProfileSuggestionMode {
    OnlinePlayers,
    NonOpOnlinePlayers,
    OpNames,
    BannedNames,
    NonWhitelistedOnlinePlayers,
    WhitelistedNames,
}

pub struct GameProfilesArgumentConsumer {
    suggestion_mode: GameProfileSuggestionMode,
    suggest_selectors: bool,
}

impl GameProfilesArgumentConsumer {
    #[must_use]
    pub const fn new(suggestion_mode: GameProfileSuggestionMode, suggest_selectors: bool) -> Self {
        Self {
            suggestion_mode,
            suggest_selectors,
        }
    }

    #[must_use]
    pub const fn online_players_with_selectors() -> Self {
        Self::new(GameProfileSuggestionMode::OnlinePlayers, true)
    }
}

impl Default for GameProfilesArgumentConsumer {
    fn default() -> Self {
        Self::online_players_with_selectors()
    }
}

impl GetClientSideArgParser for GameProfilesArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::GameProfile
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for GameProfilesArgumentConsumer {
    fn consume<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let Some(raw_arg) = args.pop() else {
            return Box::pin(async { None });
        };

        Box::pin(async move {
            resolve_profiles_from_token(sender, server, raw_arg)
                .await
                .ok()
                .map(Arg::GameProfiles)
        })
    }

    fn consume_with_syntax<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResultWithSyntax<'a> {
        let Some(raw_arg) = args.pop() else {
            return Box::pin(async { Ok(None) });
        };

        Box::pin(async move {
            let resolved = resolve_profiles_from_token(sender, server, raw_arg).await?;
            Ok(Some(Arg::GameProfiles(resolved)))
        })
    }

    fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        server: &'a Server,
        _input: &'a str,
    ) -> SuggestResult<'a> {
        Box::pin(async move {
            let mut suggestions = Vec::new();
            if self.suggest_selectors {
                suggestions.extend(selector_suggestions());
            }

            let mut names = Vec::new();
            match self.suggestion_mode {
                GameProfileSuggestionMode::OnlinePlayers => {
                    for player in server.get_all_players() {
                        push_name_if_missing(&mut names, player.gameprofile.name.clone());
                    }
                }
                GameProfileSuggestionMode::NonOpOnlinePlayers => {
                    let ops = server.data.operator_config.read().await;
                    for player in server.get_all_players() {
                        if ops.ops.iter().all(|op| op.uuid != player.gameprofile.id) {
                            push_name_if_missing(&mut names, player.gameprofile.name.clone());
                        }
                    }
                }
                GameProfileSuggestionMode::OpNames => {
                    let ops = server.data.operator_config.read().await;
                    for op in &ops.ops {
                        push_name_if_missing(&mut names, op.name.clone());
                    }
                }
                GameProfileSuggestionMode::BannedNames => {
                    let banned = server.data.banned_player_list.read().await;
                    for entry in &banned.banned_players {
                        push_name_if_missing(&mut names, entry.name.clone());
                    }
                }
                GameProfileSuggestionMode::NonWhitelistedOnlinePlayers => {
                    let whitelist = server.data.whitelist_config.read().await;
                    for player in server.get_all_players() {
                        if !whitelist.is_whitelisted(&player.gameprofile) {
                            push_name_if_missing(&mut names, player.gameprofile.name.clone());
                        }
                    }
                }
                GameProfileSuggestionMode::WhitelistedNames => {
                    let whitelist = server.data.whitelist_config.read().await;
                    for entry in &whitelist.whitelist {
                        push_name_if_missing(&mut names, entry.name.clone());
                    }
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

async fn resolve_profiles_from_token(
    sender: &CommandSender,
    server: &Server,
    raw_arg: RawArg<'_>,
) -> Result<Vec<GameProfile>, CommandSyntaxError> {
    if raw_arg.value.starts_with('@') {
        let selector = parse_target_selector_with_context(raw_arg)?;
        ensure_player_only_selector(&selector, raw_arg)?;

        let players = server.select_players(&selector, Some(sender));
        if players.is_empty() {
            return Err(syntax_player_unknown(raw_arg));
        }

        return Ok(players
            .into_iter()
            .map(|player| player.gameprofile.clone())
            .collect());
    }

    if let Ok(uuid) = Uuid::parse_str(raw_arg.value) {
        if let Some(player) = server.get_player_by_uuid(uuid) {
            return Ok(vec![player.gameprofile.clone()]);
        }

        let cached_entry = server.data.user_cache.write().await.get_by_uuid(uuid);
        if let Some(entry) = cached_entry {
            return Ok(vec![profile_from_uuid_name(entry.uuid, entry.name)]);
        }

        if let Some(profile) = resolve_known_profile_by_uuid(server, uuid).await {
            return Ok(vec![profile]);
        }

        return Err(syntax_player_unknown(raw_arg));
    }

    if let Some(player) = server.get_player_by_name(raw_arg.value) {
        return Ok(vec![player.gameprofile.clone()]);
    }

    let cached_entry = server
        .data
        .user_cache
        .write()
        .await
        .get_by_name(raw_arg.value);
    if let Some(entry) = cached_entry {
        return Ok(vec![profile_from_uuid_name(entry.uuid, entry.name)]);
    }

    if let Some(profile) = resolve_known_profile_by_name(server, raw_arg.value).await {
        return Ok(vec![profile]);
    }

    if server.basic_config.online_mode {
        match lookup_profile_by_name(
            raw_arg.value,
            &server.advanced_config.networking.authentication,
        ) {
            Ok(Some((uuid, resolved_name))) => {
                server
                    .data
                    .user_cache
                    .write()
                    .await
                    .upsert(uuid, resolved_name.clone());
                return Ok(vec![profile_from_uuid_name(uuid, resolved_name)]);
            }
            Ok(None) | Err(_) => return Err(syntax_player_unknown(raw_arg)),
        }
    }

    if let Ok(uuid) = offline_uuid(raw_arg.value) {
        let profile = profile_from_uuid_name(uuid, raw_arg.value.to_string());
        server
            .data
            .user_cache
            .write()
            .await
            .upsert(profile.id, profile.name.clone());
        return Ok(vec![profile]);
    }

    Err(syntax_player_unknown(raw_arg))
}

async fn resolve_known_profile_by_name(server: &Server, name: &str) -> Option<GameProfile> {
    {
        let ops = server.data.operator_config.read().await;
        if let Some(op) = ops.ops.iter().find(|op| op.name.eq_ignore_ascii_case(name)) {
            return Some(profile_from_uuid_name(op.uuid, op.name.clone()));
        }
    }

    {
        let banned_players = server.data.banned_player_list.read().await;
        if let Some(entry) = banned_players
            .banned_players
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
        {
            return Some(profile_from_uuid_name(entry.uuid, entry.name.clone()));
        }
    }

    {
        let whitelist = server.data.whitelist_config.read().await;
        if let Some(entry) = whitelist
            .whitelist
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
        {
            return Some(profile_from_uuid_name(entry.uuid, entry.name.clone()));
        }
    }

    None
}

async fn resolve_known_profile_by_uuid(server: &Server, uuid: Uuid) -> Option<GameProfile> {
    {
        let ops = server.data.operator_config.read().await;
        if let Some(op) = ops.ops.iter().find(|op| op.uuid == uuid) {
            return Some(profile_from_uuid_name(op.uuid, op.name.clone()));
        }
    }

    {
        let banned_players = server.data.banned_player_list.read().await;
        if let Some(entry) = banned_players
            .banned_players
            .iter()
            .find(|entry| entry.uuid == uuid)
        {
            return Some(profile_from_uuid_name(entry.uuid, entry.name.clone()));
        }
    }

    {
        let whitelist = server.data.whitelist_config.read().await;
        if let Some(entry) = whitelist.whitelist.iter().find(|entry| entry.uuid == uuid) {
            return Some(profile_from_uuid_name(entry.uuid, entry.name.clone()));
        }
    }

    None
}

#[allow(clippy::missing_const_for_fn)]
fn profile_from_uuid_name(uuid: Uuid, name: String) -> GameProfile {
    GameProfile {
        id: uuid,
        name,
        properties: vec![],
        profile_actions: None,
    }
}

fn push_name_if_missing(names: &mut Vec<String>, name: String) {
    if names
        .iter()
        .any(|known_name| known_name.eq_ignore_ascii_case(&name))
    {
        return;
    }
    names.push(name);
}

fn selector_suggestions() -> Vec<CommandSuggestion> {
    vec![
        CommandSuggestion::new("@s".to_string(), None),
        CommandSuggestion::new("@p".to_string(), None),
        CommandSuggestion::new("@r".to_string(), None),
        CommandSuggestion::new("@a".to_string(), None),
        CommandSuggestion::new("@e".to_string(), None),
        CommandSuggestion::new("@n".to_string(), None),
    ]
}

fn syntax_player_unknown(raw_arg: RawArg<'_>) -> CommandSyntaxError {
    syntax_error_for_arg_with_cursor(
        raw_arg,
        TextComponent::translate(translation::ARGUMENT_PLAYER_UNKNOWN, []),
        0,
    )
}

fn syntax_error_for_arg_with_cursor(
    raw_arg: RawArg<'_>,
    message: TextComponent,
    local_cursor: usize,
) -> CommandSyntaxError {
    let mut clamped_local_cursor = local_cursor.min(raw_arg.value.len());
    while clamped_local_cursor > 0 && !raw_arg.value.is_char_boundary(clamped_local_cursor) {
        clamped_local_cursor -= 1;
    }

    CommandSyntaxError {
        error_type: &error_types::DISPATCHER_UNKNOWN_ARGUMENT,
        message,
        context: Some(CommandSyntaxErrorContext {
            input: raw_arg.input.to_string(),
            cursor: raw_arg.start + clamped_local_cursor,
        }),
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::translation;
    use pumpkin_util::text::TextContent;

    use super::syntax_player_unknown;
    use crate::command::tree::RawArg;

    #[test]
    fn unknown_player_error_uses_translation_and_arg_start_cursor() {
        let input = "ban missing_player";
        let raw_arg = RawArg {
            value: "missing_player",
            start: 4,
            end: input.len(),
            input,
        };

        let error = syntax_player_unknown(raw_arg);
        let translate_key = match error.message.0.content.as_ref() {
            TextContent::Translate { translate, .. } => translate.as_ref(),
            _ => "",
        };
        assert_eq!(translate_key, translation::ARGUMENT_PLAYER_UNKNOWN);
        assert_eq!(error.context.unwrap().cursor, 4);
    }
}
