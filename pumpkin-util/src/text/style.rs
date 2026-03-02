use super::{
    click::ClickEvent,
    color::{self, Color},
    hover::HoverEvent,
};
use crate::text::color::ARGBColor;
use serde::{Deserialize, Serialize};

/// Represents the styling options for a text component.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Style {
    /// The color to render the content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// Whether to render the content in bold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    /// Whether to render the content in italic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    /// Whether to render the content in underlined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    /// Whether to render the content in strikethrough.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    /// Whether to render the content in obfuscated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insertion: Option<String>,
    /// Allows for events to occur when the player clicks on text. Only works in chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click_event: Option<ClickEvent>,
    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover_event: Option<HoverEvent>,
    /// Allows you to change the font of the text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    /// Custom shadow color for the text.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "shadow_color"
    )]
    pub shadow_color: Option<ARGBColor>,
}

impl Style {
    /// Sets the text color using a `Color` enum value.
    ///
    /// # Arguments
    /// - `color` – The color to apply.
    ///
    /// # Returns
    /// The style instance with the color set.
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the text color using a named Minecraft color.
    ///
    /// # Arguments
    /// - `color` – The named color to apply (e.g. `NamedColor::Red`).
    ///
    /// # Returns
    /// The style instance with the named color set.
    #[must_use]
    pub const fn color_named(mut self, color: color::NamedColor) -> Self {
        self.color = Some(Color::Named(color));
        self
    }

    /// Makes the text bold.
    ///
    /// # Returns
    /// The style instance with bold enabled.
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = Some(true);
        self
    }

    /// Makes the text italic.
    ///
    /// # Returns
    /// The style instance with italic enabled.
    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.italic = Some(true);
        self
    }

    /// Makes the text underlined.
    ///
    /// # Returns
    /// The style instance with underline enabled.
    #[must_use]
    pub const fn underlined(mut self) -> Self {
        self.underlined = Some(true);
        self
    }

    /// Makes the text strikethrough.
    ///
    /// # Returns
    /// The style instance with strikethrough enabled.
    #[must_use]
    pub const fn strikethrough(mut self) -> Self {
        self.strikethrough = Some(true);
        self
    }

    /// Makes the text obfuscated (random characters).
    ///
    /// # Returns
    /// The style instance with obfuscation enabled.
    #[must_use]
    pub const fn obfuscated(mut self) -> Self {
        self.obfuscated = Some(true);
        self
    }

    /// Sets text to be inserted into the player's chat input when shift-clicked.
    ///
    /// # Arguments
    /// - `text` – The text to insert when shift-clicked.
    ///
    /// # Returns
    /// The style instance with the insertion text set.
    #[must_use]
    pub fn insertion(mut self, text: String) -> Self {
        self.insertion = Some(text);
        self
    }

    /// Sets an event to occur when the player clicks on the text.
    ///
    /// # Arguments
    /// - `event` – The click event to trigger.
    ///
    /// # Returns
    /// The style instance with the click event set.
    #[must_use]
    pub fn click_event(mut self, event: ClickEvent) -> Self {
        self.click_event = Some(event);
        self
    }

    /// Sets a tooltip to be displayed when the player hovers over the text.
    ///
    /// # Arguments
    /// - `event` – The hover event to display.
    ///
    /// # Returns
    /// The style instance with the hover event set.
    #[must_use]
    pub fn hover_event(mut self, event: HoverEvent) -> Self {
        self.hover_event = Some(event);
        self
    }

    /// Sets the font resource location for rendering.
    ///
    /// Allows changing the font face of the text. Default fonts include:
    /// - `minecraft:default` - The standard Minecraft font
    /// - `minecraft:uniform` - A uniform-width font
    /// - `minecraft:alt` - An alternative font style
    /// - `minecraft:illageralt` - The illager-themed font
    ///
    /// # Arguments
    /// - `resource_location` – The font resource location (e.g., "minecraft:uniform").
    ///
    /// # Returns
    /// The style instance with the font set.
    #[must_use]
    pub fn font(mut self, resource_location: String) -> Self {
        self.font = Some(resource_location);
        self
    }

    /// Overrides the shadow color of the text.
    ///
    /// # Arguments
    /// - `color` – The ARGB color value for the shadow.
    ///
    /// # Returns
    /// The style instance with the shadow color set.
    #[must_use]
    pub const fn shadow_color(mut self, color: ARGBColor) -> Self {
        self.shadow_color = Some(color);
        self
    }
}
