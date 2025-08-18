use std::{num::NonZero, sync::Arc};

use pumpkin_config::{BASIC_CONFIG, advanced_config};
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::{
    bedrock::{
        client::{chunk_radius_update::CChunkRadiusUpdate, container_open::CContainerOpen},
        server::{
            container_close::SContainerClose,
            interaction::{Action, SInteraction},
            player_auth_input::SPlayerAuthInput,
            request_chunk_radius::SRequestChunkRadius,
            text::SText,
        },
    },
    codec::{bedrock_block_pos::NetworkPos, var_long::VarLong},
    java::client::play::CSystemChatMessage,
};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    text::TextComponent,
};

use crate::{
    entity::player::Player, net::bedrock::BedrockClient,
    plugin::player::player_chat::PlayerChatEvent, server::seasonal_events, world::chunker,
};

impl BedrockClient {
    pub async fn handle_request_chunk_radius(
        &self,
        player: &Arc<Player>,
        packet: SRequestChunkRadius,
    ) {
        dbg!(&packet);
        player.config.write().await.view_distance =
            NonZero::new(packet.chunk_radius.0 as u8).unwrap();
        self.send_game_packet(&CChunkRadiusUpdate {
            chunk_radius: packet.chunk_radius,
        })
        .await;
    }

    pub async fn player_pos_update(&self, player: &Arc<Player>, packet: SPlayerAuthInput) {
        let pos = packet.position;
        player.living_entity.entity.set_pos(pos.to_f64());

        chunker::update_position(player).await;
        //self.send_game_packet(&CMovePlayer {
        //     player_runtime_id: VarULong(player.entity_id() as u64),
        //    position: packet.position + Vector3::new(10.0, 0.0, 0.0),
        //    pitch: packet.pitch,
        //    yaw: packet.yaw,
        //    y_head_rotation: packet.head_rotation,
        //    position_mode: 1,
        //    on_ground: false,
        //    riding_runtime_id: VarULong(0),
        //    tick: packet.client_tick,
        //})
        //.await;
    }

    pub async fn handle_interaction(&self, _player: &Arc<Player>, packet: SInteraction) {
        if matches!(packet.action, Action::OpenInventory) {
            self.send_game_packet(&CContainerOpen {
                container_id: 0,
                container_type: 0xff,
                position: NetworkPos(BlockPos(Vector3::new(0, 0, 0))),
                target_entity_id: VarLong(-1),
            })
            .await;
        }
    }

    pub async fn handle_container_close(&self, _player: &Arc<Player>, packet: SContainerClose) {
        if packet.container_id == 0 {
            self.send_game_packet(&SContainerClose {
                container_id: 0,
                container_type: 0xff,
                server_initiated: false,
            })
            .await;
        }
    }

    pub async fn handle_chat_message(&self, player: &Arc<Player>, packet: SText) {
        let gameprofile = &player.gameprofile;

        send_cancellable! {{
            PlayerChatEvent::new(player.clone(), packet.message, vec![]);

            'after: {
                log::info!("<chat> {}: {}", gameprofile.name, event.message);

                let config = advanced_config();

                let message = match seasonal_events::modify_chat_message(&event.message) {
                    Some(m) => m,
                    None => event.message.clone(),
                };

                let decorated_message = &TextComponent::chat_decorated(
                    config.chat.format.clone(),
                    gameprofile.name.clone(),
                    message.clone(),
                );

                let entity = &player.living_entity.entity;
                if BASIC_CONFIG.allow_chat_reports {
                    //TODO Alex help, what is this?
                    //world.broadcast_secure_player_chat(player, &message, decorated_message).await;
                } else {
                    let je_packet = CSystemChatMessage::new(
                        decorated_message,
                        false,
                    );

                    let be_packet = SText::new(
                        message, gameprofile.name.clone()
                    );

                    entity.world.read().await.broadcast_editioned(&je_packet, &be_packet).await;
                }
            }
        }}
    }
}
