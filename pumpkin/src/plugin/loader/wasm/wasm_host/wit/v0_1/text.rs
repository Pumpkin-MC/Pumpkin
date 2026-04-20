use std::borrow::Cow;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    DowncastResourceExt,
    state::{PluginHostState, TextComponentResource},
    wit::v0_1::pumpkin::{
        self,
        plugin::text::{ArgbColor, NamedColor, RgbColor, TextComponent},
    },
};

use pumpkin_util::text::{
    TextComponent as InternalTextComponent,
    click::ClickEvent,
    color::{self, Color},
    hover::HoverEvent,
};

// --- Trapping Helpers ---
impl PluginHostState {
    fn get_text_ref(
        &self,
        res: &Resource<TextComponent>,
    ) -> wasmtime::Result<&TextComponentResource> {
        self.resource_table
            .get::<TextComponentResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn get_text_mut(
        &mut self,
        res: &Resource<TextComponent>,
    ) -> wasmtime::Result<&mut TextComponentResource> {
        self.resource_table
            .get_mut::<TextComponentResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn take_text(
        &mut self,
        res: &Resource<TextComponent>,
    ) -> wasmtime::Result<TextComponentResource> {
        self.resource_table
            .delete::<TextComponentResource>(Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl DowncastResourceExt<TextComponentResource> for wasmtime::component::Resource<TextComponent> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a TextComponentResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid handle")
            .downcast_ref()
            .expect("type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut TextComponentResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid handle")
            .downcast_mut()
            .expect("type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> TextComponentResource {
        state
            .resource_table
            .delete(wasmtime::component::Resource::new_own(self.rep()))
            .expect("invalid handle")
    }
}

impl pumpkin::plugin::text::Host for PluginHostState {}

impl pumpkin::plugin::text::HostTextComponent for PluginHostState {
    async fn text(&mut self, plain: String) -> wasmtime::Result<Resource<TextComponent>> {
        let tc = InternalTextComponent::text(plain);
        self.add_text_component(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn translate(
        &mut self,
        key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let mut components = Vec::with_capacity(with.len());
        for r in with {
            components.push(self.take_text(&r)?.provider);
        }
        let tc = InternalTextComponent::translate(key, components);
        self.add_text_component(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn add_child(
        &mut self,
        text_component: Resource<TextComponent>,
        child: Resource<TextComponent>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let child_tc = self.take_text(&child)?.provider;
        let parent = self.get_text_mut(&text_component)?;
        parent.provider = parent.provider.clone().add_child(child_tc);
        Ok(text_component)
    }

    async fn add_text(
        &mut self,
        text_component: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let parent = self.get_text_mut(&text_component)?;
        parent.provider = parent.provider.clone().add_text(text);
        Ok(text_component)
    }

    async fn get_text(
        &mut self,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<String> {
        Ok(self
            .get_text_ref(&text_component)?
            .provider
            .clone()
            .get_text())
    }

    async fn encode(
        &mut self,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<Vec<u8>> {
        Ok(self
            .get_text_ref(&text_component)?
            .provider
            .encode()
            .into_vec())
    }

    async fn color_named(
        &mut self,
        res: Resource<TextComponent>,
        color: NamedColor,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.color =
            Some(Color::Named(map_named_color(color)));
        Ok(res)
    }

    async fn color_rgb(
        &mut self,
        res: Resource<TextComponent>,
        color: RgbColor,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.color =
            Some(Color::Rgb(color::RGBColor::new(color.r, color.g, color.b)));
        Ok(res)
    }

    async fn bold(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.bold = Some(value);
        Ok(res)
    }

    async fn italic(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.italic = Some(value);
        Ok(res)
    }

    async fn underlined(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.underlined = Some(value);
        Ok(res)
    }

    async fn strikethrough(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.strikethrough = Some(value);
        Ok(res)
    }

    async fn obfuscated(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.obfuscated = Some(value);
        Ok(res)
    }

    async fn insertion(
        &mut self,
        res: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.insertion = Some(text);
        Ok(res)
    }

    async fn font(
        &mut self,
        res: Resource<TextComponent>,
        font: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.font = Some(font);
        Ok(res)
    }

    async fn shadow_color(
        &mut self,
        res: Resource<TextComponent>,
        color: ArgbColor,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.shadow_color =
            Some(color::ARGBColor::new(color.a, color.r, color.g, color.b));
        Ok(res)
    }

    async fn click_open_url(
        &mut self,
        res: Resource<TextComponent>,
        url: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.click_event = Some(ClickEvent::OpenUrl {
            url: Cow::Owned(url),
        });
        Ok(res)
    }

    async fn click_run_command(
        &mut self,
        res: Resource<TextComponent>,
        command: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.click_event = Some(ClickEvent::RunCommand {
            command: Cow::Owned(command),
        });
        Ok(res)
    }

    async fn click_suggest_command(
        &mut self,
        res: Resource<TextComponent>,
        command: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.click_event = Some(ClickEvent::SuggestCommand {
            command: Cow::Owned(command),
        });
        Ok(res)
    }

    async fn click_copy_to_clipboard(
        &mut self,
        res: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.click_event = Some(ClickEvent::CopyToClipboard {
            value: Cow::Owned(text),
        });
        Ok(res)
    }

    async fn hover_show_text(
        &mut self,
        res: Resource<TextComponent>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let hover_tc = self.take_text(&text)?.provider;
        self.get_text_mut(&res)?.provider.0.style.hover_event = Some(HoverEvent::ShowText {
            value: vec![hover_tc.0],
        });
        Ok(res)
    }

    async fn hover_show_item(
        &mut self,
        res: Resource<TextComponent>,
        item: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        self.get_text_mut(&res)?.provider.0.style.hover_event = Some(HoverEvent::ShowItem {
            id: Cow::Owned(item),
            count: None,
        });
        Ok(res)
    }

    async fn hover_show_entity(
        &mut self,
        res: Resource<TextComponent>,
        entity_type: String,
        id: String,
        name: Option<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let name_val = match name {
            Some(r) => Some(vec![self.take_text(&r)?.provider.0]),
            None => None,
        };
        self.get_text_mut(&res)?.provider.0.style.hover_event = Some(HoverEvent::ShowEntity {
            id: Cow::Owned(entity_type),
            uuid: Cow::Owned(id),
            name: name_val,
        });
        Ok(res)
    }

    async fn drop(&mut self, rep: Resource<TextComponent>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<TextComponentResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
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
