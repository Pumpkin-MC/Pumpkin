use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::{
        self,
        plugin::text::{ArgbColor, NamedColor, RgbColor, TextComponent},
    },
};

impl pumpkin::plugin::text::Host for PluginHostState {}

impl pumpkin::plugin::text::HostTextComponent for PluginHostState {
    async fn text(&mut self, plain: String) -> Resource<TextComponent> {
        todo!()
    }

    async fn translate(
        &mut self,
        key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> Resource<TextComponent> {
        todo!()
    }

    async fn add_child(
        &mut self,
        text_component: Resource<TextComponent>,
        child: Resource<TextComponent>,
    ) -> () {
        todo!()
    }

    async fn add_text(&mut self, text_component: Resource<TextComponent>, text: String) -> () {
        todo!()
    }

    async fn get_text(&mut self, text_component: Resource<TextComponent>) -> String {
        todo!()
    }

    async fn encode(&mut self, text_component: Resource<TextComponent>) -> Vec<u8> {
        todo!()
    }

    async fn color_named(
        &mut self,
        text_component: Resource<TextComponent>,
        color: NamedColor,
    ) -> () {
        todo!()
    }

    async fn color_rgb(&mut self, text_component: Resource<TextComponent>, color: RgbColor) -> () {
        todo!()
    }

    async fn bold(&mut self, text_component: Resource<TextComponent>, value: bool) -> () {
        todo!()
    }

    async fn italic(&mut self, text_component: Resource<TextComponent>, value: bool) -> () {
        todo!()
    }

    async fn underlined(&mut self, text_component: Resource<TextComponent>, value: bool) -> () {
        todo!()
    }

    async fn strikethrough(&mut self, text_component: Resource<TextComponent>, value: bool) -> () {
        todo!()
    }

    async fn obfuscated(&mut self, text_component: Resource<TextComponent>, value: bool) -> () {
        todo!()
    }

    async fn insertion(&mut self, text_component: Resource<TextComponent>, text: String) -> () {
        todo!()
    }

    async fn font(&mut self, text_component: Resource<TextComponent>, font: String) -> () {
        todo!()
    }

    async fn shadow_color(
        &mut self,
        text_component: Resource<TextComponent>,
        color: ArgbColor,
    ) -> () {
        todo!()
    }

    async fn click_open_url(&mut self, text_component: Resource<TextComponent>, url: String) -> () {
        todo!()
    }

    async fn click_run_command(
        &mut self,
        text_component: Resource<TextComponent>,
        command: String,
    ) -> () {
        todo!()
    }

    async fn click_suggest_command(
        &mut self,
        text_component: Resource<TextComponent>,
        command: String,
    ) -> () {
        todo!()
    }

    async fn click_copy_to_clipboard(
        &mut self,
        text_component: Resource<TextComponent>,
        text: String,
    ) -> () {
        todo!()
    }

    async fn hover_show_text(
        &mut self,
        text_component: Resource<TextComponent>,
        text: Resource<TextComponent>,
    ) -> () {
        todo!()
    }

    async fn hover_show_item(
        &mut self,
        text_component: Resource<TextComponent>,
        item: String,
    ) -> () {
        todo!()
    }

    async fn hover_show_entity(
        &mut self,
        text_component: Resource<TextComponent>,
        entity_type: String,
        id: String,
        name: Option<Resource<TextComponent>>,
    ) -> () {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<TextComponent>) -> wasmtime::Result<()> {
        todo!()
    }
}
