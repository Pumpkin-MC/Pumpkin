/* This file is generated. Do not edit manually. */
use pumpkin_util::version::MinecraftVersion;
pub const CURRENT_MC_VERSION: MinecraftVersion = pumpkin_util::version::MinecraftVersion::V_1_21_11;
pub const LOWEST_SUPPORTED_MC_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_2;
#[derive(Clone, Copy, Debug)]
pub struct PacketId {
    pub v1_21_2: i32,
    pub v1_21_4: i32,
    pub v1_21_5: i32,
    pub v1_21_6: i32,
    pub v1_21_7: i32,
    pub v1_21_9: i32,
    pub v1_21_11: i32,
}
impl PacketId {
    #[doc = r" Converts the requested protocol version into the corresponding packet ID."]
    #[doc = r" Returns -1 if the packet does not exist in that version."]
    pub fn to_id(&self, version: MinecraftVersion) -> i32 {
        match version {
            pumpkin_util::version::MinecraftVersion::V_1_21_2 => self.v1_21_2,
            pumpkin_util::version::MinecraftVersion::V_1_21_4 => self.v1_21_4,
            pumpkin_util::version::MinecraftVersion::V_1_21_5 => self.v1_21_5,
            pumpkin_util::version::MinecraftVersion::V_1_21_6 => self.v1_21_6,
            pumpkin_util::version::MinecraftVersion::V_1_21_7 => self.v1_21_7,
            pumpkin_util::version::MinecraftVersion::V_1_21_9 => self.v1_21_9,
            pumpkin_util::version::MinecraftVersion::V_1_21_11 => self.v1_21_11,
            _ => self.v1_21_11,
        }
    }
}
impl PartialEq<i32> for PacketId {
    fn eq(&self, other: &i32) -> bool {
        self.v1_21_11 == *other
    }
}
impl PartialEq<PacketId> for i32 {
    fn eq(&self, other: &PacketId) -> bool {
        *self == other.v1_21_11
    }
}
pub mod serverbound {
    pub const CONFIG_ACCEPT_CODE_OF_CONDUCT: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 9i32,
        v1_21_11: 9i32,
    };
    pub const CONFIG_CLIENT_INFORMATION: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
    pub const CONFIG_COOKIE_RESPONSE: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const CONFIG_CUSTOM_CLICK_ACTION: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 8i32,
        v1_21_7: 8i32,
        v1_21_9: 8i32,
        v1_21_11: 8i32,
    };
    pub const CONFIG_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        v1_21_2: 2i32,
        v1_21_4: 2i32,
        v1_21_5: 2i32,
        v1_21_6: 2i32,
        v1_21_7: 2i32,
        v1_21_9: 2i32,
        v1_21_11: 2i32,
    };
    pub const CONFIG_FINISH_CONFIGURATION: super::PacketId = super::PacketId {
        v1_21_2: 3i32,
        v1_21_4: 3i32,
        v1_21_5: 3i32,
        v1_21_6: 3i32,
        v1_21_7: 3i32,
        v1_21_9: 3i32,
        v1_21_11: 3i32,
    };
    pub const CONFIG_KEEP_ALIVE: super::PacketId = super::PacketId {
        v1_21_2: 4i32,
        v1_21_4: 4i32,
        v1_21_5: 4i32,
        v1_21_6: 4i32,
        v1_21_7: 4i32,
        v1_21_9: 4i32,
        v1_21_11: 4i32,
    };
    pub const CONFIG_PONG: super::PacketId = super::PacketId {
        v1_21_2: 5i32,
        v1_21_4: 5i32,
        v1_21_5: 5i32,
        v1_21_6: 5i32,
        v1_21_7: 5i32,
        v1_21_9: 5i32,
        v1_21_11: 5i32,
    };
    pub const CONFIG_RESOURCE_PACK: super::PacketId = super::PacketId {
        v1_21_2: 6i32,
        v1_21_4: 6i32,
        v1_21_5: 6i32,
        v1_21_6: 6i32,
        v1_21_7: 6i32,
        v1_21_9: 6i32,
        v1_21_11: 6i32,
    };
    pub const CONFIG_SELECT_KNOWN_PACKS: super::PacketId = super::PacketId {
        v1_21_2: 7i32,
        v1_21_4: 7i32,
        v1_21_5: 7i32,
        v1_21_6: 7i32,
        v1_21_7: 7i32,
        v1_21_9: 7i32,
        v1_21_11: 7i32,
    };
    pub const HANDSHAKE_INTENTION: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
    pub const LOGIN_COOKIE_RESPONSE: super::PacketId = super::PacketId {
        v1_21_2: 4i32,
        v1_21_4: 4i32,
        v1_21_5: 4i32,
        v1_21_6: 4i32,
        v1_21_7: 4i32,
        v1_21_9: 4i32,
        v1_21_11: 4i32,
    };
    pub const LOGIN_CUSTOM_QUERY_ANSWER: super::PacketId = super::PacketId {
        v1_21_2: 2i32,
        v1_21_4: 2i32,
        v1_21_5: 2i32,
        v1_21_6: 2i32,
        v1_21_7: 2i32,
        v1_21_9: 2i32,
        v1_21_11: 2i32,
    };
    pub const LOGIN_HELLO: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
    pub const LOGIN_KEY: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const LOGIN_LOGIN_ACKNOWLEDGED: super::PacketId = super::PacketId {
        v1_21_2: 3i32,
        v1_21_4: 3i32,
        v1_21_5: 3i32,
        v1_21_6: 3i32,
        v1_21_7: 3i32,
        v1_21_9: 3i32,
        v1_21_11: 3i32,
    };
    pub const PLAY_ACCEPT_TELEPORTATION: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
    pub const PLAY_BLOCK_ENTITY_TAG_QUERY: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const PLAY_BUNDLE_ITEM_SELECTED: super::PacketId = super::PacketId {
        v1_21_2: 2i32,
        v1_21_4: 2i32,
        v1_21_5: 2i32,
        v1_21_6: 2i32,
        v1_21_7: 2i32,
        v1_21_9: 2i32,
        v1_21_11: 2i32,
    };
    pub const PLAY_CHANGE_DIFFICULTY: super::PacketId = super::PacketId {
        v1_21_2: 3i32,
        v1_21_4: 3i32,
        v1_21_5: 3i32,
        v1_21_6: 3i32,
        v1_21_7: 3i32,
        v1_21_9: 3i32,
        v1_21_11: 3i32,
    };
    pub const PLAY_CHANGE_GAME_MODE: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 4i32,
        v1_21_7: 4i32,
        v1_21_9: 4i32,
        v1_21_11: 4i32,
    };
    pub const PLAY_CHAT: super::PacketId = super::PacketId {
        v1_21_2: 7i32,
        v1_21_4: 7i32,
        v1_21_5: 7i32,
        v1_21_6: 8i32,
        v1_21_7: 8i32,
        v1_21_9: 8i32,
        v1_21_11: 8i32,
    };
    pub const PLAY_CHAT_ACK: super::PacketId = super::PacketId {
        v1_21_2: 4i32,
        v1_21_4: 4i32,
        v1_21_5: 4i32,
        v1_21_6: 5i32,
        v1_21_7: 5i32,
        v1_21_9: 5i32,
        v1_21_11: 5i32,
    };
    pub const PLAY_CHAT_COMMAND: super::PacketId = super::PacketId {
        v1_21_2: 5i32,
        v1_21_4: 5i32,
        v1_21_5: 5i32,
        v1_21_6: 6i32,
        v1_21_7: 6i32,
        v1_21_9: 6i32,
        v1_21_11: 6i32,
    };
    pub const PLAY_CHAT_COMMAND_SIGNED: super::PacketId = super::PacketId {
        v1_21_2: 6i32,
        v1_21_4: 6i32,
        v1_21_5: 6i32,
        v1_21_6: 7i32,
        v1_21_7: 7i32,
        v1_21_9: 7i32,
        v1_21_11: 7i32,
    };
    pub const PLAY_CHAT_SESSION_UPDATE: super::PacketId = super::PacketId {
        v1_21_2: 8i32,
        v1_21_4: 8i32,
        v1_21_5: 8i32,
        v1_21_6: 9i32,
        v1_21_7: 9i32,
        v1_21_9: 9i32,
        v1_21_11: 9i32,
    };
    pub const PLAY_CHUNK_BATCH_RECEIVED: super::PacketId = super::PacketId {
        v1_21_2: 9i32,
        v1_21_4: 9i32,
        v1_21_5: 9i32,
        v1_21_6: 10i32,
        v1_21_7: 10i32,
        v1_21_9: 10i32,
        v1_21_11: 10i32,
    };
    pub const PLAY_CLIENT_COMMAND: super::PacketId = super::PacketId {
        v1_21_2: 10i32,
        v1_21_4: 10i32,
        v1_21_5: 10i32,
        v1_21_6: 11i32,
        v1_21_7: 11i32,
        v1_21_9: 11i32,
        v1_21_11: 11i32,
    };
    pub const PLAY_CLIENT_INFORMATION: super::PacketId = super::PacketId {
        v1_21_2: 12i32,
        v1_21_4: 12i32,
        v1_21_5: 12i32,
        v1_21_6: 13i32,
        v1_21_7: 13i32,
        v1_21_9: 13i32,
        v1_21_11: 13i32,
    };
    pub const PLAY_CLIENT_TICK_END: super::PacketId = super::PacketId {
        v1_21_2: 11i32,
        v1_21_4: 11i32,
        v1_21_5: 11i32,
        v1_21_6: 12i32,
        v1_21_7: 12i32,
        v1_21_9: 12i32,
        v1_21_11: 12i32,
    };
    pub const PLAY_COMMAND_SUGGESTION: super::PacketId = super::PacketId {
        v1_21_2: 13i32,
        v1_21_4: 13i32,
        v1_21_5: 13i32,
        v1_21_6: 14i32,
        v1_21_7: 14i32,
        v1_21_9: 14i32,
        v1_21_11: 14i32,
    };
    pub const PLAY_CONFIGURATION_ACKNOWLEDGED: super::PacketId = super::PacketId {
        v1_21_2: 14i32,
        v1_21_4: 14i32,
        v1_21_5: 14i32,
        v1_21_6: 15i32,
        v1_21_7: 15i32,
        v1_21_9: 15i32,
        v1_21_11: 15i32,
    };
    pub const PLAY_CONTAINER_BUTTON_CLICK: super::PacketId = super::PacketId {
        v1_21_2: 15i32,
        v1_21_4: 15i32,
        v1_21_5: 15i32,
        v1_21_6: 16i32,
        v1_21_7: 16i32,
        v1_21_9: 16i32,
        v1_21_11: 16i32,
    };
    pub const PLAY_CONTAINER_CLICK: super::PacketId = super::PacketId {
        v1_21_2: 16i32,
        v1_21_4: 16i32,
        v1_21_5: 16i32,
        v1_21_6: 17i32,
        v1_21_7: 17i32,
        v1_21_9: 17i32,
        v1_21_11: 17i32,
    };
    pub const PLAY_CONTAINER_CLOSE: super::PacketId = super::PacketId {
        v1_21_2: 17i32,
        v1_21_4: 17i32,
        v1_21_5: 17i32,
        v1_21_6: 18i32,
        v1_21_7: 18i32,
        v1_21_9: 18i32,
        v1_21_11: 18i32,
    };
    pub const PLAY_CONTAINER_SLOT_STATE_CHANGED: super::PacketId = super::PacketId {
        v1_21_2: 18i32,
        v1_21_4: 18i32,
        v1_21_5: 18i32,
        v1_21_6: 19i32,
        v1_21_7: 19i32,
        v1_21_9: 19i32,
        v1_21_11: 19i32,
    };
    pub const PLAY_COOKIE_RESPONSE: super::PacketId = super::PacketId {
        v1_21_2: 19i32,
        v1_21_4: 19i32,
        v1_21_5: 19i32,
        v1_21_6: 20i32,
        v1_21_7: 20i32,
        v1_21_9: 20i32,
        v1_21_11: 20i32,
    };
    pub const PLAY_CUSTOM_CLICK_ACTION: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 65i32,
        v1_21_7: 65i32,
        v1_21_9: 65i32,
        v1_21_11: 65i32,
    };
    pub const PLAY_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        v1_21_2: 20i32,
        v1_21_4: 20i32,
        v1_21_5: 20i32,
        v1_21_6: 21i32,
        v1_21_7: 21i32,
        v1_21_9: 21i32,
        v1_21_11: 21i32,
    };
    pub const PLAY_DEBUG_SUBSCRIPTION_REQUEST: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 22i32,
        v1_21_11: 22i32,
    };
    pub const PLAY_EDIT_BOOK: super::PacketId = super::PacketId {
        v1_21_2: 22i32,
        v1_21_4: 22i32,
        v1_21_5: 22i32,
        v1_21_6: 23i32,
        v1_21_7: 23i32,
        v1_21_9: 23i32,
        v1_21_11: 23i32,
    };
    pub const PLAY_ENTITY_TAG_QUERY: super::PacketId = super::PacketId {
        v1_21_2: 23i32,
        v1_21_4: 23i32,
        v1_21_5: 23i32,
        v1_21_6: 24i32,
        v1_21_7: 24i32,
        v1_21_9: 24i32,
        v1_21_11: 24i32,
    };
    pub const PLAY_INTERACT: super::PacketId = super::PacketId {
        v1_21_2: 24i32,
        v1_21_4: 24i32,
        v1_21_5: 24i32,
        v1_21_6: 25i32,
        v1_21_7: 25i32,
        v1_21_9: 25i32,
        v1_21_11: 25i32,
    };
    pub const PLAY_JIGSAW_GENERATE: super::PacketId = super::PacketId {
        v1_21_2: 25i32,
        v1_21_4: 25i32,
        v1_21_5: 25i32,
        v1_21_6: 26i32,
        v1_21_7: 26i32,
        v1_21_9: 26i32,
        v1_21_11: 26i32,
    };
    pub const PLAY_KEEP_ALIVE: super::PacketId = super::PacketId {
        v1_21_2: 26i32,
        v1_21_4: 26i32,
        v1_21_5: 26i32,
        v1_21_6: 27i32,
        v1_21_7: 27i32,
        v1_21_9: 27i32,
        v1_21_11: 27i32,
    };
    pub const PLAY_LOCK_DIFFICULTY: super::PacketId = super::PacketId {
        v1_21_2: 27i32,
        v1_21_4: 27i32,
        v1_21_5: 27i32,
        v1_21_6: 28i32,
        v1_21_7: 28i32,
        v1_21_9: 28i32,
        v1_21_11: 28i32,
    };
    pub const PLAY_MOVE_PLAYER_POS: super::PacketId = super::PacketId {
        v1_21_2: 28i32,
        v1_21_4: 28i32,
        v1_21_5: 28i32,
        v1_21_6: 29i32,
        v1_21_7: 29i32,
        v1_21_9: 29i32,
        v1_21_11: 29i32,
    };
    pub const PLAY_MOVE_PLAYER_POS_ROT: super::PacketId = super::PacketId {
        v1_21_2: 29i32,
        v1_21_4: 29i32,
        v1_21_5: 29i32,
        v1_21_6: 30i32,
        v1_21_7: 30i32,
        v1_21_9: 30i32,
        v1_21_11: 30i32,
    };
    pub const PLAY_MOVE_PLAYER_ROT: super::PacketId = super::PacketId {
        v1_21_2: 30i32,
        v1_21_4: 30i32,
        v1_21_5: 30i32,
        v1_21_6: 31i32,
        v1_21_7: 31i32,
        v1_21_9: 31i32,
        v1_21_11: 31i32,
    };
    pub const PLAY_MOVE_PLAYER_STATUS_ONLY: super::PacketId = super::PacketId {
        v1_21_2: 31i32,
        v1_21_4: 31i32,
        v1_21_5: 31i32,
        v1_21_6: 32i32,
        v1_21_7: 32i32,
        v1_21_9: 32i32,
        v1_21_11: 32i32,
    };
    pub const PLAY_MOVE_VEHICLE: super::PacketId = super::PacketId {
        v1_21_2: 32i32,
        v1_21_4: 32i32,
        v1_21_5: 32i32,
        v1_21_6: 33i32,
        v1_21_7: 33i32,
        v1_21_9: 33i32,
        v1_21_11: 33i32,
    };
    pub const PLAY_PADDLE_BOAT: super::PacketId = super::PacketId {
        v1_21_2: 33i32,
        v1_21_4: 33i32,
        v1_21_5: 33i32,
        v1_21_6: 34i32,
        v1_21_7: 34i32,
        v1_21_9: 34i32,
        v1_21_11: 34i32,
    };
    pub const PLAY_PICK_ITEM_FROM_BLOCK: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: 34i32,
        v1_21_5: 34i32,
        v1_21_6: 35i32,
        v1_21_7: 35i32,
        v1_21_9: 35i32,
        v1_21_11: 35i32,
    };
    pub const PLAY_PICK_ITEM_FROM_ENTITY: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: 35i32,
        v1_21_5: 35i32,
        v1_21_6: 36i32,
        v1_21_7: 36i32,
        v1_21_9: 36i32,
        v1_21_11: 36i32,
    };
    pub const PLAY_PING_REQUEST: super::PacketId = super::PacketId {
        v1_21_2: 35i32,
        v1_21_4: 36i32,
        v1_21_5: 36i32,
        v1_21_6: 37i32,
        v1_21_7: 37i32,
        v1_21_9: 37i32,
        v1_21_11: 37i32,
    };
    pub const PLAY_PLACE_RECIPE: super::PacketId = super::PacketId {
        v1_21_2: 36i32,
        v1_21_4: 37i32,
        v1_21_5: 37i32,
        v1_21_6: 38i32,
        v1_21_7: 38i32,
        v1_21_9: 38i32,
        v1_21_11: 38i32,
    };
    pub const PLAY_PLAYER_ABILITIES: super::PacketId = super::PacketId {
        v1_21_2: 37i32,
        v1_21_4: 38i32,
        v1_21_5: 38i32,
        v1_21_6: 39i32,
        v1_21_7: 39i32,
        v1_21_9: 39i32,
        v1_21_11: 39i32,
    };
    pub const PLAY_PLAYER_ACTION: super::PacketId = super::PacketId {
        v1_21_2: 38i32,
        v1_21_4: 39i32,
        v1_21_5: 39i32,
        v1_21_6: 40i32,
        v1_21_7: 40i32,
        v1_21_9: 40i32,
        v1_21_11: 40i32,
    };
    pub const PLAY_PLAYER_COMMAND: super::PacketId = super::PacketId {
        v1_21_2: 39i32,
        v1_21_4: 40i32,
        v1_21_5: 40i32,
        v1_21_6: 41i32,
        v1_21_7: 41i32,
        v1_21_9: 41i32,
        v1_21_11: 41i32,
    };
    pub const PLAY_PLAYER_INPUT: super::PacketId = super::PacketId {
        v1_21_2: 40i32,
        v1_21_4: 41i32,
        v1_21_5: 41i32,
        v1_21_6: 42i32,
        v1_21_7: 42i32,
        v1_21_9: 42i32,
        v1_21_11: 42i32,
    };
    pub const PLAY_PLAYER_LOADED: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: 42i32,
        v1_21_5: 42i32,
        v1_21_6: 43i32,
        v1_21_7: 43i32,
        v1_21_9: 43i32,
        v1_21_11: 43i32,
    };
    pub const PLAY_PONG: super::PacketId = super::PacketId {
        v1_21_2: 41i32,
        v1_21_4: 43i32,
        v1_21_5: 43i32,
        v1_21_6: 44i32,
        v1_21_7: 44i32,
        v1_21_9: 44i32,
        v1_21_11: 44i32,
    };
    pub const PLAY_RECIPE_BOOK_CHANGE_SETTINGS: super::PacketId = super::PacketId {
        v1_21_2: 42i32,
        v1_21_4: 44i32,
        v1_21_5: 44i32,
        v1_21_6: 45i32,
        v1_21_7: 45i32,
        v1_21_9: 45i32,
        v1_21_11: 45i32,
    };
    pub const PLAY_RECIPE_BOOK_SEEN_RECIPE: super::PacketId = super::PacketId {
        v1_21_2: 43i32,
        v1_21_4: 45i32,
        v1_21_5: 45i32,
        v1_21_6: 46i32,
        v1_21_7: 46i32,
        v1_21_9: 46i32,
        v1_21_11: 46i32,
    };
    pub const PLAY_RENAME_ITEM: super::PacketId = super::PacketId {
        v1_21_2: 44i32,
        v1_21_4: 46i32,
        v1_21_5: 46i32,
        v1_21_6: 47i32,
        v1_21_7: 47i32,
        v1_21_9: 47i32,
        v1_21_11: 47i32,
    };
    pub const PLAY_RESOURCE_PACK: super::PacketId = super::PacketId {
        v1_21_2: 45i32,
        v1_21_4: 47i32,
        v1_21_5: 47i32,
        v1_21_6: 48i32,
        v1_21_7: 48i32,
        v1_21_9: 48i32,
        v1_21_11: 48i32,
    };
    pub const PLAY_SEEN_ADVANCEMENTS: super::PacketId = super::PacketId {
        v1_21_2: 46i32,
        v1_21_4: 48i32,
        v1_21_5: 48i32,
        v1_21_6: 49i32,
        v1_21_7: 49i32,
        v1_21_9: 49i32,
        v1_21_11: 49i32,
    };
    pub const PLAY_SELECT_TRADE: super::PacketId = super::PacketId {
        v1_21_2: 47i32,
        v1_21_4: 49i32,
        v1_21_5: 49i32,
        v1_21_6: 50i32,
        v1_21_7: 50i32,
        v1_21_9: 50i32,
        v1_21_11: 50i32,
    };
    pub const PLAY_SET_BEACON: super::PacketId = super::PacketId {
        v1_21_2: 48i32,
        v1_21_4: 50i32,
        v1_21_5: 50i32,
        v1_21_6: 51i32,
        v1_21_7: 51i32,
        v1_21_9: 51i32,
        v1_21_11: 51i32,
    };
    pub const PLAY_SET_CARRIED_ITEM: super::PacketId = super::PacketId {
        v1_21_2: 49i32,
        v1_21_4: 51i32,
        v1_21_5: 51i32,
        v1_21_6: 52i32,
        v1_21_7: 52i32,
        v1_21_9: 52i32,
        v1_21_11: 52i32,
    };
    pub const PLAY_SET_COMMAND_BLOCK: super::PacketId = super::PacketId {
        v1_21_2: 50i32,
        v1_21_4: 52i32,
        v1_21_5: 52i32,
        v1_21_6: 53i32,
        v1_21_7: 53i32,
        v1_21_9: 53i32,
        v1_21_11: 53i32,
    };
    pub const PLAY_SET_COMMAND_MINECART: super::PacketId = super::PacketId {
        v1_21_2: 51i32,
        v1_21_4: 53i32,
        v1_21_5: 53i32,
        v1_21_6: 54i32,
        v1_21_7: 54i32,
        v1_21_9: 54i32,
        v1_21_11: 54i32,
    };
    pub const PLAY_SET_CREATIVE_MODE_SLOT: super::PacketId = super::PacketId {
        v1_21_2: 52i32,
        v1_21_4: 54i32,
        v1_21_5: 54i32,
        v1_21_6: 55i32,
        v1_21_7: 55i32,
        v1_21_9: 55i32,
        v1_21_11: 55i32,
    };
    pub const PLAY_SET_JIGSAW_BLOCK: super::PacketId = super::PacketId {
        v1_21_2: 53i32,
        v1_21_4: 55i32,
        v1_21_5: 55i32,
        v1_21_6: 56i32,
        v1_21_7: 56i32,
        v1_21_9: 56i32,
        v1_21_11: 56i32,
    };
    pub const PLAY_SET_STRUCTURE_BLOCK: super::PacketId = super::PacketId {
        v1_21_2: 54i32,
        v1_21_4: 56i32,
        v1_21_5: 56i32,
        v1_21_6: 57i32,
        v1_21_7: 57i32,
        v1_21_9: 57i32,
        v1_21_11: 57i32,
    };
    pub const PLAY_SET_TEST_BLOCK: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: 57i32,
        v1_21_6: 58i32,
        v1_21_7: 58i32,
        v1_21_9: 58i32,
        v1_21_11: 58i32,
    };
    pub const PLAY_SIGN_UPDATE: super::PacketId = super::PacketId {
        v1_21_2: 55i32,
        v1_21_4: 57i32,
        v1_21_5: 58i32,
        v1_21_6: 59i32,
        v1_21_7: 59i32,
        v1_21_9: 59i32,
        v1_21_11: 59i32,
    };
    pub const PLAY_SWING: super::PacketId = super::PacketId {
        v1_21_2: 56i32,
        v1_21_4: 58i32,
        v1_21_5: 59i32,
        v1_21_6: 60i32,
        v1_21_7: 60i32,
        v1_21_9: 60i32,
        v1_21_11: 60i32,
    };
    pub const PLAY_TELEPORT_TO_ENTITY: super::PacketId = super::PacketId {
        v1_21_2: 57i32,
        v1_21_4: 59i32,
        v1_21_5: 60i32,
        v1_21_6: 61i32,
        v1_21_7: 61i32,
        v1_21_9: 61i32,
        v1_21_11: 61i32,
    };
    pub const PLAY_TEST_INSTANCE_BLOCK_ACTION: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: 61i32,
        v1_21_6: 62i32,
        v1_21_7: 62i32,
        v1_21_9: 62i32,
        v1_21_11: 62i32,
    };
    pub const PLAY_USE_ITEM: super::PacketId = super::PacketId {
        v1_21_2: 59i32,
        v1_21_4: 61i32,
        v1_21_5: 63i32,
        v1_21_6: 64i32,
        v1_21_7: 64i32,
        v1_21_9: 64i32,
        v1_21_11: 64i32,
    };
    pub const PLAY_USE_ITEM_ON: super::PacketId = super::PacketId {
        v1_21_2: 58i32,
        v1_21_4: 60i32,
        v1_21_5: 62i32,
        v1_21_6: 63i32,
        v1_21_7: 63i32,
        v1_21_9: 63i32,
        v1_21_11: 63i32,
    };
    pub const STATUS_PING_REQUEST: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const STATUS_STATUS_REQUEST: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
}
pub mod clientbound {
    pub const CONFIG_CLEAR_DIALOG: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 17i32,
        v1_21_7: 17i32,
        v1_21_9: 17i32,
        v1_21_11: 17i32,
    };
    pub const CONFIG_CODE_OF_CONDUCT: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 19i32,
        v1_21_11: 19i32,
    };
    pub const CONFIG_COOKIE_REQUEST: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
    pub const CONFIG_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const CONFIG_CUSTOM_REPORT_DETAILS: super::PacketId = super::PacketId {
        v1_21_2: 15i32,
        v1_21_4: 15i32,
        v1_21_5: 15i32,
        v1_21_6: 15i32,
        v1_21_7: 15i32,
        v1_21_9: 15i32,
        v1_21_11: 15i32,
    };
    pub const CONFIG_DISCONNECT: super::PacketId = super::PacketId {
        v1_21_2: 2i32,
        v1_21_4: 2i32,
        v1_21_5: 2i32,
        v1_21_6: 2i32,
        v1_21_7: 2i32,
        v1_21_9: 2i32,
        v1_21_11: 2i32,
    };
    pub const CONFIG_FINISH_CONFIGURATION: super::PacketId = super::PacketId {
        v1_21_2: 3i32,
        v1_21_4: 3i32,
        v1_21_5: 3i32,
        v1_21_6: 3i32,
        v1_21_7: 3i32,
        v1_21_9: 3i32,
        v1_21_11: 3i32,
    };
    pub const CONFIG_KEEP_ALIVE: super::PacketId = super::PacketId {
        v1_21_2: 4i32,
        v1_21_4: 4i32,
        v1_21_5: 4i32,
        v1_21_6: 4i32,
        v1_21_7: 4i32,
        v1_21_9: 4i32,
        v1_21_11: 4i32,
    };
    pub const CONFIG_PING: super::PacketId = super::PacketId {
        v1_21_2: 5i32,
        v1_21_4: 5i32,
        v1_21_5: 5i32,
        v1_21_6: 5i32,
        v1_21_7: 5i32,
        v1_21_9: 5i32,
        v1_21_11: 5i32,
    };
    pub const CONFIG_REGISTRY_DATA: super::PacketId = super::PacketId {
        v1_21_2: 7i32,
        v1_21_4: 7i32,
        v1_21_5: 7i32,
        v1_21_6: 7i32,
        v1_21_7: 7i32,
        v1_21_9: 7i32,
        v1_21_11: 7i32,
    };
    pub const CONFIG_RESET_CHAT: super::PacketId = super::PacketId {
        v1_21_2: 6i32,
        v1_21_4: 6i32,
        v1_21_5: 6i32,
        v1_21_6: 6i32,
        v1_21_7: 6i32,
        v1_21_9: 6i32,
        v1_21_11: 6i32,
    };
    pub const CONFIG_RESOURCE_PACK_POP: super::PacketId = super::PacketId {
        v1_21_2: 8i32,
        v1_21_4: 8i32,
        v1_21_5: 8i32,
        v1_21_6: 8i32,
        v1_21_7: 8i32,
        v1_21_9: 8i32,
        v1_21_11: 8i32,
    };
    pub const CONFIG_RESOURCE_PACK_PUSH: super::PacketId = super::PacketId {
        v1_21_2: 9i32,
        v1_21_4: 9i32,
        v1_21_5: 9i32,
        v1_21_6: 9i32,
        v1_21_7: 9i32,
        v1_21_9: 9i32,
        v1_21_11: 9i32,
    };
    pub const CONFIG_SELECT_KNOWN_PACKS: super::PacketId = super::PacketId {
        v1_21_2: 14i32,
        v1_21_4: 14i32,
        v1_21_5: 14i32,
        v1_21_6: 14i32,
        v1_21_7: 14i32,
        v1_21_9: 14i32,
        v1_21_11: 14i32,
    };
    pub const CONFIG_SERVER_LINKS: super::PacketId = super::PacketId {
        v1_21_2: 16i32,
        v1_21_4: 16i32,
        v1_21_5: 16i32,
        v1_21_6: 16i32,
        v1_21_7: 16i32,
        v1_21_9: 16i32,
        v1_21_11: 16i32,
    };
    pub const CONFIG_SHOW_DIALOG: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 18i32,
        v1_21_7: 18i32,
        v1_21_9: 18i32,
        v1_21_11: 18i32,
    };
    pub const CONFIG_STORE_COOKIE: super::PacketId = super::PacketId {
        v1_21_2: 10i32,
        v1_21_4: 10i32,
        v1_21_5: 10i32,
        v1_21_6: 10i32,
        v1_21_7: 10i32,
        v1_21_9: 10i32,
        v1_21_11: 10i32,
    };
    pub const CONFIG_TRANSFER: super::PacketId = super::PacketId {
        v1_21_2: 11i32,
        v1_21_4: 11i32,
        v1_21_5: 11i32,
        v1_21_6: 11i32,
        v1_21_7: 11i32,
        v1_21_9: 11i32,
        v1_21_11: 11i32,
    };
    pub const CONFIG_UPDATE_ENABLED_FEATURES: super::PacketId = super::PacketId {
        v1_21_2: 12i32,
        v1_21_4: 12i32,
        v1_21_5: 12i32,
        v1_21_6: 12i32,
        v1_21_7: 12i32,
        v1_21_9: 12i32,
        v1_21_11: 12i32,
    };
    pub const CONFIG_UPDATE_TAGS: super::PacketId = super::PacketId {
        v1_21_2: 13i32,
        v1_21_4: 13i32,
        v1_21_5: 13i32,
        v1_21_6: 13i32,
        v1_21_7: 13i32,
        v1_21_9: 13i32,
        v1_21_11: 13i32,
    };
    pub const LOGIN_COOKIE_REQUEST: super::PacketId = super::PacketId {
        v1_21_2: 5i32,
        v1_21_4: 5i32,
        v1_21_5: 5i32,
        v1_21_6: 5i32,
        v1_21_7: 5i32,
        v1_21_9: 5i32,
        v1_21_11: 5i32,
    };
    pub const LOGIN_CUSTOM_QUERY: super::PacketId = super::PacketId {
        v1_21_2: 4i32,
        v1_21_4: 4i32,
        v1_21_5: 4i32,
        v1_21_6: 4i32,
        v1_21_7: 4i32,
        v1_21_9: 4i32,
        v1_21_11: 4i32,
    };
    pub const LOGIN_HELLO: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const LOGIN_LOGIN_COMPRESSION: super::PacketId = super::PacketId {
        v1_21_2: 3i32,
        v1_21_4: 3i32,
        v1_21_5: 3i32,
        v1_21_6: 3i32,
        v1_21_7: 3i32,
        v1_21_9: 3i32,
        v1_21_11: 3i32,
    };
    pub const LOGIN_LOGIN_DISCONNECT: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
    pub const LOGIN_LOGIN_FINISHED: super::PacketId = super::PacketId {
        v1_21_2: 2i32,
        v1_21_4: 2i32,
        v1_21_5: 2i32,
        v1_21_6: 2i32,
        v1_21_7: 2i32,
        v1_21_9: 2i32,
        v1_21_11: 2i32,
    };
    pub const PLAY_ADD_ENTITY: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const PLAY_ANIMATE: super::PacketId = super::PacketId {
        v1_21_2: 3i32,
        v1_21_4: 3i32,
        v1_21_5: 2i32,
        v1_21_6: 2i32,
        v1_21_7: 2i32,
        v1_21_9: 2i32,
        v1_21_11: 2i32,
    };
    pub const PLAY_AWARD_STATS: super::PacketId = super::PacketId {
        v1_21_2: 4i32,
        v1_21_4: 4i32,
        v1_21_5: 3i32,
        v1_21_6: 3i32,
        v1_21_7: 3i32,
        v1_21_9: 3i32,
        v1_21_11: 3i32,
    };
    pub const PLAY_BLOCK_CHANGED_ACK: super::PacketId = super::PacketId {
        v1_21_2: 5i32,
        v1_21_4: 5i32,
        v1_21_5: 4i32,
        v1_21_6: 4i32,
        v1_21_7: 4i32,
        v1_21_9: 4i32,
        v1_21_11: 4i32,
    };
    pub const PLAY_BLOCK_DESTRUCTION: super::PacketId = super::PacketId {
        v1_21_2: 6i32,
        v1_21_4: 6i32,
        v1_21_5: 5i32,
        v1_21_6: 5i32,
        v1_21_7: 5i32,
        v1_21_9: 5i32,
        v1_21_11: 5i32,
    };
    pub const PLAY_BLOCK_ENTITY_DATA: super::PacketId = super::PacketId {
        v1_21_2: 7i32,
        v1_21_4: 7i32,
        v1_21_5: 6i32,
        v1_21_6: 6i32,
        v1_21_7: 6i32,
        v1_21_9: 6i32,
        v1_21_11: 6i32,
    };
    pub const PLAY_BLOCK_EVENT: super::PacketId = super::PacketId {
        v1_21_2: 8i32,
        v1_21_4: 8i32,
        v1_21_5: 7i32,
        v1_21_6: 7i32,
        v1_21_7: 7i32,
        v1_21_9: 7i32,
        v1_21_11: 7i32,
    };
    pub const PLAY_BLOCK_UPDATE: super::PacketId = super::PacketId {
        v1_21_2: 9i32,
        v1_21_4: 9i32,
        v1_21_5: 8i32,
        v1_21_6: 8i32,
        v1_21_7: 8i32,
        v1_21_9: 8i32,
        v1_21_11: 8i32,
    };
    pub const PLAY_BOSS_EVENT: super::PacketId = super::PacketId {
        v1_21_2: 10i32,
        v1_21_4: 10i32,
        v1_21_5: 9i32,
        v1_21_6: 9i32,
        v1_21_7: 9i32,
        v1_21_9: 9i32,
        v1_21_11: 9i32,
    };
    pub const PLAY_BUNDLE_DELIMITER: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
    pub const PLAY_CHANGE_DIFFICULTY: super::PacketId = super::PacketId {
        v1_21_2: 11i32,
        v1_21_4: 11i32,
        v1_21_5: 10i32,
        v1_21_6: 10i32,
        v1_21_7: 10i32,
        v1_21_9: 10i32,
        v1_21_11: 10i32,
    };
    pub const PLAY_CHUNK_BATCH_FINISHED: super::PacketId = super::PacketId {
        v1_21_2: 12i32,
        v1_21_4: 12i32,
        v1_21_5: 11i32,
        v1_21_6: 11i32,
        v1_21_7: 11i32,
        v1_21_9: 11i32,
        v1_21_11: 11i32,
    };
    pub const PLAY_CHUNK_BATCH_START: super::PacketId = super::PacketId {
        v1_21_2: 13i32,
        v1_21_4: 13i32,
        v1_21_5: 12i32,
        v1_21_6: 12i32,
        v1_21_7: 12i32,
        v1_21_9: 12i32,
        v1_21_11: 12i32,
    };
    pub const PLAY_CHUNKS_BIOMES: super::PacketId = super::PacketId {
        v1_21_2: 14i32,
        v1_21_4: 14i32,
        v1_21_5: 13i32,
        v1_21_6: 13i32,
        v1_21_7: 13i32,
        v1_21_9: 13i32,
        v1_21_11: 13i32,
    };
    pub const PLAY_CLEAR_DIALOG: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 132i32,
        v1_21_7: 132i32,
        v1_21_9: 137i32,
        v1_21_11: 137i32,
    };
    pub const PLAY_CLEAR_TITLES: super::PacketId = super::PacketId {
        v1_21_2: 15i32,
        v1_21_4: 15i32,
        v1_21_5: 14i32,
        v1_21_6: 14i32,
        v1_21_7: 14i32,
        v1_21_9: 14i32,
        v1_21_11: 14i32,
    };
    pub const PLAY_COMMAND_SUGGESTIONS: super::PacketId = super::PacketId {
        v1_21_2: 16i32,
        v1_21_4: 16i32,
        v1_21_5: 15i32,
        v1_21_6: 15i32,
        v1_21_7: 15i32,
        v1_21_9: 15i32,
        v1_21_11: 15i32,
    };
    pub const PLAY_COMMANDS: super::PacketId = super::PacketId {
        v1_21_2: 17i32,
        v1_21_4: 17i32,
        v1_21_5: 16i32,
        v1_21_6: 16i32,
        v1_21_7: 16i32,
        v1_21_9: 16i32,
        v1_21_11: 16i32,
    };
    pub const PLAY_CONTAINER_CLOSE: super::PacketId = super::PacketId {
        v1_21_2: 18i32,
        v1_21_4: 18i32,
        v1_21_5: 17i32,
        v1_21_6: 17i32,
        v1_21_7: 17i32,
        v1_21_9: 17i32,
        v1_21_11: 17i32,
    };
    pub const PLAY_CONTAINER_SET_CONTENT: super::PacketId = super::PacketId {
        v1_21_2: 19i32,
        v1_21_4: 19i32,
        v1_21_5: 18i32,
        v1_21_6: 18i32,
        v1_21_7: 18i32,
        v1_21_9: 18i32,
        v1_21_11: 18i32,
    };
    pub const PLAY_CONTAINER_SET_DATA: super::PacketId = super::PacketId {
        v1_21_2: 20i32,
        v1_21_4: 20i32,
        v1_21_5: 19i32,
        v1_21_6: 19i32,
        v1_21_7: 19i32,
        v1_21_9: 19i32,
        v1_21_11: 19i32,
    };
    pub const PLAY_CONTAINER_SET_SLOT: super::PacketId = super::PacketId {
        v1_21_2: 21i32,
        v1_21_4: 21i32,
        v1_21_5: 20i32,
        v1_21_6: 20i32,
        v1_21_7: 20i32,
        v1_21_9: 20i32,
        v1_21_11: 20i32,
    };
    pub const PLAY_COOKIE_REQUEST: super::PacketId = super::PacketId {
        v1_21_2: 22i32,
        v1_21_4: 22i32,
        v1_21_5: 21i32,
        v1_21_6: 21i32,
        v1_21_7: 21i32,
        v1_21_9: 21i32,
        v1_21_11: 21i32,
    };
    pub const PLAY_COOLDOWN: super::PacketId = super::PacketId {
        v1_21_2: 23i32,
        v1_21_4: 23i32,
        v1_21_5: 22i32,
        v1_21_6: 22i32,
        v1_21_7: 22i32,
        v1_21_9: 22i32,
        v1_21_11: 22i32,
    };
    pub const PLAY_CUSTOM_CHAT_COMPLETIONS: super::PacketId = super::PacketId {
        v1_21_2: 24i32,
        v1_21_4: 24i32,
        v1_21_5: 23i32,
        v1_21_6: 23i32,
        v1_21_7: 23i32,
        v1_21_9: 23i32,
        v1_21_11: 23i32,
    };
    pub const PLAY_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        v1_21_2: 25i32,
        v1_21_4: 25i32,
        v1_21_5: 24i32,
        v1_21_6: 24i32,
        v1_21_7: 24i32,
        v1_21_9: 24i32,
        v1_21_11: 24i32,
    };
    pub const PLAY_CUSTOM_REPORT_DETAILS: super::PacketId = super::PacketId {
        v1_21_2: 129i32,
        v1_21_4: 129i32,
        v1_21_5: 129i32,
        v1_21_6: 129i32,
        v1_21_7: 129i32,
        v1_21_9: 134i32,
        v1_21_11: 134i32,
    };
    pub const PLAY_DAMAGE_EVENT: super::PacketId = super::PacketId {
        v1_21_2: 26i32,
        v1_21_4: 26i32,
        v1_21_5: 25i32,
        v1_21_6: 25i32,
        v1_21_7: 25i32,
        v1_21_9: 25i32,
        v1_21_11: 25i32,
    };
    pub const PLAY_DEBUG_BLOCK_VALUE: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 26i32,
        v1_21_11: 26i32,
    };
    pub const PLAY_DEBUG_CHUNK_VALUE: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 27i32,
        v1_21_11: 27i32,
    };
    pub const PLAY_DEBUG_ENTITY_VALUE: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 28i32,
        v1_21_11: 28i32,
    };
    pub const PLAY_DEBUG_EVENT: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 29i32,
        v1_21_11: 29i32,
    };
    pub const PLAY_DEBUG_SAMPLE: super::PacketId = super::PacketId {
        v1_21_2: 27i32,
        v1_21_4: 27i32,
        v1_21_5: 26i32,
        v1_21_6: 26i32,
        v1_21_7: 26i32,
        v1_21_9: 30i32,
        v1_21_11: 30i32,
    };
    pub const PLAY_DELETE_CHAT: super::PacketId = super::PacketId {
        v1_21_2: 28i32,
        v1_21_4: 28i32,
        v1_21_5: 27i32,
        v1_21_6: 27i32,
        v1_21_7: 27i32,
        v1_21_9: 31i32,
        v1_21_11: 31i32,
    };
    pub const PLAY_DISCONNECT: super::PacketId = super::PacketId {
        v1_21_2: 29i32,
        v1_21_4: 29i32,
        v1_21_5: 28i32,
        v1_21_6: 28i32,
        v1_21_7: 28i32,
        v1_21_9: 32i32,
        v1_21_11: 32i32,
    };
    pub const PLAY_DISGUISED_CHAT: super::PacketId = super::PacketId {
        v1_21_2: 30i32,
        v1_21_4: 30i32,
        v1_21_5: 29i32,
        v1_21_6: 29i32,
        v1_21_7: 29i32,
        v1_21_9: 33i32,
        v1_21_11: 33i32,
    };
    pub const PLAY_ENTITY_EVENT: super::PacketId = super::PacketId {
        v1_21_2: 31i32,
        v1_21_4: 31i32,
        v1_21_5: 30i32,
        v1_21_6: 30i32,
        v1_21_7: 30i32,
        v1_21_9: 34i32,
        v1_21_11: 34i32,
    };
    pub const PLAY_ENTITY_POSITION_SYNC: super::PacketId = super::PacketId {
        v1_21_2: 32i32,
        v1_21_4: 32i32,
        v1_21_5: 31i32,
        v1_21_6: 31i32,
        v1_21_7: 31i32,
        v1_21_9: 35i32,
        v1_21_11: 35i32,
    };
    pub const PLAY_EXPLODE: super::PacketId = super::PacketId {
        v1_21_2: 33i32,
        v1_21_4: 33i32,
        v1_21_5: 32i32,
        v1_21_6: 32i32,
        v1_21_7: 32i32,
        v1_21_9: 36i32,
        v1_21_11: 36i32,
    };
    pub const PLAY_FORGET_LEVEL_CHUNK: super::PacketId = super::PacketId {
        v1_21_2: 34i32,
        v1_21_4: 34i32,
        v1_21_5: 33i32,
        v1_21_6: 33i32,
        v1_21_7: 33i32,
        v1_21_9: 37i32,
        v1_21_11: 37i32,
    };
    pub const PLAY_GAME_EVENT: super::PacketId = super::PacketId {
        v1_21_2: 35i32,
        v1_21_4: 35i32,
        v1_21_5: 34i32,
        v1_21_6: 34i32,
        v1_21_7: 34i32,
        v1_21_9: 38i32,
        v1_21_11: 38i32,
    };
    pub const PLAY_GAME_TEST_HIGHLIGHT_POS: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: 39i32,
        v1_21_11: 39i32,
    };
    pub const PLAY_HURT_ANIMATION: super::PacketId = super::PacketId {
        v1_21_2: 37i32,
        v1_21_4: 37i32,
        v1_21_5: 36i32,
        v1_21_6: 36i32,
        v1_21_7: 36i32,
        v1_21_9: 41i32,
        v1_21_11: 41i32,
    };
    pub const PLAY_INITIALIZE_BORDER: super::PacketId = super::PacketId {
        v1_21_2: 38i32,
        v1_21_4: 38i32,
        v1_21_5: 37i32,
        v1_21_6: 37i32,
        v1_21_7: 37i32,
        v1_21_9: 42i32,
        v1_21_11: 42i32,
    };
    pub const PLAY_KEEP_ALIVE: super::PacketId = super::PacketId {
        v1_21_2: 39i32,
        v1_21_4: 39i32,
        v1_21_5: 38i32,
        v1_21_6: 38i32,
        v1_21_7: 38i32,
        v1_21_9: 43i32,
        v1_21_11: 43i32,
    };
    pub const PLAY_LEVEL_CHUNK_WITH_LIGHT: super::PacketId = super::PacketId {
        v1_21_2: 40i32,
        v1_21_4: 40i32,
        v1_21_5: 39i32,
        v1_21_6: 39i32,
        v1_21_7: 39i32,
        v1_21_9: 44i32,
        v1_21_11: 44i32,
    };
    pub const PLAY_LEVEL_EVENT: super::PacketId = super::PacketId {
        v1_21_2: 41i32,
        v1_21_4: 41i32,
        v1_21_5: 40i32,
        v1_21_6: 40i32,
        v1_21_7: 40i32,
        v1_21_9: 45i32,
        v1_21_11: 45i32,
    };
    pub const PLAY_LEVEL_PARTICLES: super::PacketId = super::PacketId {
        v1_21_2: 42i32,
        v1_21_4: 42i32,
        v1_21_5: 41i32,
        v1_21_6: 41i32,
        v1_21_7: 41i32,
        v1_21_9: 46i32,
        v1_21_11: 46i32,
    };
    pub const PLAY_LIGHT_UPDATE: super::PacketId = super::PacketId {
        v1_21_2: 43i32,
        v1_21_4: 43i32,
        v1_21_5: 42i32,
        v1_21_6: 42i32,
        v1_21_7: 42i32,
        v1_21_9: 47i32,
        v1_21_11: 47i32,
    };
    pub const PLAY_LOGIN: super::PacketId = super::PacketId {
        v1_21_2: 44i32,
        v1_21_4: 44i32,
        v1_21_5: 43i32,
        v1_21_6: 43i32,
        v1_21_7: 43i32,
        v1_21_9: 48i32,
        v1_21_11: 48i32,
    };
    pub const PLAY_MAP_ITEM_DATA: super::PacketId = super::PacketId {
        v1_21_2: 45i32,
        v1_21_4: 45i32,
        v1_21_5: 44i32,
        v1_21_6: 44i32,
        v1_21_7: 44i32,
        v1_21_9: 49i32,
        v1_21_11: 49i32,
    };
    pub const PLAY_MERCHANT_OFFERS: super::PacketId = super::PacketId {
        v1_21_2: 46i32,
        v1_21_4: 46i32,
        v1_21_5: 45i32,
        v1_21_6: 45i32,
        v1_21_7: 45i32,
        v1_21_9: 50i32,
        v1_21_11: 50i32,
    };
    pub const PLAY_MOUNT_SCREEN_OPEN: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: -1i32,
        v1_21_7: -1i32,
        v1_21_9: -1i32,
        v1_21_11: 40i32,
    };
    pub const PLAY_MOVE_ENTITY_POS: super::PacketId = super::PacketId {
        v1_21_2: 47i32,
        v1_21_4: 47i32,
        v1_21_5: 46i32,
        v1_21_6: 46i32,
        v1_21_7: 46i32,
        v1_21_9: 51i32,
        v1_21_11: 51i32,
    };
    pub const PLAY_MOVE_ENTITY_POS_ROT: super::PacketId = super::PacketId {
        v1_21_2: 48i32,
        v1_21_4: 48i32,
        v1_21_5: 47i32,
        v1_21_6: 47i32,
        v1_21_7: 47i32,
        v1_21_9: 52i32,
        v1_21_11: 52i32,
    };
    pub const PLAY_MOVE_ENTITY_ROT: super::PacketId = super::PacketId {
        v1_21_2: 50i32,
        v1_21_4: 50i32,
        v1_21_5: 49i32,
        v1_21_6: 49i32,
        v1_21_7: 49i32,
        v1_21_9: 54i32,
        v1_21_11: 54i32,
    };
    pub const PLAY_MOVE_MINECART_ALONG_TRACK: super::PacketId = super::PacketId {
        v1_21_2: 49i32,
        v1_21_4: 49i32,
        v1_21_5: 48i32,
        v1_21_6: 48i32,
        v1_21_7: 48i32,
        v1_21_9: 53i32,
        v1_21_11: 53i32,
    };
    pub const PLAY_MOVE_VEHICLE: super::PacketId = super::PacketId {
        v1_21_2: 51i32,
        v1_21_4: 51i32,
        v1_21_5: 50i32,
        v1_21_6: 50i32,
        v1_21_7: 50i32,
        v1_21_9: 55i32,
        v1_21_11: 55i32,
    };
    pub const PLAY_OPEN_BOOK: super::PacketId = super::PacketId {
        v1_21_2: 52i32,
        v1_21_4: 52i32,
        v1_21_5: 51i32,
        v1_21_6: 51i32,
        v1_21_7: 51i32,
        v1_21_9: 56i32,
        v1_21_11: 56i32,
    };
    pub const PLAY_OPEN_SCREEN: super::PacketId = super::PacketId {
        v1_21_2: 53i32,
        v1_21_4: 53i32,
        v1_21_5: 52i32,
        v1_21_6: 52i32,
        v1_21_7: 52i32,
        v1_21_9: 57i32,
        v1_21_11: 57i32,
    };
    pub const PLAY_OPEN_SIGN_EDITOR: super::PacketId = super::PacketId {
        v1_21_2: 54i32,
        v1_21_4: 54i32,
        v1_21_5: 53i32,
        v1_21_6: 53i32,
        v1_21_7: 53i32,
        v1_21_9: 58i32,
        v1_21_11: 58i32,
    };
    pub const PLAY_PING: super::PacketId = super::PacketId {
        v1_21_2: 55i32,
        v1_21_4: 55i32,
        v1_21_5: 54i32,
        v1_21_6: 54i32,
        v1_21_7: 54i32,
        v1_21_9: 59i32,
        v1_21_11: 59i32,
    };
    pub const PLAY_PLACE_GHOST_RECIPE: super::PacketId = super::PacketId {
        v1_21_2: 57i32,
        v1_21_4: 57i32,
        v1_21_5: 56i32,
        v1_21_6: 56i32,
        v1_21_7: 56i32,
        v1_21_9: 61i32,
        v1_21_11: 61i32,
    };
    pub const PLAY_PLAYER_ABILITIES: super::PacketId = super::PacketId {
        v1_21_2: 58i32,
        v1_21_4: 58i32,
        v1_21_5: 57i32,
        v1_21_6: 57i32,
        v1_21_7: 57i32,
        v1_21_9: 62i32,
        v1_21_11: 62i32,
    };
    pub const PLAY_PLAYER_CHAT: super::PacketId = super::PacketId {
        v1_21_2: 59i32,
        v1_21_4: 59i32,
        v1_21_5: 58i32,
        v1_21_6: 58i32,
        v1_21_7: 58i32,
        v1_21_9: 63i32,
        v1_21_11: 63i32,
    };
    pub const PLAY_PLAYER_COMBAT_END: super::PacketId = super::PacketId {
        v1_21_2: 60i32,
        v1_21_4: 60i32,
        v1_21_5: 59i32,
        v1_21_6: 59i32,
        v1_21_7: 59i32,
        v1_21_9: 64i32,
        v1_21_11: 64i32,
    };
    pub const PLAY_PLAYER_COMBAT_ENTER: super::PacketId = super::PacketId {
        v1_21_2: 61i32,
        v1_21_4: 61i32,
        v1_21_5: 60i32,
        v1_21_6: 60i32,
        v1_21_7: 60i32,
        v1_21_9: 65i32,
        v1_21_11: 65i32,
    };
    pub const PLAY_PLAYER_COMBAT_KILL: super::PacketId = super::PacketId {
        v1_21_2: 62i32,
        v1_21_4: 62i32,
        v1_21_5: 61i32,
        v1_21_6: 61i32,
        v1_21_7: 61i32,
        v1_21_9: 66i32,
        v1_21_11: 66i32,
    };
    pub const PLAY_PLAYER_INFO_REMOVE: super::PacketId = super::PacketId {
        v1_21_2: 63i32,
        v1_21_4: 63i32,
        v1_21_5: 62i32,
        v1_21_6: 62i32,
        v1_21_7: 62i32,
        v1_21_9: 67i32,
        v1_21_11: 67i32,
    };
    pub const PLAY_PLAYER_INFO_UPDATE: super::PacketId = super::PacketId {
        v1_21_2: 64i32,
        v1_21_4: 64i32,
        v1_21_5: 63i32,
        v1_21_6: 63i32,
        v1_21_7: 63i32,
        v1_21_9: 68i32,
        v1_21_11: 68i32,
    };
    pub const PLAY_PLAYER_LOOK_AT: super::PacketId = super::PacketId {
        v1_21_2: 65i32,
        v1_21_4: 65i32,
        v1_21_5: 64i32,
        v1_21_6: 64i32,
        v1_21_7: 64i32,
        v1_21_9: 69i32,
        v1_21_11: 69i32,
    };
    pub const PLAY_PLAYER_POSITION: super::PacketId = super::PacketId {
        v1_21_2: 66i32,
        v1_21_4: 66i32,
        v1_21_5: 65i32,
        v1_21_6: 65i32,
        v1_21_7: 65i32,
        v1_21_9: 70i32,
        v1_21_11: 70i32,
    };
    pub const PLAY_PLAYER_ROTATION: super::PacketId = super::PacketId {
        v1_21_2: 67i32,
        v1_21_4: 67i32,
        v1_21_5: 66i32,
        v1_21_6: 66i32,
        v1_21_7: 66i32,
        v1_21_9: 71i32,
        v1_21_11: 71i32,
    };
    pub const PLAY_PONG_RESPONSE: super::PacketId = super::PacketId {
        v1_21_2: 56i32,
        v1_21_4: 56i32,
        v1_21_5: 55i32,
        v1_21_6: 55i32,
        v1_21_7: 55i32,
        v1_21_9: 60i32,
        v1_21_11: 60i32,
    };
    pub const PLAY_PROJECTILE_POWER: super::PacketId = super::PacketId {
        v1_21_2: 128i32,
        v1_21_4: 128i32,
        v1_21_5: 128i32,
        v1_21_6: 128i32,
        v1_21_7: 128i32,
        v1_21_9: 133i32,
        v1_21_11: 133i32,
    };
    pub const PLAY_RECIPE_BOOK_ADD: super::PacketId = super::PacketId {
        v1_21_2: 68i32,
        v1_21_4: 68i32,
        v1_21_5: 67i32,
        v1_21_6: 67i32,
        v1_21_7: 67i32,
        v1_21_9: 72i32,
        v1_21_11: 72i32,
    };
    pub const PLAY_RECIPE_BOOK_REMOVE: super::PacketId = super::PacketId {
        v1_21_2: 69i32,
        v1_21_4: 69i32,
        v1_21_5: 68i32,
        v1_21_6: 68i32,
        v1_21_7: 68i32,
        v1_21_9: 73i32,
        v1_21_11: 73i32,
    };
    pub const PLAY_RECIPE_BOOK_SETTINGS: super::PacketId = super::PacketId {
        v1_21_2: 70i32,
        v1_21_4: 70i32,
        v1_21_5: 69i32,
        v1_21_6: 69i32,
        v1_21_7: 69i32,
        v1_21_9: 74i32,
        v1_21_11: 74i32,
    };
    pub const PLAY_REMOVE_ENTITIES: super::PacketId = super::PacketId {
        v1_21_2: 71i32,
        v1_21_4: 71i32,
        v1_21_5: 70i32,
        v1_21_6: 70i32,
        v1_21_7: 70i32,
        v1_21_9: 75i32,
        v1_21_11: 75i32,
    };
    pub const PLAY_REMOVE_MOB_EFFECT: super::PacketId = super::PacketId {
        v1_21_2: 72i32,
        v1_21_4: 72i32,
        v1_21_5: 71i32,
        v1_21_6: 71i32,
        v1_21_7: 71i32,
        v1_21_9: 76i32,
        v1_21_11: 76i32,
    };
    pub const PLAY_RESET_SCORE: super::PacketId = super::PacketId {
        v1_21_2: 73i32,
        v1_21_4: 73i32,
        v1_21_5: 72i32,
        v1_21_6: 72i32,
        v1_21_7: 72i32,
        v1_21_9: 77i32,
        v1_21_11: 77i32,
    };
    pub const PLAY_RESOURCE_PACK_POP: super::PacketId = super::PacketId {
        v1_21_2: 74i32,
        v1_21_4: 74i32,
        v1_21_5: 73i32,
        v1_21_6: 73i32,
        v1_21_7: 73i32,
        v1_21_9: 78i32,
        v1_21_11: 78i32,
    };
    pub const PLAY_RESOURCE_PACK_PUSH: super::PacketId = super::PacketId {
        v1_21_2: 75i32,
        v1_21_4: 75i32,
        v1_21_5: 74i32,
        v1_21_6: 74i32,
        v1_21_7: 74i32,
        v1_21_9: 79i32,
        v1_21_11: 79i32,
    };
    pub const PLAY_RESPAWN: super::PacketId = super::PacketId {
        v1_21_2: 76i32,
        v1_21_4: 76i32,
        v1_21_5: 75i32,
        v1_21_6: 75i32,
        v1_21_7: 75i32,
        v1_21_9: 80i32,
        v1_21_11: 80i32,
    };
    pub const PLAY_ROTATE_HEAD: super::PacketId = super::PacketId {
        v1_21_2: 77i32,
        v1_21_4: 77i32,
        v1_21_5: 76i32,
        v1_21_6: 76i32,
        v1_21_7: 76i32,
        v1_21_9: 81i32,
        v1_21_11: 81i32,
    };
    pub const PLAY_SECTION_BLOCKS_UPDATE: super::PacketId = super::PacketId {
        v1_21_2: 78i32,
        v1_21_4: 78i32,
        v1_21_5: 77i32,
        v1_21_6: 77i32,
        v1_21_7: 77i32,
        v1_21_9: 82i32,
        v1_21_11: 82i32,
    };
    pub const PLAY_SELECT_ADVANCEMENTS_TAB: super::PacketId = super::PacketId {
        v1_21_2: 79i32,
        v1_21_4: 79i32,
        v1_21_5: 78i32,
        v1_21_6: 78i32,
        v1_21_7: 78i32,
        v1_21_9: 83i32,
        v1_21_11: 83i32,
    };
    pub const PLAY_SERVER_DATA: super::PacketId = super::PacketId {
        v1_21_2: 80i32,
        v1_21_4: 80i32,
        v1_21_5: 79i32,
        v1_21_6: 79i32,
        v1_21_7: 79i32,
        v1_21_9: 84i32,
        v1_21_11: 84i32,
    };
    pub const PLAY_SERVER_LINKS: super::PacketId = super::PacketId {
        v1_21_2: 130i32,
        v1_21_4: 130i32,
        v1_21_5: 130i32,
        v1_21_6: 130i32,
        v1_21_7: 130i32,
        v1_21_9: 135i32,
        v1_21_11: 135i32,
    };
    pub const PLAY_SET_ACTION_BAR_TEXT: super::PacketId = super::PacketId {
        v1_21_2: 81i32,
        v1_21_4: 81i32,
        v1_21_5: 80i32,
        v1_21_6: 80i32,
        v1_21_7: 80i32,
        v1_21_9: 85i32,
        v1_21_11: 85i32,
    };
    pub const PLAY_SET_BORDER_CENTER: super::PacketId = super::PacketId {
        v1_21_2: 82i32,
        v1_21_4: 82i32,
        v1_21_5: 81i32,
        v1_21_6: 81i32,
        v1_21_7: 81i32,
        v1_21_9: 86i32,
        v1_21_11: 86i32,
    };
    pub const PLAY_SET_BORDER_LERP_SIZE: super::PacketId = super::PacketId {
        v1_21_2: 83i32,
        v1_21_4: 83i32,
        v1_21_5: 82i32,
        v1_21_6: 82i32,
        v1_21_7: 82i32,
        v1_21_9: 87i32,
        v1_21_11: 87i32,
    };
    pub const PLAY_SET_BORDER_SIZE: super::PacketId = super::PacketId {
        v1_21_2: 84i32,
        v1_21_4: 84i32,
        v1_21_5: 83i32,
        v1_21_6: 83i32,
        v1_21_7: 83i32,
        v1_21_9: 88i32,
        v1_21_11: 88i32,
    };
    pub const PLAY_SET_BORDER_WARNING_DELAY: super::PacketId = super::PacketId {
        v1_21_2: 85i32,
        v1_21_4: 85i32,
        v1_21_5: 84i32,
        v1_21_6: 84i32,
        v1_21_7: 84i32,
        v1_21_9: 89i32,
        v1_21_11: 89i32,
    };
    pub const PLAY_SET_BORDER_WARNING_DISTANCE: super::PacketId = super::PacketId {
        v1_21_2: 86i32,
        v1_21_4: 86i32,
        v1_21_5: 85i32,
        v1_21_6: 85i32,
        v1_21_7: 85i32,
        v1_21_9: 90i32,
        v1_21_11: 90i32,
    };
    pub const PLAY_SET_CAMERA: super::PacketId = super::PacketId {
        v1_21_2: 87i32,
        v1_21_4: 87i32,
        v1_21_5: 86i32,
        v1_21_6: 86i32,
        v1_21_7: 86i32,
        v1_21_9: 91i32,
        v1_21_11: 91i32,
    };
    pub const PLAY_SET_CHUNK_CACHE_CENTER: super::PacketId = super::PacketId {
        v1_21_2: 88i32,
        v1_21_4: 88i32,
        v1_21_5: 87i32,
        v1_21_6: 87i32,
        v1_21_7: 87i32,
        v1_21_9: 92i32,
        v1_21_11: 92i32,
    };
    pub const PLAY_SET_CHUNK_CACHE_RADIUS: super::PacketId = super::PacketId {
        v1_21_2: 89i32,
        v1_21_4: 89i32,
        v1_21_5: 88i32,
        v1_21_6: 88i32,
        v1_21_7: 88i32,
        v1_21_9: 93i32,
        v1_21_11: 93i32,
    };
    pub const PLAY_SET_CURSOR_ITEM: super::PacketId = super::PacketId {
        v1_21_2: 90i32,
        v1_21_4: 90i32,
        v1_21_5: 89i32,
        v1_21_6: 89i32,
        v1_21_7: 89i32,
        v1_21_9: 94i32,
        v1_21_11: 94i32,
    };
    pub const PLAY_SET_DEFAULT_SPAWN_POSITION: super::PacketId = super::PacketId {
        v1_21_2: 91i32,
        v1_21_4: 91i32,
        v1_21_5: 90i32,
        v1_21_6: 90i32,
        v1_21_7: 90i32,
        v1_21_9: 95i32,
        v1_21_11: 95i32,
    };
    pub const PLAY_SET_DISPLAY_OBJECTIVE: super::PacketId = super::PacketId {
        v1_21_2: 92i32,
        v1_21_4: 92i32,
        v1_21_5: 91i32,
        v1_21_6: 91i32,
        v1_21_7: 91i32,
        v1_21_9: 96i32,
        v1_21_11: 96i32,
    };
    pub const PLAY_SET_ENTITY_DATA: super::PacketId = super::PacketId {
        v1_21_2: 93i32,
        v1_21_4: 93i32,
        v1_21_5: 92i32,
        v1_21_6: 92i32,
        v1_21_7: 92i32,
        v1_21_9: 97i32,
        v1_21_11: 97i32,
    };
    pub const PLAY_SET_ENTITY_LINK: super::PacketId = super::PacketId {
        v1_21_2: 94i32,
        v1_21_4: 94i32,
        v1_21_5: 93i32,
        v1_21_6: 93i32,
        v1_21_7: 93i32,
        v1_21_9: 98i32,
        v1_21_11: 98i32,
    };
    pub const PLAY_SET_ENTITY_MOTION: super::PacketId = super::PacketId {
        v1_21_2: 95i32,
        v1_21_4: 95i32,
        v1_21_5: 94i32,
        v1_21_6: 94i32,
        v1_21_7: 94i32,
        v1_21_9: 99i32,
        v1_21_11: 99i32,
    };
    pub const PLAY_SET_EQUIPMENT: super::PacketId = super::PacketId {
        v1_21_2: 96i32,
        v1_21_4: 96i32,
        v1_21_5: 95i32,
        v1_21_6: 95i32,
        v1_21_7: 95i32,
        v1_21_9: 100i32,
        v1_21_11: 100i32,
    };
    pub const PLAY_SET_EXPERIENCE: super::PacketId = super::PacketId {
        v1_21_2: 97i32,
        v1_21_4: 97i32,
        v1_21_5: 96i32,
        v1_21_6: 96i32,
        v1_21_7: 96i32,
        v1_21_9: 101i32,
        v1_21_11: 101i32,
    };
    pub const PLAY_SET_HEALTH: super::PacketId = super::PacketId {
        v1_21_2: 98i32,
        v1_21_4: 98i32,
        v1_21_5: 97i32,
        v1_21_6: 97i32,
        v1_21_7: 97i32,
        v1_21_9: 102i32,
        v1_21_11: 102i32,
    };
    pub const PLAY_SET_HELD_SLOT: super::PacketId = super::PacketId {
        v1_21_2: 99i32,
        v1_21_4: 99i32,
        v1_21_5: 98i32,
        v1_21_6: 98i32,
        v1_21_7: 98i32,
        v1_21_9: 103i32,
        v1_21_11: 103i32,
    };
    pub const PLAY_SET_OBJECTIVE: super::PacketId = super::PacketId {
        v1_21_2: 100i32,
        v1_21_4: 100i32,
        v1_21_5: 99i32,
        v1_21_6: 99i32,
        v1_21_7: 99i32,
        v1_21_9: 104i32,
        v1_21_11: 104i32,
    };
    pub const PLAY_SET_PASSENGERS: super::PacketId = super::PacketId {
        v1_21_2: 101i32,
        v1_21_4: 101i32,
        v1_21_5: 100i32,
        v1_21_6: 100i32,
        v1_21_7: 100i32,
        v1_21_9: 105i32,
        v1_21_11: 105i32,
    };
    pub const PLAY_SET_PLAYER_INVENTORY: super::PacketId = super::PacketId {
        v1_21_2: 102i32,
        v1_21_4: 102i32,
        v1_21_5: 101i32,
        v1_21_6: 101i32,
        v1_21_7: 101i32,
        v1_21_9: 106i32,
        v1_21_11: 106i32,
    };
    pub const PLAY_SET_PLAYER_TEAM: super::PacketId = super::PacketId {
        v1_21_2: 103i32,
        v1_21_4: 103i32,
        v1_21_5: 102i32,
        v1_21_6: 102i32,
        v1_21_7: 102i32,
        v1_21_9: 107i32,
        v1_21_11: 107i32,
    };
    pub const PLAY_SET_SCORE: super::PacketId = super::PacketId {
        v1_21_2: 104i32,
        v1_21_4: 104i32,
        v1_21_5: 103i32,
        v1_21_6: 103i32,
        v1_21_7: 103i32,
        v1_21_9: 108i32,
        v1_21_11: 108i32,
    };
    pub const PLAY_SET_SIMULATION_DISTANCE: super::PacketId = super::PacketId {
        v1_21_2: 105i32,
        v1_21_4: 105i32,
        v1_21_5: 104i32,
        v1_21_6: 104i32,
        v1_21_7: 104i32,
        v1_21_9: 109i32,
        v1_21_11: 109i32,
    };
    pub const PLAY_SET_SUBTITLE_TEXT: super::PacketId = super::PacketId {
        v1_21_2: 106i32,
        v1_21_4: 106i32,
        v1_21_5: 105i32,
        v1_21_6: 105i32,
        v1_21_7: 105i32,
        v1_21_9: 110i32,
        v1_21_11: 110i32,
    };
    pub const PLAY_SET_TIME: super::PacketId = super::PacketId {
        v1_21_2: 107i32,
        v1_21_4: 107i32,
        v1_21_5: 106i32,
        v1_21_6: 106i32,
        v1_21_7: 106i32,
        v1_21_9: 111i32,
        v1_21_11: 111i32,
    };
    pub const PLAY_SET_TITLE_TEXT: super::PacketId = super::PacketId {
        v1_21_2: 108i32,
        v1_21_4: 108i32,
        v1_21_5: 107i32,
        v1_21_6: 107i32,
        v1_21_7: 107i32,
        v1_21_9: 112i32,
        v1_21_11: 112i32,
    };
    pub const PLAY_SET_TITLES_ANIMATION: super::PacketId = super::PacketId {
        v1_21_2: 109i32,
        v1_21_4: 109i32,
        v1_21_5: 108i32,
        v1_21_6: 108i32,
        v1_21_7: 108i32,
        v1_21_9: 113i32,
        v1_21_11: 113i32,
    };
    pub const PLAY_SHOW_DIALOG: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 133i32,
        v1_21_7: 133i32,
        v1_21_9: 138i32,
        v1_21_11: 138i32,
    };
    pub const PLAY_SOUND: super::PacketId = super::PacketId {
        v1_21_2: 111i32,
        v1_21_4: 111i32,
        v1_21_5: 110i32,
        v1_21_6: 110i32,
        v1_21_7: 110i32,
        v1_21_9: 115i32,
        v1_21_11: 115i32,
    };
    pub const PLAY_SOUND_ENTITY: super::PacketId = super::PacketId {
        v1_21_2: 110i32,
        v1_21_4: 110i32,
        v1_21_5: 109i32,
        v1_21_6: 109i32,
        v1_21_7: 109i32,
        v1_21_9: 114i32,
        v1_21_11: 114i32,
    };
    pub const PLAY_START_CONFIGURATION: super::PacketId = super::PacketId {
        v1_21_2: 112i32,
        v1_21_4: 112i32,
        v1_21_5: 111i32,
        v1_21_6: 111i32,
        v1_21_7: 111i32,
        v1_21_9: 116i32,
        v1_21_11: 116i32,
    };
    pub const PLAY_STOP_SOUND: super::PacketId = super::PacketId {
        v1_21_2: 113i32,
        v1_21_4: 113i32,
        v1_21_5: 112i32,
        v1_21_6: 112i32,
        v1_21_7: 112i32,
        v1_21_9: 117i32,
        v1_21_11: 117i32,
    };
    pub const PLAY_STORE_COOKIE: super::PacketId = super::PacketId {
        v1_21_2: 114i32,
        v1_21_4: 114i32,
        v1_21_5: 113i32,
        v1_21_6: 113i32,
        v1_21_7: 113i32,
        v1_21_9: 118i32,
        v1_21_11: 118i32,
    };
    pub const PLAY_SYSTEM_CHAT: super::PacketId = super::PacketId {
        v1_21_2: 115i32,
        v1_21_4: 115i32,
        v1_21_5: 114i32,
        v1_21_6: 114i32,
        v1_21_7: 114i32,
        v1_21_9: 119i32,
        v1_21_11: 119i32,
    };
    pub const PLAY_TAB_LIST: super::PacketId = super::PacketId {
        v1_21_2: 116i32,
        v1_21_4: 116i32,
        v1_21_5: 115i32,
        v1_21_6: 115i32,
        v1_21_7: 115i32,
        v1_21_9: 120i32,
        v1_21_11: 120i32,
    };
    pub const PLAY_TAG_QUERY: super::PacketId = super::PacketId {
        v1_21_2: 117i32,
        v1_21_4: 117i32,
        v1_21_5: 116i32,
        v1_21_6: 116i32,
        v1_21_7: 116i32,
        v1_21_9: 121i32,
        v1_21_11: 121i32,
    };
    pub const PLAY_TAKE_ITEM_ENTITY: super::PacketId = super::PacketId {
        v1_21_2: 118i32,
        v1_21_4: 118i32,
        v1_21_5: 117i32,
        v1_21_6: 117i32,
        v1_21_7: 117i32,
        v1_21_9: 122i32,
        v1_21_11: 122i32,
    };
    pub const PLAY_TELEPORT_ENTITY: super::PacketId = super::PacketId {
        v1_21_2: 119i32,
        v1_21_4: 119i32,
        v1_21_5: 118i32,
        v1_21_6: 118i32,
        v1_21_7: 118i32,
        v1_21_9: 123i32,
        v1_21_11: 123i32,
    };
    pub const PLAY_TEST_INSTANCE_BLOCK_STATUS: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: 119i32,
        v1_21_6: 119i32,
        v1_21_7: 119i32,
        v1_21_9: 124i32,
        v1_21_11: 124i32,
    };
    pub const PLAY_TICKING_STATE: super::PacketId = super::PacketId {
        v1_21_2: 120i32,
        v1_21_4: 120i32,
        v1_21_5: 120i32,
        v1_21_6: 120i32,
        v1_21_7: 120i32,
        v1_21_9: 125i32,
        v1_21_11: 125i32,
    };
    pub const PLAY_TICKING_STEP: super::PacketId = super::PacketId {
        v1_21_2: 121i32,
        v1_21_4: 121i32,
        v1_21_5: 121i32,
        v1_21_6: 121i32,
        v1_21_7: 121i32,
        v1_21_9: 126i32,
        v1_21_11: 126i32,
    };
    pub const PLAY_TRANSFER: super::PacketId = super::PacketId {
        v1_21_2: 122i32,
        v1_21_4: 122i32,
        v1_21_5: 122i32,
        v1_21_6: 122i32,
        v1_21_7: 122i32,
        v1_21_9: 127i32,
        v1_21_11: 127i32,
    };
    pub const PLAY_UPDATE_ADVANCEMENTS: super::PacketId = super::PacketId {
        v1_21_2: 123i32,
        v1_21_4: 123i32,
        v1_21_5: 123i32,
        v1_21_6: 123i32,
        v1_21_7: 123i32,
        v1_21_9: 128i32,
        v1_21_11: 128i32,
    };
    pub const PLAY_UPDATE_ATTRIBUTES: super::PacketId = super::PacketId {
        v1_21_2: 124i32,
        v1_21_4: 124i32,
        v1_21_5: 124i32,
        v1_21_6: 124i32,
        v1_21_7: 124i32,
        v1_21_9: 129i32,
        v1_21_11: 129i32,
    };
    pub const PLAY_UPDATE_MOB_EFFECT: super::PacketId = super::PacketId {
        v1_21_2: 125i32,
        v1_21_4: 125i32,
        v1_21_5: 125i32,
        v1_21_6: 125i32,
        v1_21_7: 125i32,
        v1_21_9: 130i32,
        v1_21_11: 130i32,
    };
    pub const PLAY_UPDATE_RECIPES: super::PacketId = super::PacketId {
        v1_21_2: 126i32,
        v1_21_4: 126i32,
        v1_21_5: 126i32,
        v1_21_6: 126i32,
        v1_21_7: 126i32,
        v1_21_9: 131i32,
        v1_21_11: 131i32,
    };
    pub const PLAY_UPDATE_TAGS: super::PacketId = super::PacketId {
        v1_21_2: 127i32,
        v1_21_4: 127i32,
        v1_21_5: 127i32,
        v1_21_6: 127i32,
        v1_21_7: 127i32,
        v1_21_9: 132i32,
        v1_21_11: 132i32,
    };
    pub const PLAY_WAYPOINT: super::PacketId = super::PacketId {
        v1_21_2: -1i32,
        v1_21_4: -1i32,
        v1_21_5: -1i32,
        v1_21_6: 131i32,
        v1_21_7: 131i32,
        v1_21_9: 136i32,
        v1_21_11: 136i32,
    };
    pub const STATUS_PONG_RESPONSE: super::PacketId = super::PacketId {
        v1_21_2: 1i32,
        v1_21_4: 1i32,
        v1_21_5: 1i32,
        v1_21_6: 1i32,
        v1_21_7: 1i32,
        v1_21_9: 1i32,
        v1_21_11: 1i32,
    };
    pub const STATUS_STATUS_RESPONSE: super::PacketId = super::PacketId {
        v1_21_2: 0i32,
        v1_21_4: 0i32,
        v1_21_5: 0i32,
        v1_21_6: 0i32,
        v1_21_7: 0i32,
        v1_21_9: 0i32,
        v1_21_11: 0i32,
    };
}
