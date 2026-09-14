use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::advancement::{
    AdvancementDisplay as WitAdvancementDisplay, AdvancementInfo as WitAdvancementInfo,
    FrameType as WitFrameType,
};
use pumpkin_data::Advancement;
use pumpkin_data::advancement_data::FrameType;

#[must_use]
pub fn find_advancement(id: &str) -> Option<&'static Advancement> {
    Advancement::from_name(id)
        .or_else(|| Advancement::from_minecraft_name(id))
        .or_else(|| {
            id.strip_prefix("minecraft:")
                .and_then(Advancement::from_name)
        })
}

#[must_use]
pub const fn to_wasm_frame_type(frame: FrameType) -> WitFrameType {
    match frame {
        FrameType::Task => WitFrameType::Task,
        FrameType::Challenge => WitFrameType::Challenge,
        FrameType::Goal => WitFrameType::Goal,
    }
}

pub fn to_wasm_advancement_info(
    state: &mut PluginHostState,
    advancement: &'static Advancement,
) -> wasmtime::Result<WitAdvancementInfo> {
    let display = if let Some(disp) = advancement.display {
        let title = state.add_text_component(disp.get_title())?;
        let description = state.add_text_component(disp.get_description())?;
        Some(WitAdvancementDisplay {
            title,
            description,
            frame: to_wasm_frame_type(disp.frame_type),
            show_toast: disp.show_toast,
            hidden: disp.hidden,
            announce_to_chat: disp.announce_to_chat,
            background: disp.background_texture.map(ToString::to_string),
            x: disp.x,
            y: disp.y,
        })
    } else {
        None
    };

    Ok(WitAdvancementInfo {
        id: advancement.id.to_string(),
        parent_id: advancement.parent.as_ref().map(ToString::to_string),
        criteria: advancement
            .criteria
            .iter()
            .map(ToString::to_string)
            .collect(),
        display,
    })
}
