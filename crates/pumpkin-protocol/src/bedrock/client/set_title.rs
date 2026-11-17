// Last verified for v2169

use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

/// Displays or controls on-screen titles, subtitles, or action bar notices.
#[derive(PacketWrite)]
#[packet(88)]
pub struct CSetTitle {
    /// Operation type to perform (show text, adjust timing, clear, or reset).
    pub title_type: TitleType,
    /// Content string or raw JSON component displayed on screen.
    pub title_text: String,
    /// Duration in ticks for the title to fade in.
    pub fade_in_time: VarInt,
    /// Duration in ticks for the title to remain fully visible.
    pub stay_time: VarInt,
    /// Duration in ticks for the title to fade out.
    pub fade_out_time: VarInt,
    /// Xbox Live user identifier of the recipient, if applicable.
    pub xuid: String,
    /// Platform-specific online identifier.
    pub platform_online_id: String,
    /// Sanitized version of the title message with profanity removed.
    pub filtered_title_message: String,
}

/// Action or display target of a title packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(i32)]
#[serial(varint)]
pub enum TitleType {
    /// Clears any currently displayed title.
    Clear,
    /// Resets title animations and display durations to default values.
    Reset,
    /// Displays primary title text.
    Title,
    /// Displays secondary subtitle text beneath the title.
    Subtitle,
    /// Displays text above the player's hotbar (action bar).
    Actionbar,
    /// Configures fade-in, stay, and fade-out durations without displaying new text.
    Times,
    /// Displays localized or formatted JSON text as primary title.
    TitleTextObject,
    /// Displays localized or formatted JSON text as subtitle.
    SubtitleTextObject,
    /// Displays localized or formatted JSON text on the action bar.
    ActionbarTextObject,
}

impl CSetTitle {
    /// Constructs a title packet with specified timing parameters and type.
    #[must_use]
    pub const fn new(
        title_type: TitleType,
        title_text: String,
        fade_in_time: i32,
        stay_time: i32,
        fade_out_time: i32,
    ) -> Self {
        Self {
            title_type,
            title_text,
            fade_in_time: VarInt(fade_in_time),
            stay_time: VarInt(stay_time),
            fade_out_time: VarInt(fade_out_time),
            xuid: String::new(),
            platform_online_id: String::new(),
            filtered_title_message: String::new(),
        }
    }
}
