use core::str;
use std::borrow::Cow;

use crate::{text::color::ARGBColor, translation::get_translation_en_us};
use click::ClickEvent;
use color::Color;
use colored::Colorize;
use hover::HoverEvent;
use serde::{Deserialize, Serialize};
use style::Style;

pub mod click;
pub mod color;
pub mod hover;
pub mod style;

/// Represents a Text component
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TextComponent(pub TextComponentBase);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TextComponentBase {
    /// The actual text
    #[serde(flatten)]
    pub content: TextContent,
    /// Style of the text. Bold, Italic, underline, Color...
    /// Also has `ClickEvent
    #[serde(flatten)]
    pub style: Style,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Extra text components
    pub extra: Vec<TextComponentBase>,
}

impl TextComponentBase {
    pub fn to_pretty_console(self) -> String {
        let mut text = match self.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate { translate, with } => {
                let translate = translate.into_owned();
                get_translation_en_us(&translate, with)
                    .unwrap_or(translate.to_string())
                    .clone()
            }
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
        };
        let style = self.style;
        let color = style.color;
        if let Some(color) = color {
            text = color.console_color(&text).to_string();
        }
        if style.bold.is_some() {
            text = text.bold().to_string();
        }
        if style.italic.is_some() {
            text = text.italic().to_string();
        }
        if style.underlined.is_some() {
            text = text.underline().to_string();
        }
        if style.strikethrough.is_some() {
            text = text.strikethrough().to_string();
        }
        if style.click_event.is_some() {
            if let Some(ClickEvent::OpenUrl(url)) = style.click_event {
                //TODO: check if term supports hyperlinks before
                text = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text).to_string()
            }
        }
        for child in self.extra {
            text += &*child.to_pretty_console();
        }
        text
    }
}

impl TextComponent {
    pub fn text<P>(plain: P) -> Self
    where
        P: Into<Cow<'static, str>>,
    {
        Self(TextComponentBase {
            content: TextContent::Text { text: plain.into() },
            style: Style::default(),
            extra: vec![],
        })
    }

    pub fn translate<K, W>(key: K, with: W) -> Self
    where
        K: Into<Cow<'static, str>>,
        W: Into<Vec<TextComponent>>,
    {
        Self(TextComponentBase {
            content: TextContent::Translate {
                translate: key.into(),
                with: with.into().into_iter().map(|x| x.0).collect(),
            },
            style: Style::default(),
            extra: vec![],
        })
    }

    pub fn add_child(mut self, child: TextComponent) -> Self {
        self.0.extra.push(child.0);
        self
    }

    pub fn from_content(content: TextContent) -> Self {
        Self(TextComponentBase {
            content,
            style: Style::default(),
            extra: vec![],
        })
    }

    pub fn add_text<P>(mut self, text: P) -> Self
    where
        P: Into<Cow<'static, str>>,
    {
        self.0.extra.push(TextComponentBase {
            content: TextContent::Text { text: text.into() },
            style: Style::default(),
            extra: vec![],
        });
        self
    }

    pub fn get_text(self) -> String {
        match self.0.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate { translate, with } => {
                let translate = translate.into_owned();
                get_translation_en_us(&translate, with)
                    .unwrap_or(translate.to_string())
                    .clone()
            }
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
        }
    }

    pub fn to_pretty_console(self) -> String {
        self.0.to_pretty_console()
    }
}

impl TextComponent {
    pub fn encode(&self) -> Box<[u8]> {
        let mut buf = Vec::new();
        // TODO: Properly handle errors
        pumpkin_nbt::serializer::to_bytes_unnamed(&self.0, &mut buf)
            .expect("Failed to serialize text component NBT for encode");

        buf.into_boxed_slice()
    }

    pub fn color(mut self, color: Color) -> Self {
        self.0.style.color = Some(color);
        self
    }

    pub fn color_named(mut self, color: color::NamedColor) -> Self {
        self.0.style.color = Some(Color::Named(color));
        self
    }

    pub fn color_rgb(mut self, color: color::RGBColor) -> Self {
        self.0.style.color = Some(Color::Rgb(color));
        self
    }

    /// Makes the text bold
    pub fn bold(mut self) -> Self {
        self.0.style.bold = Some(true);
        self
    }

    /// Makes the text italic
    pub fn italic(mut self) -> Self {
        self.0.style.italic = Some(true);
        self
    }

    /// Makes the text underlined
    pub fn underlined(mut self) -> Self {
        self.0.style.underlined = Some(true);
        self
    }

    /// Makes the text strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.0.style.strikethrough = Some(true);
        self
    }

    /// Makes the text obfuscated
    pub fn obfuscated(mut self) -> Self {
        self.0.style.obfuscated = Some(true);
        self
    }

    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages
    pub fn insertion(mut self, text: String) -> Self {
        self.0.style.insertion = Some(text);
        self
    }

    /// Allows for events to occur when the player clicks on text. Only work in chat.
    pub fn click_event(mut self, event: ClickEvent) -> Self {
        self.0.style.click_event = Some(event);
        self
    }

    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    pub fn hover_event(mut self, event: HoverEvent) -> Self {
        self.0.style.hover_event = Some(event);
        self
    }

    /// Allows you to change the font of the text.
    /// Default fonts: `minecraft:default`, `minecraft:uniform`, `minecraft:alt`, `minecraft:illageralt`
    pub fn font(mut self, identifier: String) -> Self {
        self.0.style.font = Some(identifier);
        self
    }

    /// Overrides the shadow properties of text.
    pub fn shadow_color(mut self, color: ARGBColor) -> Self {
        self.0.style.shadow_color = Some(color);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum TextContent {
    /// Raw Text
    Text { text: Cow<'static, str> },
    /// Translated text
    Translate {
        translate: Cow<'static, str>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        with: Vec<TextComponentBase>,
    },
    /// Displays the name of one or more entities found by a selector.
    EntityNames {
        selector: Cow<'static, str>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        separator: Option<Cow<'static, str>>,
    },
    /// A keybind identifier
    /// https://minecraft.wiki/w/Controls#Configurable_controls
    Keybind { keybind: Cow<'static, str> },
}

#[cfg(test)]
mod test {
    use pumpkin_nbt::serializer::to_bytes_unnamed;

    use crate::text::{color::NamedColor, TextComponent};

    #[test]
    fn test_serialize_text_component() {
        let msg_comp = TextComponent::translate(
            "multiplayer.player.joined",
            [TextComponent::text("NAME".to_string())],
        )
        .color_named(NamedColor::Yellow);

        let mut bytes = Vec::new();
        to_bytes_unnamed(&msg_comp.0, &mut bytes).unwrap();

        let expected_bytes = [
            0x0A, 0x08, 0x00, 0x09, 0x74, 0x72, 0x61, 0x6E, 0x73, 0x6C, 0x61, 0x74, 0x65, 0x00,
            0x19, 0x6D, 0x75, 0x6C, 0x74, 0x69, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x2E, 0x70,
            0x6C, 0x61, 0x79, 0x65, 0x72, 0x2E, 0x6A, 0x6F, 0x69, 0x6E, 0x65, 0x64, 0x09, 0x00,
            0x04, 0x77, 0x69, 0x74, 0x68, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x08, 0x00, 0x04, 0x74,
            0x65, 0x78, 0x74, 0x00, 0x04, 0x4E, 0x41, 0x4D, 0x45, 0x00, 0x08, 0x00, 0x05, 0x63,
            0x6F, 0x6C, 0x6F, 0x72, 0x00, 0x06, 0x79, 0x65, 0x6C, 0x6C, 0x6F, 0x77, 0x00,
        ];

        assert_eq!(bytes, expected_bytes);
    }
}
