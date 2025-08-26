use crate::text::color::ARGBColor;
use click::ClickEvent;
use color::Color;
use content::TextContent;
use core::str;
use hover::HoverEvent;
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt::Formatter;
use style::Style;

pub mod click;
pub mod color;
pub mod content;
pub mod hover;
pub mod style;
pub mod translation;

/// Represents a text component
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct TextComponent(pub TextComponentBase);

impl<'de> Deserialize<'de> for TextComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TextComponentVisitor;

        impl<'de> Visitor<'de> for TextComponentVisitor {
            type Value = TextComponentBase;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a TextComponentBase or a sequence of TextComponentBase")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(TextComponentBase {
                    content: TextContent::Text {
                        text: Cow::from(v.to_string()),
                    },
                    style: Default::default(),
                    extra: vec![],
                })
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut bases = Vec::new();
                while let Some(element) = seq.next_element::<TextComponent>()? {
                    bases.push(element.0);
                }

                Ok(TextComponentBase {
                    content: TextContent::Text { text: "".into() },
                    style: Default::default(),
                    extra: bases,
                })
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                TextComponentBase::deserialize(serde::de::value::MapAccessDeserializer::new(map))
            }
        }

        deserializer
            .deserialize_any(TextComponentVisitor)
            .map(TextComponent)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TextComponentBase {
    /// The actual text
    #[serde(flatten)]
    pub content: TextContent,
    /// Style of the text. Bold, Italic, underline, Color...
    /// Also has Click and Hover events
    #[serde(flatten)]
    pub style: Style,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Extra text components
    pub extra: Vec<TextComponentBase>,
}

/// TextComponent constructors
impl TextComponent {
    pub fn text<P: Into<Cow<'static, str>>>(plain: P) -> Self {
        Self(TextComponentBase {
            content: TextContent::Text { text: plain.into() },
            style: Style::default(),
            extra: vec![],
        })
    }

    pub fn translate<K: Into<Cow<'static, str>>, W: Into<Vec<TextComponent>>>(
        key: K,
        fallback: Option<K>,
        with: W,
    ) -> Self {
        Self(TextComponentBase {
            content: TextContent::Translate {
                translate: key.into(),
                fallback: fallback.map(Into::into),
                with: with.into().into_iter().map(|x| x.0).collect(),
            },
            style: Style::default(),
            extra: vec![],
        })
    }

    pub fn custom<K: Into<Cow<'static, str>>, W: Into<Vec<TextComponent>>>(
        namespace: K,
        key: K,
        with: W,
    ) -> Self {
        Self(TextComponentBase {
            content: TextContent::Custom {
                key: format!("{}:{}", namespace.into(), key.into())
                    .to_lowercase()
                    .into(),
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

    pub fn add_text<P: Into<Cow<'static, str>>>(mut self, text: P) -> Self {
        self.0.extra.push(TextComponentBase {
            content: TextContent::Text { text: text.into() },
            style: Style::default(),
            extra: vec![],
        });
        self
    }

    pub fn chat_decorated(format: String, player_name: String, content: String) -> Self {
        // Todo: maybe allow players to use & in chat contingent on permissions
        let with_resolved_fields = format
            .replace("&", "§")
            .replace("{DISPLAYNAME}", player_name.as_str())
            .replace("{MESSAGE}", content.as_str());

        Self(TextComponentBase {
            content: TextContent::Text {
                text: Cow::Owned(with_resolved_fields),
            },
            style: Style::default(),
            extra: vec![],
        })
    }
}

impl TextComponent {
    pub fn encode(&self) -> Box<[u8]> {
        let mut buf = Vec::new();
        // TODO: Properly handle errors
        pumpkin_nbt::serializer::to_bytes_unnamed(&self, &mut buf)
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

    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages.
    pub fn insertion(mut self, text: String) -> Self {
        self.0.style.insertion = Some(text);
        self
    }

    /// Allows for events to occur when the player clicks on text. Only works in chat.
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
    pub fn font(mut self, resource_location: String) -> Self {
        self.0.style.font = Some(resource_location);
        self
    }

    /// Overrides the shadow properties of text.
    pub fn shadow_color(mut self, color: ARGBColor) -> Self {
        self.0.style.shadow_color = Some(color);
        self
    }
}

#[cfg(test)]
mod test {
    use pumpkin_nbt::serializer::to_bytes_unnamed;

    use crate::text::{TextComponent, color::NamedColor};

    #[test]
    fn test_serialize_text_component() {
        let msg_comp = TextComponent::translate(
            "multiplayer.player.joined",
            None,
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
