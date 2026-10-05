use super::AccessorExt;
use crate::pumpkin::plugin::boss_bar::{
    self, BossBar, BossBarColor as WitColor, BossBarDivision as WitDivision,
    BossBarMetadata as WitMetadata,
};
use pumpkin_core::{
    server::Server,
    world::bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags},
};
use pumpkin_wasm_host_common::state::PluginHostState;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;
use uuid::Uuid;
use wasmtime::component::{Accessor, HasSelf, Resource};

pub struct PluginBossBar {
    pub bossbar: Bossbar,
    pub players: Vec<Uuid>,
    pub server: Weak<Server>,
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

async fn remove_all_players(pbb: &Arc<Mutex<PluginBossBar>>) {
    let pbb = pbb.lock().await;
    if let Some(server) = pbb.server.upgrade() {
        for uuid in &pbb.players {
            if let Some(player) = server.get_player_by_uuid(*uuid) {
                player.remove_bossbar(pbb.bossbar.uuid);
            }
        }
    }
}

impl boss_bar::Host for PluginHostState {}

impl boss_bar::HostBossBar for PluginHostState {
    async fn drop(&mut self, res: Resource<BossBar>) -> wasmtime::Result<()> {
        let pbb_handle = self.get(&res)?.clone();
        remove_all_players(&pbb_handle).await;
        self.drop(res)
    }

    fn create(
        &mut self,
        title: Resource<crate::pumpkin::plugin::text::TextComponent>,
        color: WitColor,
        division: WitDivision,
    ) -> wasmtime::Result<Resource<BossBar>> {
        let state = self;
        let title = state.take(title)?;
        let mut bossbar = Bossbar::new(title);

        bossbar.color = from_wit_color(color);
        bossbar.division = from_wit_division(division);

        let server = state
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("server not available"))?
            .clone();
        let plugin_bossbar = Arc::new(Mutex::new(PluginBossBar::new(
            bossbar,
            Arc::downgrade(&server),
        )));
        state.add(plugin_bossbar)
    }
}

impl boss_bar::HostBossBarWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_title(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<Resource<crate::pumpkin::plugin::text::TextComponent>> {
        let title = accessor.get_res(&res)?.lock().await.bossbar.title.clone();
        accessor.add_res(title)
    }

    async fn set_title(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        title: Resource<crate::pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let title = accessor.take_res(title)?;
        let pbb_handle = accessor.get_res(&res)?;
        let mut pbb = pbb_handle.lock().await;
        pbb.bossbar.title = title.clone();
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player.update_bossbar_title(&pbb.bossbar.uuid, title.clone());
                }
            }
        }
        Ok(())
    }

    async fn get_health(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<f32> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.lock().await;
        Ok(pbb.bossbar.health)
    }

    async fn set_health(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        health: f32,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        let mut pbb = pbb_handle.lock().await;
        pbb.bossbar.health = health;
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player.update_bossbar_health(&pbb.bossbar.uuid, health);
                }
            }
        }
        Ok(())
    }

    async fn get_color(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<WitColor> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.lock().await;
        Ok(to_wit_color(pbb.bossbar.color))
    }

    async fn set_color(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        color: WitColor,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        let mut pbb = pbb_handle.lock().await;
        pbb.bossbar.color = from_wit_color(color);
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player.update_bossbar_style(
                        &pbb.bossbar.uuid,
                        pbb.bossbar.color,
                        pbb.bossbar.division,
                    );
                }
            }
        }
        Ok(())
    }

    async fn get_division(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<WitDivision> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.lock().await;
        Ok(to_wit_division(pbb.bossbar.division))
    }

    async fn set_division(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        division: WitDivision,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        let mut pbb = pbb_handle.lock().await;
        pbb.bossbar.division = from_wit_division(division);
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player.update_bossbar_style(
                        &pbb.bossbar.uuid,
                        pbb.bossbar.color,
                        pbb.bossbar.division,
                    );
                }
            }
        }
        Ok(())
    }

    async fn get_metadata(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<WitMetadata> {
        let pbb_handle = accessor.get_res(&res)?;
        let pbb = pbb_handle.lock().await;
        Ok(to_wit_metadata(pbb.bossbar.flags))
    }

    async fn set_metadata(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        metadata: WitMetadata,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        let mut pbb = pbb_handle.lock().await;
        pbb.bossbar.flags = from_wit_metadata(metadata);
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player.update_bossbar_flags(&pbb.bossbar.uuid, pbb.bossbar.flags);
                }
            }
        }
        Ok(())
    }

    async fn get_players(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<Vec<Resource<crate::pumpkin::plugin::player::Player>>> {
        let players = {
            let pbb_handle = accessor.get_res(&res)?;
            let pbb = pbb_handle.lock().await;
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
        let mut pbb = pbb_handle.lock().await;
        let uuid = player.gameprofile.id;

        if !pbb.players.contains(&uuid) {
            pbb.players.push(uuid);
            player.send_bossbar(&pbb.bossbar);
        }
        Ok(())
    }

    async fn remove_player(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
        player: Resource<crate::pumpkin::plugin::player::Player>,
    ) -> wasmtime::Result<()> {
        let player = accessor.take_res(player)?;
        let pbb_handle = accessor.get_res(&res)?;
        let mut pbb = pbb_handle.lock().await;
        let uuid = player.gameprofile.id;

        if let Some(idx) = pbb.players.iter().position(|&x| x == uuid) {
            pbb.players.remove(idx);
            player.remove_bossbar(pbb.bossbar.uuid);
        }
        Ok(())
    }

    async fn remove_all(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<()> {
        let pbb_handle = accessor.get_res(&res)?;
        remove_all_players(&pbb_handle).await;
        Ok(())
    }
}
