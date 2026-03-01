use std::borrow::Cow;

use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, TextComponentResource},
    wit::v0_1_0::pumpkin::{
        self,
        plugin::text::{ArgbColor, NamedColor, RgbColor, TextComponent},
    },
};

use pumpkin_util::text::{
    click::ClickEvent,
    color::{self, Color},
    hover::HoverEvent,
};

impl pumpkin::plugin::text::Host for PluginHostState {}

trait TextComponentResourceExt {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a TextComponentResource;
    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut TextComponentResource;
    fn consume(self, state: &mut PluginHostState) -> TextComponentResource;
}

// TODO - Change the pumpkin_util::text::TextComponent to use &mut self instead of self for the builder pattern.
// right now we have to do a bunch of cloning due to the fact that the builder pattern doesn't accept &mut self.
impl TextComponentResourceExt for Resource<TextComponent> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a TextComponentResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid text-component resource handle")
            .downcast_ref::<TextComponentResource>()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut TextComponentResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid text-component resource handle")
            .downcast_mut::<TextComponentResource>()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> TextComponentResource {
        state
            .resource_table
            .delete::<TextComponentResource>(Resource::new_own(self.rep()))
            .expect("invalid text-component resource handle")
    }
}

const fn map_named_color(color: NamedColor) -> color::NamedColor {
    match color {
        NamedColor::Black => color::NamedColor::Black,
        NamedColor::DarkBlue => color::NamedColor::DarkBlue,
        NamedColor::DarkGreen => color::NamedColor::DarkGreen,
        NamedColor::DarkAqua => color::NamedColor::DarkAqua,
        NamedColor::DarkRed => color::NamedColor::DarkRed,
        NamedColor::DarkPurple => color::NamedColor::DarkPurple,
        NamedColor::Gold => color::NamedColor::Gold,
        NamedColor::Gray => color::NamedColor::Gray,
        NamedColor::DarkGray => color::NamedColor::DarkGray,
        NamedColor::Blue => color::NamedColor::Blue,
        NamedColor::Green => color::NamedColor::Green,
        NamedColor::Aqua => color::NamedColor::Aqua,
        NamedColor::Red => color::NamedColor::Red,
        NamedColor::LightPurple => color::NamedColor::LightPurple,
        NamedColor::Yellow => color::NamedColor::Yellow,
        NamedColor::White => color::NamedColor::White,
    }
}

impl pumpkin::plugin::text::HostTextComponent for PluginHostState {
    async fn text(&mut self, plain: String) -> Resource<TextComponent> {
        let tc = pumpkin_util::text::TextComponent::text(plain);
        self.add_text_component(tc).unwrap()
    }

    async fn translate(
        &mut self,
        key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> Resource<TextComponent> {
        let with: Vec<pumpkin_util::text::TextComponent> =
            with.into_iter().map(|r| r.consume(self).provider).collect();
        let tc = pumpkin_util::text::TextComponent::translate(key, with);
        self.add_text_component(tc).unwrap()
    }

    async fn add_child(
        &mut self,
        text_component: Resource<TextComponent>,
        child: Resource<TextComponent>,
    ) {
        let child = child.consume(self).provider;
        let parent = &mut text_component.downcast_mut(self).provider;
        *parent = parent.clone().add_child(child);
    }

    async fn add_text(&mut self, text_component: Resource<TextComponent>, text: String) {
        let parent = &mut text_component.downcast_mut(self).provider;
        *parent = parent.clone().add_text(text);
    }

    async fn get_text(&mut self, text_component: Resource<TextComponent>) -> String {
        text_component
            .downcast_ref(self)
            .provider
            .clone()
            .get_text()
    }

    async fn encode(&mut self, text_component: Resource<TextComponent>) -> Vec<u8> {
        text_component
            .downcast_ref(self)
            .provider
            .encode()
            .into_vec()
    }

    async fn color_named(&mut self, text_component: Resource<TextComponent>, color: NamedColor) {
        text_component.downcast_mut(self).provider.0.style.color =
            Some(Color::Named(map_named_color(color)));
    }

    async fn color_rgb(&mut self, text_component: Resource<TextComponent>, color: RgbColor) {
        text_component.downcast_mut(self).provider.0.style.color =
            Some(Color::Rgb(color::RGBColor::new(color.r, color.g, color.b)));
    }

    async fn bold(&mut self, text_component: Resource<TextComponent>, value: bool) {
        text_component.downcast_mut(self).provider.0.style.bold = Some(value);
    }

    async fn italic(&mut self, text_component: Resource<TextComponent>, value: bool) {
        text_component.downcast_mut(self).provider.0.style.italic = Some(value);
    }

    async fn underlined(&mut self, text_component: Resource<TextComponent>, value: bool) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .underlined = Some(value);
    }

    async fn strikethrough(&mut self, text_component: Resource<TextComponent>, value: bool) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .strikethrough = Some(value);
    }

    async fn obfuscated(&mut self, text_component: Resource<TextComponent>, value: bool) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .obfuscated = Some(value);
    }

    async fn insertion(&mut self, text_component: Resource<TextComponent>, text: String) {
        text_component.downcast_mut(self).provider.0.style.insertion = Some(text);
    }

    async fn font(&mut self, text_component: Resource<TextComponent>, font: String) {
        text_component.downcast_mut(self).provider.0.style.font = Some(font);
    }

    async fn shadow_color(&mut self, text_component: Resource<TextComponent>, color: ArgbColor) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .shadow_color = Some(color::ARGBColor::new(color.a, color.r, color.g, color.b));
    }

    async fn click_open_url(&mut self, text_component: Resource<TextComponent>, url: String) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .click_event = Some(ClickEvent::OpenUrl {
            url: Cow::Owned(url),
        });
    }

    async fn click_run_command(
        &mut self,
        text_component: Resource<TextComponent>,
        command: String,
    ) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .click_event = Some(ClickEvent::RunCommand {
            command: Cow::Owned(command),
        });
    }

    async fn click_suggest_command(
        &mut self,
        text_component: Resource<TextComponent>,
        command: String,
    ) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .click_event = Some(ClickEvent::SuggestCommand {
            command: Cow::Owned(command),
        });
    }

    async fn click_copy_to_clipboard(
        &mut self,
        text_component: Resource<TextComponent>,
        text: String,
    ) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .click_event = Some(ClickEvent::CopyToClipboard {
            value: Cow::Owned(text),
        });
    }

    async fn hover_show_text(
        &mut self,
        text_component: Resource<TextComponent>,
        text: Resource<TextComponent>,
    ) {
        let hover_tc = text.consume(self).provider;
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .hover_event = Some(HoverEvent::ShowText {
            value: vec![hover_tc.0],
        });
    }

    async fn hover_show_item(&mut self, text_component: Resource<TextComponent>, item: String) {
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .hover_event = Some(HoverEvent::ShowItem {
            id: Cow::Owned(item),
            count: None,
        });
    }

    async fn hover_show_entity(
        &mut self,
        text_component: Resource<TextComponent>,
        entity_type: String,
        id: String,
        name: Option<Resource<TextComponent>>,
    ) {
        let name = name.map(|r| vec![r.consume(self).provider.0]);
        text_component
            .downcast_mut(self)
            .provider
            .0
            .style
            .hover_event = Some(HoverEvent::ShowEntity {
            id: Cow::Owned(entity_type),
            uuid: Cow::Owned(id),
            name,
        });
    }

    async fn drop(&mut self, rep: Resource<TextComponent>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<TextComponentResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
