use crate::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_data::packet::clientbound::PLAY_UPDATE_ADVANCEMENTS;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use serde::Serialize;
use pumpkin_data::Advancement;

#[derive(Serialize)]
#[java_packet(PLAY_UPDATE_ADVANCEMENTS)]
pub struct CUpdateAdvancements<'a> {
    pub reset: bool,
    pub advancement: &'a [Advancement],
    pub progress: &'a [AdvancementProgress<'a>],
    pub identifiers: &'a [ResourceLocation],
    pub show_advancements: bool,
}

impl<'a> CUpdateAdvancements<'a> {
    #[must_use]
    pub fn new(
        reset: bool,
        advancement: &'a [Advancement],
        progress: &'a [AdvancementProgress<'a>],
        identifiers: &'a [ResourceLocation],
        show_advancements: bool,
    ) -> Self {
        Self {
            reset,
            advancement,
            progress,
            identifiers,
            show_advancements,
        }
    }
}
