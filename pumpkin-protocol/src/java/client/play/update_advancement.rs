use crate::codec::item_stack_seralizer::ItemStackTemplate;
use crate::codec::var_int::VarInt;
use pumpkin_data::Advancement;
use pumpkin_data::advancement_data::{AdvancementDisplay, AdvancementProgressData};
use pumpkin_data::packet::clientbound::PLAY_UPDATE_ADVANCEMENTS;
use pumpkin_macros::java_packet;
use pumpkin_util::identifier::Identifier;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer, ser::SerializeSeq};

fn serialize_advancements<S: Serializer>(
    advancements: &[&'static Advancement],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(advancements.len()))?;
    for adv in advancements {
        seq.serialize_element(&AdvancementSer(adv))?;
    }
    seq.end()
}

pub struct AdvancementSer<'a>(pub &'a Advancement);

impl Serialize for AdvancementSer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let adv = self.0;
        let mut state = serializer.serialize_struct("Advancement", 5)?;
        state.serialize_field("id", &adv.id.to_string())?;
        state.serialize_field("parent", &adv.parent.clone().map(|p| p.to_string()))?;
        if let Some(display) = adv.display {
            state.serialize_field("display", &Some(DisplaySerializer(display)))?;
        } else {
            state.serialize_field("display", &None::<&DisplaySerializer>)?;
        }
        state.serialize_field("requirements", adv.requirements)?;
        state.serialize_field("send_telemetry", &adv.send_telemetry)?;
        state.end()
    }
}

pub struct DisplaySerializer<'a>(pub &'a AdvancementDisplay);

impl Serialize for DisplaySerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let display = self.0;
        let mut state = serializer.serialize_struct("AdvancementDisplay", 8)?;

        state.serialize_field("title", &display.get_title())?;
        state.serialize_field("description", &display.get_description())?;
        state.serialize_field("icon", &ItemStackTemplate::from(display.item_icon.clone()))?;
        state.serialize_field("frame_type", &VarInt(display.frame_type as i32))?;
        let flags = (display.has_background() as i32)
            | ((display.show_toast as i32) << 1)
            | ((display.hidden as i32) << 2);
        state.serialize_field("flags", &flags)?;
        if let Some(bg) = display.background_texture {
            state.serialize_field("background_texture", bg)?;
        }
        state.serialize_field("x", &display.x)?;
        state.serialize_field("y", &display.y)?;
        state.end()
    }
}

#[derive(Serialize)]
#[java_packet(PLAY_UPDATE_ADVANCEMENTS)]
#[allow(unused)]
pub struct CUpdateAdvancements {
    pub reset: bool,
    #[serde(serialize_with = "serialize_advancements")]
    pub added: Vec<&'static Advancement>,
    pub removed: Vec<Identifier>,
    pub progress: Vec<AdvancementProgressData>,
    pub show_advancements: bool,
}

impl CUpdateAdvancements {
    #[must_use]
    #[allow(unused)]
    pub const fn new(
        reset: bool,
        added: Vec<&'static Advancement>,
        progress: Vec<AdvancementProgressData>,
        removed: Vec<Identifier>,
        show_advancements: bool,
    ) -> Self {
        Self {
            reset,
            added,
            removed,
            progress,
            show_advancements,
        }
    }
}
