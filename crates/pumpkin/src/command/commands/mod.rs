use crate::command::node::dispatcher::CommandDispatcher;
use pumpkin_config::CommandsConfig;
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionManager, PermissionRegistry},
};
use tracing::{info, warn};

mod advancement;
mod attribute;
mod ban;
mod banip;
mod banlist;
mod bossbar;
mod clear;
mod clone;
mod damage;
mod data;
mod datapack;
mod debug;
pub mod defaultgamemode;
mod deop;
mod dialog;
mod difficulty;
mod effect;
mod enchant;
mod execute;
mod experience;
mod fetchprofile;
mod fill;
mod fillbiome;
mod forceload;
mod function;
mod gamemode;
mod gamerule;
mod give;
mod help;
mod item;
mod kick;
mod kill;
mod list;
mod locate;
mod loot;
mod me;
mod msg;
mod op;
mod pardon;
mod pardonip;
mod particle;
mod place;
mod playsound;
mod plugin;
mod plugins;
mod pumpkin;
mod raid;
mod random;
mod recipe;
mod reload;
mod r#return;
mod ride;
mod rotate;
mod saveall;
mod saveoff;
mod saveon;
mod say;
mod schedule;
mod scoreboard;
mod seed;
mod setblock;
mod setidletimeout;
mod setworldspawn;
mod spawnpoint;
mod spectate;
mod spreadplayers;
mod stop;
mod stopsound;
mod stopwatch;
mod summon;
mod tag;
mod team;
mod teammsg;
mod teleport;
mod tellraw;
mod tick;
mod time;
mod title;
mod tps;
mod transfer;
mod trigger;
mod waypoint;
mod weather;
mod whitelist;
mod worldborder;

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn default_dispatcher(
    permission_manager: &PermissionManager,
    commands_config: &CommandsConfig,
) -> CommandDispatcher {
    let registry = &permission_manager.registry;

    register_permissions(registry);

    let mut dispatcher = CommandDispatcher::new();

    say::register(&mut dispatcher, registry);
    banlist::register(&mut dispatcher, registry);
    difficulty::register(&mut dispatcher, registry);
    debug::register(&mut dispatcher, registry);
    dialog::register(&mut dispatcher, registry);
    execute::register(&mut dispatcher, registry);
    fillbiome::register(&mut dispatcher, registry);
    forceload::register(&mut dispatcher, registry);
    ride::register(&mut dispatcher, registry);
    recipe::register(&mut dispatcher, registry);
    reload::register(&mut dispatcher, registry);
    help::register(&mut dispatcher, registry);
    kill::register(&mut dispatcher, registry);
    op::register(&mut dispatcher, registry);
    place::register(&mut dispatcher, registry);
    random::register(&mut dispatcher, registry);
    list::register(&mut dispatcher, registry);
    locate::register(&mut dispatcher, registry);
    loot::register(&mut dispatcher, registry);
    seed::register(&mut dispatcher, registry);
    saveall::register(&mut dispatcher, registry);
    saveoff::register(&mut dispatcher, registry);
    saveon::register(&mut dispatcher, registry);
    setidletimeout::register(&mut dispatcher, registry);
    spreadplayers::register(&mut dispatcher, registry);
    stop::register(&mut dispatcher, registry);
    tag::register(&mut dispatcher, registry);
    tick::register(&mut dispatcher, registry);
    advancement::register(&mut dispatcher, registry);
    data::register(&mut dispatcher, registry);
    stopwatch::register(&mut dispatcher, registry);
    trigger::register(&mut dispatcher, registry);
    scoreboard::register(&mut dispatcher, registry);
    team::register(&mut dispatcher, registry);
    teammsg::register(&mut dispatcher, registry);
    clone::register(&mut dispatcher, registry);
    attribute::register(&mut dispatcher, registry);
    datapack::register(&mut dispatcher, registry);
    function::register(&mut dispatcher, registry);
    r#return::register(&mut dispatcher, registry);
    schedule::register(&mut dispatcher, registry);
    fetchprofile::register(&mut dispatcher, registry);
    worldborder::register(&mut dispatcher, registry);
    particle::register(&mut dispatcher, registry);
    playsound::register(&mut dispatcher, registry);
    fill::register(&mut dispatcher, registry);
    clear::register(&mut dispatcher, registry);
    pumpkin::register(&mut dispatcher, registry);
    me::register(&mut dispatcher, registry);
    msg::register(&mut dispatcher, registry);
    tps::register(&mut dispatcher, registry);
    transfer::register(&mut dispatcher, registry);
    gamemode::register(&mut dispatcher, registry);
    defaultgamemode::register(&mut dispatcher, registry);
    weather::register(&mut dispatcher, registry);
    time::register(&mut dispatcher, registry);
    teleport::register(&mut dispatcher, registry);
    setworldspawn::register(&mut dispatcher, registry);
    spawnpoint::register(&mut dispatcher, registry);
    spectate::register(&mut dispatcher, registry);
    setblock::register(&mut dispatcher, registry);
    give::register(&mut dispatcher, registry);
    item::register(&mut dispatcher, registry);
    enchant::register(&mut dispatcher, registry);
    effect::register(&mut dispatcher, registry);
    summon::register(&mut dispatcher, registry);
    damage::register(&mut dispatcher, registry);
    rotate::register(&mut dispatcher, registry);
    tellraw::register(&mut dispatcher, registry);
    title::register(&mut dispatcher, registry);
    experience::register(&mut dispatcher, registry);
    bossbar::register(&mut dispatcher, registry);
    gamerule::register(&mut dispatcher, registry);
    stopsound::register(&mut dispatcher, registry);
    waypoint::register(&mut dispatcher, registry);
    raid::register(&mut dispatcher, registry);
    deop::register(&mut dispatcher, registry);
    kick::register(&mut dispatcher, registry);
    plugin::register(&mut dispatcher, registry);
    plugins::register(&mut dispatcher, registry);
    ban::register(&mut dispatcher, registry);
    banip::register(&mut dispatcher, registry);
    pardon::register(&mut dispatcher, registry);
    pardonip::register(&mut dispatcher, registry);
    whitelist::register(&mut dispatcher, registry);

    apply_command_overrides(&mut dispatcher, registry, commands_config);

    dispatcher
}

/// Applies the per-command settings from the server configuration on top of the
/// freshly built dispatcher.
///
/// Two kinds of override are supported:
/// - Disabling a command, which removes it from the legacy dispatcher and marks
///   its name so the wrapper dispatcher hides it everywhere else.
/// - Changing a command's required permission level, which is done by rewriting
///   the default of its permission node in the registry. Because command
///   requirements look their permission up in the registry at execution time,
///   this affects both the legacy and the node-based dispatchers uniformly.
fn apply_command_overrides(
    dispatcher: &mut CommandDispatcher,
    registry: &PermissionRegistry,
    commands_config: &CommandsConfig,
) {
    for (raw_name, settings) in &commands_config.overrides {
        // Command names are always lowercase, so normalise here to be forgiving
        // of how the owner wrote them in the config file.
        let name = raw_name.to_ascii_lowercase();

        // Catch typos: an override for a command that does not exist almost
        // always means the owner misspelled it, so tell them instead of silently
        // doing nothing.
        if !dispatcher.has_command(&name) {
            warn!(
                "Ignoring the command setting for \"{raw_name}\" because there is no command with that name (check the spelling in your config)"
            );
            continue;
        }

        if !settings.enabled {
            // If the owner named an alias (e.g. `tp` for `teleport`), turn off
            // the whole command, not just that one alias.
            let primary = dispatcher.primary_command_name(&name);

            dispatcher.disable_command(name.clone());
            dispatcher.disable_command(primary.clone());
            // Node-based commands keep their aliases as redirecting root nodes,
            // so flag those too.
            for alias in dispatcher.tree_alias_names(&primary) {
                dispatcher.disable_command(alias);
            }
            info!("The /{primary} command has been turned off in the configuration");
            // A disabled command can never be run, so its permission level is
            // irrelevant; skip the rest.
            continue;
        }

        if let Some(level) = settings.permission_level {
            let default = if level == PermissionLvl::Zero {
                PermissionDefault::Allow
            } else {
                PermissionDefault::Op(level)
            };

            if let Some(node) = resolve_permission_node(dispatcher, registry, &name) {
                if registry.set_default(&node, default) {
                    info!(
                        "The /{name} command now needs permission level {} to use",
                        level as u8
                    );
                } else {
                    warn!(
                        "Command override for /{name} sets a permission level, but matching permission node could not be updated; leaving it unchanged"
                    );
                }
            } else {
                warn!(
                    "Command override for /{name} sets a permission level, but no matching permission node could be found; leaving it unchanged"
                );
            }
        }
    }
}

/// Finds the permission node associated with a command name.
///
/// Follows the `<namespace>:command.<name>` convention by probing the registry.
fn resolve_permission_node(
    _dispatcher: &CommandDispatcher,
    registry: &PermissionRegistry,
    name: &str,
) -> Option<String> {
    for namespace in ["minecraft", "pumpkin"] {
        let candidate = format!("{namespace}:command.{name}");
        if registry.get_permission(&candidate).is_some() {
            return Some(candidate);
        }
    }

    None
}

fn register_permissions(registry: &PermissionRegistry) {
    // Register our entity selector permission as well.
    registry
        .register_permission(Permission::new(
            "minecraft:command.selector",
            "Allows a player to use selector variables",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
}

#[cfg(test)]
mod override_tests {
    use pumpkin_config::{CommandOverride, CommandsConfig};
    use pumpkin_util::PermissionLvl;
    use pumpkin_util::permission::{PermissionDefault, PermissionManager};

    use super::default_dispatcher;

    fn disabled(config: &mut CommandsConfig, name: &str) {
        config.overrides.insert(
            name.to_string(),
            CommandOverride {
                enabled: false,
                permission_level: None,
            },
        );
    }

    fn permission(config: &mut CommandsConfig, name: &str, level: PermissionLvl) {
        config.overrides.insert(
            name.to_string(),
            CommandOverride {
                enabled: true,
                permission_level: Some(level),
            },
        );
    }

    #[test]
    fn disabling_a_command_removes_and_hides_it() {
        let mut commands = CommandsConfig::default();
        disabled(&mut commands, "gamemode");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &commands);

        assert!(dispatcher.is_disabled("gamemode"));
        assert!(
            !dispatcher.is_disabled("give"),
            "untouched commands stay on"
        );
    }

    #[test]
    fn disabling_an_alias_turns_off_the_whole_command() {
        let mut commands = CommandsConfig::default();
        // `tp` is an alias of `teleport`; disabling it should take the whole
        // command down, including the primary name.
        disabled(&mut commands, "tp");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &commands);

        assert!(dispatcher.is_disabled("tp"));
        assert!(dispatcher.is_disabled("teleport"));
    }

    #[test]
    fn disabling_a_node_command_also_disables_its_aliases() {
        let mut commands = CommandsConfig::default();
        // `help` is a node-based command with the aliases `h` and `?`.
        disabled(&mut commands, "help");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &commands);

        assert!(dispatcher.is_disabled("help"));
        assert!(dispatcher.is_disabled("h"));
        assert!(dispatcher.is_disabled("?"));
    }

    #[test]
    fn override_for_unknown_command_is_ignored() {
        let mut commands = CommandsConfig::default();
        // A command name that does not exist (usually a typo in the config)
        // should be ignored, not silently swallow a real command or panic.
        disabled(&mut commands, "notacommand");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &commands);

        assert!(!dispatcher.is_disabled("notacommand"));
        assert!(dispatcher.has_command("gamemode"));
    }

    #[test]
    fn override_is_case_insensitive() {
        let mut commands = CommandsConfig::default();
        disabled(&mut commands, "GameMode");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &commands);

        assert!(dispatcher.is_disabled("gamemode"));
    }

    #[test]
    fn permission_level_override_rewrites_the_registry_default() {
        let mut commands = CommandsConfig::default();
        // `gamemode` is normally level 2; bump it to owner-only.
        permission(&mut commands, "gamemode", PermissionLvl::Four);

        let manager = PermissionManager::new();
        let _dispatcher = default_dispatcher(&manager, &commands);

        let permission = manager
            .get_permission("minecraft:command.gamemode")
            .expect("gamemode permission should be registered");
        assert_eq!(
            permission.default,
            PermissionDefault::Op(PermissionLvl::Four)
        );
    }

    #[test]
    fn permission_override_resolves_node_command_by_convention() {
        let mut commands = CommandsConfig::default();
        // `kill` is a node-based command whose permission node is not recorded in
        // the legacy dispatcher, so the override must fall back to the
        // `minecraft:command.kill` naming convention.
        permission(&mut commands, "kill", PermissionLvl::Four);

        let manager = PermissionManager::new();
        let _dispatcher = default_dispatcher(&manager, &commands);

        let permission = manager
            .get_permission("minecraft:command.kill")
            .expect("kill permission should be registered");
        assert_eq!(
            permission.default,
            PermissionDefault::Op(PermissionLvl::Four)
        );
    }

    #[test]
    fn permission_level_zero_allows_everyone() {
        let mut commands = CommandsConfig::default();
        permission(&mut commands, "gamemode", PermissionLvl::Zero);

        let manager = PermissionManager::new();
        let _dispatcher = default_dispatcher(&manager, &commands);

        let permission = manager
            .get_permission("minecraft:command.gamemode")
            .expect("gamemode permission should be registered");
        assert_eq!(permission.default, PermissionDefault::Allow);
    }
}
