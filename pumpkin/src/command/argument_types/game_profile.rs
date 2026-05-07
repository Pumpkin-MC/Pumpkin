use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::entity::ONLY_PLAYERS_ALLOWED_ERROR_TYPE;
use crate::command::argument_types::entity_selector::EntitySelector;
use crate::command::argument_types::entity_selector::parser::EntitySelectorParser;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::net::authentication::lookup_profile_by_name;
use crate::net::{GameProfile, offline_uuid};
use crate::server::Server;
use pumpkin_data::translation;
use uuid::Uuid;

pub const UNKNOWN_PLAYER_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_PLAYER_UNKNOWN,
    translation::java::ARGUMENT_PLAYER_UNKNOWN,
);

/// A result from the [`GameProfileArgumentType`], which can be resolved into
/// one or more [`GameProfile`]s, successfully or not.
pub enum GameProfileResult {
    Selector(Box<EntitySelector>),
    Name(String),
    Uuid(Uuid),
}

impl GameProfileResult {
    fn unknown_player_syntax_error() -> CommandSyntaxError {
        UNKNOWN_PLAYER_ERROR_TYPE.create_without_context()
    }

    /// Resolves this result with the help of a [`CommandSource`].
    ///
    /// # Warning
    ///
    /// Do not lock write/read access to one of these data locks
    /// before calling this method, as that may cause a *deadlock*:
    /// - `server.data.user_cache`
    /// - `server.data.operator_config`
    /// - `server.data.banned_player_list`
    /// - `server.data.whitelist_config`
    ///
    /// Instead, call this method *before* using `write()`/`read()` on a lock.
    pub async fn resolve(
        &self,
        source: &CommandSource,
    ) -> Result<Vec<GameProfile>, CommandSyntaxError> {
        let players = match self {
            Self::Selector(selector) => selector.find_players(source).await,
            Self::Name(name) => {
                let server = source.server();
                if let Some(player) = server.get_player_by_name(name) {
                    return Ok(vec![player.gameprofile.clone()]);
                }

                let cached_entry = server.data.user_cache.write().await.get_by_name(name);
                if let Some(entry) = cached_entry {
                    return Ok(vec![Self::profile_from_uuid_name(entry.uuid, entry.name)]);
                }

                if let Some(profile) = Self::resolve_known_profile_by_name(server, name).await {
                    return Ok(vec![profile]);
                }

                if server.basic_config.online_mode {
                    return match lookup_profile_by_name(
                        name,
                        &server.advanced_config.networking.authentication,
                    ) {
                        Ok(Some((uuid, resolved_name))) => {
                            server
                                .data
                                .user_cache
                                .write()
                                .await
                                .upsert(uuid, resolved_name.clone());
                            Ok(vec![Self::profile_from_uuid_name(uuid, resolved_name)])
                        }
                        _ => Err(Self::unknown_player_syntax_error()),
                    };
                } else if let Ok(uuid) = offline_uuid(name) {
                    let profile = Self::profile_from_uuid_name(uuid, name.clone());
                    server
                        .data
                        .user_cache
                        .write()
                        .await
                        .upsert(profile.id, profile.name.clone());
                    return Ok(vec![profile]);
                }

                return Err(Self::unknown_player_syntax_error());
            }
            Self::Uuid(uuid) => {
                let server = source.server();

                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    return Ok(vec![player.gameprofile.clone()]);
                }

                let cached_entry = server.data.user_cache.write().await.get_by_uuid(*uuid);
                if let Some(entry) = cached_entry {
                    return Ok(vec![Self::profile_from_uuid_name(entry.uuid, entry.name)]);
                }

                if let Some(profile) = Self::resolve_known_profile_by_uuid(server, *uuid).await {
                    return Ok(vec![profile]);
                }

                return Err(Self::unknown_player_syntax_error());
            }
        }?;

        Ok(players.iter().map(|p| &p.gameprofile).cloned().collect())
    }

    async fn resolve_known_profile_by_name(server: &Server, name: &str) -> Option<GameProfile> {
        let ops = server.data.operator_config.read().await;
        if let Some(op) = ops.ops.iter().find(|op| op.name.eq_ignore_ascii_case(name)) {
            return Some(Self::profile_from_uuid_name(op.uuid, op.name.clone()));
        }

        let banned_players = server.data.banned_player_list.read().await;
        if let Some(entry) = banned_players
            .banned_players
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
        {
            return Some(Self::profile_from_uuid_name(entry.uuid, entry.name.clone()));
        }

        let whitelist = server.data.whitelist_config.read().await;
        if let Some(entry) = whitelist
            .whitelist
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
        {
            return Some(Self::profile_from_uuid_name(entry.uuid, entry.name.clone()));
        }

        None
    }

    async fn resolve_known_profile_by_uuid(server: &Server, uuid: Uuid) -> Option<GameProfile> {
        let ops = server.data.operator_config.read().await;
        if let Some(op) = ops.ops.iter().find(|op| op.uuid == uuid) {
            return Some(Self::profile_from_uuid_name(op.uuid, op.name.clone()));
        }

        let banned_players = server.data.banned_player_list.read().await;
        if let Some(entry) = banned_players
            .banned_players
            .iter()
            .find(|entry| entry.uuid == uuid)
        {
            return Some(Self::profile_from_uuid_name(entry.uuid, entry.name.clone()));
        }

        let whitelist = server.data.whitelist_config.read().await;
        if let Some(entry) = whitelist.whitelist.iter().find(|entry| entry.uuid == uuid) {
            return Some(Self::profile_from_uuid_name(entry.uuid, entry.name.clone()));
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
}

/// An argument type to parse one or more [`GameProfile`]s.
///
/// Use [`GameProfileArgumentType::get`] to automatically get a `Vec` of
/// [`GameProfile`]s for an argument by providing a [`CommandContext`] and the
/// argument's name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GameProfileArgumentType;

impl ArgumentType for GameProfileArgumentType {
    type Item = GameProfileResult;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        Self::parse_with_allow_selectors(reader, true)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::GameProfile
    }

    fn examples(&self) -> Vec<String> {
        examples!("Herobrine", "98765", "@a", "@p[limit=2]")
    }
}

impl GameProfileArgumentType {
    fn parse_with_allow_selectors(
        reader: &mut StringReader,
        allow_selectors: bool,
    ) -> Result<<Self as ArgumentType>::Item, CommandSyntaxError> {
        if reader.peek() == Some('@') {
            // We read a selector variable.
            let parser = EntitySelectorParser::new(reader, allow_selectors);
            let selector = parser.parse()?;
            if selector.includes_entities {
                Err(ONLY_PLAYERS_ALLOWED_ERROR_TYPE.create(reader))
            } else {
                Ok(GameProfileResult::Selector(Box::new(selector)))
            }
        } else {
            // We read a UUID or player name.
            let i = reader.cursor();
            while reader.can_read_char() && reader.peek() != Some(' ') {
                reader.skip();
            }
            let string = &reader.string()[i..reader.cursor()];
            Ok(Uuid::try_parse(string).map_or_else(
                |_| GameProfileResult::Name(string.to_owned()),
                GameProfileResult::Uuid,
            ))
        }
    }

    /// Tries to get any number of [`GameProfile`]s from a parsed argument of the provided [`CommandContext`].
    ///
    /// # Warning
    ///
    /// Do not lock write/read access to one of these data locks
    /// before calling this method, as that may cause a *deadlock*:
    /// - `server.data.user_cache`
    /// - `server.data.operator_config`
    /// - `server.data.banned_player_list`
    /// - `server.data.whitelist_config`
    ///
    /// Instead, call this function *before* using `write()`/`read()` on a lock.
    pub async fn get(
        context: &CommandContext<'_>,
        name: &str,
    ) -> Result<Vec<GameProfile>, CommandSyntaxError> {
        context
            .get_argument::<GameProfileResult>(name)?
            .resolve(context.source.as_ref())
            .await
    }
}
