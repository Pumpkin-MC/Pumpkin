use std::fmt::Write;

use crate::text::color::Color;
use crate::text::{TextComponent, TextContent};
use crate::translation::{Locale, get_translation_text};

impl TextComponent {
    /// Parses a legacy Minecraft formatted string (using section signs '§' by default) into a text component.
    ///
    /// Legacy formatting uses formatting codes after a color symbol:
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
        Self::from_legacy_string_with_code(input, '§')
    }

    /// Parses a legacy formatted string using a custom color code symbol (e.g. '&' or '§').
    ///
    /// # Arguments
    /// - `input` – The legacy formatted string.
    /// - `code_symbol` – The delimiter character (e.g. `'§'` or `'&'`).
    ///
    /// # Returns
    /// A `TextComponent` with the parsed formatting applied.
    #[must_use]
    #[expect(clippy::too_many_lines)]
    pub fn from_legacy_string_with_code(input: &str, code_symbol: char) -> Self {
        let mut root = Self::text("");
        let mut parts = input.split(code_symbol);

        if let Some(first) = parts.next()
            && !first.is_empty()
        {
            root = root.add_child(Self::text(first.to_string()));
        }

        let mut current_color: Option<Color> = None;
        let mut bold = false;
        let mut italic = false;
        let mut underlined = false;
        let mut strikethrough = false;
        let mut obfuscated = false;

        while let Some(part) = parts.next() {
            if part.is_empty() {
                continue;
            }

            let code = part.chars().next().unwrap_or(' ').to_ascii_lowercase();
            let remainder = &part[code.len_utf8()..];

            match code {
                'x' => {
                    let mut hex = [0u8; 6];
                    let mut valid_hex = true;
                    let mut trailing_remainder = String::new();
                    for hex_byte in &mut hex {
                        if let Some(next_part) = parts.next() {
                            if let Some(c) = next_part.chars().next() {
                                *hex_byte = c as u8;
                                trailing_remainder.push_str(&next_part[c.len_utf8()..]);
                            } else {
                                valid_hex = false;
                                break;
                            }
                        } else {
                            valid_hex = false;
                            break;
                        }
                    }
                    if valid_hex && let Ok(hex_str) = std::str::from_utf8(&hex) {
                        current_color = Color::from_hex_str(hex_str);
                    }
                    if !remainder.is_empty() || !trailing_remainder.is_empty() {
                        let text = format!("{remainder}{trailing_remainder}");
                        if !text.is_empty() {
                            let mut child = Self::text(text);
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
                    }
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
        }

        root
    }

    /// Serializes this component into a Java legacy formatted string using `§`.
    ///
    /// # Arguments
    /// - `locale` – The locale to use for resolving translation components.
    ///
    /// # Returns
    /// A legacy Minecraft formatted string with `§` color and style codes.
    #[must_use]
    pub fn to_legacy_string(&self, locale: Locale) -> String {
        self.to_legacy_string_for_version(&crate::version::JavaMinecraftVersion::V_26_2, locale)
    }

    /// Serializes this component into a Java legacy formatted string for a specific Minecraft version.
    ///
    /// # Arguments
    /// - `version` – The Minecraft version to format for.
    /// - `locale` – The locale to use for resolving translation components.
    ///
    /// # Returns
    /// A legacy Minecraft formatted string with `§` color and style codes.
    #[must_use]
    pub fn to_legacy_string_for_version(
        &self,
        version: &crate::version::JavaMinecraftVersion,
        locale: Locale,
    ) -> String {
        self.to_legacy_string_with_code_for_version(version, '§', locale)
    }

    /// Serializes this component into a legacy formatted string using a custom color code symbol.
    ///
    /// # Arguments
    /// - `code_symbol` – The formatting character symbol (e.g., `'§'` or `'&'`).
    /// - `locale` – The locale to use for resolving translation components.
    ///
    /// # Returns
    /// A legacy formatted string with color and style codes using `code_symbol`.
    #[must_use]
    pub fn to_legacy_string_with_code(&self, code_symbol: char, locale: Locale) -> String {
        self.to_legacy_string_with_code_for_version(
            &crate::version::JavaMinecraftVersion::V_26_2,
            code_symbol,
            locale,
        )
    }

    /// Serializes this component into a legacy formatted string for a specific Minecraft version.
    ///
    /// # Arguments
    /// - `version` – The Minecraft version to format for.
    /// - `code_symbol` – The formatting character symbol (e.g., `'§'` or `'&'`).
    /// - `locale` – The locale to use for resolving translation components.
    ///
    /// # Returns
    /// A legacy formatted string with color and style codes using `code_symbol`.
    #[must_use]
    pub fn to_legacy_string_with_code_for_version(
        &self,
        version: &crate::version::JavaMinecraftVersion,
        code_symbol: char,
        locale: Locale,
    ) -> String {
        let mut text = String::new();

        // 1. Color formatting
        if let Some(color) = &self.0.style.color {
            match color {
                Color::Named(named) => {
                    let _ = write!(text, "{code_symbol}{}", named.to_legacy_char());
                }
                Color::Rgb(rgb) => {
                    if *version >= crate::version::JavaMinecraftVersion::V_1_16 {
                        let hex = format!("{:02x}{:02x}{:02x}", rgb.red, rgb.green, rgb.blue);
                        let _ = write!(text, "{code_symbol}x");
                        for ch in hex.chars() {
                            let _ = write!(text, "{code_symbol}{ch}");
                        }
                    } else {
                        let named = rgb.to_nearest_named();
                        let _ = write!(text, "{code_symbol}{}", named.to_legacy_char());
                    }
                }
                Color::Reset => {
                    let _ = write!(text, "{code_symbol}r");
                }
            }
        }

        // 2. Style formatting
        if self.0.style.bold == Some(true) {
            let _ = write!(text, "{code_symbol}l");
        }
        if self.0.style.italic == Some(true) {
            let _ = write!(text, "{code_symbol}o");
        }
        if self.0.style.underlined == Some(true) {
            let _ = write!(text, "{code_symbol}n");
        }
        if self.0.style.strikethrough == Some(true) {
            let _ = write!(text, "{code_symbol}m");
        }
        if self.0.style.obfuscated == Some(true) {
            let _ = write!(text, "{code_symbol}k");
        }

        // 3. Resolve Content
        match &*self.0.content {
            TextContent::Text { text: t } => text.push_str(t),
            TextContent::Translate {
                translate,
                bedrock_translate: _,
                with,
            } => {
                text.push_str(&get_translation_text(
                    format!("minecraft:{translate}"),
                    locale,
                    with.clone(),
                ));
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

        // 4. Recursively append extra components
        for child in &self.0.extra {
            text.push_str(&Self(child.clone()).to_legacy_string_with_code_for_version(
                version,
                code_symbol,
                locale,
            ));
            let _ = write!(text, "{code_symbol}r");
        }

        text
    }
}

#[cfg(test)]
mod test {
    use crate::text::TextComponent;
    use crate::text::color::{Color, NamedColor};
    use crate::translation::Locale;

    #[test]
    fn from_legacy_string_hex() {
        let comp = TextComponent::from_legacy_string("§x§f§c§0§0§0§0Test String ▽");
        assert_eq!(comp.0.extra.len(), 1);
        if let crate::text::TextContent::Text { text } = &*comp.0.extra[0].content {
            assert_eq!(text, "Test String ▽");
        } else {
            panic!("Expected Text content");
        }
        assert_eq!(comp.0.extra[0].style.color, Color::from_hex_str("fc0000"));
    }

    #[test]
    fn from_legacy_string_alt_code() {
        let comp = TextComponent::from_legacy_string_with_code("&cRed &lBold", '&');
        assert_eq!(comp.0.extra.len(), 2);
        assert_eq!(
            comp.0.extra[0].style.color,
            Some(Color::Named(NamedColor::Red))
        );
        assert_eq!(comp.0.extra[1].style.bold, Some(true));
    }

    #[test]
    fn to_legacy_string() {
        let comp = TextComponent::text("Hello ").add_child(
            TextComponent::text("World")
                .color_named(NamedColor::Red)
                .bold(),
        );
        let legacy = comp.to_legacy_string(Locale::EnUs);
        assert_eq!(legacy, "Hello §c§lWorld§r");
    }
}
