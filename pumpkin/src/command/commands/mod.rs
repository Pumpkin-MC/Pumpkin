use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
};

use crate::PERMISSION_REGISTRY;

use super::dispatcher::CommandDispatcher;

mod ban;
mod banip;
mod banlist;
mod bossbar;
mod clear;
mod damage;
pub mod defaultgamemode;
mod deop;
mod difficulty;
mod effect;
mod experience;
mod fill;
mod gamemode;
mod give;
mod help;
mod kick;
mod kill;
mod list;
mod me;
mod msg;
mod op;
mod pardon;
mod pardonip;
mod particle;
mod playsound;
mod plugin;
mod plugins;
mod pumpkin;
mod say;
mod seed;
mod setblock;
mod stop;
mod stopsound;
mod summon;
mod teleport;
mod time;
mod title;
mod transfer;
mod weather;
mod whitelist;
mod worldborder;

#[cfg(feature = "dhat-heap")]
mod profile;
mod tellraw;

#[must_use]
pub async fn default_dispatcher() -> CommandDispatcher {
    let mut dispatcher = CommandDispatcher::default();

    register_permissions().await;

    // Zero
    dispatcher.register(pumpkin::init_command_tree(), "pumpkin:command.pumpkin");
    dispatcher.register(help::init_command_tree(), "minecraft:command.help");
    dispatcher.register(list::init_command_tree(), "minecraft:command.list");
    dispatcher.register(transfer::init_command_tree(), "minecraft:command.transfer");
    dispatcher.register(me::init_command_tree(), "minecraft:command.me");
    dispatcher.register(msg::init_command_tree(), "minecraft:command.msg");
    // Two
    dispatcher.register(kill::init_command_tree(), "minecraft:command.kill");
    dispatcher.register(
        worldborder::init_command_tree(),
        "minecraft:command.worldborder",
    );
    dispatcher.register(effect::init_command_tree(), "minecraft:command.effect");
    dispatcher.register(teleport::init_command_tree(), "minecraft:command.teleport");
    dispatcher.register(time::init_command_tree(), "minecraft:command.time");
    dispatcher.register(give::init_command_tree(), "minecraft:command.give");
    dispatcher.register(clear::init_command_tree(), "minecraft:command.clear");
    dispatcher.register(setblock::init_command_tree(), "minecraft:command.setblock");
    dispatcher.register(seed::init_command_tree(), "minecraft:command.seed");
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
    dispatcher.register(damage::init_command_tree(), "minecraft:command.damage");
    dispatcher.register(bossbar::init_command_tree(), "minecraft:command.bossbar");
    dispatcher.register(say::init_command_tree(), "minecraft:command.say");
    dispatcher.register(gamemode::init_command_tree(), "minecraft:command.gamemode");
    dispatcher.register(
        difficulty::init_command_tree(),
        "minecraft:command.difficulty",
    );
    dispatcher.register(
        stopsound::init_command_tree(),
        "minecraft:command.stopsound",
    );
    dispatcher.register(
        defaultgamemode::init_command_tree(),
        "minecraft:command.defaultgamemode",
    );
    // Three
    dispatcher.register(op::init_command_tree(), "minecraft:command.op");
    dispatcher.register(deop::init_command_tree(), "minecraft:command.deop");
    dispatcher.register(kick::init_command_tree(), "minecraft:command.kick");
    dispatcher.register(plugin::init_command_tree(), "pumpkin:command.plugin");
    dispatcher.register(plugins::init_command_tree(), "pumpkin:command.plugins");
    dispatcher.register(ban::init_command_tree(), "minecraft:command.ban");
    dispatcher.register(banip::init_command_tree(), "minecraft:command.banip");
    dispatcher.register(banlist::init_command_tree(), "minecraft:command.banlist");
    dispatcher.register(pardon::init_command_tree(), "minecraft:command.pardon");
    dispatcher.register(pardonip::init_command_tree(), "minecraft:command.pardonip");
    dispatcher.register(
        whitelist::init_command_tree(),
        "minecraft:command.whitelist",
    );
    // Four
    dispatcher.register(stop::init_command_tree(), "minecraft:command.stop");

    #[cfg(feature = "dhat-heap")]
    dispatcher.register(profile::init_command_tree(), "pumpkin:command.profile");

    dispatcher
}

async fn register_permissions() {
    let mut registry = PERMISSION_REGISTRY.write().await;

    // Register level 0 permissions (allowed by default)
    register_level_0_permissions(&mut registry);

    // Register level 2 permissions (OP level 2)
    register_level_2_permissions(&mut registry);

    // Register level 3 permissions (OP level 3)
    register_level_3_permissions(&mut registry);

    // Register level 4 permissions (OP level 4)
    register_level_4_permissions(&mut registry);
}

fn register_level_0_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for builtin commands that are allowed for everyone
    registry
        .register_permission(Permission::new(
            "pumpkin:command.pumpkin",
            "Shows information about the Pumpkin server",
            PermissionDefault::Allow,
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.help",
            "Lists available commands and their usage",
            PermissionDefault::Allow,
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.list",
            "Lists players that are currently online",
            PermissionDefault::Allow,
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.transfer",
            "Transfers the player to another server",
            PermissionDefault::Allow,
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.me",
            "Broadcasts a narrative message about the player",
            PermissionDefault::Allow,
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.msg",
            "Sends a private message to another player",
            PermissionDefault::Allow,
        ))
        .unwrap();
}

#[allow(clippy::too_many_lines)]
fn register_level_2_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Two
    registry
        .register_permission(Permission::new(
            "minecraft:command.kill",
            "Kills entities (players, mobs, items, etc.)",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.worldborder",
            "Manages the world border",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.effect",
            "Adds or removes status effects",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.teleport",
            "Teleports entities to other locations",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.time",
            "Changes or queries the world's game time",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.give",
            "Gives an item to a player",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.clear",
            "Clears items from player inventory",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.setblock",
            "Changes a block to another block",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.seed",
            "Displays the world seed",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.fill",
            "Fills a region with a specific block",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.playsound",
            "Plays a sound to players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.tellraw",
            "Displays a JSON message to players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.title",
            "Controls screen titles displayed to players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.summon",
            "Summons an entity",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.experience",
            "Adds, removes or queries player experience",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.weather",
            "Sets the weather in the server",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.particle",
            "Creates particles in the world",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.damage",
            "Damages entities",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.bossbar",
            "Creates and manages boss bars",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.say",
            "Broadcasts a message to multiple players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.gamemode",
            "Sets a player's game mode",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.stopsound",
            "Stops sounds from playing",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.defaultgamemode",
            "Sets the default game mode for new players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.difficulty",
            "Sets the difficulty of the world",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
}

fn register_level_3_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Three
    registry
        .register_permission(Permission::new(
            "minecraft:command.op",
            "Grants operator status to a player",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.deop",
            "Revokes operator status from a player",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.kick",
            "Removes players from the server",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "pumpkin:command.plugin",
            "Manages server plugins",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "pumpkin:command.plugins",
            "Lists all plugins loaded on the server",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.ban",
            "Adds players to banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.banip",
            "Adds IP addresses to banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.banlist",
            "Displays banned players or IP addresses",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.pardon",
            "Removes entries from the player banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.pardonip",
            "Removes entries from the IP banlist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.whitelist",
            "Manages server whitelist",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
}

fn register_level_4_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Four
    registry
        .register_permission(Permission::new(
            "minecraft:command.stop",
            "Stops the server",
            PermissionDefault::Op(PermissionLvl::Four),
        ))
        .unwrap();

    #[cfg(feature = "dhat-heap")]
    registry
        .register_permission(Permission::new(
            "pumpkin:command.profile",
            "Controls heap profiling",
            PermissionDefault::Op(PermissionLvl::Four),
        ))
        .unwrap();
}
