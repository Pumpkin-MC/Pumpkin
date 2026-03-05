use crate::text::color::{ARGBColor, hsv_to_rgb};
use crate::translation::{
    Locale, get_translation, get_translation_text, reorder_substitutions, translation_to_pretty,
};
use click::ClickEvent;
use color::Color;
use colored::Colorize;
use core::str;
use hover::HoverEvent;
use pumpkin_nbt::serializer::Serializer;
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt::Formatter;
use style::Style;

pub mod click;
pub mod color;
pub mod hover;
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
    /// Converts this component to a human-readable string for console output.
    ///
    /// # Returns
    /// A formatted string ready for console output.
    #[must_use]
    pub fn to_pretty_console(self) -> String {
        let mut text = match *self.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate { translate, with } => {
                translation_to_pretty(format!("minecraft:{translate}"), Locale::EnUs, with)
            }
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
            TextContent::Custom { key, with, .. } => translation_to_pretty(key, Locale::EnUs, with),
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
        for child in self.extra {
            text += &*child.to_pretty_console();
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
        match *self.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate { translate, with } => {
                get_translation_text(format!("minecraft:{translate}"), locale, with)
            }
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
            TextContent::Custom { key, with, .. } => get_translation_text(key, locale, with),
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
        let mut extra = vec![];
        for extra_component in component.extra {
            let translated = extra_component.to_translated();
            extra.push(translated);
        }
        // If the hover event is present, it will also be translated
        let style = match component.style.hover_event {
            None => component.style,
            Some(ref hover) => {
                let mut style = component.style.clone();
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
                                name: Some(
                                    name.iter().map(|x| x.to_owned().to_translated()).collect(),
                                ),
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
                style
            }
        };
        Self {
            content: component.content,
            style,
            extra,
        }
    }
}

impl TextComponent {
    /// Creates a new text component with plain text content.
    ///
    /// # Arguments
    /// - `plain` – The text content (can be `String`, `&str`, or `Cow <'static, str>`).
    ///
    /// # Returns
    /// A new `TextComponent` containing the given text.
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
    pub fn translate<K: Into<Cow<'static, str>>, W: Into<Vec<Self>>>(key: K, with: W) -> Self {
        Self(TextComponentBase {
            content: Box::new(TextContent::Translate {
                translate: key.into(),
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

    /// Parses a legacy Minecraft formatted string (using section signs '§') into a text component.
    ///
    /// Legacy formatting uses the section sign (§) followed by a formatting code:
    /// - Colors: 0-9, a-f
    /// - Styles: l (bold), o (italic), n (underline), m (strikethrough), k (obfuscated)
    /// - Reset: r
    /// - RGB hex colors: §x§R§R§G§G§B§B
    ///
    /// # Arguments
    /// - `input` – The legacy formatted string.
    ///
    /// # Returns
    /// A `TextComponent` with the parsed formatting applied.
    #[must_use]
    pub fn from_legacy_string(input: &str) -> Self {
        let mut root = Self::text("");
        let parts: Vec<&str> = input.split('§').collect();

        if !parts[0].is_empty() {
            root = root.add_child(Self::text(parts[0].to_string()));
        }

        let mut current_color: Option<Color> = None;
        let mut bold = false;
        let mut italic = false;
        let mut underlined = false;
        let mut strikethrough = false;
        let mut obfuscated = false;

        let mut i = 1;
        while i < parts.len() {
            let part = parts[i];
            if part.is_empty() {
                i += 1;
                continue;
            }

            let mut chars = part.chars();
            let code = chars.next().unwrap_or(' ').to_ascii_lowercase();
            let remainder = &part[1..];

            match code {
                'x' if i + 6 < parts.len() => {
                    let mut hex = String::new();
                    for j in 1..=6 {
                        if let Some(c) = parts[i + j].chars().next() {
                            hex.push(c);
                        }
                    }
                    current_color = Color::from_hex_str(&hex);

                    i += 6;

                    let last_part = parts[i];
                    if last_part.len() > 1 {
                        let mut child = Self::text(last_part[1..].to_string());
                        if let Some(c) = current_color {
                            child = child.color(c);
                        }
                        root = root.add_child(child);
                    }
                    i += 1;
                    continue;
                }
                '0'..='9' | 'a'..='f' => {
                    current_color = Color::from_legacy_code(code);
                    bold = false;
                    italic = false;
                    underlined = false;
                    strikethrough = false;
                    obfuscated = false;
                }
                'l' => bold = true,
                'o' => italic = true,
                'n' => underlined = true,
                'm' => strikethrough = true,
                'k' => obfuscated = true,
                'r' => {
                    current_color = None;
                    bold = false;
                    italic = false;
                    underlined = false;
                    strikethrough = false;
                    obfuscated = false;
                }
                _ => {}
            }

            if !remainder.is_empty() {
                let mut child = Self::text(remainder.to_string());
                if let Some(c) = current_color {
                    child = child.color(c);
                }
                if bold {
                    child = child.bold();
                }
                if italic {
                    child = child.italic();
                }
                if underlined {
                    child = child.underlined();
                }
                if strikethrough {
                    child = child.strikethrough();
                }
                if obfuscated {
                    child = child.obfuscated();
                }
                root = root.add_child(child);
            }
            i += 1;
        }

        root
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
    /// Encodes this component into a byte array using NBT serialization.
    ///
    /// # Returns
    /// A boxed byte slice containing the NBT-encoded component.
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let mut buf = Vec::new();
        // TODO: Properly handle errors
        let mut serializer = Serializer::new(&mut buf, None);
        self.0
            .clone()
            .to_translated()
            .serialize(&mut serializer)
            .expect("Failed to serialize text component NBT for encode");

        buf.into_boxed_slice()
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
        self
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

    use crate::text::{TextComponent, color::NamedColor};

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
}
