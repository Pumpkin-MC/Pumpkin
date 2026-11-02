#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::bedrock::server::block_actor_data::SBlockActorData;

fn parse_bedrock_sign_lines(text: &str) -> [Box<str>; 4] {
    let mut lines = [Box::from(""), Box::from(""), Box::from(""), Box::from("")];
    for (i, line) in text.split('\n').take(4).enumerate() {
        lines[i] = Box::from(line);
    }
    lines
}

fn apply_sign_text(
    player: &Arc<Player>,
    world: &Arc<crate::world::World>,
    sign_entity: &crate::block::entities::sign::SignEntityRef,
    block_pos: pumpkin_util::math::position::BlockPos,
    is_front: bool,
    text_comp: &pumpkin_nbt::compound::NbtCompound,
) {
    let Some(raw_text) = text_comp.get_string("Text") else {
        return;
    };
    let lines = parse_bedrock_sign_lines(raw_text);
    let event_lines = lines.iter().map(ToString::to_string).collect();
    let mut event = crate::plugin::api::events::block::sign_change::SignChangeEvent::new(
        player.clone(),
        block_pos,
        event_lines,
    );
    if let Some(server) = world.server.upgrade() {
        server.plugin_manager.fire_blocking(&server, &mut event);
    }
    if event.cancelled {
        return;
    }

    let sign_text = sign_entity.get_text(is_front);
    sign_text
        .messages
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone_from(&lines);
    *sign_text
        .filtered_messages
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = lines;

    if let Some(color_val) = text_comp.get_int("SignTextColor") {
        let dye = crate::block::entities::bedrock::bedrock_sign_text_color_to_dye_color(color_val);
        sign_text.set_color(dye);
    }
    if let Some(glowing) = text_comp
        .get_bool("IgnoreLighting")
        .or_else(|| text_comp.get_byte("IgnoreLighting").map(|b| b != 0))
    {
        sign_text.set_has_glowing_text(glowing);
    }
}

impl BedrockClient {
    pub fn handle_block_actor_data(&self, player: &Arc<Player>, packet: &SBlockActorData) {
        let id = packet.actor_data_tags.get_string("id").unwrap_or("");
        if id == "Sign" || id == "HangingSign" {
            Self::handle_sign_block_actor_data(player, packet);
        }
    }

    fn handle_sign_block_actor_data(player: &Arc<Player>, packet: &SBlockActorData) {
        let world = player.get_entity().world.load_full();
        let Some(block_entity) = world.get_block_entity(&packet.block_position) else {
            return;
        };
        let Some(sign_entity) =
            crate::block::entities::sign::SignEntityRef::from_block_entity(&*block_entity)
        else {
            return;
        };
        if sign_entity.is_waxed() {
            return;
        }

        let currently_editing = *sign_entity
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(editor_id) = currently_editing
            && editor_id != player.gameprofile.id
        {
            return;
        }

        if let Some(front_comp) = packet.actor_data_tags.get_compound("FrontText") {
            apply_sign_text(
                player,
                &world,
                &sign_entity,
                packet.block_position,
                true,
                front_comp,
            );
        } else if let Some(raw_text) = packet.actor_data_tags.get_string("Text") {
            let mut temp_comp = pumpkin_nbt::compound::NbtCompound::new();
            temp_comp.put_string("Text", raw_text.to_string());
            if let Some(color_val) = packet.actor_data_tags.get_int("SignTextColor") {
                temp_comp.put_int("SignTextColor", color_val);
            }
            apply_sign_text(
                player,
                &world,
                &sign_entity,
                packet.block_position,
                true,
                &temp_comp,
            );
        }

        if let Some(back_comp) = packet.actor_data_tags.get_compound("BackText") {
            apply_sign_text(
                player,
                &world,
                &sign_entity,
                packet.block_position,
                false,
                back_comp,
            );
        }

        *sign_entity
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        world.update_block_entity(&block_entity);
    }
}
