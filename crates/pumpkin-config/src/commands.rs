use std::collections::HashMap;

use pumpkin_util::PermissionLvl;
use serde::{Deserialize, Serialize};

/// Configuration for command handling and execution.
///
/// Controls how commands are accepted, logged, and which permission
/// level non-operator players receive by default.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct CommandsConfig {
    /// Whether commands from the console are accepted.
    pub use_console: bool,
    /// Whether to use rusty line for tty input.
    pub use_tty: bool,
    /// Whether commands from players are logged in the console.
    pub log_console: bool,
    /// Whether console and RCON command output is broadcast to online operators.
    /// Corresponds to vanilla's `broadcast-console-to-ops` server property.
    pub broadcast_console_to_ops: bool,
    /// The `op` permission level of everyone that is not in the `ops` file.
    pub default_op_level: PermissionLvl,
    /// Per-command settings, so you can turn individual commands off or change
    /// who is allowed to use them.
    ///
    /// Each entry is named after the command it applies to (without the leading
    /// slash), for example `gamemode` or `tp`. You only need to list the
    /// commands you actually want to change; everything you leave out keeps its
    /// normal behaviour.
    ///
    /// Example:
    ///
    /// ```toml
    /// # Only server owners may change gamemodes
    /// [commands.overrides.gamemode]
    /// permission_level = 4
    ///
    /// # Turn the /tp command off completely
    /// [commands.overrides.tp]
    /// enabled = false
    /// ```
    pub overrides: HashMap<String, CommandOverride>,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            use_console: true,
            log_console: true,
            use_tty: true,
            broadcast_console_to_ops: true,
            default_op_level: PermissionLvl::Zero,
            overrides: HashMap::new(),
        }
    }
}

/// Settings for a single command, letting a server owner turn it off or change
/// who is allowed to run it.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct CommandOverride {
    /// Whether this command can be used at all. When set to `false` the command
    /// is hidden completely: it won't run, won't show up in the command list,
    /// and won't appear in tab-completion. Players who try to use it simply get
    /// the normal "unknown command" message. Set to `true` (the default) to
    /// leave the command available.
    pub enabled: bool,
    /// Who is allowed to use this command, given as a permission level:
    ///
    /// - `0` = everyone can use it
    /// - `2` = normal operators (the usual level for most cheat-style commands)
    /// - `3` = admins (player management, kicking, banning, and so on)
    /// - `4` = the server owner only (full server management)
    ///
    /// Leave this out to keep the command's normal requirement.
    pub permission_level: Option<PermissionLvl>,
}

impl Default for CommandOverride {
    fn default() -> Self {
        Self {
            enabled: true,
            permission_level: None,
        }
    }
}
