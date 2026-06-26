use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::tr_plain;
use pumpkin_config::BasicConfiguration;
use pumpkin_i18n::server_command_locale;
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
};
use tokio::sync::RwLock;

mod advancement;
mod ban;
mod banip;
mod banlist;
mod bossbar;
mod clear;
mod damage;
mod data;
pub mod defaultgamemode;
mod deop;
mod difficulty;
mod effect;
mod enchant;
mod experience;
mod fill;
mod gamemode;
mod gamerule;
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
mod rotate;
mod say;
mod seed;
mod setblock;
mod setidletimeout;
mod setworldspawn;
mod spawnpoint;
mod stop;
mod stopsound;
mod summon;
mod teleport;
mod tellraw;
mod tick;
mod time;
mod title;
mod tps;
mod transfer;
mod weather;
mod whitelist;
mod worldborder;

#[must_use]
pub async fn default_dispatcher(
    registry: &RwLock<PermissionRegistry>,
    _basic_config: &BasicConfiguration,
) -> CommandDispatcher {
    let mut dispatcher = crate::command::dispatcher::CommandDispatcher::default();

    let mut registry_lock = registry.write().await;
    let registry = &mut *registry_lock;

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
    dispatcher.register(data::init_command_tree(), "minecraft:command.data");
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
    help::register(&mut dispatcher, registry);
    kill::register(&mut dispatcher, registry);
    op::register(&mut dispatcher, registry);
    list::register(&mut dispatcher, registry);
    seed::register(&mut dispatcher, registry);
    setidletimeout::register(&mut dispatcher, registry);
    stop::register(&mut dispatcher, registry);
    tick::register(&mut dispatcher, registry);
    advancement::register(&mut dispatcher, registry);
    dispatcher
}

fn register_permissions(registry: &mut PermissionRegistry) {
    // Register level 0 permissions (allowed by default)
    register_level_0_permissions(registry);

    // Register level 2 permissions (OP level 2)
    register_level_2_permissions(registry);

    // Register level 3 permissions (OP level 3)
    register_level_3_permissions(registry);

    // Register our entity selector permission as well.
    registry
        .register_permission(translated_permission(
            "minecraft:command.selector",
            "permissions.selector.description",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
}

fn translated_permission(
    node: &str,
    description_key: &str,
    default: PermissionDefault,
) -> Permission {
    Permission::new(
        node,
        &tr_plain(description_key, server_command_locale()),
        default,
    )
}

fn register_level_0_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for builtin commands that are allowed for everyone
    registry
        .register_permission(translated_permission(
            "pumpkin:command.pumpkin",
            "permissions.pumpkin.description",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.me",
            "permissions.me.description",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.msg",
            "permissions.msg.description",
            PermissionDefault::Allow,
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
}

#[expect(clippy::too_many_lines)]
fn register_level_2_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Two
    registry
        .register_permission(translated_permission(
            "minecraft:command.worldborder",
            "permissions.worldborder.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.effect",
            "permissions.effect.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.teleport",
            "permissions.teleport.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.time",
            "permissions.time.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.give",
            "permissions.give.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.clear",
            "permissions.clear.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.setblock",
            "permissions.setblock.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.fill",
            "permissions.fill.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.playsound",
            "permissions.playsound.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.tellraw",
            "permissions.tellraw.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.title",
            "permissions.title.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.summon",
            "permissions.summon.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.experience",
            "permissions.experience.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.weather",
            "permissions.weather.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.particle",
            "permissions.particle.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.rotate",
            "permissions.rotate.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.damage",
            "permissions.damage.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.bossbar",
            "permissions.bossbar.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.say",
            "permissions.say.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.gamemode",
            "permissions.gamemode.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.gamerule",
            "permissions.gamerule.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.stopsound",
            "permissions.stopsound.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.defaultgamemode",
            "permissions.defaultgamemode.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.data",
            "permissions.data.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.enchant",
            "permissions.enchant.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.spawnpoint",
            "permissions.spawnpoint.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "pumpkin:command.tps",
            "permissions.tps.description",
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
}

#[allow(clippy::too_many_lines)]
fn register_level_3_permissions(registry: &mut PermissionRegistry) {
    // Register permissions for commands with PermissionLvl::Three
    registry
        .register_permission(translated_permission(
            "minecraft:command.setworldspawn",
            "permissions.setworldspawn.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.deop",
            "permissions.deop.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.kick",
            "permissions.kick.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "pumpkin:command.plugin",
            "permissions.plugin.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "pumpkin:command.plugins",
            "permissions.plugins.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.ban",
            "permissions.ban.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.banip",
            "permissions.banip.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.pardon",
            "permissions.pardon.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.pardonip",
            "permissions.pardonip.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.whitelist",
            "permissions.whitelist.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
    registry
        .register_permission(translated_permission(
            "minecraft:command.transfer",
            "permissions.transfer.description",
            PermissionDefault::Op(PermissionLvl::Three),
        ))
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.permission_already_registered",
                    server_command_locale(),
                )
            )
        });
}
