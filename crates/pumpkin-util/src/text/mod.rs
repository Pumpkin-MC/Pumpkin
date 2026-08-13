use crate::text::color::{ARGBColor, hsv_to_rgb};
use crate::translation::{
    Locale, get_translation, get_translation_text, reorder_substitutions, translation_to_pretty,
};
use click::ClickEvent;
use color::Color;
use colored::Colorize;
use core::str;
use hover::HoverEvent;
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt::Formatter;
use std::fmt::Write;
use std::sync::LazyLock;
use style::Style;

pub mod click;
pub mod color;
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

impl TextComponentBase {
    /// Converts this component to an NBT compound tag.
    #[expect(clippy::too_many_lines)]
    #[must_use]
    pub fn to_nbt_compound(&self) -> pumpkin_nbt::NbtCompound {
        let mut compound = pumpkin_nbt::NbtCompound::new();
        match &*self.content {
            TextContent::Text { text } => {
                compound.put_string("text", text.to_string());
            }
            TextContent::Translate {
                translate, with, ..
            } => {
                compound.put_string("translate", translate.to_string());
                if !with.is_empty() {
                    let list = with
                        .iter()
                        .map(|w| pumpkin_nbt::tag::NbtTag::Compound(w.to_nbt_compound()))
                        .collect();
                    compound.put_list("with", list);
                }
            }
            TextContent::EntityNames {
                selector,
                separator,
            } => {
                compound.put_string("selector", selector.to_string());
                if let Some(sep) = separator {
                    compound.put_string("separator", sep.to_string());
                }
            }
            TextContent::Keybind { keybind } => {
                compound.put_string("keybind", keybind.to_string());
            }
            TextContent::Custom { key, with, .. } => {
                compound.put_string("translate", key.to_string());
                if !with.is_empty() {
                    let list = with
                        .iter()
                        .map(|w| pumpkin_nbt::tag::NbtTag::Compound(w.to_nbt_compound()))
                        .collect();
                    compound.put_list("with", list);
                }
            }
            TextContent::PlayerSprite {
                type_name,
                profile,
                hat,
            } => {
                let full_type = if type_name.contains(':') {
                    type_name.to_string()
                } else {
                    format!("minecraft:{type_name}")
                };
                compound.put_string("type", full_type);
                compound.put_compound("player", profile.0.clone());
                compound.put_byte("hat", i8::from(*hat));
            }
        }

        if let Some(ref color) = self.style.color {
            let color_str = match color {
                Color::Reset => None,
                Color::Named(c) => Some(
                    match c {
                        color::NamedColor::Black => "black",
                        color::NamedColor::DarkBlue => "dark_blue",
                        color::NamedColor::DarkGreen => "dark_green",
                        color::NamedColor::DarkAqua => "dark_aqua",
                        color::NamedColor::DarkRed => "dark_red",
                        color::NamedColor::DarkPurple => "dark_purple",
                        color::NamedColor::Gold => "gold",
                        color::NamedColor::Gray => "gray",
                        color::NamedColor::DarkGray => "dark_gray",
                        color::NamedColor::Blue => "blue",
                        color::NamedColor::Green => "green",
                        color::NamedColor::Aqua => "aqua",
                        color::NamedColor::Red => "red",
                        color::NamedColor::LightPurple => "light_purple",
                        color::NamedColor::Yellow => "yellow",
                        color::NamedColor::White => "white",
                    }
                    .to_string(),
                ),
                Color::Rgb(rgb) => {
                    Some(format!("#{:02X}{:02X}{:02X}", rgb.red, rgb.green, rgb.blue))
                }
            };
            if let Some(cs) = color_str {
                compound.put_string("color", cs);
            }
        }

        if let Some(bold) = self.style.bold {
            compound.put_byte("bold", i8::from(bold));
        }
        if let Some(italic) = self.style.italic {
            compound.put_byte("italic", i8::from(italic));
        }
        if let Some(underlined) = self.style.underlined {
            compound.put_byte("underlined", i8::from(underlined));
        }
        if let Some(strikethrough) = self.style.strikethrough {
            compound.put_byte("strikethrough", i8::from(strikethrough));
        }
        if let Some(obfuscated) = self.style.obfuscated {
            compound.put_byte("obfuscated", i8::from(obfuscated));
        }

        if let Some(ref click) = self.style.click_event {
            let mut click_tag = pumpkin_nbt::NbtCompound::new();
            match click {
                ClickEvent::OpenUrl { url } => {
                    click_tag.put_string("action", "open_url".to_string());
                    click_tag.put_string("url", url.to_string());
                }
                ClickEvent::OpenFile { path } => {
                    click_tag.put_string("action", "open_file".to_string());
                    click_tag.put_string("path", path.to_string());
                }
                ClickEvent::RunCommand { command } => {
                    click_tag.put_string("action", "run_command".to_string());
                    click_tag.put_string("command", command.to_string());
                }
                ClickEvent::SuggestCommand { command } => {
                    click_tag.put_string("action", "suggest_command".to_string());
                    click_tag.put_string("command", command.to_string());
                }
                ClickEvent::ChangePage { page } => {
                    click_tag.put_string("action", "change_page".to_string());
                    click_tag.put_int("page", *page as i32);
                }
                ClickEvent::CopyToClipboard { value } => {
                    click_tag.put_string("action", "copy_to_clipboard".to_string());
                    click_tag.put_string("value", value.to_string());
                }
            }
            compound.put_compound("click_event", click_tag);
        }

        if let Some(ref hover) = self.style.hover_event {
            let mut hover_tag = pumpkin_nbt::NbtCompound::new();
            match hover {
                HoverEvent::ShowText { value } => {
                    hover_tag.put_string("action", "show_text".to_string());
                    if value.len() == 1 {
                        hover_tag.put_compound("value", value[0].to_nbt_compound());
                    } else {
                        let list = value
                            .iter()
                            .map(|e| pumpkin_nbt::tag::NbtTag::Compound(e.to_nbt_compound()))
                            .collect();
                        hover_tag.put_list("value", list);
                    }
                }
                HoverEvent::ShowItem { id, count } => {
                    hover_tag.put_string("action", "show_item".to_string());
                    hover_tag.put_string("id", id.to_string());
                    if let Some(cnt) = count {
                        hover_tag.put_int("count", *cnt);
                    }
                }
                HoverEvent::ShowEntity { id, uuid, name } => {
                    hover_tag.put_string("action", "show_entity".to_string());
                    hover_tag.put_string("id", id.to_string());
                    hover_tag.put_string("uuid", uuid.to_string());
                    if let Some(n) = name {
                        if n.len() == 1 {
                            hover_tag.put_compound("name", n[0].to_nbt_compound());
                        } else {
                            let list = n
                                .iter()
                                .map(|e| pumpkin_nbt::tag::NbtTag::Compound(e.to_nbt_compound()))
                                .collect();
                            hover_tag.put_list("name", list);
                        }
                    }
                }
            }
            compound.put_compound("hover_event", hover_tag);
        }

        if !self.extra.is_empty() {
            let list = self
                .extra
                .iter()
                .map(|e| pumpkin_nbt::tag::NbtTag::Compound(e.to_nbt_compound()))
                .collect();
            compound.put_list("extra", list);
        }

        compound
    }
    /// Converts this component to a human-readable string for console output.
    ///
    /// # Returns
    /// A formatted string ready for console output.
    #[must_use]
    pub fn to_pretty_console(self) -> String {
        fn osc8_link(url: &str, text: &str) -> String {
            format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
        }

        let mut text = match *self.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate {
                translate,
                bedrock_translate,
                with,
            } => {
                let key = bedrock_translate.as_ref().unwrap_or(&translate);
                translation_to_pretty(format!("minecraft:{key}"), Locale::EnUs, with)
            }
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
            TextContent::Custom { key, with, .. } => translation_to_pretty(key, Locale::EnUs, with),
            TextContent::PlayerSprite { ref profile, .. } => profile
                .0
                .get_string("name")
                .map_or_else(|| "player_sprite".to_string(), ToString::to_string),
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
        if let Some(ClickEvent::OpenUrl { url }) = style.click_event.as_ref() {
            text = osc8_link(url, &text);
        }
        if let Some(ClickEvent::OpenFile { path }) = style.click_event.as_ref() {
            text = osc8_link(&format!("file://{path}"), &text);
        }

        for child in self.extra {
            text += &*child.to_pretty_console();
        }
        text
    }

    /// Converts this component into a raw Bedrock string, specifically for translation parameters.
    /// Translations are emitted as `%translation.key` so Bedrock evaluates them natively.
    #[must_use]
    pub fn to_bedrock_string(&self) -> String {
        let mut text = String::new();

        match &*self.content {
            TextContent::Text { text: t } => text.push_str(t),
            TextContent::Translate {
                translate,
                bedrock_translate,
                with: _,
            } => {
                let key = bedrock_translate.as_deref().unwrap_or(translate.as_ref());
                let _ = write!(text, "%{key}");
            }
            TextContent::EntityNames { selector, .. } => text.push_str(selector),
            TextContent::Keybind { keybind } => text.push_str(keybind),
            TextContent::Custom { key, .. } => {
                let _ = write!(text, "%{key}");
            }
            TextContent::PlayerSprite { profile, .. } => {
                if let Some(name) = profile.0.get_string("name") {
                    text.push_str(name);
                }
            }
        }

        for child in &self.extra {
            text.push_str(&child.to_bedrock_string());
        }

        text
    }

    #[must_use]
    pub fn to_bedrock_legacy(&self, locale: Locale) -> String {
        let mut text = String::new();

        // 1. Inject Bedrock formatting codes
        if let Some(color) = &self.style.color {
            match color {
                Color::Named(named) => {
                    let _ = write!(text, "§{}", named.to_legacy_char());
                }
                Color::Rgb(_rgb) => {
                    // Bedrock doesn't strictly support Java's §x hex format.
                    // Most Bedrock implementations fallback to Gray or ignore it.
                }
                Color::Reset => {
                    // Explicitly handle the Reset variant
                    text.push_str("§r");
                }
            }
        }

        if self.style.bold == Some(true) {
            text.push_str("§l");
        }
        if self.style.italic == Some(true) {
            text.push_str("§o");
        }
        if self.style.underlined == Some(true) {
            text.push_str("§n");
        }
        if self.style.obfuscated == Some(true) {
            text.push_str("§k");
        }
        // Note: Bedrock does not support strikethrough natively without resource packs.

        // 2. Resolve Content
        match &*self.content {
            TextContent::Text { text: t } => text.push_str(t),
            TextContent::Translate {
                translate,
                bedrock_translate,
                with,
            } => {
                let key = bedrock_translate.as_ref().unwrap_or(translate);
                text.push_str(&get_translation_text(key.to_string(), locale, with.clone()));
            }
            TextContent::EntityNames { selector, .. } => text.push_str(selector),
            TextContent::Keybind { keybind } => text.push_str(keybind),
            TextContent::Custom { key, with, .. } => {
                text.push_str(&get_translation_text(key.clone(), locale, with.clone()));
            }
            TextContent::PlayerSprite { profile, .. } => {
                if let Some(name) = profile.0.get_string("name") {
                    text.push_str(name);
                }
            }
        }

        // 3. Recursively append extra components
        for child in &self.extra {
            text.push_str(&child.to_bedrock_legacy(locale));
            // Bedrock styles bleed into subsequent text. We append a reset code
            // to ensure child styles are properly isolated from one another.
            text.push_str("§r");
        }

        text
    }

    /// Extracts the raw text content of this component for the given locale.
    ///
    /// # Arguments
    /// - `locale` – The locale to use for translations.
    ///
    /// # Returns
    /// The plain text content of the component.
    #[must_use]
    pub fn get_text(self, locale: Locale) -> String {
        let mut text = match *self.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate {
                translate,
                bedrock_translate,
                with,
            } => {
                let key = bedrock_translate.as_ref().unwrap_or(&translate);
                get_translation_text(format!("minecraft:{key}"), locale, with)
            }
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
            TextContent::Custom { key, with, .. } => get_translation_text(key, locale, with),
            TextContent::PlayerSprite { profile, .. } => profile
                .0
                .get_string("name")
                .map(ToString::to_string)
                .unwrap_or_default(),
        };

        // Recursively append the text of all child components
        for child in self.extra {
            text += &child.get_text(locale);
        }

        text
    }

    /// Converts this component by resolving all translations.
    ///
    /// # Returns
    /// A new component with all translations resolved.
    fn translate_hover_event(style: &mut Style) {
        if let Some(ref hover) = style.hover_event {
            style.hover_event = match hover {
                HoverEvent::ShowText { value } => {
                    let mut hover_components = vec![];
                    for hover_component in value {
                        hover_components.push(hover_component.to_owned().to_translated());
                    }
                    Some(HoverEvent::ShowText {
                        value: hover_components,
                    })
                }
                HoverEvent::ShowEntity { name, id, uuid } => name.as_ref().map_or_else(
                    || {
                        Some(HoverEvent::ShowEntity {
                            name: None,
                            id: id.clone(),
                            uuid: uuid.clone(),
                        })
                    },
                    |name| {
                        Some(HoverEvent::ShowEntity {
                            name: Some(name.iter().map(|x| x.to_owned().to_translated()).collect()),
                            id: id.clone(),
                            uuid: uuid.clone(),
                        })
                    },
                ),
                HoverEvent::ShowItem { id, count } => Some(HoverEvent::ShowItem {
                    id: id.clone(),
                    count: count.to_owned(),
                }),
            };
        }
    }

    /// Converts this component by resolving all translations.
    ///
    /// # Returns
    /// A new component with all translations resolved.
    #[must_use]
    pub fn to_translated(self) -> Self {
        // NOTE: Divide the translation into slices and inserts the substitutions.
        let component = match *self.content {
            TextContent::Translate {
                translate,
                bedrock_translate,
                with,
            } => {
                let mut translated_with = vec![];
                for w in with {
                    translated_with.push(w.to_translated());
                }
                Self {
                    content: Box::new(TextContent::Translate {
                        translate,
                        bedrock_translate,
                        with: translated_with,
                    }),
                    style: self.style,
                    extra: self.extra,
                }
            }
            TextContent::Custom { key, with, locale } => {
                let translation = get_translation(&key, locale);
                let mut translation_parent = translation.clone();
                let mut translation_slices = vec![];

                if translation.contains('%') {
                    let (substitutions, ranges) = reorder_substitutions(&translation, with);
                    for (idx, &range) in ranges.iter().enumerate() {
                        if idx == 0 {
                            translation_parent = translation[..range.start].to_string();
                        }
                        translation_slices.push(substitutions[idx].clone());
                        if range.end >= translation.len() - 1 {
                            continue;
                        }

                        translation_slices.push(Self {
                            content: Box::new(TextContent::Text {
                                text: if idx == ranges.len() - 1 {
                                    // Last substitution, append the rest of the translation
                                    Cow::Owned(translation[range.end + 1..].to_string())
                                } else {
                                    Cow::Owned(
                                        translation[range.end + 1..ranges[idx + 1].start]
                                            .to_string(),
                                    )
                                },
                            }),
                            style: Box::new(Style::default()),
                            extra: vec![],
                        });
                    }
                }
                for i in self.extra {
                    translation_slices.push(i);
                }
                Self {
                    content: Box::new(TextContent::Text {
                        text: translation_parent.into(),
                    }),
                    style: self.style,
                    extra: translation_slices,
                }
            }
            _ => self, // If not a translation, return as is
        };
        // Ensure that the extra components are translated
        let extra = component
            .extra
            .into_iter()
            .map(Self::to_translated)
            .collect();

        // If the hover event is present, it will also be translated
        let mut style = component.style;
        Self::translate_hover_event(&mut style);

        Self {
            content: component.content,
            style,
            extra,
        }
    }
}

impl TextComponent {
    /// Creates a new text component without any text content.
    ///
    /// Useful to join multiple text components together into one
    /// by putting them all as a child of an empty text component
    /// in the required order.
    ///
    /// # Returns
    /// An empty `TextComponent`.
    #[must_use]
    pub fn empty() -> Self {
        Self::text("")
    }

    /// Creates a new text component with plain text content.
    ///
    /// # Arguments
    /// - `plain` – The text content (can be `String`, `&str`, or `Cow <'static, str>`).
    ///
    /// # Returns
    /// A new `TextComponent` containing the given text.
    #[must_use]
    pub fn text<P: Into<Cow<'static, str>>>(plain: P) -> Self {
        Self(TextComponentBase {
            content: Box::new(TextContent::Text { text: plain.into() }),
            style: Box::new(Style::default()),
            extra: vec![],
        })
    }

    /// Creates a new text component with a translation key.
    ///
    /// # Arguments
    /// - `key` – The translation key (e.g., "multiplayer.player.joined").
    /// - `with` – The substitution parameters for the translation.
    ///
    /// # Returns
    /// A new `TextComponent` that will be translated on the client.
    #[deprecated(
        since = "0.1.0",
        note = "Use the `pumpkin_macros::translate_java!` macro instead for compile-time translation key and parameter checking."
    )]
    #[must_use]
    pub fn translate<K: Into<Cow<'static, str>>, W: Into<Vec<Self>>>(key: K, with: W) -> Self {
        Self(TextComponentBase {
            content: Box::new(TextContent::Translate {
                translate: key.into(),
                bedrock_translate: None,
                with: with.into().into_iter().map(|x| x.0).collect(),
            }),
            style: Box::new(Style::default()),
            extra: vec![],
        })
    }

    /// Creates a new text component with a translation key that has a Bedrock-specific fallback.
    ///
    /// # Arguments
    /// - `java_key` – The translation key for Java (e.g., "multiplayer.player.joined").
    /// - `bedrock_key` – The translation key for Bedrock (e.g., "multiplayer.player.joined").
    /// - `with` – The substitution parameters for the translation.
    ///
    /// # Returns
    /// A new `TextComponent` that will be translated natively on both clients.
    #[deprecated(
        since = "0.1.0",
        note = "Use the `pumpkin_macros::translate_cross!` macro instead for compile-time translation key and parameter checking."
    )]
    #[must_use]
    pub fn translate_cross<
        K1: Into<Cow<'static, str>>,
        K2: Into<Cow<'static, str>>,
        W: Into<Vec<Self>>,
    >(
        java_key: K1,
        bedrock_key: K2,
        with: W,
    ) -> Self {
        Self(TextComponentBase {
            content: Box::new(TextContent::Translate {
                translate: java_key.into(),
                bedrock_translate: Some(bedrock_key.into()),
                with: with.into().into_iter().map(|x| x.0).collect(),
            }),
            style: Box::new(Style::default()),
            extra: vec![],
        })
    }

    /// Creates a new text component with a custom translation key.
    ///
    /// # Arguments
    /// - `namespace` – The namespace for the translation (e.g. "pumpkinplus").
    /// - `key` – The translation key within the namespace.
    /// - `locale` – The locale to use for translation.
    /// - `with` – The substitution parameters for the translation.
    ///
    /// # Returns
    /// A new `TextComponent` with custom translation.
    #[must_use]
    pub fn custom<K: Into<Cow<'static, str>>, W: Into<Vec<Self>>>(
        namespace: K,
        key: K,
        locale: Locale,
        with: W,
    ) -> Self {
        Self(TextComponentBase {
            content: Box::new(TextContent::Custom {
                key: format!("{}:{}", namespace.into(), key.into())
                    .to_lowercase()
                    .into(),
                locale,
                with: with.into().into_iter().map(|x| x.0).collect(),
            }),
            style: Box::new(Style::default()),
            extra: vec![],
        })
    }

    /// Appends a child component to this component.
    ///
    /// # Arguments
    /// - `child` – The component to append.
    ///
    /// # Returns
    /// The component with the child added.
    #[must_use]
    pub fn add_child(mut self, child: Self) -> Self {
        self.0.extra.push(child.0);
        self
    }

    /// Creates a new component from raw content.
    ///
    /// # Arguments
    /// - `content` – The text content.
    ///
    /// # Returns
    /// A new component with the given content.
    #[must_use]
    pub fn from_content(content: TextContent) -> Self {
        Self(TextComponentBase {
            content: Box::new(content),
            style: Box::new(Style::default()),
            extra: vec![],
        })
    }

    /// Appends plain text to this component.
    ///
    /// # Arguments
    /// - `text` – The text to append.
    ///
    /// # Returns
    /// The component with the text appended.
    #[must_use]
    pub fn add_text<P: Into<Cow<'static, str>>>(mut self, text: P) -> Self {
        self.0.extra.push(TextComponentBase {
            content: Box::new(TextContent::Text { text: text.into() }),
            style: Box::new(Style::default()),
            extra: vec![],
        });
        self
    }

    /// Extracts the raw text content for English (US).
    ///
    /// # Returns
    /// The plain text content.
    #[must_use]
    pub fn get_text(self) -> String {
        self.0.get_text(Locale::EnUs)
    }

    /// Creates a chat message with formatting placeholders replaced.
    ///
    /// Replaces:
    /// - `&` with `§` for legacy formatting
    /// - `{DISPLAYNAME}` with the player's name
    /// - `{MESSAGE}` with the chat message content
    ///
    /// # Arguments
    /// - `format` – The message format string.
    /// - `player_name` – The player's display name.
    /// - `content` – The chat message content.
    ///
    /// # Returns
    /// A formatted chat component.
    #[must_use]
    pub fn chat_decorated(format: &str, player_name: &str, content: &str) -> Self {
        // Todo: maybe allow players to use & in chat contingent on permissions
        let with_resolved_fields = format
            .replace('&', "§")
            .replace("{DISPLAYNAME}", player_name)
            .replace("{MESSAGE}", content);

        Self(TextComponentBase {
            content: Box::new(TextContent::Text {
                text: Cow::Owned(with_resolved_fields),
            }),
            style: Box::new(Style::default()),
            extra: vec![],
        })
    }

    /// Converts this component to a pretty console string.
    ///
    /// # Returns
    /// A formatted string ready for console output.
    #[must_use]
    pub fn to_pretty_console(self) -> String {
        self.0.to_pretty_console()
    }
}

impl TextComponent {
    /// Creates a player sprite component.
    #[must_use]
    pub fn player_sprite(profile: pumpkin_nbt::NbtCompound, hat: bool) -> Self {
        Self(TextComponentBase {
            content: Box::new(TextContent::PlayerSprite {
                type_name: Cow::Borrowed("minecraft:player_sprite"),
                profile: ProfileNbt(profile),
                hat,
            }),
            style: Box::default(),
            extra: vec![],
        })
    }

    /// Encodes this component into a byte array using NBT serialization.
    ///
    /// # Returns
    /// A boxed byte slice containing the NBT-encoded component.
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let compound = self.0.clone().to_translated().to_nbt_compound();
        pumpkin_nbt::Nbt::from(compound)
            .write_unnamed()
            .as_ref()
            .into()
    }

    /// Sets the text color.
    ///
    /// # Arguments
    /// - `color` – The color to apply.
    ///
    /// # Returns
    /// The component with the color set.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.0.style.color = Some(color);
        self
    }

    /// Sets the text color using a named Minecraft color.
    ///
    /// # Arguments
    /// - `color` – The named color to apply.
    ///
    /// # Returns
    /// The component with the color set.
    #[must_use]
    pub fn color_named(mut self, color: color::NamedColor) -> Self {
        self.0.style.color = Some(Color::Named(color));
        self
    }

    /// Sets the text color using an RGB color.
    ///
    /// # Arguments
    /// - `color` – The RGB color to apply.
    ///
    /// # Returns
    /// The component with the color set.
    #[must_use]
    pub fn color_rgb(mut self, color: color::RGBColor) -> Self {
        self.0.style.color = Some(Color::Rgb(color));
        self
    }

    /// Appends a new line/line break.
    ///
    /// # Returns
    /// The component with a new line appended.
    #[must_use]
    pub fn new_line(self) -> Self {
        self.add_child(Self::text("\n"))
    }

    /// Applies a color gradient to the text using named colors.
    ///
    /// # Arguments
    /// - `colors` – The gradient colors to apply.
    ///
    /// # Returns
    /// The component with the gradient applied.
    #[must_use]
    pub fn gradient_named(self, colors: &[color::NamedColor]) -> Self {
        let rgb_colors: Vec<color::RGBColor> =
            colors.iter().map(color::NamedColor::to_rgb).collect();
        self.gradient(&rgb_colors)
    }

    /// Applies a color gradient to the text using RGB colors.
    ///
    /// # Arguments
    /// - `colors` – The gradient colors to apply.
    ///
    /// # Returns
    /// The component with the gradient applied.
    #[must_use]
    pub fn gradient(self, colors: &[color::RGBColor]) -> Self {
        if colors.len() < 2 {
            return self;
        }

        self.apply_color_effect(|i, len| {
            if len <= 1 {
                return colors[0];
            }
            let total_segments = colors.len() - 1;
            let position = i as f32 / (len - 1) as f32;
            let segment_f = position * total_segments as f32;
            let segment_index = (segment_f.floor() as usize).min(total_segments - 1);

            let local_t = segment_f - segment_index as f32;
            let start = colors[segment_index];
            let end = colors[segment_index + 1];

            // LERP logic
            color::RGBColor::new(
                (f32::from(end.red) - f32::from(start.red)).mul_add(local_t, f32::from(start.red))
                    as u8,
                (f32::from(end.green) - f32::from(start.green))
                    .mul_add(local_t, f32::from(start.green)) as u8,
                (f32::from(end.blue) - f32::from(start.blue))
                    .mul_add(local_t, f32::from(start.blue)) as u8,
            )
        })
    }

    /// Applies a rainbow effect to the text.
    ///
    /// Each character gets a different hue, creating a smooth rainbow transition.
    ///
    /// # Returns
    /// The component with the rainbow effect applied.
    #[must_use]
    pub fn rainbow(self) -> Self {
        self.apply_color_effect(|i, len| {
            let hue = (i as f32 / len as f32) * 360.0;
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            color::RGBColor::new(r, g, b)
        })
    }

    /// Applies a per-character color effect to the text content.
    ///
    /// # Arguments
    /// - `color_gen` – A function that takes the character index and total length
    ///   and returns an RGB color for that character.
    ///
    /// # Returns
    /// A new text component where each character is individually colored according
    /// to the generator function. The original component's content becomes empty,
    /// and the colored characters are placed in the `extra` field.
    fn apply_color_effect<F>(mut self, color_gen: F) -> Self
    where
        F: Fn(usize, usize) -> color::RGBColor,
    {
        let raw_text = self.0.clone().get_text(Locale::EnUs);
        let chars: Vec<char> = raw_text.chars().collect();
        let len = chars.len();

        if len == 0 {
            return self;
        }

        let mut colored_extra = Vec::new();
        for (i, c) in chars.into_iter().enumerate() {
            let rgb = color_gen(i, len);

            let mut char_base = TextComponentBase {
                content: Box::new(TextContent::Text {
                    text: Cow::Owned(c.to_string()),
                }),
                style: self.0.style.clone(),
                extra: vec![],
            };
            char_base.style.color = Some(Color::Rgb(rgb));
            colored_extra.push(char_base);
        }

        self.0.content = Box::new(TextContent::Text { text: "".into() });
        self.0.extra = colored_extra;
        self.0.style.click_event = None;
        self.0.style.hover_event = None;
        self
    }

    /// Wraps a component in square brackets.
    ///
    /// # Returns
    /// The new component.
    #[allow(deprecated)]
    #[must_use]
    pub fn wrap_in_square_brackets(self) -> Self {
        Self::translate("chat.square_brackets", [self])
    }

    /// Makes the text bold.
    ///
    /// # Returns
    /// The component with bold enabled.
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.0.style.bold = Some(true);
        self
    }

    /// Makes the text italic.
    ///
    /// # Returns
    /// The component with italic enabled.
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.0.style.italic = Some(true);
        self
    }

    /// Makes the text underlined.
    ///
    /// # Returns
    /// The component with underline enabled.
    #[must_use]
    pub fn underlined(mut self) -> Self {
        self.0.style.underlined = Some(true);
        self
    }

    /// Makes the text strikethrough.
    ///
    /// # Returns
    /// The component with strikethrough enabled.
    #[must_use]
    pub fn strikethrough(mut self) -> Self {
        self.0.style.strikethrough = Some(true);
        self
    }

    /// Makes the text obfuscated (random characters).
    ///
    /// # Returns
    /// The component with obfuscation enabled.
    #[must_use]
    pub fn obfuscated(mut self) -> Self {
        self.0.style.obfuscated = Some(true);
        self
    }

    /// Sets text to be inserted into the player's chat input when shift-clicked.
    ///
    /// When the text is shift-clicked by a player, this string is inserted in their
    /// chat input. It does not overwrite any existing text the player was writing.
    /// This only works in chat messages.
    ///
    /// # Arguments
    /// - `text` – The text to insert when shift-clicked.
    ///
    /// # Returns
    /// The component with the insertion text set.
    #[must_use]
    pub fn insertion(mut self, text: String) -> Self {
        self.0.style.insertion = Some(text);
        self
    }

    /// Sets an event to occur when the player clicks on the text.
    ///
    /// Allows for actions like running commands, opening URLs, suggesting commands,
    /// or copying text to clipboard. Only works in chat.
    ///
    /// # Arguments
    /// - `event` – The click event to trigger.
    ///
    /// # Returns
    /// The component with the click event set.
    #[must_use]
    pub fn click_event(mut self, event: ClickEvent) -> Self {
        self.0.style.click_event = Some(event);
        self
    }

    /// Sets a tooltip to be displayed when the player hovers over the text.
    ///
    /// Can show plain text, item information, or entity details.
    ///
    /// # Arguments
    /// - `event` – The hover event to display.
    ///
    /// # Returns
    /// The component with the hover event set.
    #[must_use]
    pub fn hover_event(mut self, event: HoverEvent) -> Self {
        self.0.style.hover_event = Some(event);
        self
    }

    /// Sets the font resource location for rendering.
    ///
    /// Allows changing the font face of the text. Default fonts include:
    /// - `minecraft:default` - The standard Minecraft font.
    /// - `minecraft:uniform` - A uniform-width font.
    /// - `minecraft:alt` - An alternative font style.
    /// - `minecraft:illageralt` - The illager-themed font.
    ///
    /// # Arguments
    /// - `resource_location` – The font resource location (e.g., "minecraft:uniform").
    ///
    /// # Returns
    /// The component with the font set.
    #[must_use]
    pub fn font(mut self, resource_location: String) -> Self {
        self.0.style.font = Some(resource_location);
        self
    }

    /// Overrides the shadow color of the text.
    ///
    /// # Arguments
    /// - `color` – The ARGB color value for the shadow.
    ///
    /// # Returns
    /// The component with the shadow color set.
    #[must_use]
    pub fn shadow_color(mut self, color: ARGBColor) -> Self {
        self.0.style.shadow_color = Some(color);
        self
    }
}

impl TextComponent {
    /// Joins multiple text components into one with a separator containing a gray comma
    /// and a space after it.
    ///
    /// # Arguments
    /// - `elements` - The elements to join.
    ///
    /// # Returns
    /// The resultant text component with all the elements joined in it.
    #[must_use]
    pub fn join_with_comma(elements: Vec<Self>) -> Self {
        static DEFAULT_SEPARATOR: LazyLock<TextComponent> = LazyLock::new(|| {
            TextComponent::text(", ").color(Color::Named(color::NamedColor::Gray))
        });

        Self::join(elements, &DEFAULT_SEPARATOR)
    }

    /// Joins multiple text components into one with the given separator text component.
    /// Use [`TextComponent::join_with_comma`] instead if you just want to join text components with
    /// a comma in between.
    ///
    /// # Arguments
    /// - `elements` - The elements to join.
    /// - `separator` - The separator to use for joining the elements provided.
    ///
    /// # Returns
    /// The resultant text component with all the elements joined in it.
    #[must_use]
    pub fn join(elements: Vec<Self>, separator: &Self) -> Self {
        let mut result = Self::empty();
        let mut first = true;

        for element in elements {
            if !first {
                result = result.add_child(separator.clone());
            }

            result = result.add_child(element);
            first = false;
        }

        result
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProfileNbt(pub pumpkin_nbt::NbtCompound);

impl Eq for ProfileNbt {}

impl std::hash::Hash for ProfileNbt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_string().hash(state);
    }
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
    /// A player sprite object component.
    #[serde(skip)]
    PlayerSprite {
        type_name: Cow<'static, str>,
        profile: ProfileNbt,
        hat: bool,
    },
}

/// Tests for the text component implementations.
#[cfg(test)]
mod test {
    use crate::text::{TextComponent, color::NamedColor, hover::HoverEvent};
    use std::borrow::Cow;

    #[test]
    fn serialize_text_component() {
        #[allow(deprecated)]
        let msg_comp = TextComponent::translate(
            "multiplayer.player.joined",
            [TextComponent::text("NAME".to_string())],
        )
        .color_named(NamedColor::Yellow);

        let bytes = msg_comp.encode();

        let expected_compound = msg_comp.0.to_translated().to_nbt_compound();
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let decoded = pumpkin_nbt::Nbt::read_unnamed(&mut reader).unwrap();
        assert_eq!(decoded, expected_compound.into());
    }

    /// The client expects the hover event payload to be inlined next to `action`.
    /// Nesting it under `item`/`entity` makes the component fail to decode and
    /// kicks the player off the server.
    #[test]
    fn hover_event_payload_is_inlined() {
        let show_item = TextComponent::text("sword")
            .hover_event(HoverEvent::ShowItem {
                id: Cow::Borrowed("minecraft:diamond_sword"),
                count: Some(1),
            })
            .0
            .to_nbt_compound();
        let hover = show_item.get_compound("hover_event").unwrap();
        assert_eq!(hover.get_string("action"), Some("show_item"));
        assert_eq!(hover.get_string("id"), Some("minecraft:diamond_sword"));
        assert_eq!(hover.get_int("count"), Some(1));
        assert!(hover.get_compound("item").is_none());

        let show_entity = TextComponent::text("pig")
            .hover_event(HoverEvent::show_entity(
                "6ba1a740-9a3b-4b7c-8f2c-8f5a5c1a0a11",
                "minecraft:pig",
                None,
            ))
            .0
            .to_nbt_compound();
        let hover = show_entity.get_compound("hover_event").unwrap();
        assert_eq!(hover.get_string("action"), Some("show_entity"));
        assert_eq!(hover.get_string("id"), Some("minecraft:pig"));
        assert_eq!(
            hover.get_string("uuid"),
            Some("6ba1a740-9a3b-4b7c-8f2c-8f5a5c1a0a11")
        );
        assert!(hover.get_compound("entity").is_none());
    }

    /// `count` is optional for the client, so it must stay out of the payload
    /// when it was never set.
    #[test]
    fn hover_show_item_omits_unset_count() {
        let compound = TextComponent::text("sword")
            .hover_event(HoverEvent::ShowItem {
                id: Cow::Borrowed("minecraft:diamond_sword"),
                count: None,
            })
            .0
            .to_nbt_compound();
        let hover = compound.get_compound("hover_event").unwrap();
        assert!(hover.get_int("count").is_none());
    }
}
