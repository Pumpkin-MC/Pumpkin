use crate::error::PumpkinError;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use thiserror::Error;
use tracing::Level;

mod chat;
mod interaction;
mod inventory;
mod misc;
mod movement;
mod use_item;

#[derive(Debug, Error)]
pub enum BlockPlacingError {
    BlockOutOfReach,
    InvalidHand,
    InvalidBlockFace,
    BlockOutOfWorld,
    InvalidGamemode,
}

impl std::fmt::Display for BlockPlacingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PumpkinError for BlockPlacingError {
    fn is_kick(&self) -> bool {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld | Self::InvalidGamemode => false,
            Self::InvalidBlockFace | Self::InvalidHand => true,
        }
    }

    fn severity(&self) -> Level {
        match self {
            Self::BlockOutOfWorld | Self::InvalidGamemode => Level::TRACE,
            Self::BlockOutOfReach | Self::InvalidBlockFace | Self::InvalidHand => Level::WARN,
        }
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld | Self::InvalidGamemode => None,
            Self::InvalidBlockFace => Some("Invalid block face".into()),
            Self::InvalidHand => Some("Invalid hand".into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("sent an oversized message")]
    OversizedMessage,
    #[error("sent a message with illegal characters")]
    IllegalCharacters,
    #[error("sent a chat with invalid/no signature")]
    UnsignedChat,
    #[error("has too many unacknowledged chats queued")]
    TooManyPendingChats,
    #[error("sent a chat that couldn't be validated")]
    ChatValidationFailed,
    #[error("sent a chat with an out of order timestamp")]
    OutOfOrderChat,
    #[error("has an expired public key")]
    ExpiredPublicKey,
    #[error("attempted to initialize a session with an invalid public key")]
    InvalidPublicKey,
}

impl PumpkinError for ChatError {
    fn is_kick(&self) -> bool {
        true
    }

    fn severity(&self) -> Level {
        Level::WARN
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::OversizedMessage => Some("Chat message too long".into()),
            Self::IllegalCharacters => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_ILLEGAL_CHARACTERS,
                    translation::java::MULTIPLAYER_DISCONNECT_ILLEGAL_CHARACTERS,
                    [],
                )
                .get_text(),
            ),
            Self::UnsignedChat => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_UNSIGNED_CHAT,
                    translation::java::MULTIPLAYER_DISCONNECT_UNSIGNED_CHAT,
                    [],
                )
                .get_text(),
            ),
            Self::TooManyPendingChats => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_TOO_MANY_PENDING_CHATS,
                    translation::java::MULTIPLAYER_DISCONNECT_TOO_MANY_PENDING_CHATS,
                    [],
                )
                .get_text(),
            ),
            Self::ChatValidationFailed => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                    translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                    [],
                )
                .get_text(),
            ),
            Self::OutOfOrderChat => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUT_OF_ORDER_CHAT,
                    translation::java::MULTIPLAYER_DISCONNECT_OUT_OF_ORDER_CHAT,
                    [],
                )
                .get_text(),
            ),
            Self::ExpiredPublicKey => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_EXPIRED_PUBLIC_KEY,
                    translation::java::MULTIPLAYER_DISCONNECT_EXPIRED_PUBLIC_KEY,
                    [],
                )
                .get_text(),
            ),
            Self::InvalidPublicKey => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_INVALID_PUBLIC_KEY_SIGNATURE,
                    translation::java::MULTIPLAYER_DISCONNECT_INVALID_PUBLIC_KEY_SIGNATURE,
                    [],
                )
                .get_text(),
            ),
        }
    }
}
