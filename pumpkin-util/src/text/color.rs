use colored::{ColoredString, Colorize};
use serde::{Deserialize, Deserializer, Serialize};

/// Text color
#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Color {
    /// The default color for the text will be used, which varies by context
    /// (in some cases, it's white; in others, it's black; in still others, it
    /// is a shade of gray that isn't normally used on text).
    #[default]
    Reset,
    /// RGB Color
    Rgb(RGBColor),
    /// One of the 16 named Minecraft colors
    Named(NamedColor),
}

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
}

#[derive(Debug, Deserialize, Clone, Copy, Eq, Hash, PartialEq)]
pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RGBColor {
    #[must_use]
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
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

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
pub struct ARGBColor {
    alpha: u8,
    red: u8,
    green: u8,
    blue: u8,
}

impl ARGBColor {
    #[must_use]
    pub fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
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

/// Named Minecraft color
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    Black = 0,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl NamedColor {
    #[must_use]
    pub fn to_rgb(&self) -> RGBColor {
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
