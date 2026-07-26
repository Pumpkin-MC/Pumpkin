use crate::translation::Locale;
use core::str;
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt::Formatter;
use style::Style;

mod base;
pub mod click;
pub mod color;
mod component;
pub mod hover;
pub mod legacy;
pub mod style;

/// Represents a Minecraft chat component.
///
/// Text components are the building blocks of Minecraft's chat system, allowing for
/// rich formatted text with colors, styles, click events, hover tooltips, and
/// translations. They can be nested and combined to create complex messages.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
                    content: Box::new(TextContent::Text {
                        text: Cow::from(v.to_string()),
                    }),
                    style: Box::default(),
                    extra: vec![],
                })
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut bases = Vec::new();
                while let Some(element) = seq.next_element::<TextComponent>()? {
                    bases.push(element.0);
                }

                Ok(TextComponentBase {
                    content: Box::new(TextContent::Text { text: "".into() }),
                    style: Box::default(),
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

impl Serialize for TextComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct("TextComponent", &self.0.clone().to_translated())
    }
}

/// The base structure for a text component containing content, style, and children.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TextComponentBase {
    /// The actual content of this component (text, translation, etc.).
    #[serde(flatten)]
    pub content: Box<TextContent>,
    /// The styling applied to this component (color, bold, click events, etc.).
    #[serde(flatten)]
    pub style: Box<Style>,
    /// Child text components that are appended after this component's content.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Self>,
}

/// The content type of the text component.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum TextContent {
    /// Raw, untranslated text.
    Text { text: Cow<'static, str> },
    /// Text that should be translated on the client.
    Translate {
        /// The translation key (e.g. "multiplayer.player.joined").
        translate: Cow<'static, str>,
        /// Bedrock translation key. If specified, Bedrock clients receive an `SText::translation` packet.
        #[serde(skip, default)]
        bedrock_translate: Option<Cow<'static, str>>,
        /// Substitution parameters for the translation.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        with: Vec<TextComponentBase>,
    },
    /// Displays the name of one or more entities found by a selector.
    EntityNames {
        /// The entity selector string (e.g., "@e[type=pig]").
        selector: Cow<'static, str>,
        /// Optional separator between multiple entity names.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        separator: Option<Cow<'static, str>>,
    },
    /// A keybind identifier for a configurable control.
    ///
    /// See <https://minecraft.wiki/w/Controls#Configurable_controls> for available keybinds.
    Keybind {
        /// The keybind identifier (e.g., "key.forward").
        keybind: Cow<'static, str>,
    },
    /// A custom translation key for modded content.
    ///
    /// This variant is not serialized directly; translations are resolved
    /// before serialization using `to_translated()`.
    #[serde(skip)]
    Custom {
        /// The full translation key with namespace (e.g. "pumpkinplus:some.text").
        key: Cow<'static, str>,
        /// The locale to use for translation.
        locale: Locale,
        /// Substitution parameters for the translation.
        with: Vec<TextComponentBase>,
    },
}

/// Tests for the text component implementations.
#[cfg(test)]
mod test {
    use pumpkin_nbt::serializer::to_bytes_unnamed;

    use crate::text::{TextComponent, TextComponentBase, color::NamedColor};
    use crate::translation::Locale;

    #[test]
    fn serialize_text_component() {
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

    #[test]
    fn deserialize_plain_string() {
        let component: TextComponent = serde_json::from_str("\"hello\"").unwrap();
        assert_eq!(component, TextComponent::text("hello"));
    }

    #[test]
    fn deserialize_component_sequence() {
        let component: TextComponent = serde_json::from_str(r#"["a", "b"]"#).unwrap();
        assert_eq!(
            component,
            TextComponent::empty().add_text("a").add_text("b")
        );
    }

    #[test]
    fn serialize_json_named_color() {
        let component = TextComponent::text("Hi").color_named(NamedColor::Gold);
        assert_eq!(
            serde_json::to_string(&component).unwrap(),
            r#"{"text":"Hi","color":"gold"}"#
        );
    }

    #[test]
    fn json_roundtrip_preserves_style() {
        let component =
            TextComponent::translate("multiplayer.player.joined", [TextComponent::text("NAME")])
                .color_named(NamedColor::Yellow)
                .bold()
                .italic();
        let json = serde_json::to_string(&component).unwrap();
        let parsed: TextComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, component);
    }

    #[test]
    fn plain_text_with_children_get_text() {
        let component = TextComponent::text("Hello").add_text(" World");
        assert_eq!(component.0.get_text(Locale::EnUs), "Hello World");
    }

    #[test]
    fn bedrock_string_uses_translation_key() {
        let component = TextComponent::translate("chat.type.text", Vec::new());
        assert_eq!(component.0.to_bedrock_string(), "%chat.type.text");
    }

    #[test]
    fn moved_impl_paths_reachable() {
        let base_pretty: fn(TextComponentBase) -> String = TextComponentBase::to_pretty_console;
        let _: fn(&TextComponentBase) -> String = TextComponentBase::to_bedrock_string;
        let _: fn(&TextComponentBase, Locale) -> String = TextComponentBase::to_bedrock_legacy;
        let _: fn(TextComponentBase) -> TextComponentBase = TextComponentBase::to_translated;
        let _: fn(TextComponent, TextComponent) -> TextComponent = TextComponent::add_child;
        let _: fn(&TextComponent) -> Box<[u8]> = TextComponent::encode;
        let _: fn(Vec<TextComponent>) -> TextComponent = TextComponent::join_with_comma;
        assert_eq!(base_pretty(TextComponent::text("plain").0), "plain");
    }
}
