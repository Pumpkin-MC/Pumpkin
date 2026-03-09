use pumpkin_config::BasicConfiguration;
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
};
use tokio::sync::RwLock;

use super::dispatcher::CommandDispatcher;

mod attribute;
mod ban;
mod banip;
mod banlist;
mod bossbar;
mod clear;
mod clone;
mod damage;
mod data;
mod debug;
pub mod defaultgamemode;
mod deop;
mod difficulty;
mod effect;
mod enchant;
mod execute;
mod experience;
mod fill;
mod fillbiome;
mod gamemode;
mod gamerule;
mod give;
mod help;
mod item;
mod kick;
mod kill;
mod list;
mod me;
mod msg;
mod op;
mod pardon;
mod pardonip;
mod particle;
mod perf;
mod playsound;
mod plugin;
mod plugins;
mod pumpkin;
mod random;
mod reload;
mod ride;
mod rotate;
mod save;
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
mod weather;
mod whitelist;
mod worldborder;

#[must_use]
#[expect(clippy::too_many_lines)]
pub async fn default_dispatcher(
    registry: &RwLock<PermissionRegistry>,
    basic_config: &BasicConfiguration,
) -> CommandDispatcher {
    let mut dispatcher = CommandDispatcher::default();

    register_permissions(registry).await;

    // Zero
    dispatcher.register(pumpkin::init_command_tree(), "pumpkin:command.pumpkin");
    dispatcher.register(help::init_command_tree(), "minecraft:command.help");
    dispatcher.register(list::init_command_tree(), "minecraft:command.list");
    dispatcher.register(me::init_command_tree(), "minecraft:command.me");
    dispatcher.register(msg::init_command_tree(), "minecraft:command.msg");
    dispatcher.register(random::init_command_tree(), "minecraft:command.random");
    dispatcher.register(trigger::init_command_tree(), "minecraft:command.trigger");
    dispatcher.register(teammsg::init_command_tree(), "minecraft:command.teammsg");
    // Two
    dispatcher.register(kill::init_command_tree(), "minecraft:command.kill");
    dispatcher.register(
        worldborder::init_command_tree(),
        "minecraft:command.worldborder",
    );
    dispatcher.register(effect::init_command_tree(), "minecraft:command.effect");
    dispatcher.register(teleport::init_command_tree(), "minecraft:command.teleport");
    dispatcher.register(time::init_command_tree(), "minecraft:command.time");
    dispatcher.register(
        tick::init_command_tree(basic_config.tps),
        "minecraft:command.tick",
    );
    dispatcher.register(give::init_command_tree(), "minecraft:command.give");
    dispatcher.register(enchant::init_command_tree(), "minecraft:command.enchant");
    dispatcher.register(clear::init_command_tree(), "minecraft:command.clear");
    dispatcher.register(setblock::init_command_tree(), "minecraft:command.setblock");
    dispatcher.register(seed::init_command_tree(), "minecraft:command.seed");
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
    dispatcher.register(
        setworldspawn::init_command_tree(),
        "minecraft:command.setworldspawn",
    );
    dispatcher.register(
        spawnpoint::init_command_tree(),
        "minecraft:command.spawnpoint",
    );
    dispatcher.register(data::init_command_tree(), "minecraft:command.data");
    dispatcher.register(tag::init_command_tree(), "minecraft:command.tag");
    dispatcher.register(ride::init_command_tree(), "minecraft:command.ride");
    dispatcher.register(
        attribute::init_command_tree(),
        "minecraft:command.attribute",
    );
    dispatcher.register(
        spectate::init_command_tree(),
        "minecraft:command.spectate",
    );
    dispatcher.register(clone::init_command_tree(), "minecraft:command.clone");
    dispatcher.register(
        fillbiome::init_command_tree(),
        "minecraft:command.fillbiome",
    );
    dispatcher.register(
        spreadplayers::init_command_tree(),
        "minecraft:command.spreadplayers",
    );
    dispatcher.register(
        scoreboard::init_command_tree(),
        "minecraft:command.scoreboard",
    );
    dispatcher.register(team::init_command_tree(), "minecraft:command.team");
    dispatcher.register(item::init_command_tree(), "minecraft:command.item");
    dispatcher.register(execute::init_command_tree(), "minecraft:command.execute");
    dispatcher.register(perf::init_command_tree(), "minecraft:command.perf");
    dispatcher.register(debug::init_command_tree(), "minecraft:command.debug");
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
    dispatcher.register(transfer::init_command_tree(), "minecraft:command.transfer");
    dispatcher.register(
        setidletimeout::init_command_tree(),
        "minecraft:command.setidletimeout",
    );
    dispatcher.register(reload::init_command_tree(), "minecraft:command.reload");
    // Four
    dispatcher.register(stop::init_command_tree(), "minecraft:command.stop");
    dispatcher.register(
        save::init_command_tree_save_all(),
        "minecraft:command.save-all",
    );
    dispatcher.register(
        save::init_command_tree_save_off(),
        "minecraft:command.save-off",
    );
    dispatcher.register(
        save::init_command_tree_save_on(),
        "minecraft:command.save-on",
    );

    dispatcher
}

async fn register_permissions(permission_registry: &RwLock<PermissionRegistry>) {
    let mut registry = permission_registry.write().await;

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
    registry
        .register_permission(Permission::new(
            "minecraft:command.random",
            "Draw a random value or control random sequences",
            PermissionDefault::Allow,
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.trigger",
            "Sets a trigger to be activated",
            PermissionDefault::Allow,
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.teammsg",
            "Sends a message to all players on the sender's team",
            PermissionDefault::Allow,
        ))
        .unwrap();
}

#[expect(clippy::too_many_lines)]
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
            "minecraft:command.rotate",
            "Changes the rotation of an entity",
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
            "minecraft:command.gamerule",
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
    registry
        .register_permission(Permission::new(
            "minecraft:command.data",
            "Query and modify data of entities and blocks",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.enchant",
            "Adds an enchantment to a player's selected item, subject to the same restrictions as an anvil. Also works on any mob or entity holding a weapon/tool/armor in its main hand.",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.spawnpoint",
            "Sets the spawn point for a player",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "pumpkin:command.tps",
            "Displays the server TPS and MSPT",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.tag",
            "Controls entity tags",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.ride",
            "Mounts or dismounts entities",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.spectate",
            "Makes a player in Spectator mode spectate an entity",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.attribute",
            "Queries, adds, removes, or sets an entity attribute",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.clone",
            "Copies blocks from one region to another",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.fillbiome",
            "Fills a region with a specific biome",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.spreadplayers",
            "Teleports entities to random surface locations in an area",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.scoreboard",
            "Manages scoreboard objectives and players",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.team",
            "Controls teams",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.item",
            "Manipulates items in inventories",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.execute",
            "Executes a command",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.perf",
            "Captures info and metrics about the server",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.debug",
            "Starts or stops a debug profiling session",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap();
}

#[expect(clippy::too_many_lines)]
fn register_level_3_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Three
    registry
        .register_permission(Permission::new(
            "minecraft:command.setworldspawn",
            "Sets the world spawn point",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
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
    registry
        .register_permission(Permission::new(
            "minecraft:command.tick",
            "Triggers the tick event",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.transfer",
            "Transfers the player to another server",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.setidletimeout",
            "Sets the time before idle players are kicked",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.reload",
            "Reloads loot tables, advancements, and functions from disk",
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
    registry
        .register_permission(Permission::new(
            "minecraft:command.save-all",
            "Saves the server to disk",
            PermissionDefault::Op(PermissionLvl::Four),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.save-off",
            "Disables automatic saving",
            PermissionDefault::Op(PermissionLvl::Four),
        ))
        .unwrap();
    registry
        .register_permission(Permission::new(
            "minecraft:command.save-on",
            "Enables automatic saving",
            PermissionDefault::Op(PermissionLvl::Four),
        ))
        .unwrap();
}
