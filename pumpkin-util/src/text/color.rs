use colored::{ColoredString, Colorize};
use serde::{Deserialize, Deserializer, Serialize};

/// Text color for chat components.
///
/// Colors can be specified in three ways:
/// - `Reset` - Uses the default color for the context
/// - `Rgb` - A custom RGB color (e.g. "#FF55AA")
/// - `Named` - One of the 16 standard Minecraft named colors
#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Color {
    /// The default color for the text will be used, which varies by context
    /// (in some cases, it's white; in others, it's black; in still others, it
    /// is a shade of gray that isn't normally used on text).
    #[default]
    Reset,
    /// An RGB color specified as a hex string like "#RRGGBB".
    Rgb(RGBColor),
    /// One of the 16 named Minecraft colors.
    Named(NamedColor),
}

/// Converts HSV (Hue, Saturation, Value) color values to RGB.
///
/// # Arguments
/// - `h` – Hue in degrees (0-360)
/// - `s` – Saturation as a float (0-1)
/// - `v` – Value (brightness) as a float (0-1)
///
/// # Returns
/// A tuple of (red, green, blue) as u8 values (0-255).
#[must_use]
#[expect(clippy::many_single_char_names)]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h as i32 / 60) % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        if s == "reset" {
            Ok(Self::Reset)
        } else if let Some(hex) = s.strip_prefix('#') {
            if s.len() != 7 {
                return Err(serde::de::Error::custom(
                    "Hex color must be in the format '#RRGGBB'",
                ));
            }

            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| serde::de::Error::custom("Invalid red component in hex color"))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| serde::de::Error::custom("Invalid green component in hex color"))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| serde::de::Error::custom("Invalid blue component in hex color"))?;

            Ok(Self::Rgb(RGBColor::new(r, g, b)))
        } else {
            Ok(Self::Named(NamedColor::try_from(s.as_str()).map_err(
                |()| serde::de::Error::custom("Invalid named color"),
            )?))
        }
    }
}

impl Color {
    /// Converts this color to a colored string for terminal output.
    ///
    /// # Arguments
    /// - `text` – The text to colorize.
    ///
    /// # Returns
    /// A `ColoredString` that can be printed to the terminal.
    #[must_use]
    pub fn console_color(&self, text: &str) -> ColoredString {
        match self {
            Self::Reset => text.clear(),
            Self::Named(color) => match color {
                NamedColor::Black => text.black(),
                NamedColor::DarkBlue => text.blue(),
                NamedColor::DarkGreen => text.green(),
                NamedColor::DarkAqua => text.cyan(),
                NamedColor::DarkRed => text.red(),
                NamedColor::DarkPurple => text.purple(),
                NamedColor::Gold => text.yellow(),
                NamedColor::Gray | NamedColor::DarkGray => text.bright_black(), // ?
                NamedColor::Blue => text.bright_blue(),
                NamedColor::Green => text.bright_green(),
                NamedColor::Aqua => text.bright_cyan(),
                NamedColor::Red => text.bright_red(),
                NamedColor::LightPurple => text.bright_purple(),
                NamedColor::Yellow => text.bright_yellow(),
                NamedColor::White => text.white(),
            },
            // TODO: Check if terminal supports true color
            Self::Rgb(color) => text.truecolor(color.red, color.green, color.blue),
        }
    }

    /// Creates a color from a legacy Minecraft color code.
    ///
    /// # Arguments
    /// - `code` – The legacy color code character (0-9, a-f).
    ///
    /// # Returns
    /// The corresponding `Color`, or `None` if the code is invalid.
    #[must_use]
    pub const fn from_legacy_code(code: char) -> Option<Self> {
        let named = match code.to_ascii_lowercase() {
            '0' => NamedColor::Black,
            '1' => NamedColor::DarkBlue,
            '2' => NamedColor::DarkGreen,
            '3' => NamedColor::DarkAqua,
            '4' => NamedColor::DarkRed,
            '5' => NamedColor::DarkPurple,
            '6' => NamedColor::Gold,
            '7' => NamedColor::Gray,
            '8' => NamedColor::DarkGray,
            '9' => NamedColor::Blue,
            'a' => NamedColor::Green,
            'b' => NamedColor::Aqua,
            'c' => NamedColor::Red,
            'd' => NamedColor::LightPurple,
            'e' => NamedColor::Yellow,
            'f' => NamedColor::White,
            _ => return None,
        };
        Some(Self::Named(named))
    }

    /// Creates an RGB color from a hex string.
    ///
    /// # Arguments
    /// - `hex` – The hex color string without '#' prefix, exactly 6 characters (RRGGBB).
    ///
    /// # Returns
    /// An RGB color, or `None` if the hex string is invalid.
    #[must_use]
    pub fn from_hex_str(hex: &str) -> Option<Self> {
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self::Rgb(RGBColor::new(r, g, b)))
    }
}

/// An RGB color with red, green, and blue components.
#[derive(Debug, Deserialize, Clone, Copy, Eq, Hash, PartialEq)]
pub struct RGBColor {
    /// The red component (0-255).
    pub red: u8,
    /// The green component (0-255).
    pub green: u8,
    /// The blue component (0-255).
    pub blue: u8,
}

impl RGBColor {
    /// Creates a new RGB color from component values.
    ///
    /// # Arguments
    /// - `red` – The red component (0-255).
    /// - `green` – The green component (0-255).
    /// - `blue` – The blue component (0-255).
    ///
    /// # Returns
    /// A new `RGBColor` instance.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl Serialize for RGBColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!(
            "#{:02X}{:02X}{:02X}",
            self.red, self.green, self.blue
        ))
    }
}

/// An ARGB color with alpha, red, green, and blue components.
///
/// Used for advanced color effects like custom text shadows.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
pub struct ARGBColor {
    /// The alpha (transparency) component (0-255).
    alpha: u8,
    /// The red component (0-255).
    red: u8,
    /// The green component (0-255).
    green: u8,
    /// The blue component (0-255).
    blue: u8,
}

impl ARGBColor {
    /// Creates a new ARGB color from component values.
    ///
    /// # Arguments
    /// - `alpha` – The alpha (transparency) component (0-255, 0 = transparent, 255 = opaque).
    /// - `red` – The red component (0-255).
    /// - `green` – The green component (0-255).
    /// - `blue` – The blue component (0-255).
    ///
    /// # Returns
    /// A new `ARGBColor` instance.
    #[must_use]
    pub const fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self {
            alpha,
            red,
            green,
            blue,
        }
    }
}

impl Serialize for ARGBColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes([self.alpha, self.red, self.green, self.blue].as_ref())
    }
}

/// One of the 16 standard Minecraft named colors.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    /// Black (#000000)
    Black = 0,
    /// Dark Blue (#0000AA)
    DarkBlue,
    /// Dark Green (#00AA00)
    DarkGreen,
    /// Dark Aqua (#00AAAA)
    DarkAqua,
    /// Dark Red (#AA0000)
    DarkRed,
    /// Dark Purple (#AA00AA)
    DarkPurple,
    /// Gold (#FFAA00)
    Gold,
    /// Gray (#AAAAAA)
    Gray,
    /// Dark Gray (#555555)
    DarkGray,
    /// Blue (#5555FF)
    Blue,
    /// Green (#55FF55)
    Green,
    /// Aqua (#55FFFF)
    Aqua,
    /// Red (#FF5555)
    Red,
    /// Light Purple (#FF55FF)
    LightPurple,
    /// Yellow (#FFFF55)
    Yellow,
    /// White (#FFFFFF)
    White,
}

impl NamedColor {
    /// Converts this named color to its corresponding RGB values.
    ///
    /// # Returns
    /// The RGB color equivalent of this named color.
    #[must_use]
    pub const fn to_rgb(&self) -> RGBColor {
        match self {
            Self::Black => RGBColor::new(0, 0, 0),
            Self::DarkBlue => RGBColor::new(0, 0, 170),
            Self::DarkGreen => RGBColor::new(0, 170, 0),
            Self::DarkAqua => RGBColor::new(0, 170, 170),
            Self::DarkRed => RGBColor::new(170, 0, 0),
            Self::DarkPurple => RGBColor::new(170, 0, 170),
            Self::Gold => RGBColor::new(255, 170, 0),
            Self::Gray => RGBColor::new(170, 170, 170),
            Self::DarkGray => RGBColor::new(85, 85, 85),
            Self::Blue => RGBColor::new(85, 85, 255),
            Self::Green => RGBColor::new(85, 255, 85),
            Self::Aqua => RGBColor::new(85, 255, 255),
            Self::Red => RGBColor::new(255, 85, 85),
            Self::LightPurple => RGBColor::new(255, 85, 255),
            Self::Yellow => RGBColor::new(255, 255, 85),
            Self::White => RGBColor::new(255, 255, 255),
        }
    }
}

impl TryFrom<&str> for NamedColor {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "black" => Ok(Self::Black),
            "dark_blue" => Ok(Self::DarkBlue),
            "dark_green" => Ok(Self::DarkGreen),
            "dark_aqua" => Ok(Self::DarkAqua),
            "dark_red" => Ok(Self::DarkRed),
            "dark_purple" => Ok(Self::DarkPurple),
            "gold" => Ok(Self::Gold),
            "gray" => Ok(Self::Gray),
            "dark_gray" => Ok(Self::DarkGray),
            "blue" => Ok(Self::Blue),
            "green" => Ok(Self::Green),
            "aqua" => Ok(Self::Aqua),
            "red" => Ok(Self::Red),
            "light_purple" => Ok(Self::LightPurple),
            "yellow" => Ok(Self::Yellow),
            "white" => Ok(Self::White),
            _ => Err(()),
        }
    }
}
