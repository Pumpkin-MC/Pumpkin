use core::str;
use std::borrow::Cow;

use crate::text::color::ARGBColor;
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
#[serde(rename_all = "camelCase")]
pub struct TextComponent {
    /// The actual text
    #[serde(flatten)]
    pub content: TextContent,
    /// Style of the text. Bold, Italic, underline, Color...
    /// Also has `ClickEvent
    #[serde(flatten)]
    pub style: Style,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Extra text components
    pub extra: Vec<TextComponent>,
}

pub struct TextComponentNbt {
    pub wrappee: TextComponent,
}

impl TextComponentNbt {
    pub fn new(wrappee: TextComponent) -> Self {
        Self { wrappee }
    }

    pub fn encode(&self) -> bytes::BytesMut {
        self.wrappee.encode()
    }
}

impl serde::Serialize for TextComponentNbt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // NBT-encode the underlying component, then
        // pass the bytes to `serializer`.
        let bytes = self.wrappee.encode();
        serializer.serialize_bytes(&bytes)
    }
}

impl TextComponent {
    pub fn to_nbt(&self) -> TextComponentNbt {
        TextComponentNbt::new(self.clone())
    }
}

impl TextComponent {
    pub fn text<P>(plain: P) -> Self
    where
        P: Into<Cow<'static, str>>,
    {
        Self {
            content: TextContent::Text { text: plain.into() },
            style: Style::default(),
            extra: vec![],
        }
    }

    pub fn add_child(mut self, child: TextComponent) -> Self {
        self.extra.push(child);
        self
    }

    pub fn to_pretty_console(self) -> String {
        let style = self.style;
        let color = style.color;
        let mut text = match self.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate { translate, with: _ } => translate.into_owned(),
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
        };
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
    pub fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }

    pub fn color_named(mut self, color: color::NamedColor) -> Self {
        self.style.color = Some(Color::Named(color));
        self
    }

    pub fn color_rgb(mut self, color: color::RGBColor) -> Self {
        self.style.color = Some(Color::Rgb(color));
        self
    }

    /// Makes the text bold
    pub fn bold(mut self) -> Self {
        self.style.bold = Some(true);
        self
    }

    /// Makes the text italic
    pub fn italic(mut self) -> Self {
        self.style.italic = Some(true);
        self
    }

    /// Makes the text underlined
    pub fn underlined(mut self) -> Self {
        self.style.underlined = Some(true);
        self
    }

    /// Makes the text strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.style.strikethrough = Some(true);
        self
    }

    /// Makes the text obfuscated
    pub fn obfuscated(mut self) -> Self {
        self.style.obfuscated = Some(true);
        self
    }

    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages
    pub fn insertion(mut self, text: String) -> Self {
        self.style.insertion = Some(text);
        self
    }

    /// Allows for events to occur when the player clicks on text. Only work in chat.
    pub fn click_event(mut self, event: ClickEvent) -> Self {
        self.style.click_event = Some(event);
        self
    }

    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    pub fn hover_event(mut self, event: HoverEvent) -> Self {
        self.style.hover_event = Some(event);
        self
    }

    /// Allows you to change the font of the text.
    /// Default fonts: `minecraft:default`, `minecraft:uniform`, `minecraft:alt`, `minecraft:illageralt`
    pub fn font(mut self, identifier: String) -> Self {
        self.style.font = Some(identifier);
        self
    }

    /// Overrides the shadow properties of text.
    pub fn shadow_color(mut self, color: ARGBColor) -> Self {
        self.style.shadow_color = Some(color);
        self
    }

    pub fn encode(&self) -> bytes::BytesMut {
        pumpkin_nbt::serializer::to_bytes_unnamed(&self).unwrap()
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
        with: Vec<TextComponent>,
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
    use crate::text::style::Style;
    use super::{TextComponent, TextContent};

    #[test]
    fn text_component_json() {
        let text = "%s is cool!";
        let value = "testing";

        let component = TextComponent {
            content: TextContent::Translate {
                translate: text.into(),
                with: vec![
                    TextComponent::text(value.to_string()),
                ],
            },
            style: Style::default(),
            extra: vec![],
        };

        let json_text = &serde_json::to_string(&component).unwrap();
        assert_eq!(json_text, "{\"translate\":\"%s is cool!\",\"with\":[{\"text\":\"testing\"}]}");
    }
}