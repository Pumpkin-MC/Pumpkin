use crate::plugin::loader::wasm::wasm_host::{
    state::{BossBarResource, PlayerResource, PluginHostState},
    wit::v0_1::pumpkin::plugin::boss_bar::{
        self, BossBar, BossBarColor as WitColor, BossBarDivision as WitDivision,
        BossBarMetadata as WitMetadata,
    },
};
use crate::server::Server;
use crate::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags};
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;
use uuid::Uuid;
use wasmtime::component::Resource;

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

fn player_from_resource(
    state: &PluginHostState,
    player: &Resource<
        crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::player::Player,
    >,
) -> wasmtime::Result<std::sync::Arc<crate::entity::player::Player>> {
    state
        .resource_table
        .get::<PlayerResource>(&Resource::new_own(player.rep()))
        .map_err(wasmtime::Error::from)
        .map(|resource| resource.provider.clone())
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

impl PluginHostState {
    fn get_bossbar_res(&self, res: &Resource<BossBar>) -> wasmtime::Result<&BossBarResource> {
        self.resource_table
            .get::<BossBarResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl boss_bar::Host for PluginHostState {}

impl boss_bar::HostBossBar for PluginHostState {
    async fn new(
        &mut self,
        title: Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::text::TextComponent,
        >,
        color: WitColor,
        division: WitDivision,
    ) -> wasmtime::Result<Resource<BossBar>> {
        let title = self.get_text_provider(&title)?;
        let mut bossbar = Bossbar::new(title);

        bossbar.color = from_wit_color(color);
        bossbar.division = from_wit_division(division);

        let server = self.server.as_ref().expect("server not available").clone();
        let plugin_bossbar = Arc::new(Mutex::new(PluginBossBar::new(
            bossbar,
            Arc::downgrade(&server),
        )));
        self.add_boss_bar(plugin_bossbar)
    }

    async fn get_title(
        &mut self,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<
        Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::text::TextComponent,
        >,
    > {
        let title = {
            let bossbar = self.get_bossbar_res(&res)?.provider.lock().await;
            bossbar.bossbar.title.clone()
        };
        self.add_text_component(title)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn set_title(
        &mut self,
        res: Resource<BossBar>,
        title: Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::text::TextComponent,
        >,
    ) -> wasmtime::Result<()> {
        let title = self.get_text_provider(&title)?;
        let mut pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        pbb.bossbar.title = title.clone();
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player
                        .update_bossbar_title(&pbb.bossbar.uuid, title.clone())
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn get_health(&mut self, res: Resource<BossBar>) -> wasmtime::Result<f32> {
        let pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        Ok(pbb.bossbar.health)
    }

    async fn set_health(&mut self, res: Resource<BossBar>, health: f32) -> wasmtime::Result<()> {
        let mut pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        pbb.bossbar.health = health;
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player
                        .update_bossbar_health(&pbb.bossbar.uuid, health)
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn get_color(&mut self, res: Resource<BossBar>) -> wasmtime::Result<WitColor> {
        let pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        Ok(to_wit_color(pbb.bossbar.color))
    }

    async fn set_color(&mut self, res: Resource<BossBar>, color: WitColor) -> wasmtime::Result<()> {
        let mut pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        pbb.bossbar.color = from_wit_color(color);
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player
                        .update_bossbar_style(
                            &pbb.bossbar.uuid,
                            pbb.bossbar.color,
                            pbb.bossbar.division,
                        )
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn get_division(&mut self, res: Resource<BossBar>) -> wasmtime::Result<WitDivision> {
        let pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        Ok(to_wit_division(pbb.bossbar.division))
    }

    async fn set_division(
        &mut self,
        res: Resource<BossBar>,
        division: WitDivision,
    ) -> wasmtime::Result<()> {
        let mut pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        pbb.bossbar.division = from_wit_division(division);
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player
                        .update_bossbar_style(
                            &pbb.bossbar.uuid,
                            pbb.bossbar.color,
                            pbb.bossbar.division,
                        )
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn get_metadata(&mut self, res: Resource<BossBar>) -> wasmtime::Result<WitMetadata> {
        let pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        Ok(to_wit_metadata(pbb.bossbar.flags))
    }

    async fn set_metadata(
        &mut self,
        res: Resource<BossBar>,
        metadata: WitMetadata,
    ) -> wasmtime::Result<()> {
        let mut pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        pbb.bossbar.flags = from_wit_metadata(metadata);
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player
                        .update_bossbar_flags(&pbb.bossbar.uuid, pbb.bossbar.flags)
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn get_players(
        &mut self,
        res: Resource<BossBar>,
    ) -> wasmtime::Result<
        Vec<
            Resource<
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::player::Player,
            >,
        >,
    > {
        let players = {
            let pbb = self.get_bossbar_res(&res)?.provider.lock().await;
            pbb.players.clone()
        };

        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        let mut wit_players = Vec::new();
        for uuid in players {
            if let Some(player) = server.get_player_by_uuid(uuid) {
                wit_players.push(self.add_player(player)?);
            }
        }
        Ok(wit_players)
    }

    async fn add_player(
        &mut self,
        res: Resource<BossBar>,
        player: Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::player::Player,
        >,
    ) -> wasmtime::Result<()> {
        let mut pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        let player = player_from_resource(self, &player)?;
        let uuid = player.gameprofile.id;

        if !pbb.players.contains(&uuid) {
            pbb.players.push(uuid);
            player.send_bossbar(&pbb.bossbar).await;
        }
        Ok(())
    }

    async fn remove_player(
        &mut self,
        res: Resource<BossBar>,
        player: Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::player::Player,
        >,
    ) -> wasmtime::Result<()> {
        let mut pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        let player = player_from_resource(self, &player)?;
        let uuid = player.gameprofile.id;

        if let Some(idx) = pbb.players.iter().position(|&x| x == uuid) {
            pbb.players.remove(idx);
            player.remove_bossbar(pbb.bossbar.uuid).await;
        }
        Ok(())
    }

    async fn remove_all(&mut self, res: Resource<BossBar>) -> wasmtime::Result<()> {
        let pbb = self.get_bossbar_res(&res)?.provider.lock().await;
        if let Some(server) = pbb.server.upgrade() {
            for uuid in &pbb.players {
                if let Some(player) = server.get_player_by_uuid(*uuid) {
                    player.remove_bossbar(pbb.bossbar.uuid).await;
                }
            }
        }
        Ok(())
    }

    async fn drop(&mut self, res: Resource<BossBar>) -> wasmtime::Result<()> {
        let rep = res.rep();
        self.remove_all(res).await?;
        self.resource_table
            .delete::<BossBarResource>(Resource::new_own(rep))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
