use std::fmt;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use args::ConsumedArgs;
use async_trait::async_trait;
use dispatcher::CommandError;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::text::TextComponent;

pub mod args;
pub mod client_suggestions;
pub mod commands;
pub mod dispatcher;
pub mod tree;

pub enum CommandSender<'a> {
    Rcon(&'a tokio::sync::Mutex<Vec<String>>),
    Console,
    Player(Arc<Player>),
}

impl fmt::Display for CommandSender<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CommandSender::Console => "Server",
                CommandSender::Rcon(_) => "Rcon",
                CommandSender::Player(p) => &p.gameprofile.name,
            }
        )
    }
}

impl CommandSender<'_> {
    pub async fn send_message(&self, text: TextComponent) {
        match self {
            CommandSender::Console => log::info!("{}", text.to_pretty_console()),
            CommandSender::Player(c) => c.send_system_message(&text).await,
            CommandSender::Rcon(s) => s.lock().await.push(text.to_pretty_console()),
        }
    }

    #[must_use]
    pub const fn is_player(&self) -> bool {
        matches!(self, CommandSender::Player(_))
    }

    #[must_use]
    pub const fn is_console(&self) -> bool {
        matches!(self, CommandSender::Console)
    }
    #[must_use]
    pub fn as_player(&self) -> Option<Arc<Player>> {
        match self {
            CommandSender::Player(player) => Some(player.clone()),
            _ => None,
        }
    }

    /// prefer using `has_permission_lvl(lvl)`
    #[must_use]
    pub fn permission_lvl(&self) -> PermissionLvl {
        match self {
            CommandSender::Console | CommandSender::Rcon(_) => PermissionLvl::Four,
            CommandSender::Player(p) => p.permission_lvl.load(),
        }
    }

    #[must_use]
    pub fn has_permission_lvl(&self, lvl: PermissionLvl) -> bool {
        match self {
            CommandSender::Console | CommandSender::Rcon(_) => true,
            CommandSender::Player(p) => p.permission_lvl.load().ge(&lvl),
        }
    }

    #[must_use]
    pub fn position(&self) -> Option<Vector3<f64>> {
        match self {
            CommandSender::Console | CommandSender::Rcon(..) => None,
            CommandSender::Player(p) => Some(p.living_entity.entity.pos.load()),
        }
    }

    #[must_use]
    pub async fn world(&self) -> Option<Arc<World>> {
        match self {
            // TODO: maybe return first world when console
            CommandSender::Console | CommandSender::Rcon(..) => None,
            CommandSender::Player(p) => Some(p.living_entity.entity.world.read().await.clone()),
        }
    }
}

#[async_trait]
pub trait CommandExecutor: Sync {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError>;
}
