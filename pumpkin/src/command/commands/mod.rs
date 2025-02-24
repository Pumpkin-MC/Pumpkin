use pumpkin_util::PermissionLvl;

use super::dispatcher::CommandDispatcher;

mod ban;
mod banip;
mod banlist;
mod bossbar;
mod clear;
mod damage;
pub mod defaultgamemode;
mod deop;
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
mod summon;
mod teleport;
mod time;
mod title;
mod transfer;
mod weather;
mod worldborder;

#[must_use]
pub fn default_dispatcher() -> CommandDispatcher {
    let mut dispatcher = CommandDispatcher::default();

    // Zero
    dispatcher.register(pumpkin::init_command_tree(), PermissionLvl::Zero);
    dispatcher.register(help::init_command_tree(), PermissionLvl::Zero);
    dispatcher.register(list::init_command_tree(), PermissionLvl::Zero);
    dispatcher.register(transfer::init_command_tree(), PermissionLvl::Zero);
    dispatcher.register(me::init_command_tree(), PermissionLvl::Zero);
    dispatcher.register(msg::init_command_tree(), PermissionLvl::Zero);
    // Two
    dispatcher.register(kill::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(worldborder::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(effect::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(teleport::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(time::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(give::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(clear::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(setblock::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(seed::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(fill::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(playsound::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(title::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(summon::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(experience::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(weather::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(particle::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(damage::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(bossbar::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(say::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(gamemode::init_command_tree(), PermissionLvl::Two);
    dispatcher.register(defaultgamemode::init_command_tree(), PermissionLvl::Two);
    // Three
    dispatcher.register(op::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(deop::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(kick::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(plugin::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(plugins::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(ban::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(banip::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(banlist::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(pardon::init_command_tree(), PermissionLvl::Three);
    dispatcher.register(pardonip::init_command_tree(), PermissionLvl::Three);
    // Four
    dispatcher.register(stop::init_command_tree(), PermissionLvl::Four);

    dispatcher
}