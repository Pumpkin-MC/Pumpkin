use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, TablistResource},
    wit::v0_1_0::pumpkin,
};
use pumpkin_protocol::java::client::play::CTabList;

fn text_component_from_resource(
    state: &PluginHostState,
    text: &Resource<pumpkin::plugin::text::TextComponent>,
) -> pumpkin_util::text::TextComponent {
    state
        .resource_table
        .get::<crate::plugin::loader::wasm::wasm_host::state::TextComponentResource>(
            &Resource::new_own(text.rep()),
        )
        .expect("invalid text-component resource handle")
        .provider
        .clone()
}

fn player_from_resource(
    state: &PluginHostState,
    tablist: &Resource<pumpkin::plugin::tablist::Tablist>,
) -> wasmtime::Result<std::sync::Arc<crate::entity::player::Player>> {
    state
        .resource_table
        .get::<TablistResource>(&Resource::new_own(tablist.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid tablist resource handle"))
        .map(|resource| resource.provider.clone())
}

impl pumpkin::plugin::tablist::Host for PluginHostState {}

impl pumpkin::plugin::tablist::HostTablist for PluginHostState {
    async fn set_header(
        &mut self,
        tablist: Resource<pumpkin::plugin::tablist::Tablist>,
        header: Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &tablist)?;
        let header = text_component_from_resource(self, &header);
        let empty = pumpkin_util::text::TextComponent::text("");
        player
            .client
            .enqueue_packet(&CTabList::new(&header, &empty))
            .await;
        Ok(())
    }

    async fn set_footer(
        &mut self,
        tablist: Resource<pumpkin::plugin::tablist::Tablist>,
        footer: Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &tablist)?;
        let footer = text_component_from_resource(self, &footer);
        let empty = pumpkin_util::text::TextComponent::text("");
        player
            .client
            .enqueue_packet(&CTabList::new(&empty, &footer))
            .await;
        Ok(())
    }

    async fn set_header_and_footer(
        &mut self,
        tablist: Resource<pumpkin::plugin::tablist::Tablist>,
        header: Resource<pumpkin::plugin::text::TextComponent>,
        footer: Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &tablist)?;
        let header = text_component_from_resource(self, &header);
        let footer = text_component_from_resource(self, &footer);
        player
            .client
            .enqueue_packet(&CTabList::new(&header, &footer))
            .await;
        Ok(())
    }

    async fn clear_header(
        &mut self,
        tablist: Resource<pumpkin::plugin::tablist::Tablist>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &tablist)?;
        let empty = pumpkin_util::text::TextComponent::text("");
        // We need to keep the current footer, but for now we just clear the header
        // A proper implementation would store header/footer state
        player
            .client
            .enqueue_packet(&CTabList::new(&empty, &empty))
            .await;
        Ok(())
    }

    async fn clear_footer(
        &mut self,
        tablist: Resource<pumpkin::plugin::tablist::Tablist>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &tablist)?;
        let empty = pumpkin_util::text::TextComponent::text("");
        player
            .client
            .enqueue_packet(&CTabList::new(&empty, &empty))
            .await;
        Ok(())
    }

    async fn clear_all(
        &mut self,
        tablist: Resource<pumpkin::plugin::tablist::Tablist>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &tablist)?;
        let empty = pumpkin_util::text::TextComponent::text("");
        player
            .client
            .enqueue_packet(&CTabList::new(&empty, &empty))
            .await;
        Ok(())
    }

    async fn drop(
        &mut self,
        rep: Resource<pumpkin::plugin::tablist::Tablist>,
    ) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<TablistResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
