use super::{AccessorExt, LockIgnorePoison, run_blocking};
use crate::RequireServer;

use crate::pumpkin::plugin::boss_bar::{
    self, BossBar, BossBarColor as WitColor, BossBarDivision as WitDivision,
    BossBarMetadata as WitMetadata,
};
use pumpkin_core::server::Server;
use pumpkin_core::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags};
use pumpkin_wasm_host_common::state::PluginHostState;
use std::sync::{Arc, Mutex, Weak};
use uuid::Uuid;
use wasmtime::component::{Accessor, HasSelf, Resource};

pub struct PluginBossBar {
    pub bossbar: Bossbar,
    pub players: Vec<Uuid>,
    pub server: Weak<Server>,
}

/// Thread-safe handle for a plugin boss bar.
///
/// `send_lock` serializes mutating updates and their outbound packet sends so clients observe
/// changes in state-mutation order, while `state` is never held across packet sends so packet
/// observers reading boss bar getters cannot deadlock.
pub struct PluginBossBarHandle {
    state: Mutex<PluginBossBar>,
    send_lock: Mutex<()>,
}

impl PluginBossBar {
    #[must_use]
    pub const fn new(bossbar: Bossbar, server: Weak<Server>) -> Self {
        Self {
            bossbar,
            players: Vec::new(),
            server,
        }
    }
}

impl PluginBossBarHandle {
    #[must_use]
    pub const fn new(bossbar: Bossbar, server: Weak<Server>) -> Self {
        Self {
            state: Mutex::new(PluginBossBar::new(bossbar, server)),
            send_lock: Mutex::new(()),
        }
    }
}

const fn to_wit_color(color: BossbarColor) -> WitColor {
    match color {
        BossbarColor::Pink => WitColor::Pink,
        BossbarColor::Blue => WitColor::Blue,
        BossbarColor::Red => WitColor::Red,
        BossbarColor::Green => WitColor::Green,
        BossbarColor::Yellow => WitColor::Yellow,
        BossbarColor::Purple => WitColor::Purple,
        BossbarColor::White => WitColor::White,
    }
}

const fn from_wit_color(color: WitColor) -> BossbarColor {
    match color {
        WitColor::Pink => BossbarColor::Pink,
        WitColor::Blue => BossbarColor::Blue,
        WitColor::Red => BossbarColor::Red,
        WitColor::Green => BossbarColor::Green,
        WitColor::Yellow => BossbarColor::Yellow,
        WitColor::Purple => BossbarColor::Purple,
        WitColor::White => BossbarColor::White,
    }
}

const fn to_wit_division(division: BossbarDivisions) -> WitDivision {
    match division {
        BossbarDivisions::NoDivision => WitDivision::NoDivision,
        BossbarDivisions::Notches6 => WitDivision::Notches6,
        BossbarDivisions::Notches10 => WitDivision::Notches10,
        BossbarDivisions::Notches12 => WitDivision::Notches12,
        BossbarDivisions::Notches20 => WitDivision::Notches20,
    }
}

const fn from_wit_division(division: WitDivision) -> BossbarDivisions {
    match division {
        WitDivision::NoDivision => BossbarDivisions::NoDivision,
        WitDivision::Notches6 => BossbarDivisions::Notches6,
        WitDivision::Notches10 => BossbarDivisions::Notches10,
        WitDivision::Notches12 => BossbarDivisions::Notches12,
        WitDivision::Notches20 => BossbarDivisions::Notches20,
    }
}

const fn to_wit_metadata(flags: BossbarFlags) -> WitMetadata {
    WitMetadata {
        darken_sky: flags.contains(BossbarFlags::DARKEN_SKY),
        dragon_bar: flags.contains(BossbarFlags::DRAGON_BAR),
        create_fog: flags.contains(BossbarFlags::CREATE_FOG),
    }
}

fn from_wit_metadata(metadata: WitMetadata) -> BossbarFlags {
    let mut f = BossbarFlags::empty();
    if metadata.darken_sky {
        f |= BossbarFlags::DARKEN_SKY;
    }
    if metadata.dragon_bar {
        f |= BossbarFlags::DRAGON_BAR;
    }
    if metadata.create_fog {
        f |= BossbarFlags::CREATE_FOG;
    }
    f
}

fn remove_all_players(handle: &PluginBossBarHandle) {
    let _send_guard = handle.send_lock.lock_ignore_poison();
    let (uuid, players, server) = {
        let mut pbb = handle.state.lock_ignore_poison();
        (
            pbb.bossbar.uuid,
            std::mem::take(&mut pbb.players),
            pbb.server.upgrade(),
        )
    };
    if let Some(server) = server {
        for player_uuid in players {
            if let Some(player) = server.get_player_by_uuid(player_uuid) {
                player.remove_bossbar(uuid);
            }
        }
    }
}

impl boss_bar::Host for PluginHostState {}

impl boss_bar::HostBossBar for PluginHostState {
    fn new(
        &mut self,
        title: Resource<crate::pumpkin::plugin::text::TextComponent>,
        color: WitColor,
        division: WitDivision,
    ) -> wasmtime::Result<Resource<BossBar>> {
        let title = self.take(title)?;
        let mut bossbar = Bossbar::new(title);

        bossbar.color = from_wit_color(color);
        bossbar.division = from_wit_division(division);

        let server = self.require_server()?.clone();
        let plugin_bossbar = Arc::new(PluginBossBarHandle::new(bossbar, Arc::downgrade(&server)));
        self.add(plugin_bossbar)
    }

    fn drop(&mut self, res: Resource<BossBar>) -> wasmtime::Result<()> {
        let pbb_handle = self.get(&res)?.clone();
        remove_all_players(&pbb_handle);
        self.drop(res)
    }
}

impl boss_bar::HostBossBarWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_title(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<Resource<crate::pumpkin::plugin::text::TextComponent>> {
        let title = accessor
            .get_res(&res)?
            .state
            .lock_ignore_poison()
            .bossbar
            .title
            .clone();
        accessor.add_res(title)
    }

    async fn set_title(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        title: Resource<crate::pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let title = accessor.take_res(title)?;
        let pbb_handle = accessor.get_res(&res)?;
        run_blocking(accessor, move || {
            let _send_guard = pbb_handle.send_lock.lock_ignore_poison();
            let (uuid, players, server) = {
                let mut pbb = pbb_handle.state.lock_ignore_poison();
                pbb.bossbar.title = title.clone();
                (pbb.bossbar.uuid, pbb.players.clone(), pbb.server.upgrade())
            };
            if let Some(server) = server {
                for player_uuid in players {
                    if let Some(player) = server.get_player_by_uuid(player_uuid) {
                        player.update_bossbar_title(&uuid, title.clone());
                    }
                }
            }
        })
        .await
    }

    async fn get_health(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<f32> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.state.lock_ignore_poison();
        Ok(pbb.bossbar.health)
    }

    async fn set_health(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        health: f32,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        run_blocking(accessor, move || {
            let _send_guard = pbb_handle.send_lock.lock_ignore_poison();
            let (uuid, players, server) = {
                let mut pbb = pbb_handle.state.lock_ignore_poison();
                pbb.bossbar.health = health;
                (pbb.bossbar.uuid, pbb.players.clone(), pbb.server.upgrade())
            };
            if let Some(server) = server {
                for player_uuid in players {
                    if let Some(player) = server.get_player_by_uuid(player_uuid) {
                        player.update_bossbar_health(&uuid, health);
                    }
                }
            }
        })
        .await
    }

    async fn get_color(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<WitColor> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.state.lock_ignore_poison();
        Ok(to_wit_color(pbb.bossbar.color))
    }

    async fn set_color(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        color: WitColor,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        let color = from_wit_color(color);
        run_blocking(accessor, move || {
            let _send_guard = pbb_handle.send_lock.lock_ignore_poison();
            let (uuid, division, players, server) = {
                let mut pbb = pbb_handle.state.lock_ignore_poison();
                pbb.bossbar.color = color;
                (
                    pbb.bossbar.uuid,
                    pbb.bossbar.division,
                    pbb.players.clone(),
                    pbb.server.upgrade(),
                )
            };
            if let Some(server) = server {
                for player_uuid in players {
                    if let Some(player) = server.get_player_by_uuid(player_uuid) {
                        player.update_bossbar_style(&uuid, color, division);
                    }
                }
            }
        })
        .await
    }

    async fn get_division(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<WitDivision> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.state.lock_ignore_poison();
        Ok(to_wit_division(pbb.bossbar.division))
    }

    async fn set_division(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        division: WitDivision,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        let division = from_wit_division(division);
        run_blocking(accessor, move || {
            let _send_guard = pbb_handle.send_lock.lock_ignore_poison();
            let (uuid, color, players, server) = {
                let mut pbb = pbb_handle.state.lock_ignore_poison();
                pbb.bossbar.division = division;
                (
                    pbb.bossbar.uuid,
                    pbb.bossbar.color,
                    pbb.players.clone(),
                    pbb.server.upgrade(),
                )
            };
            if let Some(server) = server {
                for player_uuid in players {
                    if let Some(player) = server.get_player_by_uuid(player_uuid) {
                        player.update_bossbar_style(&uuid, color, division);
                    }
                }
            }
        })
        .await
    }

    async fn get_metadata(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<WitMetadata> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.state.lock_ignore_poison();
        Ok(to_wit_metadata(pbb.bossbar.flags))
    }

    async fn set_metadata(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        metadata: WitMetadata,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        let flags = from_wit_metadata(metadata);
        run_blocking(accessor, move || {
            let _send_guard = pbb_handle.send_lock.lock_ignore_poison();
            let (uuid, players, server) = {
                let mut pbb = pbb_handle.state.lock_ignore_poison();
                pbb.bossbar.flags = flags;
                (pbb.bossbar.uuid, pbb.players.clone(), pbb.server.upgrade())
            };
            if let Some(server) = server {
                for player_uuid in players {
                    if let Some(player) = server.get_player_by_uuid(player_uuid) {
                        player.update_bossbar_flags(&uuid, flags);
                    }
                }
            }
        })
        .await
    }

    async fn get_players(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<Vec<Resource<crate::pumpkin::plugin::player::Player>>> {
        let players = {
            let pbb_handle = accessor.get_res(&res)?;
            let pbb = pbb_handle.state.lock_ignore_poison();
            pbb.players.clone()
        };

        let server = accessor.server()?;

        let mut wit_players = Vec::new();
        for uuid in players {
            if let Some(player) = server.get_player_by_uuid(uuid) {
                wit_players.push(accessor.add_res(player)?);
            }
        }
        Ok(wit_players)
    }

    async fn add_player(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        player: Resource<crate::pumpkin::plugin::player::Player>,
    ) -> wasmtime::Result<()> {
        let player = accessor.take_res(player)?;
        let pbb_handle = accessor.get_res(&res)?;
        run_blocking(accessor, move || {
            let _send_guard = pbb_handle.send_lock.lock_ignore_poison();
            let bossbar = {
                let mut pbb = pbb_handle.state.lock_ignore_poison();
                let uuid = player.gameprofile.id;
                if pbb.players.contains(&uuid) {
                    None
                } else {
                    pbb.players.push(uuid);
                    Some(pbb.bossbar.clone())
                }
            };
            if let Some(bossbar) = bossbar {
                player.send_bossbar(&bossbar);
            }
        })
        .await
    }

    async fn remove_player(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        player: Resource<crate::pumpkin::plugin::player::Player>,
    ) -> wasmtime::Result<()> {
        let player = accessor.take_res(player)?;
        let pbb_handle = accessor.get_res(&res)?;
        run_blocking(accessor, move || {
            let _send_guard = pbb_handle.send_lock.lock_ignore_poison();
            let bossbar_uuid = {
                let mut pbb = pbb_handle.state.lock_ignore_poison();
                let uuid = player.gameprofile.id;
                pbb.players.iter().position(|&x| x == uuid).map(|idx| {
                    pbb.players.remove(idx);
                    pbb.bossbar.uuid
                })
            };
            if let Some(bossbar_uuid) = bossbar_uuid {
                player.remove_bossbar(bossbar_uuid);
            }
        })
        .await
    }

    async fn remove_all(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        run_blocking(accessor, move || remove_all_players(&pbb_handle)).await
    }
}
