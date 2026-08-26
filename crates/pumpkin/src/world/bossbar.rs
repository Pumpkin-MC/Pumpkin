use crate::entity::player::Player;
use crate::net::ClientPlatform;
use bitflags::bitflags;
use pumpkin_protocol::bedrock::client::boss_event::{
    BOSS_EVENT_COLOUR_BLUE, BOSS_EVENT_COLOUR_GREEN, BOSS_EVENT_COLOUR_PINK,
    BOSS_EVENT_COLOUR_PURPLE, BOSS_EVENT_COLOUR_RED, BOSS_EVENT_COLOUR_WHITE,
    BOSS_EVENT_COLOUR_YELLOW, BOSS_EVENT_OVERLAY_NOTCHED_6, BOSS_EVENT_OVERLAY_NOTCHED_10,
    BOSS_EVENT_OVERLAY_NOTCHED_12, BOSS_EVENT_OVERLAY_NOTCHED_20, BOSS_EVENT_OVERLAY_PROGRESS,
    CBossEvent as BBossEvent,
};
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::{BosseventAction, CBossEvent};
use pumpkin_util::text::TextComponent;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BossbarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

impl BossbarColor {
    #[must_use]
    pub const fn to_bedrock(self) -> u8 {
        match self {
            Self::Pink => BOSS_EVENT_COLOUR_PINK,
            Self::Blue => BOSS_EVENT_COLOUR_BLUE,
            Self::Red => BOSS_EVENT_COLOUR_RED,
            Self::Green => BOSS_EVENT_COLOUR_GREEN,
            Self::Yellow => BOSS_EVENT_COLOUR_YELLOW,
            Self::Purple => BOSS_EVENT_COLOUR_PURPLE,
            Self::White => BOSS_EVENT_COLOUR_WHITE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BossbarDivisions {
    NoDivision,
    Notches6,
    Notches10,
    Notches12,
    Notches20,
}

impl BossbarDivisions {
    #[must_use]
    pub const fn to_bedrock(self) -> u8 {
        match self {
            Self::NoDivision => BOSS_EVENT_OVERLAY_PROGRESS,
            Self::Notches6 => BOSS_EVENT_OVERLAY_NOTCHED_6,
            Self::Notches10 => BOSS_EVENT_OVERLAY_NOTCHED_10,
            Self::Notches12 => BOSS_EVENT_OVERLAY_NOTCHED_12,
            Self::Notches20 => BOSS_EVENT_OVERLAY_NOTCHED_20,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct BossbarFlags: u8 {
        const DARKEN_SKY = 0x01;
        const DRAGON_BAR = 0x02;
        const CREATE_FOG = 0x04;
    }
}

#[derive(Clone)]
pub struct Bossbar {
    pub uuid: Uuid,
    pub title: TextComponent,
    pub health: f32,
    pub color: BossbarColor,
    pub division: BossbarDivisions,
    pub flags: BossbarFlags,
}

impl Bossbar {
    #[must_use]
    pub fn new(title: TextComponent) -> Self {
        let uuid = Uuid::new_v4();

        Self {
            uuid,
            title,
            health: 0.0,
            color: BossbarColor::White,
            division: BossbarDivisions::NoDivision,
            flags: BossbarFlags::empty(),
        }
    }
}

#[inline]
#[must_use]
pub const fn bossbar_bedrock_id(uuid: &Uuid) -> VarLong {
    let (high, low) = uuid.as_u64_pair();
    let id = (high ^ low) & 0x7FFF_FFFF_FFFF_FFFF;
    VarLong(id as i64)
}

/// Extra methods for [`Player`] to send and manage the bossbar.
impl Player {
    pub async fn send_bossbar(&self, bossbar: &Bossbar) {
        match self.client.as_ref() {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::Add {
                    title: bossbar.title.clone(),
                    health: bossbar.health,
                    color: (bossbar.color as u8).into(),
                    division: (bossbar.division as u8).into(),
                    flags: bossbar.flags.bits(),
                };

                let packet = CBossEvent::new(&bossbar.uuid, boss_action);
                java.enqueue_client_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let boss_id = bossbar_bedrock_id(&bossbar.uuid);
                let player_id = VarLong(self.entity_id() as i64);
                let packet = BBossEvent::show(
                    boss_id,
                    player_id,
                    bossbar.title.clone().get_text(),
                    bossbar.health,
                    bossbar.color.to_bedrock(),
                    bossbar.division.to_bedrock(),
                );
                bedrock.send_packet(&packet).await;

                let register_packet = BBossEvent::register_player(boss_id, player_id);
                bedrock.send_packet(&register_packet).await;
            }
        }
    }

    pub async fn remove_bossbar(&self, uuid: Uuid) {
        match self.client.as_ref() {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::Remove;

                let packet = CBossEvent::new(&uuid, boss_action);
                java.enqueue_client_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let boss_id = bossbar_bedrock_id(&uuid);
                let player_id = VarLong(self.entity_id() as i64);
                let unregister_packet = BBossEvent::unregister_player(boss_id, player_id);
                bedrock.send_packet(&unregister_packet).await;

                let packet = BBossEvent::hide(boss_id);
                bedrock.send_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_health(&self, uuid: &Uuid, health: f32) {
        match self.client.as_ref() {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateHealth(health);

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_client_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let boss_id = bossbar_bedrock_id(uuid);
                let packet = BBossEvent::update_health(boss_id, health);
                bedrock.send_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_title(&self, uuid: &Uuid, title: TextComponent) {
        match self.client.as_ref() {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateTile(title);

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_client_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let boss_id = bossbar_bedrock_id(uuid);
                let packet = BBossEvent::update_title(boss_id, title.get_text());
                bedrock.send_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_style(
        &self,
        uuid: &Uuid,
        color: BossbarColor,
        dividers: BossbarDivisions,
        _flags: BossbarFlags,
    ) {
        match self.client.as_ref() {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateStyle {
                    color: (color as u8).into(),
                    dividers: (dividers as u8).into(),
                };

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_client_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let boss_id = bossbar_bedrock_id(uuid);
                let packet = BBossEvent::update_properties(
                    boss_id,
                    color.to_bedrock(),
                    dividers.to_bedrock(),
                );
                bedrock.send_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_flags(&self, uuid: &Uuid, flags: BossbarFlags) {
        match self.client.as_ref() {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateFlags(flags.bits());

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_client_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let boss_id = bossbar_bedrock_id(uuid);
                let packet = BBossEvent::update_properties(boss_id, 0, 0);
                bedrock.send_packet(&packet).await;
            }
        }
    }
}
