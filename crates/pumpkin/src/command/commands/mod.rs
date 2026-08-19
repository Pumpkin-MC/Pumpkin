use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::tree::Command;
use pumpkin_config::{BasicConfiguration, CommandsConfig};
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
mod gamemode;
mod gamerule;
mod give;
mod help;
mod item;
mod kick;
mod kill;
mod list;
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
mod random;
mod recipe;
mod ride;
mod rotate;
mod saveall;
mod saveoff;
mod saveon;
mod say;
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
    _basic_config: &BasicConfiguration,
    commands_config: &CommandsConfig,
) -> CommandDispatcher {
    let mut dispatcher = crate::command::dispatcher::CommandDispatcher::default();
    let registry = &permission_manager.registry;

    register_permissions(registry);

    // Zero
    dispatcher.register(pumpkin::init_command_tree(), "pumpkin:command.pumpkin");
    dispatcher.register(me::init_command_tree(), "minecraft:command.me");
    dispatcher.register(msg::init_command_tree(), "minecraft:command.msg");
    // Two
    dispatcher.register(
        worldborder::init_command_tree(),
        "minecraft:command.worldborder",
    );
    dispatcher.register(effect::init_command_tree(), "minecraft:command.effect");
    dispatcher.register(teleport::init_command_tree(), "minecraft:command.teleport");
    dispatcher.register(time::init_command_tree(), "minecraft:command.time");
    dispatcher.register(give::init_command_tree(), "minecraft:command.give");
    dispatcher.register(item::init_command_tree(), "minecraft:command.item");
    dispatcher.register(enchant::init_command_tree(), "minecraft:command.enchant");
    dispatcher.register(clear::init_command_tree(), "minecraft:command.clear");
    dispatcher.register(setblock::init_command_tree(), "minecraft:command.setblock");
    dispatcher.register(tps::init_command_tree(), "pumpkin:command.tps");
    dispatcher.register(fill::init_command_tree(), "minecraft:command.fill");
    dispatcher.register(
        playsound::init_command_tree(),
        "minecraft:command.playsound",
    );
    dispatcher.register(tellraw::init_command_tree(), "minecraft:command.tellraw");
    dispatcher.register(title::init_command_tree(), "minecraft:command.title");
    dispatcher.register(summon::init_command_tree(), "minecraft:command.summon");
    dispatcher.register(
        experience::init_command_tree(),
        "minecraft:command.experience",
    );
    dispatcher.register(weather::init_command_tree(), "minecraft:command.weather");
    dispatcher.register(particle::init_command_tree(), "minecraft:command.particle");
    dispatcher.register(rotate::init_command_tree(), "minecraft:command.rotate");
    dispatcher.register(damage::init_command_tree(), "minecraft:command.damage");
    dispatcher.register(bossbar::init_command_tree(), "minecraft:command.bossbar");
    dispatcher.register(say::init_command_tree(), "minecraft:command.say");
    dispatcher.register(gamemode::init_command_tree(), "minecraft:command.gamemode");
    dispatcher.register(gamerule::init_command_tree(), "minecraft:command.gamerule");
    dispatcher.register(
        stopsound::init_command_tree(),
        "minecraft:command.stopsound",
    );
    dispatcher.register(
        defaultgamemode::init_command_tree(),
        "minecraft:command.defaultgamemode",
    );
    dispatcher.register(
        setworldspawn::init_command_tree(),
        "minecraft:command.setworldspawn",
    );
    dispatcher.register(
        spawnpoint::init_command_tree(),
        "minecraft:command.spawnpoint",
    );
    dispatcher.register(spectate::init_command_tree(), "minecraft:command.spectate");
    dispatcher.register(data::init_command_tree(), "minecraft:command.data");
    dispatcher.register(waypoint::init_command_tree(), "minecraft:command.waypoint");
    // Three
    dispatcher.register(deop::init_command_tree(), "minecraft:command.deop");
    dispatcher.register(kick::init_command_tree(), "minecraft:command.kick");
    dispatcher.register(plugin::init_command_tree(), "pumpkin:command.plugin");
    dispatcher.register(plugins::init_command_tree(), "pumpkin:command.plugins");
    dispatcher.register(ban::init_command_tree(), "minecraft:command.ban");
    dispatcher.register(banip::init_command_tree(), "minecraft:command.banip");
    dispatcher.register(pardon::init_command_tree(), "minecraft:command.pardon");
    dispatcher.register(pardonip::init_command_tree(), "minecraft:command.pardonip");
    dispatcher.register(
        whitelist::init_command_tree(),
        "minecraft:command.whitelist",
    );
    dispatcher.register(transfer::init_command_tree(), "minecraft:command.transfer");

    let mut dispatcher = {
        let mut wrapper_dispatcher = CommandDispatcher::new();
        wrapper_dispatcher.fallback_dispatcher = dispatcher;
        wrapper_dispatcher
    };

    banlist::register(&mut dispatcher, registry);
    difficulty::register(&mut dispatcher, registry);
    dialog::register(&mut dispatcher, registry);
    execute::register(&mut dispatcher, registry);
    fillbiome::register(&mut dispatcher, registry);
    forceload::register(&mut dispatcher, registry);
    ride::register(&mut dispatcher, registry);
    recipe::register(&mut dispatcher, registry);
    help::register(&mut dispatcher, registry);
    kill::register(&mut dispatcher, registry);
    op::register(&mut dispatcher, registry);
    place::register(&mut dispatcher, registry);
    random::register(&mut dispatcher, registry);
    list::register(&mut dispatcher, registry);
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
    trigger::register(&mut dispatcher, registry);
    scoreboard::register(&mut dispatcher, registry);
    team::register(&mut dispatcher, registry);
    teammsg::register(&mut dispatcher, registry);
    clone::register(&mut dispatcher, registry);
    attribute::register(&mut dispatcher, registry);
    fetchprofile::register(&mut dispatcher, registry);

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
            let primary = match dispatcher.fallback_dispatcher.commands.get(&name) {
                Some(Command::Alias(target)) => target.clone(),
                _ => name.clone(),
            };

            dispatcher.disable_command(name.clone());
            dispatcher.disable_command(primary.clone());
            // Node-based commands keep their aliases as redirecting root nodes,
            // so flag those too. (Legacy aliases are handled by the unregister
            // below, which removes them from the dispatcher outright.)
            for alias in dispatcher.tree_alias_names(&primary) {
                dispatcher.disable_command(alias);
            }
            // Unregistering the primary name cascades to every alias in the
            // legacy dispatcher.
            dispatcher.fallback_dispatcher.unregister(&primary);
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
/// Legacy commands record their node in the dispatcher directly. Node-based
/// commands do not, but they follow the `<namespace>:command.<name>` convention,
/// so we fall back to probing the registry for those.
fn resolve_permission_node(
    dispatcher: &CommandDispatcher,
    registry: &PermissionRegistry,
    name: &str,
) -> Option<String> {
    if let Some(node) = dispatcher.fallback_dispatcher.permissions.get(name) {
        return Some(node.clone());
    }

    for namespace in ["minecraft", "pumpkin"] {
        let candidate = format!("{namespace}:command.{name}");
        if registry.get_permission(&candidate).is_some() {
            return Some(candidate);
        }
    }

    None
}

fn register_permissions(registry: &PermissionRegistry) {
    // Register level 0 permissions (allowed by default)
    register_level_0_permissions(registry);

    // Register level 2 permissions (OP level 2)
    register_level_2_permissions(registry);

    // Register level 3 permissions (OP level 3)
    register_level_3_permissions(registry);

    // Register our entity selector permission as well.
    registry
        .register_permission(Permission::new(
            "minecraft:command.selector",
            "Allows a player to use selector variables",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
}

fn register_level_0_permissions(registry: &PermissionRegistry) {
    // Register permissions for builtin commands that are allowed for everyone
    registry
        .register_permission(Permission::new(
            "pumpkin:command.pumpkin",
            "Shows information about the Pumpkin server",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.me",
            "Broadcasts a narrative message about the player",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.msg",
            "Sends a private message to another player",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
}

#[expect(clippy::too_many_lines)]
fn register_level_2_permissions(registry: &PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Two
    registry
        .register_permission(Permission::new(
            "minecraft:command.worldborder",
            "Manages the world border",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.effect",
            "Adds or removes status effects",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.teleport",
            "Teleports entities to other locations",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.time",
            "Changes or queries the world's game time",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.waypoint",
            "List or modify waypoints",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.give",
            "Gives an item to a player",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.item",
            "Replace items in inventories",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.clear",
            "Clears items from player inventory",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.setblock",
            "Changes a block to another block",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.fill",
            "Fills a region with a specific block",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.playsound",
            "Plays a sound to players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.tellraw",
            "Displays a JSON message to players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.title",
            "Controls screen titles displayed to players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.summon",
            "Summons an entity",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.experience",
            "Adds, removes or queries player experience",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.weather",
            "Sets the weather in the server",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.particle",
            "Creates particles in the world",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.rotate",
            "Changes the rotation of an entity",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.damage",
            "Damages entities",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.bossbar",
            "Creates and manages boss bars",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.say",
            "Broadcasts a message to multiple players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.gamemode",
            "Sets a player's game mode",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.gamerule",
            "Sets a player's game mode",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.stopsound",
            "Stops sounds from playing",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.defaultgamemode",
            "Sets the default game mode for new players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.data",
            "Query and modify data of entities and blocks",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.enchant",
            "Adds an enchantment to a player's selected item, subject to the same restrictions as an anvil. Also works on any mob or entity holding a weapon/tool/armor in its main hand.",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.spawnpoint",
            "Sets the spawn point for a player",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.spectate",
            "Allows a player to spectate another entity",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "pumpkin:command.tps",
            "Displays the server TPS and MSPT",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
}

fn register_level_3_permissions(registry: &PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Three
    registry
        .register_permission(Permission::new(
            "minecraft:command.setworldspawn",
            "Sets the world spawn point",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.deop",
            "Revokes operator status from a player",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.kick",
            "Removes players from the server",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "pumpkin:command.plugin",
            "Manages server plugins",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "pumpkin:command.plugins",
            "Lists all plugins loaded on the server",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.ban",
            "Adds players to banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.banip",
            "Adds IP addresses to banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.pardon",
            "Removes entries from the player banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.pardonip",
            "Removes entries from the IP banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.whitelist",
            "Manages server whitelist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
    registry
        .register_permission(Permission::new(
            "minecraft:command.transfer",
            "Transfers the player to another server",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|e| tracing::warn!("{e}"));
}

#[cfg(test)]
mod override_tests {
    use pumpkin_config::{BasicConfiguration, CommandOverride, CommandsConfig};
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
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        // `gamemode` lives on the legacy dispatcher; disabling it should remove
        // it there and flag it on the wrapper.
        disabled(&mut commands, "gamemode");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &basic, &commands);

        assert!(dispatcher.is_disabled("gamemode"));
        assert!(
            dispatcher.fallback_dispatcher.get_tree("gamemode").is_err(),
            "disabled command should be unregistered from the legacy dispatcher"
        );
        assert!(
            !dispatcher.is_disabled("give"),
            "untouched commands stay on"
        );
    }

    #[test]
    fn disabling_an_alias_turns_off_the_whole_command() {
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        // `tp` is an alias of `teleport`; disabling it should take the whole
        // command down, including the primary name.
        disabled(&mut commands, "tp");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &basic, &commands);

        assert!(dispatcher.is_disabled("tp"));
        assert!(dispatcher.is_disabled("teleport"));
        assert!(dispatcher.fallback_dispatcher.get_tree("tp").is_err());
        assert!(dispatcher.fallback_dispatcher.get_tree("teleport").is_err());
    }

    #[test]
    fn disabling_a_node_command_also_disables_its_aliases() {
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        // `help` is a node-based command with the aliases `h` and `?`.
        disabled(&mut commands, "help");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &basic, &commands);

        assert!(dispatcher.is_disabled("help"));
        assert!(dispatcher.is_disabled("h"));
        assert!(dispatcher.is_disabled("?"));
    }

    #[test]
    fn override_for_unknown_command_is_ignored() {
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        // A command name that does not exist (usually a typo in the config)
        // should be ignored, not silently swallow a real command or panic.
        disabled(&mut commands, "notacommand");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &basic, &commands);

        assert!(!dispatcher.is_disabled("notacommand"));
        assert!(dispatcher.fallback_dispatcher.get_tree("gamemode").is_ok());
    }

    #[test]
    fn override_is_case_insensitive() {
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        disabled(&mut commands, "GameMode");

        let manager = PermissionManager::new();
        let dispatcher = default_dispatcher(&manager, &basic, &commands);

        assert!(dispatcher.is_disabled("gamemode"));
    }

    #[test]
    fn permission_level_override_rewrites_the_registry_default() {
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        // `gamemode` is normally level 2; bump it to owner-only.
        permission(&mut commands, "gamemode", PermissionLvl::Four);

        let manager = PermissionManager::new();
        let _dispatcher = default_dispatcher(&manager, &basic, &commands);

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
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        // `kill` is a node-based command whose permission node is not recorded in
        // the legacy dispatcher, so the override must fall back to the
        // `minecraft:command.kill` naming convention.
        permission(&mut commands, "kill", PermissionLvl::Four);

        let manager = PermissionManager::new();
        let _dispatcher = default_dispatcher(&manager, &basic, &commands);

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
        let basic = BasicConfiguration::default();
        let mut commands = CommandsConfig::default();
        permission(&mut commands, "gamemode", PermissionLvl::Zero);

        let manager = PermissionManager::new();
        let _dispatcher = default_dispatcher(&manager, &basic, &commands);

        let permission = manager
            .get_permission("minecraft:command.gamemode")
            .expect("gamemode permission should be registered");
        assert_eq!(permission.default, PermissionDefault::Allow);
    }
}
