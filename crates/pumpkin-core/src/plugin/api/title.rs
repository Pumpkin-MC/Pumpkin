use crate::entity::player::{Player, TitleMode};
use pumpkin_util::text::TextComponent;

/// A fluent Builder for constructing and displaying titles, subtitles, and action bar text with animation timings.
#[derive(Debug, Clone, Default)]
pub struct TitleBuilder {
    title: Option<TextComponent>,
    subtitle: Option<TextComponent>,
    actionbar: Option<TextComponent>,
    fade_in: Option<i32>,
    stay: Option<i32>,
    fade_out: Option<i32>,
}

impl TitleBuilder {
    /// Creates a new `TitleBuilder`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the main title text component.
    #[must_use]
    pub fn title(mut self, title: impl Into<TextComponent>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the main title from plain text.
    #[must_use]
    pub fn title_text(mut self, text: impl Into<String>) -> Self {
        self.title = Some(TextComponent::text(text.into()));
        self
    }

    /// Sets the subtitle text component.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<TextComponent>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets the subtitle from plain text.
    #[must_use]
    pub fn subtitle_text(mut self, text: impl Into<String>) -> Self {
        self.subtitle = Some(TextComponent::text(text.into()));
        self
    }

    /// Sets the action bar text component.
    #[must_use]
    pub fn actionbar(mut self, actionbar: impl Into<TextComponent>) -> Self {
        self.actionbar = Some(actionbar.into());
        self
    }

    /// Sets the action bar text from plain text.
    #[must_use]
    pub fn actionbar_text(mut self, text: impl Into<String>) -> Self {
        self.actionbar = Some(TextComponent::text(text.into()));
        self
    }

    /// Sets the fade-in, stay, and fade-out timings in ticks.
    #[must_use]
    pub const fn times(mut self, fade_in: i32, stay: i32, fade_out: i32) -> Self {
        self.fade_in = Some(fade_in);
        self.stay = Some(stay);
        self.fade_out = Some(fade_out);
        self
    }

    /// Sends the title configuration to the specified player.
    pub fn send_to(&self, player: &Player) {
        if let (Some(fade_in), Some(stay), Some(fade_out)) =
            (self.fade_in, self.stay, self.fade_out)
        {
            player.send_title_animation(fade_in, stay, fade_out);
        }

        if let Some(ref title) = self.title {
            player.show_title(title, &TitleMode::Title);
        }

        if let Some(ref subtitle) = self.subtitle {
            player.show_title(subtitle, &TitleMode::SubTitle);
        }

        if let Some(ref actionbar) = self.actionbar {
            player.show_title(actionbar, &TitleMode::ActionBar);
        }
    }
}
