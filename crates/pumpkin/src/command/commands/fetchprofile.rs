use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::uuid::UuidArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::net::authentication::{fetch_profile_by_uuid, lookup_profile_by_name};
use crate::net::{GameProfile, offline_uuid};
use crate::server::Server;
use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::borrow::Cow;
use std::sync::Arc;
use uuid::Uuid;

const DESCRIPTION: &str = "Fetches a player's profile.";
const PERMISSION: &str = "minecraft:command.fetchprofile";

const ARG_NAME: &str = "name";
const ARG_ID: &str = "id";
const ARG_ENTITY: &str = "entity";

pub const NO_PROFILE_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_FETCHPROFILE_NO_PROFILE,
    translation::java::COMMANDS_FETCHPROFILE_NO_PROFILE,
);

const fn uuid_to_int_array(uuid: &Uuid) -> [i32; 4] {
    let (most, least) = uuid.as_u64_pair();
    [
        (most >> 32) as i32,
        most as i32,
        (least >> 32) as i32,
        least as i32,
    ]
}

fn game_profile_to_nbt(profile: &GameProfile) -> NbtCompound {
    let mut compound = NbtCompound::new();
    let int_array = uuid_to_int_array(&profile.id);
    compound.put("id", NbtTag::IntArray(int_array.to_vec()));
    if !profile.name.is_empty() {
        compound.put_string("name", profile.name.clone());
    }
    let properties = profile.properties.load();
    if !properties.is_empty() {
        let mut prop_list = Vec::new();
        for prop in properties.iter() {
            let mut prop_compound = NbtCompound::new();
            prop_compound.put_string("name", prop.name.to_string());
            prop_compound.put_string("value", prop.value.to_string());
            if let Some(ref sig) = prop.signature {
                prop_compound.put_string("signature", sig.to_string());
            }
            prop_list.push(NbtTag::Compound(prop_compound));
        }
        compound.put_list("properties", prop_list);
    }
    compound
}

fn format_clickable_list(items: Vec<TextComponent>) -> TextComponent {
    let mut root = TextComponent::empty();
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            root = root.add_child(TextComponent::text(" "));
        }
        let styled = item.color_named(NamedColor::Green);
        let wrapped = styled.wrap_in_square_brackets();
        root = root.add_child(wrapped);
    }
    root
}

fn report_resolved_profile(
    source: &CommandSource,
    profile: &GameProfile,
    message_id: &'static str,
    argument: TextComponent,
) {
    let encoded_profile_compound = game_profile_to_nbt(profile);
    let encoded_profile_as_string = encoded_profile_compound.to_string();

    let head_component = TextComponent::player_sprite(encoded_profile_compound, true);
    let encoded_component_as_string = head_component
        .0
        .clone()
        .to_translated()
        .to_nbt_compound()
        .to_string();

    let clickable = format_clickable_list(vec![
        TextComponent::translate_cross(
            translation::java::COMMANDS_FETCHPROFILE_COPY_COMPONENT,
            translation::java::COMMANDS_FETCHPROFILE_COPY_COMPONENT,
            [],
        )
        .click_event(ClickEvent::CopyToClipboard {
            value: Cow::from(encoded_profile_as_string.clone()),
        }),
        TextComponent::translate_cross(
            translation::java::COMMANDS_FETCHPROFILE_GIVE_ITEM,
            translation::java::COMMANDS_FETCHPROFILE_GIVE_ITEM,
            [],
        )
        .click_event(ClickEvent::RunCommand {
            command: Cow::from(format!(
                "give @s minecraft:player_head[profile={encoded_profile_as_string}]"
            )),
        }),
        TextComponent::translate_cross(
            translation::java::COMMANDS_FETCHPROFILE_SUMMON_MANNEQUIN,
            translation::java::COMMANDS_FETCHPROFILE_SUMMON_MANNEQUIN,
            [],
        )
        .click_event(ClickEvent::RunCommand {
            command: Cow::from(format!(
                "summon minecraft:mannequin ~ ~ ~ {{profile:{encoded_profile_as_string}}}"
            )),
        }),
        TextComponent::translate_cross(
            translation::java::COMMANDS_FETCHPROFILE_COPY_TEXT,
            translation::java::COMMANDS_FETCHPROFILE_COPY_TEXT,
            [head_component.color_named(NamedColor::White)],
        )
        .click_event(ClickEvent::CopyToClipboard {
            value: Cow::from(encoded_component_as_string),
        }),
    ]);

    let msg = TextComponent::translate_cross(message_id, message_id, [argument, clickable]);

    source.send_feedback(msg, false);
}

async fn fetch_profile_by_name_helper(server: &Server, name: &str) -> Option<GameProfile> {
    if let Some(player) = server.get_player_by_name(name) {
        return Some(player.gameprofile.clone());
    }

    let cached_entry = server.data.user_cache.write().unwrap().get_by_name(name);

    let auth_config = server
        .advanced_config
        .networking
        .java
        .authentication
        .clone();
    let name_string = name.to_string();

    let mojang_res =
        tokio::task::spawn_blocking(move || lookup_profile_by_name(&name_string, &auth_config))
            .await
            .ok()
            .and_then(Result::ok)
            .flatten();

    if let Some((uuid, resolved_name)) = mojang_res {
        server
            .data
            .user_cache
            .write()
            .unwrap()
            .upsert(uuid, resolved_name.clone());
        let auth_config_clone = server
            .advanced_config
            .networking
            .java
            .authentication
            .clone();
        let full_profile =
            tokio::task::spawn_blocking(move || fetch_profile_by_uuid(uuid, &auth_config_clone))
                .await
                .ok()
                .and_then(Result::ok)
                .flatten();

        return Some(full_profile.unwrap_or_else(|| GameProfile {
            id: uuid,
            name: resolved_name,
            properties: arc_swap::ArcSwap::new(Arc::new(vec![])),
            profile_actions: None,
        }));
    }

    if let Some(entry) = cached_entry {
        return Some(GameProfile {
            id: entry.uuid,
            name: entry.name,
            properties: arc_swap::ArcSwap::new(Arc::new(vec![])),
            profile_actions: None,
        });
    }

    if !server.advanced_config.networking.java.online_mode
        && let Ok(uuid) = offline_uuid(name)
    {
        let profile = GameProfile {
            id: uuid,
            name: name.to_string(),
            properties: arc_swap::ArcSwap::new(Arc::new(vec![])),
            profile_actions: None,
        };
        server
            .data
            .user_cache
            .write()
            .unwrap()
            .upsert(uuid, name.to_string());
        return Some(profile);
    }

    None
}

async fn fetch_profile_by_id_helper(server: &Server, id: Uuid) -> Option<GameProfile> {
    if let Some(player) = server.get_player_by_uuid(id) {
        return Some(player.gameprofile.clone());
    }

    let auth_config = server
        .advanced_config
        .networking
        .java
        .authentication
        .clone();
    let mojang_res = tokio::task::spawn_blocking(move || fetch_profile_by_uuid(id, &auth_config))
        .await
        .ok()
        .and_then(Result::ok)
        .flatten();

    if let Some(profile) = mojang_res {
        server
            .data
            .user_cache
            .write()
            .unwrap()
            .upsert(profile.id, profile.name.clone());
        return Some(profile);
    }

    let cached_entry = server.data.user_cache.write().unwrap().get_by_uuid(id);
    if let Some(entry) = cached_entry {
        return Some(GameProfile {
            id: entry.uuid,
            name: entry.name,
            properties: arc_swap::ArcSwap::new(Arc::new(vec![])),
            profile_actions: None,
        });
    }

    None
}

struct ResolveNameExecutor;

impl CommandExecutor for ResolveNameExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name = StringArgumentType::get(context, ARG_NAME)?;
        let server = context.server();
        let name_owned = name.to_string();

        let name_component = TextComponent::text(name_owned.clone());
        let result = futures::executor::block_on(fetch_profile_by_name_helper(server, &name_owned));
        match result {
            Some(profile) => {
                report_resolved_profile(
                    &context.source,
                    &profile,
                    translation::java::COMMANDS_FETCHPROFILE_NAME_SUCCESS,
                    name_component,
                );
            }
            None => {
                context.source.send_error(TextComponent::translate_cross(
                    translation::java::COMMANDS_FETCHPROFILE_NAME_FAILURE,
                    translation::java::COMMANDS_FETCHPROFILE_NAME_FAILURE,
                    [name_component],
                ));
            }
        }

        Ok(1)
    }
}

struct ResolveIdExecutor;

impl CommandExecutor for ResolveIdExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let id = UuidArgumentType::get(context, ARG_ID)?;
        let server = context.server();

        let id_component = TextComponent::text(id.to_string());
        let result = futures::executor::block_on(fetch_profile_by_id_helper(server, id));
        match result {
            Some(profile) => {
                report_resolved_profile(
                    &context.source,
                    &profile,
                    translation::java::COMMANDS_FETCHPROFILE_ID_SUCCESS,
                    id_component,
                );
            }
            None => {
                context.source.send_error(TextComponent::translate_cross(
                    translation::java::COMMANDS_FETCHPROFILE_ID_FAILURE,
                    translation::java::COMMANDS_FETCHPROFILE_ID_FAILURE,
                    [id_component],
                ));
            }
        }

        Ok(1)
    }
}

struct PrintForEntityExecutor;

impl CommandExecutor for PrintForEntityExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let entity = EntityArgumentType::get_entity(context, ARG_ENTITY)?;

        entity.get_player().map_or_else(
            || Err(NO_PROFILE_ERROR_TYPE.create_without_context(entity.get_display_name())),
            |player| {
                report_resolved_profile(
                    &context.source,
                    &player.gameprofile,
                    translation::java::COMMANDS_FETCHPROFILE_ENTITY_SUCCESS,
                    player.get_display_name(),
                );
                Ok(1)
            },
        )
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("fetchprofile", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("name").then(
                argument(ARG_NAME, StringArgumentType::GreedyPhrase).executes(ResolveNameExecutor),
            ))
            .then(
                literal("id").then(argument(ARG_ID, UuidArgumentType).executes(ResolveIdExecutor)),
            )
            .then(literal("entity").then(
                argument(ARG_ENTITY, EntityArgumentType::Entity).executes(PrintForEntityExecutor),
            )),
    );
}
