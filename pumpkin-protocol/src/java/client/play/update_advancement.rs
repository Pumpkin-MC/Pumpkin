use crate::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_data::packet::clientbound::PLAY_UPDATE_ADVANCEMENTS;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_UPDATE_ADVANCEMENTS)]
pub struct CUpdateAdvancements<'a> {
    pub reset: bool,
    pub advancement: &'a [Advancement<'a>],
    pub progress: &'a [AdvancementProgress<'a>],
    pub identifiers: &'a [ResourceLocation],
    pub show_advancements: bool,
}

impl<'a> CUpdateAdvancements<'a> {
    #[must_use]
    pub fn new(
        reset: bool,
        advancement: &'a [Advancement<'a>],
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

#[derive(Clone, Copy, Serialize)]
#[repr(i32)]
pub enum FrameType {
    Task = 0,
    Challenge = 1,
    Goal = 2,
}

#[derive(Serialize)]
pub struct AdvancementProgress<'a> {
    pub id: ResourceLocation,
    pub progress: &'a [Criteria],
}

impl<'a> AdvancementProgress<'a> {
    #[must_use]
    pub fn new(id: ResourceLocation, progress: &'a [Criteria]) -> Self {
        Self { id, progress }
    }
}

#[derive(Serialize)]
pub struct Advancement<'a> {
    pub parent_id: Option<ResourceLocation>,
    pub display: Option<AdvancementDisplay<'a>>,
    pub requirements: &'a [&'a [String]],
    pub send_telemetry_data: bool,
}

impl<'a> Advancement<'a> {
    #[must_use]
    pub fn new(
        parent_id: Option<ResourceLocation>,
        display: Option<AdvancementDisplay<'a>>,
        requirements: &'a [&'a [String]],
        send_telemetry_data: bool,
    ) -> Self {
        Self {
            parent_id,
            display,
            requirements,
            send_telemetry_data,
        }
    }
}

#[derive(Serialize)]
pub struct AdvancementDisplay<'a> {
    pub title: TextComponent,
    pub description: TextComponent,
    pub icon: ItemStackSerializer<'a>,
    pub frame_type: FrameType,
    pub flags: i32,
    pub background_texture: Option<ResourceLocation>,
    pub x: f32,
    pub y: f32,
}

impl<'a> AdvancementDisplay<'a> {
    #[must_use]
    pub fn new(
        title: TextComponent,
        description: TextComponent,
        icon: ItemStackSerializer<'a>,
        frame_type: FrameType,
        flags: i32,
        background_texture: Option<ResourceLocation>,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            title,
            description,
            icon,
            frame_type,
            flags,
            background_texture,
            x,
            y,
        }
    }
}

#[derive(Serialize)]
pub struct Criteria {
    pub criterion_id: ResourceLocation,
    pub achieve_date: Option<i64>,
}

impl Criteria {
    #[must_use]
    pub fn new(criterion_id: ResourceLocation, achieve_date: Option<i64>) -> Self {
        Self {
            criterion_id,
            achieve_date,
        }
    }
}
