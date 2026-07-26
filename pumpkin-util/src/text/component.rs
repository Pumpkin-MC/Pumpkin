use crate::translation::{Locale, get_console_translation_text, server_locale};
use pumpkin_nbt::serializer::{NbtWriteHelperJava, Serializer};
use serde::Serialize;
use std::borrow::Cow;
use std::sync::LazyLock;

use super::click::ClickEvent;
use super::color::{self, ARGBColor, Color, hsv_to_rgb};
use super::hover::HoverEvent;
use super::style::Style;
use super::{TextComponent, TextComponentBase, TextContent};

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

    /// Extracts the raw text content using the server console locale.
    ///
    /// In bilingual mode, translation keys are resolved as `中文 / English`.
    ///
    /// # Returns
    /// The plain text content.
    #[must_use]
    pub fn get_text(self) -> String {
        if crate::translation::bilingual_console() {
            // Resolve translation nodes bilingually; plain text passes through.
            match *self.0.content {
                TextContent::Translate {
                    translate,
                    bedrock_translate,
                    with,
                } => {
                    let key = bedrock_translate.as_ref().unwrap_or(&translate);
                    let mut text = get_console_translation_text(format!("minecraft:{key}"), with);
                    for child in self.0.extra {
                        text += &Self(child).get_text();
                    }
                    text
                }
                TextContent::Custom { key, with, .. } => {
                    let mut text = get_console_translation_text(key, with);
                    for child in self.0.extra {
                        text += &Self(child).get_text();
                    }
                    text
                }
                _ => self.0.get_text(server_locale()),
            }
        } else {
            self.0.get_text(server_locale())
        }
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
        let writer = NbtWriteHelperJava::new(&mut buf);
        // TODO: Properly handle errors
        let mut serializer = Serializer::new(writer, None);
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

    /// Wraps a component in square brackets.
    ///
    /// # Returns
    /// The new component.
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
