use crate::translation::{
    Locale, get_translation, get_translation_text, reorder_substitutions,
    translation_to_pretty_console,
};
use colored::Colorize;
use std::borrow::Cow;
use std::fmt::Write;

use super::click::ClickEvent;
use super::color::Color;
use super::hover::HoverEvent;
use super::style::Style;
use super::{TextComponentBase, TextContent};

impl TextComponentBase {
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
                translation_to_pretty_console(format!("minecraft:{key}"), with)
            }
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
            TextContent::Custom { key, with, .. } => translation_to_pretty_console(key, with),
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
