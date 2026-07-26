use super::Player;
use crate::net::ClientPlatform;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::IdOr;
use pumpkin_protocol::SoundEvent;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CActionBar;
use pumpkin_protocol::java::client::play::CParticle;
use pumpkin_protocol::java::client::play::CPlayerInfoUpdate;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_protocol::java::client::play::CStopSound;
use pumpkin_protocol::java::client::play::CSubtitle;
use pumpkin_protocol::java::client::play::CTabList;
use pumpkin_protocol::java::client::play::CTitleAnimation;
use pumpkin_protocol::java::client::play::CTitleText;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_protocol::java::client::play::PlayerAction;
use pumpkin_protocol::java::client::play::PlayerInfoFlags;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use std::sync::atomic::Ordering;

impl Player {
    pub async fn set_tab_list_header_footer(&self, header: TextComponent, footer: TextComponent) {
        *self.tab_list_header.lock().await = header.clone();
        *self.tab_list_footer.lock().await = footer.clone();
        self.client
            .enqueue_packet(&CTabList::new(&header, &footer))
            .await;
    }

    pub async fn set_display_name(&self, display_name: Option<TextComponent>) {
        *self.display_name.lock().await = display_name.clone();
        // Update the tab list for everyone
        let world = self.world();
        world.broadcast_packet_all(&CPlayerInfoUpdate::new(
            PlayerInfoFlags::UPDATE_DISPLAY_NAME.bits(),
            &[pumpkin_protocol::java::client::play::Player {
                uuid: self.gameprofile.id,
                actions: &[PlayerAction::UpdateDisplayName(display_name.as_ref())],
            }],
        ));
    }

    pub async fn get_tab_list_name(&self) -> Option<TextComponent> {
        self.tab_list_name.lock().await.clone()
    }

    pub async fn set_tab_list_name(&self, name: Option<TextComponent>) {
        *self.tab_list_name.lock().await = name.clone();
        let world = self.world();
        world.broadcast_packet_all(&CPlayerInfoUpdate::new(
            PlayerInfoFlags::UPDATE_DISPLAY_NAME.bits(),
            &[pumpkin_protocol::java::client::play::Player {
                uuid: self.gameprofile.id,
                actions: &[PlayerAction::UpdateDisplayName(name.as_ref())],
            }],
        ));
    }

    pub fn set_tab_list_order(&self, order: i32) {
        self.tab_list_order.store(order, Ordering::Relaxed);
        let world = self.world();
        world.broadcast_packet_all(&CPlayerInfoUpdate::new(
            PlayerInfoFlags::UPDATE_LIST_PRIORITY.bits(),
            &[pumpkin_protocol::java::client::play::Player {
                uuid: self.gameprofile.id,
                actions: &[PlayerAction::UpdateListOrder(VarInt(order))],
            }],
        ));
    }

    pub fn set_tab_list_latency(&self, latency: i32) {
        self.tab_list_latency.store(latency, Ordering::Relaxed);
        let world = self.world();
        world.broadcast_packet_all(&CPlayerInfoUpdate::new(
            PlayerInfoFlags::UPDATE_LATENCY.bits(),
            &[pumpkin_protocol::java::client::play::Player {
                uuid: self.gameprofile.id,
                actions: &[PlayerAction::UpdateLatency(VarInt(latency))],
            }],
        ));
    }

    pub fn set_tab_list_listed(&self, listed: bool) {
        self.tab_list_listed.store(listed, Ordering::Relaxed);
        let world = self.world();
        world.broadcast_packet_all(&CPlayerInfoUpdate::new(
            PlayerInfoFlags::UPDATE_LISTED.bits(),
            &[pumpkin_protocol::java::client::play::Player {
                uuid: self.gameprofile.id,
                actions: &[PlayerAction::UpdateListed(listed)],
            }],
        ));
    }

    pub async fn show_title(&self, text: &TextComponent, mode: &TitleMode) {
        match mode {
            TitleMode::Title => {
                self.client
                    .enqueue_packet_editioned(
                        &CTitleText::new(text),
                        &pumpkin_protocol::bedrock::client::set_title::CSetTitle::new(
                            2,
                            text.clone().get_text(),
                            0,
                            0,
                            0,
                        ),
                    )
                    .await;
            }
            TitleMode::SubTitle => {
                self.client
                    .enqueue_packet_editioned(
                        &CSubtitle::new(text),
                        &pumpkin_protocol::bedrock::client::set_title::CSetTitle::new(
                            3,
                            text.clone().get_text(),
                            0,
                            0,
                            0,
                        ),
                    )
                    .await;
            }
            TitleMode::ActionBar => {
                self.client
                    .enqueue_packet_editioned(
                        &CActionBar::new(text),
                        &pumpkin_protocol::bedrock::client::set_title::CSetTitle::new(
                            4,
                            text.clone().get_text(),
                            0,
                            0,
                            0,
                        ),
                    )
                    .await;
            }
        }
    }

    pub async fn send_title_animation(&self, fade_in: i32, stay: i32, fade_out: i32) {
        match self.client.as_ref() {
            ClientPlatform::Java(client) => {
                client
                    .enqueue_packet(&CTitleAnimation::new(fade_in, stay, fade_out))
                    .await;
            }
            ClientPlatform::Bedrock(client) => {
                client
                    .send_game_packet(
                        &pumpkin_protocol::bedrock::client::set_title::CSetTitle::new(
                            5,
                            String::new(),
                            fade_in,
                            stay,
                            fade_out,
                        ),
                    )
                    .await;
            }
        }
    }

    pub fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle: Particle,
    ) {
        self.client.try_enqueue_packet(&CParticle::new(
            false,
            false,
            position,
            offset,
            max_speed,
            particle_count,
            VarInt(particle as i32),
            &[],
        ));
    }

    pub async fn play_sound(
        &self,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) {
        self.client
            .enqueue_packet(&CSoundEffect::new(
                IdOr::Id(sound_id),
                category,
                position,
                volume,
                pitch,
                seed,
            ))
            .await;
    }

    pub async fn play_sound_event(
        &self,
        sound: SoundEvent,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) {
        self.client
            .enqueue_packet(&CSoundEffect::new(
                IdOr::Value(sound),
                category,
                position,
                volume,
                pitch,
                seed,
            ))
            .await;
    }

    /// Stops a sound playing on the client.
    ///
    /// # Arguments
    ///
    /// * `sound_id`: An optional [`ResourceLocation`] specifying the sound to stop. If [`None`], all sounds in the specified category (if any) will be stopped.
    /// * `category`: An optional [`SoundCategory`] specifying the sound category to stop. If [`None`], all sounds with the specified resource location (if any) will be stopped.
    pub async fn stop_sound(
        &self,
        sound_id: Option<ResourceLocation>,
        category: Option<SoundCategory>,
    ) {
        self.client
            .enqueue_packet(&CStopSound::new(sound_id, category))
            .await;
    }

    /// Send the player's skin layers and used hand to all players.
    pub fn send_client_information(&self) {
        let config = self.config.load();
        self.living_entity.entity.send_meta_data(
            &[
                Metadata::new(
                    TrackedData::PLAYER_MODE_CUSTOMISATION,
                    MetaDataType::BYTE,
                    config.skin_parts,
                ),
                // Metadata::new(
                //     TrackedData::DATA_MAIN_ARM_ID,
                //     MetaDataType::ARM,
                //     VarInt(config.main_hand as u8 as i32),
                // ),
            ],
            None,
        );
    }
}

#[derive(Debug)]
pub enum TitleMode {
    Title,
    SubTitle,
    ActionBar,
}
