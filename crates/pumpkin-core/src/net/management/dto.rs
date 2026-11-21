use pumpkin_util::text::TextComponent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl PlayerDto {
    #[must_use]
    pub const fn new(id: Option<Uuid>, name: Option<String>) -> Self {
        Self { id, name }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OperatorDto {
    pub player: PlayerDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypasses_player_limit: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserBanDto {
    pub player: PlayerDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpBanDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncomingIpBanDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player: Option<PlayerDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KickPlayerDto {
    pub player: PlayerDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MessageDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translatable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translatable_params: Option<Vec<String>>,
}

impl MessageDto {
    #[must_use]
    pub fn to_text_component(&self) -> TextComponent {
        self.translatable.as_ref().map_or_else(
            || {
                self.literal.as_ref().map_or_else(
                    || TextComponent::text(""),
                    |literal| TextComponent::text(literal.clone()),
                )
            },
            |translatable| {
                self.translatable_params.as_ref().map_or_else(
                    || {
                        let empty: Vec<TextComponent> = Vec::new();
                        TextComponent::translate(translatable.clone(), empty)
                    },
                    |params| {
                        let args: Vec<TextComponent> = params
                            .iter()
                            .map(|p| TextComponent::text(p.clone()))
                            .collect();
                        TextComponent::translate(translatable.clone(), args)
                    },
                )
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionDto {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerState {
    pub started: bool,
    #[serde(default)]
    pub players: Vec<PlayerDto>,
    pub version: VersionDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SystemMessageDto {
    pub message: MessageDto,
    pub overlay: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiving_players: Option<Vec<PlayerDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypedGameRule {
    pub key: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UntypedGameRule {
    pub key: String,
    pub value: serde_json::Value,
}
