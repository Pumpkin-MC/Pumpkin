use pumpkin_data::packet::clientbound::PLAY_UPDATE_ADVANCEMENTS;
use pumpkin_data::Advancement;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use pumpkin_data::advancement_data::AdvancementDisplay;
use crate::codec::item_stack_seralizer::ItemStackSerializer;

#[derive(Serialize)]
pub struct AdvancementProgress {
    pub id: ResourceLocation,
    pub progress: Vec<Criteria>,
}

#[derive(Serialize)]
pub struct Criteria {
    pub criterion_id: ResourceLocation,
    pub achieve_date: Option<i64>,
}

fn serialize_advancements<S: Serializer>(
    advancements: &[Advancement],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(advancements.len()))?;
    for adv in advancements {
        seq.serialize_element(&AdvancementSer(adv))?;
    }
    seq.end()
}

pub struct AdvancementSer<'a>(pub &'a Advancement);

impl<'a> Serialize for AdvancementSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let adv = self.0;
        let mut state = serializer.serialize_struct("Advancement", 5)?;
        state.serialize_field("id", adv.id)?;
        state.serialize_field("parent", &adv.parent)?;
        if let Some(display) = adv.display {
            state.serialize_field("display", &DisplaySerializer(display))?;
        } else {
            state.serialize_field("display", &None::<&DisplaySerializer>)?;
        }
        state.serialize_field("requirements", &[] as &[i32])?;
        state.serialize_field("send_telemetry", &adv.send_telemetry)?;
        state.end()
    }
}

pub struct DisplaySerializer<'a>(pub &'a AdvancementDisplay);

impl<'a> Serialize for DisplaySerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let disp = self.0;
        let mut state = serializer.serialize_struct("AdvancementDisplay", 8)?;
        state.serialize_field("title", &disp.get_title())?;
        state.serialize_field("description", &disp.get_description())?;
        state.serialize_field(
            "item_icon",
            &ItemStackSerializer::from(disp.item_icon.clone()),
        )?;
        state.serialize_field("frame_type", &(disp.frame_type as i32))?;
        let flags = (disp.has_background() as i32)
            | ((disp.show_toast as i32) << 1)
            | ((disp.hidden as i32) << 2);
        state.serialize_field("flags", &flags)?;

        if let Some(bg) = disp.background_texture {
            state.serialize_field("background_texture", bg)?;
        }

        state.serialize_field("x", &disp.x)?;
        state.serialize_field("y", &disp.y)?;
        state.end()
    }
}

#[derive(Serialize)]
#[java_packet(PLAY_UPDATE_ADVANCEMENTS)]
#[allow(unused)]
pub struct CUpdateAdvancements {
    pub reset: bool,
    #[serde(serialize_with = "serialize_advancements")]
    pub advancement: Vec<Advancement>,
    pub identifiers: Vec<ResourceLocation>,
    pub progress: Vec<AdvancementProgress>,
    pub show_advancements: bool,
}

impl CUpdateAdvancements {
    #[must_use]
    #[allow(unused)]
    pub fn new(
        reset: bool,
        advancement: Vec<Advancement>,
        progress: Vec<AdvancementProgress>,
        identifiers: Vec<ResourceLocation>,
        show_advancements: bool,
    ) -> Self {
        Self {
            reset,
            advancement,
            identifiers,
            progress,
            show_advancements,
        }
    }
}

