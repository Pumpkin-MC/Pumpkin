use std::borrow::Cow;
use std::str::FromStr;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
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
use pumpkin_util::translation::Locale;

impl pumpkin::plugin::text::Host for PluginHostState {}

impl pumpkin::plugin::text::HostTextComponent for PluginHostState {
    async fn text(&mut self, plain: String) -> wasmtime::Result<Resource<TextComponent>> {
        let tc = InternalTextComponent::text(plain);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn translate(
        &mut self,
        key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let mut components = Vec::with_capacity(with.len());
        for r in with {
            components.push(self.take(r)?);
        }
        #[allow(deprecated)]
        let tc = InternalTextComponent::translate(key, components);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn translate_cross(
        &mut self,
        java_key: String,
        bedrock_key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let mut components = Vec::with_capacity(with.len());
        for r in with {
            components.push(self.take(r)?);
        }
        #[allow(deprecated)]
        let tc = InternalTextComponent::translate_cross(java_key, bedrock_key, components);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn entity_names(
        &mut self,
        selector: String,
        separator: Option<String>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let tc = InternalTextComponent::entity_names(selector, separator);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn keybind(&mut self, keybind: String) -> wasmtime::Result<Resource<TextComponent>> {
        let tc = InternalTextComponent::keybind(keybind);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn custom(
        &mut self,
        namespace: String,
        key: String,
        locale: String,
        with: Vec<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let loc = Locale::from_str(&locale).unwrap_or(Locale::EnUs);
        let mut components = Vec::with_capacity(with.len());
        for r in with {
            components.push(self.take(r)?);
        }
        let tc = InternalTextComponent::custom(namespace, key, loc, components);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn from_legacy_string(
        &mut self,
        input: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let tc = InternalTextComponent::from_legacy_string(&input);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn from_legacy_string_with_code(
        &mut self,
        input: String,
        code_symbol: char,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let tc = InternalTextComponent::from_legacy_string_with_code(&input, code_symbol);
        self.add(tc)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
    }

    async fn from_json(
        &mut self,
        json: String,
    ) -> wasmtime::Result<Result<Resource<TextComponent>, String>> {
        match serde_json::from_str::<InternalTextComponent>(&json) {
            Ok(tc) => match self.add(tc) {
                Ok(res) => Ok(Ok(res)),
                Err(err) => Ok(Err(err.to_string())),
            },
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn to_json(
        &mut self,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<String> {
        let tc = &self.get(&text_component)?;
        Ok(serde_json::to_string(tc).unwrap_or_default())
    }

    async fn add_child(
        &mut self,
        text_component: Resource<TextComponent>,
        child: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let child_tc = self.take(child)?;
        let parent = self.get_mut(&text_component)?;
        *parent = parent.clone().add_child(child_tc);
        Ok(())
    }

    async fn add_text(
        &mut self,
        text_component: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<()> {
        let parent = self.get_mut(&text_component)?;
        *parent = parent.clone().add_text(text);
        Ok(())
    }

    async fn get_text(
        &mut self,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<String> {
        Ok(self.get(&text_component)?.clone().get_text())
    }

    async fn encode(
        &mut self,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<Vec<u8>> {
        Ok(self.get(&text_component)?.encode().into_vec())
    }

    async fn to_pretty_console(
        &mut self,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<String> {
        Ok(self.get(&text_component)?.clone().to_pretty_console())
    }

    async fn color_named(
        &mut self,
        res: Resource<TextComponent>,
        color: NamedColor,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.color = Some(Color::Named(map_named_color(color)));
        Ok(())
    }

    async fn color_rgb(
        &mut self,
        res: Resource<TextComponent>,
        color: RgbColor,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.color =
            Some(Color::Rgb(color::RGBColor::new(color.r, color.g, color.b)));
        Ok(())
    }

    async fn gradient_named(
        &mut self,
        res: Resource<TextComponent>,
        colors: Vec<NamedColor>,
    ) -> wasmtime::Result<()> {
        let mapped: Vec<_> = colors.into_iter().map(map_named_color).collect();
        let parent = self.get_mut(&res)?;
        *parent = parent.clone().gradient_named(&mapped);
        Ok(())
    }

    async fn gradient(
        &mut self,
        res: Resource<TextComponent>,
        colors: Vec<RgbColor>,
    ) -> wasmtime::Result<()> {
        let mapped: Vec<_> = colors
            .into_iter()
            .map(|c| color::RGBColor::new(c.r, c.g, c.b))
            .collect();
        let parent = self.get_mut(&res)?;
        *parent = parent.clone().gradient(&mapped);
        Ok(())
    }

    async fn rainbow(&mut self, res: Resource<TextComponent>) -> wasmtime::Result<()> {
        let parent = self.get_mut(&res)?;
        *parent = parent.clone().rainbow();
        Ok(())
    }

    async fn bold(&mut self, res: Resource<TextComponent>, value: bool) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.bold = Some(value);
        Ok(())
    }

    async fn italic(&mut self, res: Resource<TextComponent>, value: bool) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.italic = Some(value);
        Ok(())
    }

    async fn underlined(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.underlined = Some(value);
        Ok(())
    }

    async fn strikethrough(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.strikethrough = Some(value);
        Ok(())
    }

    async fn obfuscated(
        &mut self,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.obfuscated = Some(value);
        Ok(())
    }

    async fn insertion(
        &mut self,
        res: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.insertion = Some(text);
        Ok(())
    }

    async fn font(&mut self, res: Resource<TextComponent>, font: String) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.font = Some(font);
        Ok(())
    }

    async fn shadow_color(
        &mut self,
        res: Resource<TextComponent>,
        color: ArgbColor,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.shadow_color =
            Some(color::ARGBColor::new(color.a, color.r, color.g, color.b));
        Ok(())
    }

    async fn click_open_url(
        &mut self,
        res: Resource<TextComponent>,
        url: String,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.click_event = Some(ClickEvent::OpenUrl {
            url: Cow::Owned(url),
        });
        Ok(())
    }

    async fn click_open_file(
        &mut self,
        res: Resource<TextComponent>,
        path: String,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.click_event = Some(ClickEvent::OpenFile {
            path: Cow::Owned(path),
        });
        Ok(())
    }

    async fn click_run_command(
        &mut self,
        res: Resource<TextComponent>,
        command: String,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.click_event = Some(ClickEvent::RunCommand {
            command: Cow::Owned(command),
        });
        Ok(())
    }

    async fn click_suggest_command(
        &mut self,
        res: Resource<TextComponent>,
        command: String,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.click_event = Some(ClickEvent::SuggestCommand {
            command: Cow::Owned(command),
        });
        Ok(())
    }

    async fn click_change_page(
        &mut self,
        res: Resource<TextComponent>,
        page: u32,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.click_event = Some(ClickEvent::ChangePage { page });
        Ok(())
    }

    async fn click_copy_to_clipboard(
        &mut self,
        res: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.click_event = Some(ClickEvent::CopyToClipboard {
            value: Cow::Owned(text),
        });
        Ok(())
    }

    async fn hover_show_text(
        &mut self,
        res: Resource<TextComponent>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let hover_tc = self.take(text)?;
        self.get_mut(&res)?.0.style.hover_event = Some(HoverEvent::ShowText {
            value: vec![hover_tc.0],
        });
        Ok(())
    }

    async fn hover_show_item(
        &mut self,
        res: Resource<TextComponent>,
        item: String,
    ) -> wasmtime::Result<()> {
        self.get_mut(&res)?.0.style.hover_event = Some(HoverEvent::ShowItem {
            id: Cow::Owned(item),
            count: None,
        });
        Ok(())
    }

    async fn hover_show_entity(
        &mut self,
        res: Resource<TextComponent>,
        entity_type: String,
        id: String,
        name: Option<Resource<TextComponent>>,
    ) -> wasmtime::Result<()> {
        let name_val = match name {
            Some(r) => Some(vec![self.take(r)?.0]),
            None => None,
        };
        self.get_mut(&res)?.0.style.hover_event = Some(HoverEvent::ShowEntity {
            id: Cow::Owned(entity_type),
            uuid: Cow::Owned(id),
            name: name_val,
        });
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<TextComponent>) -> wasmtime::Result<()> {
        self.drop(rep)
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
