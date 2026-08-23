use std::fs;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use tracing::error;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::server::Server;

const DESCRIPTION: &str = "Controls loaded data packs.";
const PERMISSION: &str = "minecraft:command.datapack";

static ERROR_UNKNOWN_DATAPACK: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_UNKNOWN,
    translation::java::COMMANDS_DATAPACK_UNKNOWN,
);

static ERROR_ENABLE_FAILED: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_ENABLE_FAILED,
    translation::java::COMMANDS_DATAPACK_ENABLE_FAILED,
);

static ERROR_DISABLE_FAILED: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_DISABLE_FAILED,
    translation::java::COMMANDS_DATAPACK_DISABLE_FAILED,
);

static ERROR_CREATE_ALREADY_EXISTS: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_CREATE_ALREADY_EXISTS,
    translation::java::COMMANDS_DATAPACK_CREATE_ALREADY_EXISTS,
);

static ERROR_CREATE_INVALID_NAME: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_CREATE_INVALID_NAME,
    translation::java::COMMANDS_DATAPACK_CREATE_INVALID_NAME,
);

static ERROR_CREATE_IO_FAILURE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_CREATE_IO_FAILURE,
    translation::java::COMMANDS_DATAPACK_CREATE_IO_FAILURE,
);

fn get_all_known_packs(server: &Server) -> Vec<String> {
    let mut packs = Vec::new();
    packs.push("vanilla".to_string());

    // Bundled feature packs
    for bundled in [
        "trade_rebalance",
        "minecart_improvements",
        "redstone_experiments",
    ] {
        if !packs.iter().any(|p| p == bundled) {
            packs.push(bundled.to_string());
        }
    }

    // World datapacks directory
    let datapacks_dir = server.basic_config.get_world_path().join("datapacks");
    if let Ok(entries) = fs::read_dir(datapacks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with('.') {
                continue;
            }
            if path.is_dir()
                || path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            {
                let pack_name = format!("file/{file_name}");
                if !packs.iter().any(|p| p == &pack_name) {
                    packs.push(pack_name);
                }
            }
        }
    }

    let level_info = server.level_info.load();
    for pack in &level_info.data_packs.enabled {
        if !packs.iter().any(|p| p == pack) {
            packs.push(pack.clone());
        }
    }
    for pack in &level_info.data_packs.disabled {
        if !packs.iter().any(|p| p == pack) {
            packs.push(pack.clone());
        }
    }

    packs
}

fn get_enabled_packs(server: &Server) -> Vec<String> {
    server.level_info.load().data_packs.enabled.clone()
}

fn get_available_packs(server: &Server) -> Vec<String> {
    let enabled = get_enabled_packs(server);
    let all = get_all_known_packs(server);
    all.into_iter().filter(|p| !enabled.contains(p)).collect()
}

fn find_pack_name(server: &Server, input: &str) -> Option<String> {
    let known = get_all_known_packs(server);
    if let Some(p) = known.iter().find(|p| *p == input) {
        return Some(p.clone());
    }
    let file_input = format!("file/{input}");
    if let Some(p) = known.iter().find(|p| **p == file_input) {
        return Some(p.clone());
    }
    if let Some(p) = known
        .iter()
        .find(|p| p.strip_prefix("file/") == Some(input))
    {
        return Some(p.clone());
    }
    None
}

fn format_pack(name: &str) -> TextComponent {
    TextComponent::text(format!("[{name}]")).color_named(NamedColor::Green)
}

struct AvailablePackSuggestionProvider;

impl SuggestionProvider for AvailablePackSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        context: &'a CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult<'a> {
        Box::pin(async move {
            let server = context.server();
            for pack in get_available_packs(server) {
                if pack.contains(' ') || pack.contains('/') {
                    builder = builder.suggest(format!("\"{pack}\""));
                } else {
                    builder = builder.suggest(pack.clone());
                }
                if let Some(short) = pack.strip_prefix("file/") {
                    if short.contains(' ') {
                        builder = builder.suggest(format!("\"{short}\""));
                    } else {
                        builder = builder.suggest(short.to_string());
                    }
                }
            }
            builder.build()
        })
    }
}

struct EnabledPackSuggestionProvider;

impl SuggestionProvider for EnabledPackSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        context: &'a CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult<'a> {
        Box::pin(async move {
            let server = context.server();
            for pack in get_enabled_packs(server) {
                if pack.contains(' ') || pack.contains('/') {
                    builder = builder.suggest(format!("\"{pack}\""));
                } else {
                    builder = builder.suggest(pack.clone());
                }
                if let Some(short) = pack.strip_prefix("file/") {
                    if short.contains(' ') {
                        builder = builder.suggest(format!("\"{short}\""));
                    } else {
                        builder = builder.suggest(short.to_string());
                    }
                }
            }
            builder.build()
        })
    }
}

#[derive(Clone, Copy)]
enum ListFilter {
    All,
    Available,
    Enabled,
}

struct DatapackListExecutor(ListFilter);

impl CommandExecutor for DatapackListExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let enabled = get_enabled_packs(server);
            let available = get_available_packs(server);

            match self.0 {
                ListFilter::All => {
                    send_enabled_list(context, &enabled).await;
                    send_available_list(context, &available).await;
                    Ok(enabled.len() as i32)
                }
                ListFilter::Enabled => {
                    send_enabled_list(context, &enabled).await;
                    Ok(enabled.len() as i32)
                }
                ListFilter::Available => {
                    send_available_list(context, &available).await;
                    Ok(available.len() as i32)
                }
            }
        })
    }
}

async fn send_enabled_list(context: &CommandContext<'_>, enabled: &[String]) {
    if enabled.is_empty() {
        context
            .source
            .send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_DATAPACK_LIST_ENABLED_NONE,
                    translation::java::COMMANDS_DATAPACK_LIST_ENABLED_NONE,
                    [],
                ),
                false,
            )
            .await;
    } else {
        let packs_component =
            TextComponent::join_with_comma(enabled.iter().map(|p| format_pack(p)).collect());
        context
            .source
            .send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_DATAPACK_LIST_ENABLED_SUCCESS,
                    translation::java::COMMANDS_DATAPACK_LIST_ENABLED_SUCCESS,
                    [
                        TextComponent::text(enabled.len().to_string()),
                        packs_component,
                    ],
                ),
                false,
            )
            .await;
    }
}

async fn send_available_list(context: &CommandContext<'_>, available: &[String]) {
    if available.is_empty() {
        context
            .source
            .send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_NONE,
                    translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_NONE,
                    [],
                ),
                false,
            )
            .await;
    } else {
        let packs_component =
            TextComponent::join_with_comma(available.iter().map(|p| format_pack(p)).collect());
        context
            .source
            .send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_SUCCESS,
                    translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_SUCCESS,
                    [
                        TextComponent::text(available.len().to_string()),
                        packs_component,
                    ],
                ),
                false,
            )
            .await;
    }
}

#[derive(Clone, Copy)]
enum EnablePosition {
    First,
    Last,
}

struct DatapackEnableExecutor(EnablePosition);

impl CommandExecutor for DatapackEnableExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let name_str = StringArgumentType::get(context, "name")?;
            let server = context.server();

            let Some(resolved_name) = find_pack_name(server, name_str) else {
                return Err(ERROR_UNKNOWN_DATAPACK
                    .create_without_context(TextComponent::text(name_str.to_string())));
            };

            let enabled = get_enabled_packs(server);
            if enabled.contains(&resolved_name) {
                return Err(
                    ERROR_ENABLE_FAILED.create_without_context(TextComponent::text(resolved_name))
                );
            }

            let target = resolved_name.clone();
            let pos = self.0;
            server.level_info.rcu(|level_info| {
                let mut new_info = (**level_info).clone();
                new_info.data_packs.disabled.retain(|p| p != &target);
                new_info.data_packs.enabled.retain(|p| p != &target);
                match pos {
                    EnablePosition::First => {
                        new_info.data_packs.enabled.insert(0, target.clone());
                    }
                    EnablePosition::Last => {
                        new_info.data_packs.enabled.push(target.clone());
                    }
                }
                new_info
            });

            if let Err(err) = server.save_world_info() {
                error!("Failed to save world info: {err}");
            }

            server.reload_datapacks(server).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                        translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                        [format_pack(&resolved_name)],
                    ),
                    true,
                )
                .await;

            Ok(server.level_info.load().data_packs.enabled.len() as i32)
        })
    }
}

#[derive(Clone, Copy)]
enum BeforeOrAfter {
    Before,
    After,
}

struct DatapackEnableExistingExecutor(BeforeOrAfter);

impl CommandExecutor for DatapackEnableExistingExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let name_str = StringArgumentType::get(context, "name")?;
            let existing_str = StringArgumentType::get(context, "existing")?;
            let server = context.server();

            let Some(target_pack) = find_pack_name(server, name_str) else {
                return Err(ERROR_UNKNOWN_DATAPACK
                    .create_without_context(TextComponent::text(name_str.to_string())));
            };

            let Some(existing_pack) = find_pack_name(server, existing_str) else {
                return Err(ERROR_UNKNOWN_DATAPACK
                    .create_without_context(TextComponent::text(existing_str.to_string())));
            };

            let enabled = get_enabled_packs(server);
            if enabled.contains(&target_pack) {
                return Err(
                    ERROR_ENABLE_FAILED.create_without_context(TextComponent::text(target_pack))
                );
            }

            if !enabled.contains(&existing_pack) {
                return Err(
                    ERROR_DISABLE_FAILED.create_without_context(TextComponent::text(existing_pack))
                );
            }

            let target = target_pack.clone();
            let existing = existing_pack.clone();
            let before_or_after = self.0;
            server.level_info.rcu(|level_info| {
                let mut new_info = (**level_info).clone();
                new_info.data_packs.disabled.retain(|p| p != &target);
                new_info.data_packs.enabled.retain(|p| p != &target);

                if let Some(idx) = new_info
                    .data_packs
                    .enabled
                    .iter()
                    .position(|p| p == &existing)
                {
                    let insert_pos = match before_or_after {
                        BeforeOrAfter::Before => idx,
                        BeforeOrAfter::After => idx + 1,
                    };
                    new_info
                        .data_packs
                        .enabled
                        .insert(insert_pos, target.clone());
                } else {
                    new_info.data_packs.enabled.push(target.clone());
                }
                new_info
            });

            if let Err(err) = server.save_world_info() {
                error!("Failed to save world info: {err}");
            }

            server.reload_datapacks(server).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                        translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                        [format_pack(&target_pack)],
                    ),
                    true,
                )
                .await;

            Ok(server.level_info.load().data_packs.enabled.len() as i32)
        })
    }
}

struct DatapackDisableExecutor;

impl CommandExecutor for DatapackDisableExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let name_str = StringArgumentType::get(context, "name")?;
            let server = context.server();

            let Some(target_pack) = find_pack_name(server, name_str) else {
                return Err(ERROR_DISABLE_FAILED
                    .create_without_context(TextComponent::text(name_str.to_string())));
            };

            let enabled = get_enabled_packs(server);
            if !enabled.contains(&target_pack) {
                return Err(ERROR_DISABLE_FAILED
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }

            if target_pack == "vanilla" {
                return Err(ERROR_DISABLE_FAILED
                    .create_without_context(TextComponent::text("vanilla".to_string())));
            }

            let target = target_pack.clone();
            server.level_info.rcu(|level_info| {
                let mut new_info = (**level_info).clone();
                new_info.data_packs.enabled.retain(|p| p != &target);
                if !new_info.data_packs.disabled.contains(&target) {
                    new_info.data_packs.disabled.push(target.clone());
                }
                new_info
            });

            if let Err(err) = server.save_world_info() {
                error!("Failed to save world info: {err}");
            }

            server.reload_datapacks(server).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_DATAPACK_MODIFY_DISABLE,
                        translation::java::COMMANDS_DATAPACK_MODIFY_DISABLE,
                        [format_pack(&target_pack)],
                    ),
                    true,
                )
                .await;

            Ok(server.level_info.load().data_packs.enabled.len() as i32)
        })
    }
}

fn is_valid_pack_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

struct DatapackCreateExecutor;

impl CommandExecutor for DatapackCreateExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let name_str = StringArgumentType::get(context, "name")?;
            let description = StringArgumentType::get(context, "description").unwrap_or(name_str);

            if !is_valid_pack_name(name_str) {
                return Err(ERROR_CREATE_INVALID_NAME
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }

            let server = context.server();
            let datapacks_dir = server.basic_config.get_world_path().join("datapacks");
            let pack_dir = datapacks_dir.join(name_str);

            if pack_dir.exists() {
                return Err(ERROR_CREATE_ALREADY_EXISTS
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }

            let data_dir = pack_dir.join("data");
            if let Err(err) = fs::create_dir_all(&data_dir) {
                error!("Failed to create datapack directory: {err}");
                return Err(ERROR_CREATE_IO_FAILURE
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }

            let mcmeta_content = serde_json::json!({
                "pack": {
                    "pack_format": 61,
                    "description": description
                }
            });

            let mcmeta_path = pack_dir.join("pack.mcmeta");
            let mcmeta_str = match serde_json::to_string_pretty(&mcmeta_content) {
                Ok(s) => s,
                Err(err) => {
                    error!("Failed to serialize pack.mcmeta: {err}");
                    return Err(ERROR_CREATE_IO_FAILURE
                        .create_without_context(TextComponent::text(name_str.to_string())));
                }
            };

            if let Err(err) = fs::write(&mcmeta_path, mcmeta_str) {
                error!("Failed to write pack.mcmeta: {err}");
                return Err(ERROR_CREATE_IO_FAILURE
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_DATAPACK_CREATE_SUCCESS,
                        translation::java::COMMANDS_DATAPACK_CREATE_SUCCESS,
                        [TextComponent::text(name_str.to_string())],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let list_builder = literal("list")
        .then(literal("available").executes(DatapackListExecutor(ListFilter::Available)))
        .then(literal("enabled").executes(DatapackListExecutor(ListFilter::Enabled)))
        .executes(DatapackListExecutor(ListFilter::All));

    let enable_builder = literal("enable").then(
        argument("name", StringArgumentType::QuotablePhrase)
            .suggests(AvailablePackSuggestionProvider)
            .then(literal("first").executes(DatapackEnableExecutor(EnablePosition::First)))
            .then(literal("last").executes(DatapackEnableExecutor(EnablePosition::Last)))
            .then(
                literal("before").then(
                    argument("existing", StringArgumentType::QuotablePhrase)
                        .suggests(EnabledPackSuggestionProvider)
                        .executes(DatapackEnableExistingExecutor(BeforeOrAfter::Before)),
                ),
            )
            .then(
                literal("after").then(
                    argument("existing", StringArgumentType::QuotablePhrase)
                        .suggests(EnabledPackSuggestionProvider)
                        .executes(DatapackEnableExistingExecutor(BeforeOrAfter::After)),
                ),
            )
            .executes(DatapackEnableExecutor(EnablePosition::Last)),
    );

    let disable_builder = literal("disable").then(
        argument("name", StringArgumentType::QuotablePhrase)
            .suggests(EnabledPackSuggestionProvider)
            .executes(DatapackDisableExecutor),
    );

    let create_builder = literal("create").then(
        argument("name", StringArgumentType::SingleWord)
            .then(
                argument("description", StringArgumentType::GreedyPhrase)
                    .executes(DatapackCreateExecutor),
            )
            .executes(DatapackCreateExecutor),
    );

    dispatcher.register(
        command("datapack", DESCRIPTION)
            .requires(PERMISSION)
            .then(list_builder)
            .then(enable_builder)
            .then(disable_builder)
            .then(create_builder),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pack_name_check() {
        assert!(is_valid_pack_name("my_pack"));
        assert!(is_valid_pack_name("test-1.0"));
        assert!(is_valid_pack_name("pack123"));

        assert!(!is_valid_pack_name(""));
        assert!(!is_valid_pack_name("../hack"));
        assert!(!is_valid_pack_name("folder/pack"));
        assert!(!is_valid_pack_name("pack\\test"));
        assert!(!is_valid_pack_name("invalid char"));
    }

    #[test]
    fn format_pack_brackets() {
        let formatted = format_pack("vanilla");
        assert!(formatted.to_pretty_console().contains("[vanilla]"));
    }

    #[test]
    fn pack_mcmeta_json_structure() {
        let mcmeta_content = serde_json::json!({
            "pack": {
                "pack_format": 61,
                "description": "Test description"
            }
        });
        let mcmeta_str = serde_json::to_string(&mcmeta_content).unwrap();
        assert!(mcmeta_str.contains("\"pack_format\":61"));
        assert!(mcmeta_str.contains("\"description\":\"Test description\""));
    }
}
