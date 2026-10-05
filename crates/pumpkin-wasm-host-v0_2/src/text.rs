use crate::pumpkin::{
    self,
    plugin::text::{ArgbColor, NamedColor, RgbColor, TextComponent},
};
use pumpkin_util::{
    text::{
        TextComponent as InternalTextComponent,
        click::ClickEvent,
        color::{self, Color},
        hover::HoverEvent,
    },
    translation::Locale,
};
use pumpkin_wasm_host_common::state::PluginHostState;
use std::{borrow::Cow, str::FromStr};
use wasmtime::component::{Accessor, HasSelf, Resource};

impl pumpkin::plugin::text::Host for PluginHostState {}

impl pumpkin::plugin::text::HostTextComponent for PluginHostState {
    async fn drop(&mut self, rep: Resource<TextComponent>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl pumpkin::plugin::text::HostTextComponentWithStore<PluginHostState>
    for HasSelf<PluginHostState>
{
    async fn text(
        accessor: &Accessor<PluginHostState, Self>,
        plain: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let tc = InternalTextComponent::text(plain);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn translate(
        accessor: &Accessor<PluginHostState, Self>,
        key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let mut components = Vec::with_capacity(with.len());
            for r in with {
                components.push(state.take(r)?);
            }
            #[allow(deprecated)]
            let tc = InternalTextComponent::translate(key, components);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn translate_cross(
        accessor: &Accessor<PluginHostState, Self>,
        java_key: String,
        bedrock_key: String,
        with: Vec<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let mut components = Vec::with_capacity(with.len());
            for r in with {
                components.push(state.take(r)?);
            }
            #[allow(deprecated)]
            let tc = InternalTextComponent::translate_cross(java_key, bedrock_key, components);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn entity_names(
        accessor: &Accessor<PluginHostState, Self>,
        selector: String,
        separator: Option<String>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let tc = InternalTextComponent::entity_names(selector, separator);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn keybind(
        accessor: &Accessor<PluginHostState, Self>,
        keybind: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let tc = InternalTextComponent::keybind(keybind);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn custom(
        accessor: &Accessor<PluginHostState, Self>,
        namespace: String,
        key: String,
        locale: String,
        with: Vec<Resource<TextComponent>>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let loc = Locale::from_str(&locale).unwrap_or(Locale::EnUs);
            let mut components = Vec::with_capacity(with.len());
            for r in with {
                components.push(state.take(r)?);
            }
            let tc = InternalTextComponent::custom(namespace, key, loc, components);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn from_legacy_string(
        accessor: &Accessor<PluginHostState, Self>,
        input: String,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let tc = InternalTextComponent::from_legacy_string(&input);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn from_legacy_string_with_code(
        accessor: &Accessor<PluginHostState, Self>,
        input: String,
        code_symbol: char,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let tc = InternalTextComponent::from_legacy_string_with_code(&input, code_symbol);
            state
                .add(tc)
                .map_err(|_| wasmtime::Error::msg("Failed to add text component"))
        })
    }

    async fn from_json(
        accessor: &Accessor<PluginHostState, Self>,
        json: String,
    ) -> wasmtime::Result<Result<Resource<TextComponent>, String>> {
        accessor.with(|mut host| {
            let state = host.get();
            match serde_json::from_str::<InternalTextComponent>(&json) {
                Ok(tc) => match state.add(tc) {
                    Ok(res) => Ok(Ok(res)),
                    Err(err) => Ok(Err(err.to_string())),
                },
                Err(err) => Ok(Err(err.to_string())),
            }
        })
    }

    async fn to_json(
        accessor: &Accessor<PluginHostState, Self>,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            let tc = &state.get(&text_component)?;
            Ok(serde_json::to_string(tc).unwrap_or_default())
        })
    }

    async fn add_child(
        accessor: &Accessor<PluginHostState, Self>,
        text_component: Resource<TextComponent>,
        child: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let child_tc = state.take(child)?;
            let parent = state.get_mut(&text_component)?;
            *parent = parent.clone().add_child(child_tc);
            Ok(())
        })
    }

    async fn add_text(
        accessor: &Accessor<PluginHostState, Self>,
        text_component: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let parent = state.get_mut(&text_component)?;
            *parent = parent.clone().add_text(text);
            Ok(())
        })
    }

    async fn get_text(
        accessor: &Accessor<PluginHostState, Self>,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&text_component)?.clone().get_text())
        })
    }

    async fn encode(
        accessor: &Accessor<PluginHostState, Self>,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<Vec<u8>> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&text_component)?.encode().into_vec())
        })
    }

    async fn to_pretty_console(
        accessor: &Accessor<PluginHostState, Self>,
        text_component: Resource<TextComponent>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&text_component)?.clone().to_pretty_console())
        })
    }

    async fn color_named(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        color: NamedColor,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.color = Some(Color::Named(map_named_color(color)));
            Ok(())
        })
    }

    async fn color_rgb(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        color: RgbColor,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.color =
                Some(Color::Rgb(color::RGBColor::new(color.r, color.g, color.b)));
            Ok(())
        })
    }

    async fn gradient_named(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        colors: Vec<NamedColor>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let mapped: Vec<_> = colors.into_iter().map(map_named_color).collect();
            let parent = state.get_mut(&res)?;
            *parent = parent.clone().gradient_named(&mapped);
            Ok(())
        })
    }

    async fn gradient(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        colors: Vec<RgbColor>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let mapped: Vec<_> = colors
                .into_iter()
                .map(|c| color::RGBColor::new(c.r, c.g, c.b))
                .collect();
            let parent = state.get_mut(&res)?;
            *parent = parent.clone().gradient(&mapped);
            Ok(())
        })
    }

    async fn rainbow(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let parent = state.get_mut(&res)?;
            *parent = parent.clone().rainbow();
            Ok(())
        })
    }

    async fn bold(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.bold = Some(value);
            Ok(())
        })
    }

    async fn italic(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.italic = Some(value);
            Ok(())
        })
    }

    async fn underlined(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.underlined = Some(value);
            Ok(())
        })
    }

    async fn strikethrough(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.strikethrough = Some(value);
            Ok(())
        })
    }

    async fn obfuscated(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        value: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.obfuscated = Some(value);
            Ok(())
        })
    }

    async fn insertion(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.insertion = Some(text);
            Ok(())
        })
    }

    async fn font(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        font: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.font = Some(font);
            Ok(())
        })
    }

    async fn shadow_color(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        color: ArgbColor,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.shadow_color =
                Some(color::ARGBColor::new(color.a, color.r, color.g, color.b));
            Ok(())
        })
    }

    async fn click_open_url(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        url: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.click_event = Some(ClickEvent::OpenUrl {
                url: Cow::Owned(url),
            });
            Ok(())
        })
    }

    async fn click_open_file(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        path: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.click_event = Some(ClickEvent::OpenFile {
                path: Cow::Owned(path),
            });
            Ok(())
        })
    }

    async fn click_run_command(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        command: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.click_event = Some(ClickEvent::RunCommand {
                command: Cow::Owned(command),
            });
            Ok(())
        })
    }

    async fn click_suggest_command(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        command: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.click_event = Some(ClickEvent::SuggestCommand {
                command: Cow::Owned(command),
            });
            Ok(())
        })
    }

    async fn click_change_page(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        page: u32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.click_event = Some(ClickEvent::ChangePage { page });
            Ok(())
        })
    }

    async fn click_copy_to_clipboard(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        text: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.click_event = Some(ClickEvent::CopyToClipboard {
                value: Cow::Owned(text),
            });
            Ok(())
        })
    }

    async fn hover_show_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let hover_tc = state.take(text)?;
            state.get_mut(&res)?.0.style.hover_event = Some(HoverEvent::ShowText {
                value: vec![hover_tc.0],
            });
            Ok(())
        })
    }

    async fn hover_show_item(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        item: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get_mut(&res)?.0.style.hover_event = Some(HoverEvent::ShowItem {
                id: Cow::Owned(item),
                count: None,
            });
            Ok(())
        })
    }

    async fn hover_show_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TextComponent>,
        entity_type: String,
        id: String,
        name: Option<Resource<TextComponent>>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let name_val = match name {
                Some(r) => Some(vec![state.take(r)?.0]),
                None => None,
            };
            state.get_mut(&res)?.0.style.hover_event = Some(HoverEvent::ShowEntity {
                id: Cow::Owned(entity_type),
                uuid: Cow::Owned(id),
                name: name_val,
            });
            Ok(())
        })
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
