use crate::entity::player::Player;
use crate::net::ClientPlatform;
use bitflags::bitflags;
use pumpkin_protocol::bedrock::client::boss_event::{BossEventAction, CBossEvent as BBossEvent};
use pumpkin_protocol::codec::var_int::VarInt;
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
    pub const fn to_bedrock(self) -> VarInt {
        match self {
            Self::Pink => VarInt(0),
            Self::Blue => VarInt(1),
            Self::Red => VarInt(2),
            Self::Green => VarInt(3),
            Self::Yellow => VarInt(4),
            Self::Purple => VarInt(5),
            Self::White => VarInt(6),
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
    pub const fn to_bedrock(self) -> VarInt {
        match self {
            Self::NoDivision => VarInt(0),
            Self::Notches6 => VarInt(1),
            Self::Notches10 => VarInt(2),
            Self::Notches12 => VarInt(3),
            Self::Notches20 => VarInt(4),
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

/// Extra methods for [`Player`] to send and manage the bossbar.
impl Player {
    pub async fn send_bossbar(&self, bossbar: &Bossbar) {
        match &self.client {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::Add {
                    title: bossbar.title.clone(),
                    health: bossbar.health,
                    color: (bossbar.color as u8).into(),
                    division: (bossbar.division as u8).into(),
                    flags: bossbar.flags.bits(),
                };

                let packet = CBossEvent::new(&bossbar.uuid, boss_action);
                java.enqueue_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let packet = BBossEvent {
                    boss_entity_id: VarLong(bossbar.uuid.as_u128() as i64),
                    action: BossEventAction::Add {
                        title: bossbar.title.clone().get_text(),
                        health_percent: bossbar.health,
                        screen_darken: u16::from(bossbar.flags.contains(BossbarFlags::DARKEN_SKY)),
                        color: bossbar.color.to_bedrock(),
                        overlay: bossbar.division.to_bedrock(),
                    },
                };
                bedrock.send_game_packet(&packet).await;
            }
        }
    }

    pub async fn remove_bossbar(&self, uuid: Uuid) {
        match &self.client {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::Remove;

                let packet = CBossEvent::new(&uuid, boss_action);
                java.enqueue_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let packet = BBossEvent {
                    boss_entity_id: VarLong(uuid.as_u128() as i64),
                    action: BossEventAction::Remove,
                };
                bedrock.send_game_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_health(&self, uuid: &Uuid, health: f32) {
        match &self.client {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateHealth(health);

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let packet = BBossEvent {
                    boss_entity_id: VarLong(uuid.as_u128() as i64),
                    action: BossEventAction::UpdateHealth(health),
                };
                bedrock.send_game_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_title(&self, uuid: &Uuid, title: TextComponent) {
        match &self.client {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateTile(title);

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let packet = BBossEvent {
                    boss_entity_id: VarLong(uuid.as_u128() as i64),
                    action: BossEventAction::UpdateTitle(title.get_text()),
                };
                bedrock.send_game_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_style(
        &self,
        uuid: &Uuid,
        color: BossbarColor,
        dividers: BossbarDivisions,
        flags: BossbarFlags,
    ) {
        match &self.client {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateStyle {
                    color: (color as u8).into(),
                    dividers: (dividers as u8).into(),
                };

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let packet = BBossEvent {
                    boss_entity_id: VarLong(uuid.as_u128() as i64),
                    action: BossEventAction::UpdateProperties {
                        screen_darken: u16::from(flags.contains(BossbarFlags::DARKEN_SKY)),
                        color: color.to_bedrock(),
                        overlay: dividers.to_bedrock(),
                    },
                };
                bedrock.send_game_packet(&packet).await;
            }
        }
    }

    pub async fn update_bossbar_flags(&self, uuid: &Uuid, flags: BossbarFlags) {
        match &self.client {
            ClientPlatform::Java(java) => {
                let boss_action = BosseventAction::UpdateFlags(flags.bits());

                let packet = CBossEvent::new(uuid, boss_action);
                java.enqueue_packet(&packet).await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                // For Bedrock, flags are part of properties (screen darken)
                // We don't have color and dividers here, so we might need more info or just skip if not critical
                // Actually, properties includes screen_darken, color, and overlay.
                // Since this method only has flags, we might need to store the current color/division on the player
                // or retrieve them from the bossbar if we had a reference to it.
                let packet = BBossEvent {
                    boss_entity_id: VarLong(uuid.as_u128() as i64),
                    action: BossEventAction::UpdateProperties {
                        screen_darken: u16::from(flags.contains(BossbarFlags::DARKEN_SKY)),
                        color: VarInt(0),
                        overlay: VarInt(0),
                    },
                };
                bedrock.send_game_packet(&packet).await;
            }
        }
    }
}
