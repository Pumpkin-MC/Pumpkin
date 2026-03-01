use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, TextComponentResource},
    wit::v0_1_0::pumpkin::{
        self,
        plugin::text::{ArgbColor, NamedColor, RgbColor, TextComponent},
    },
};

impl pumpkin::plugin::text::Host for PluginHostState {}

trait TextComponentResourceExt {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a TextComponentResource;
    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut TextComponentResource;
    fn consume(self, state: &mut PluginHostState) -> TextComponentResource;
}

impl TextComponentResourceExt for Resource<TextComponent> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a TextComponentResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid server resource handle")
            .downcast_ref::<TextComponentResource>()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut TextComponentResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid server resource handle")
            .downcast_mut::<TextComponentResource>()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> TextComponentResource {
        state
            .resource_table
            .delete::<TextComponentResource>(Resource::new_own(self.rep()))
            .expect("invalid child resource handle")
    }
}

impl pumpkin::plugin::text::HostTextComponent for PluginHostState {
    async fn text(&mut self, plain: String) -> Resource<TextComponent> {
        let text_component = pumpkin_util::text::TextComponent::text(plain);
        self.add_text_component(text_component).unwrap()
    }

    async fn translate(
        &mut self,
        key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> Resource<TextComponent> {
        let with: Vec<pumpkin_util::text::TextComponent> = with
            .into_iter()
            .map(|component| component.consume(self).provider)
            .collect();
        let text_component = pumpkin_util::text::TextComponent::translate(key, with);
        self.add_text_component(text_component).unwrap()
    }

    async fn add_child(
        &mut self,
        text_component: Resource<TextComponent>,
        child: Resource<TextComponent>,
    ) {
        let child = child.consume(self).provider;
        let parent = &mut text_component.downcast_mut(self).provider;

        // The current builder pattern for text components doesn't accept &mut references of self.
        *parent = parent.clone().add_child(child);
    }

    async fn add_text(&mut self, text_component: Resource<TextComponent>, text: String) {
        todo!()
    }

    async fn get_text(&mut self, text_component: Resource<TextComponent>) -> String {
        todo!()
    }

    async fn encode(&mut self, text_component: Resource<TextComponent>) -> Vec<u8> {
        todo!()
    }

    async fn color_named(&mut self, text_component: Resource<TextComponent>, color: NamedColor) {
        todo!()
    }

    async fn color_rgb(&mut self, text_component: Resource<TextComponent>, color: RgbColor) {
        todo!()
    }

    async fn bold(&mut self, text_component: Resource<TextComponent>, value: bool) {
        todo!()
    }

    async fn italic(&mut self, text_component: Resource<TextComponent>, value: bool) {
        todo!()
    }

    async fn underlined(&mut self, text_component: Resource<TextComponent>, value: bool) {
        todo!()
    }

    async fn strikethrough(&mut self, text_component: Resource<TextComponent>, value: bool) {
        todo!()
    }

    async fn obfuscated(&mut self, text_component: Resource<TextComponent>, value: bool) {
        todo!()
    }

    async fn insertion(&mut self, text_component: Resource<TextComponent>, text: String) {
        todo!()
    }

    async fn font(&mut self, text_component: Resource<TextComponent>, font: String) {
        todo!()
    }

    async fn shadow_color(&mut self, text_component: Resource<TextComponent>, color: ArgbColor) {
        todo!()
    }

    async fn click_open_url(&mut self, text_component: Resource<TextComponent>, url: String) {
        todo!()
    }

    async fn click_run_command(
        &mut self,
        text_component: Resource<TextComponent>,
        command: String,
    ) {
        todo!()
    }

    async fn click_suggest_command(
        &mut self,
        text_component: Resource<TextComponent>,
        command: String,
    ) {
        todo!()
    }

    async fn click_copy_to_clipboard(
        &mut self,
        text_component: Resource<TextComponent>,
        text: String,
    ) {
        todo!()
    }

    async fn hover_show_text(
        &mut self,
        text_component: Resource<TextComponent>,
        text: Resource<TextComponent>,
    ) {
        todo!()
    }

    async fn hover_show_item(&mut self, text_component: Resource<TextComponent>, item: String) {
        todo!()
    }

    async fn hover_show_entity(
        &mut self,
        text_component: Resource<TextComponent>,
        entity_type: String,
        id: String,
        name: Option<Resource<TextComponent>>,
    ) {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<TextComponent>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<TextComponentResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
