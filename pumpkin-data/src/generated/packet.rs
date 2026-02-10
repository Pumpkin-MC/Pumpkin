/* This file is generated. Do not edit manually. */
use pumpkin_util::version::MinecraftVersion;
pub const CURRENT_MC_PROTOCOL: u32 = 774u32;
pub struct PacketId {
    pub latest_id: i32,
    pub v1_21_7_id: i32,
}
impl PacketId {
    #[doc = r" Converts the latest packet ID to the ID used in the requested version."]
    #[doc = r" Returns -1 if the packet does not exist in that version."]
    pub fn to_id(&self, version: MinecraftVersion) -> i32 {
        match version {
            MinecraftVersion::V_1_21_11 => self.latest_id,
            MinecraftVersion::V_1_21_7 => self.v1_21_7_id,
            _ => self.latest_id,
        }
    }
}
impl PartialEq<i32> for PacketId {
    fn eq(&self, other: &i32) -> bool {
        self.latest_id == *other
    }
}
impl PartialEq<PacketId> for i32 {
    fn eq(&self, other: &PacketId) -> bool {
        *self == other.latest_id
    }
}
pub mod serverbound {
    pub const CONFIG_ACCEPT_CODE_OF_CONDUCT: super::PacketId = super::PacketId {
        latest_id: 9i32,
        v1_21_7_id: -1i32,
    };
    pub const CONFIG_CLIENT_INFORMATION: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
    pub const CONFIG_COOKIE_RESPONSE: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const CONFIG_CUSTOM_CLICK_ACTION: super::PacketId = super::PacketId {
        latest_id: 8i32,
        v1_21_7_id: 8i32,
    };
    pub const CONFIG_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        latest_id: 2i32,
        v1_21_7_id: 2i32,
    };
    pub const CONFIG_FINISH_CONFIGURATION: super::PacketId = super::PacketId {
        latest_id: 3i32,
        v1_21_7_id: 3i32,
    };
    pub const CONFIG_KEEP_ALIVE: super::PacketId = super::PacketId {
        latest_id: 4i32,
        v1_21_7_id: 4i32,
    };
    pub const CONFIG_PONG: super::PacketId = super::PacketId {
        latest_id: 5i32,
        v1_21_7_id: 5i32,
    };
    pub const CONFIG_RESOURCE_PACK: super::PacketId = super::PacketId {
        latest_id: 6i32,
        v1_21_7_id: 6i32,
    };
    pub const CONFIG_SELECT_KNOWN_PACKS: super::PacketId = super::PacketId {
        latest_id: 7i32,
        v1_21_7_id: 7i32,
    };
    pub const HANDSHAKE_INTENTION: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
    pub const LOGIN_COOKIE_RESPONSE: super::PacketId = super::PacketId {
        latest_id: 4i32,
        v1_21_7_id: 4i32,
    };
    pub const LOGIN_CUSTOM_QUERY_ANSWER: super::PacketId = super::PacketId {
        latest_id: 2i32,
        v1_21_7_id: 2i32,
    };
    pub const LOGIN_HELLO: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
    pub const LOGIN_KEY: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const LOGIN_LOGIN_ACKNOWLEDGED: super::PacketId = super::PacketId {
        latest_id: 3i32,
        v1_21_7_id: 3i32,
    };
    pub const PLAY_ACCEPT_TELEPORTATION: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
    pub const PLAY_BLOCK_ENTITY_TAG_QUERY: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const PLAY_BUNDLE_ITEM_SELECTED: super::PacketId = super::PacketId {
        latest_id: 2i32,
        v1_21_7_id: 2i32,
    };
    pub const PLAY_CHANGE_DIFFICULTY: super::PacketId = super::PacketId {
        latest_id: 3i32,
        v1_21_7_id: 3i32,
    };
    pub const PLAY_CHANGE_GAME_MODE: super::PacketId = super::PacketId {
        latest_id: 4i32,
        v1_21_7_id: 4i32,
    };
    pub const PLAY_CHAT: super::PacketId = super::PacketId {
        latest_id: 8i32,
        v1_21_7_id: 8i32,
    };
    pub const PLAY_CHAT_ACK: super::PacketId = super::PacketId {
        latest_id: 5i32,
        v1_21_7_id: 5i32,
    };
    pub const PLAY_CHAT_COMMAND: super::PacketId = super::PacketId {
        latest_id: 6i32,
        v1_21_7_id: 6i32,
    };
    pub const PLAY_CHAT_COMMAND_SIGNED: super::PacketId = super::PacketId {
        latest_id: 7i32,
        v1_21_7_id: 7i32,
    };
    pub const PLAY_CHAT_SESSION_UPDATE: super::PacketId = super::PacketId {
        latest_id: 9i32,
        v1_21_7_id: 9i32,
    };
    pub const PLAY_CHUNK_BATCH_RECEIVED: super::PacketId = super::PacketId {
        latest_id: 10i32,
        v1_21_7_id: 10i32,
    };
    pub const PLAY_CLIENT_COMMAND: super::PacketId = super::PacketId {
        latest_id: 11i32,
        v1_21_7_id: 11i32,
    };
    pub const PLAY_CLIENT_INFORMATION: super::PacketId = super::PacketId {
        latest_id: 13i32,
        v1_21_7_id: 13i32,
    };
    pub const PLAY_CLIENT_TICK_END: super::PacketId = super::PacketId {
        latest_id: 12i32,
        v1_21_7_id: 12i32,
    };
    pub const PLAY_COMMAND_SUGGESTION: super::PacketId = super::PacketId {
        latest_id: 14i32,
        v1_21_7_id: 14i32,
    };
    pub const PLAY_CONFIGURATION_ACKNOWLEDGED: super::PacketId = super::PacketId {
        latest_id: 15i32,
        v1_21_7_id: 15i32,
    };
    pub const PLAY_CONTAINER_BUTTON_CLICK: super::PacketId = super::PacketId {
        latest_id: 16i32,
        v1_21_7_id: 16i32,
    };
    pub const PLAY_CONTAINER_CLICK: super::PacketId = super::PacketId {
        latest_id: 17i32,
        v1_21_7_id: 17i32,
    };
    pub const PLAY_CONTAINER_CLOSE: super::PacketId = super::PacketId {
        latest_id: 18i32,
        v1_21_7_id: 18i32,
    };
    pub const PLAY_CONTAINER_SLOT_STATE_CHANGED: super::PacketId = super::PacketId {
        latest_id: 19i32,
        v1_21_7_id: 19i32,
    };
    pub const PLAY_COOKIE_RESPONSE: super::PacketId = super::PacketId {
        latest_id: 20i32,
        v1_21_7_id: 20i32,
    };
    pub const PLAY_CUSTOM_CLICK_ACTION: super::PacketId = super::PacketId {
        latest_id: 65i32,
        v1_21_7_id: 65i32,
    };
    pub const PLAY_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        latest_id: 21i32,
        v1_21_7_id: 21i32,
    };
    pub const PLAY_DEBUG_SUBSCRIPTION_REQUEST: super::PacketId = super::PacketId {
        latest_id: 22i32,
        v1_21_7_id: -1i32,
    };
    pub const PLAY_EDIT_BOOK: super::PacketId = super::PacketId {
        latest_id: 23i32,
        v1_21_7_id: 23i32,
    };
    pub const PLAY_ENTITY_TAG_QUERY: super::PacketId = super::PacketId {
        latest_id: 24i32,
        v1_21_7_id: 24i32,
    };
    pub const PLAY_INTERACT: super::PacketId = super::PacketId {
        latest_id: 25i32,
        v1_21_7_id: 25i32,
    };
    pub const PLAY_JIGSAW_GENERATE: super::PacketId = super::PacketId {
        latest_id: 26i32,
        v1_21_7_id: 26i32,
    };
    pub const PLAY_KEEP_ALIVE: super::PacketId = super::PacketId {
        latest_id: 27i32,
        v1_21_7_id: 27i32,
    };
    pub const PLAY_LOCK_DIFFICULTY: super::PacketId = super::PacketId {
        latest_id: 28i32,
        v1_21_7_id: 28i32,
    };
    pub const PLAY_MOVE_PLAYER_POS: super::PacketId = super::PacketId {
        latest_id: 29i32,
        v1_21_7_id: 29i32,
    };
    pub const PLAY_MOVE_PLAYER_POS_ROT: super::PacketId = super::PacketId {
        latest_id: 30i32,
        v1_21_7_id: 30i32,
    };
    pub const PLAY_MOVE_PLAYER_ROT: super::PacketId = super::PacketId {
        latest_id: 31i32,
        v1_21_7_id: 31i32,
    };
    pub const PLAY_MOVE_PLAYER_STATUS_ONLY: super::PacketId = super::PacketId {
        latest_id: 32i32,
        v1_21_7_id: 32i32,
    };
    pub const PLAY_MOVE_VEHICLE: super::PacketId = super::PacketId {
        latest_id: 33i32,
        v1_21_7_id: 33i32,
    };
    pub const PLAY_PADDLE_BOAT: super::PacketId = super::PacketId {
        latest_id: 34i32,
        v1_21_7_id: 34i32,
    };
    pub const PLAY_PICK_ITEM_FROM_BLOCK: super::PacketId = super::PacketId {
        latest_id: 35i32,
        v1_21_7_id: 35i32,
    };
    pub const PLAY_PICK_ITEM_FROM_ENTITY: super::PacketId = super::PacketId {
        latest_id: 36i32,
        v1_21_7_id: 36i32,
    };
    pub const PLAY_PING_REQUEST: super::PacketId = super::PacketId {
        latest_id: 37i32,
        v1_21_7_id: 37i32,
    };
    pub const PLAY_PLACE_RECIPE: super::PacketId = super::PacketId {
        latest_id: 38i32,
        v1_21_7_id: 38i32,
    };
    pub const PLAY_PLAYER_ABILITIES: super::PacketId = super::PacketId {
        latest_id: 39i32,
        v1_21_7_id: 39i32,
    };
    pub const PLAY_PLAYER_ACTION: super::PacketId = super::PacketId {
        latest_id: 40i32,
        v1_21_7_id: 40i32,
    };
    pub const PLAY_PLAYER_COMMAND: super::PacketId = super::PacketId {
        latest_id: 41i32,
        v1_21_7_id: 41i32,
    };
    pub const PLAY_PLAYER_INPUT: super::PacketId = super::PacketId {
        latest_id: 42i32,
        v1_21_7_id: 42i32,
    };
    pub const PLAY_PLAYER_LOADED: super::PacketId = super::PacketId {
        latest_id: 43i32,
        v1_21_7_id: 43i32,
    };
    pub const PLAY_PONG: super::PacketId = super::PacketId {
        latest_id: 44i32,
        v1_21_7_id: 44i32,
    };
    pub const PLAY_RECIPE_BOOK_CHANGE_SETTINGS: super::PacketId = super::PacketId {
        latest_id: 45i32,
        v1_21_7_id: 45i32,
    };
    pub const PLAY_RECIPE_BOOK_SEEN_RECIPE: super::PacketId = super::PacketId {
        latest_id: 46i32,
        v1_21_7_id: 46i32,
    };
    pub const PLAY_RENAME_ITEM: super::PacketId = super::PacketId {
        latest_id: 47i32,
        v1_21_7_id: 47i32,
    };
    pub const PLAY_RESOURCE_PACK: super::PacketId = super::PacketId {
        latest_id: 48i32,
        v1_21_7_id: 48i32,
    };
    pub const PLAY_SEEN_ADVANCEMENTS: super::PacketId = super::PacketId {
        latest_id: 49i32,
        v1_21_7_id: 49i32,
    };
    pub const PLAY_SELECT_TRADE: super::PacketId = super::PacketId {
        latest_id: 50i32,
        v1_21_7_id: 50i32,
    };
    pub const PLAY_SET_BEACON: super::PacketId = super::PacketId {
        latest_id: 51i32,
        v1_21_7_id: 51i32,
    };
    pub const PLAY_SET_CARRIED_ITEM: super::PacketId = super::PacketId {
        latest_id: 52i32,
        v1_21_7_id: 52i32,
    };
    pub const PLAY_SET_COMMAND_BLOCK: super::PacketId = super::PacketId {
        latest_id: 53i32,
        v1_21_7_id: 53i32,
    };
    pub const PLAY_SET_COMMAND_MINECART: super::PacketId = super::PacketId {
        latest_id: 54i32,
        v1_21_7_id: 54i32,
    };
    pub const PLAY_SET_CREATIVE_MODE_SLOT: super::PacketId = super::PacketId {
        latest_id: 55i32,
        v1_21_7_id: 55i32,
    };
    pub const PLAY_SET_JIGSAW_BLOCK: super::PacketId = super::PacketId {
        latest_id: 56i32,
        v1_21_7_id: 56i32,
    };
    pub const PLAY_SET_STRUCTURE_BLOCK: super::PacketId = super::PacketId {
        latest_id: 57i32,
        v1_21_7_id: 57i32,
    };
    pub const PLAY_SET_TEST_BLOCK: super::PacketId = super::PacketId {
        latest_id: 58i32,
        v1_21_7_id: 58i32,
    };
    pub const PLAY_SIGN_UPDATE: super::PacketId = super::PacketId {
        latest_id: 59i32,
        v1_21_7_id: 59i32,
    };
    pub const PLAY_SWING: super::PacketId = super::PacketId {
        latest_id: 60i32,
        v1_21_7_id: 60i32,
    };
    pub const PLAY_TELEPORT_TO_ENTITY: super::PacketId = super::PacketId {
        latest_id: 61i32,
        v1_21_7_id: 61i32,
    };
    pub const PLAY_TEST_INSTANCE_BLOCK_ACTION: super::PacketId = super::PacketId {
        latest_id: 62i32,
        v1_21_7_id: 62i32,
    };
    pub const PLAY_USE_ITEM: super::PacketId = super::PacketId {
        latest_id: 64i32,
        v1_21_7_id: 64i32,
    };
    pub const PLAY_USE_ITEM_ON: super::PacketId = super::PacketId {
        latest_id: 63i32,
        v1_21_7_id: 63i32,
    };
    pub const STATUS_PING_REQUEST: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const STATUS_STATUS_REQUEST: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
}
pub mod clientbound {
    pub const CONFIG_CLEAR_DIALOG: super::PacketId = super::PacketId {
        latest_id: 17i32,
        v1_21_7_id: 17i32,
    };
    pub const CONFIG_CODE_OF_CONDUCT: super::PacketId = super::PacketId {
        latest_id: 19i32,
        v1_21_7_id: -1i32,
    };
    pub const CONFIG_COOKIE_REQUEST: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
    pub const CONFIG_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const CONFIG_CUSTOM_REPORT_DETAILS: super::PacketId = super::PacketId {
        latest_id: 15i32,
        v1_21_7_id: 15i32,
    };
    pub const CONFIG_DISCONNECT: super::PacketId = super::PacketId {
        latest_id: 2i32,
        v1_21_7_id: 2i32,
    };
    pub const CONFIG_FINISH_CONFIGURATION: super::PacketId = super::PacketId {
        latest_id: 3i32,
        v1_21_7_id: 3i32,
    };
    pub const CONFIG_KEEP_ALIVE: super::PacketId = super::PacketId {
        latest_id: 4i32,
        v1_21_7_id: 4i32,
    };
    pub const CONFIG_PING: super::PacketId = super::PacketId {
        latest_id: 5i32,
        v1_21_7_id: 5i32,
    };
    pub const CONFIG_REGISTRY_DATA: super::PacketId = super::PacketId {
        latest_id: 7i32,
        v1_21_7_id: 7i32,
    };
    pub const CONFIG_RESET_CHAT: super::PacketId = super::PacketId {
        latest_id: 6i32,
        v1_21_7_id: 6i32,
    };
    pub const CONFIG_RESOURCE_PACK_POP: super::PacketId = super::PacketId {
        latest_id: 8i32,
        v1_21_7_id: 8i32,
    };
    pub const CONFIG_RESOURCE_PACK_PUSH: super::PacketId = super::PacketId {
        latest_id: 9i32,
        v1_21_7_id: 9i32,
    };
    pub const CONFIG_SELECT_KNOWN_PACKS: super::PacketId = super::PacketId {
        latest_id: 14i32,
        v1_21_7_id: 14i32,
    };
    pub const CONFIG_SERVER_LINKS: super::PacketId = super::PacketId {
        latest_id: 16i32,
        v1_21_7_id: 16i32,
    };
    pub const CONFIG_SHOW_DIALOG: super::PacketId = super::PacketId {
        latest_id: 18i32,
        v1_21_7_id: 18i32,
    };
    pub const CONFIG_STORE_COOKIE: super::PacketId = super::PacketId {
        latest_id: 10i32,
        v1_21_7_id: 10i32,
    };
    pub const CONFIG_TRANSFER: super::PacketId = super::PacketId {
        latest_id: 11i32,
        v1_21_7_id: 11i32,
    };
    pub const CONFIG_UPDATE_ENABLED_FEATURES: super::PacketId = super::PacketId {
        latest_id: 12i32,
        v1_21_7_id: 12i32,
    };
    pub const CONFIG_UPDATE_TAGS: super::PacketId = super::PacketId {
        latest_id: 13i32,
        v1_21_7_id: 13i32,
    };
    pub const LOGIN_COOKIE_REQUEST: super::PacketId = super::PacketId {
        latest_id: 5i32,
        v1_21_7_id: 5i32,
    };
    pub const LOGIN_CUSTOM_QUERY: super::PacketId = super::PacketId {
        latest_id: 4i32,
        v1_21_7_id: 4i32,
    };
    pub const LOGIN_HELLO: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const LOGIN_LOGIN_COMPRESSION: super::PacketId = super::PacketId {
        latest_id: 3i32,
        v1_21_7_id: 3i32,
    };
    pub const LOGIN_LOGIN_DISCONNECT: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
    pub const LOGIN_LOGIN_FINISHED: super::PacketId = super::PacketId {
        latest_id: 2i32,
        v1_21_7_id: 2i32,
    };
    pub const PLAY_ADD_ENTITY: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const PLAY_ANIMATE: super::PacketId = super::PacketId {
        latest_id: 2i32,
        v1_21_7_id: 2i32,
    };
    pub const PLAY_AWARD_STATS: super::PacketId = super::PacketId {
        latest_id: 3i32,
        v1_21_7_id: 3i32,
    };
    pub const PLAY_BLOCK_CHANGED_ACK: super::PacketId = super::PacketId {
        latest_id: 4i32,
        v1_21_7_id: 4i32,
    };
    pub const PLAY_BLOCK_DESTRUCTION: super::PacketId = super::PacketId {
        latest_id: 5i32,
        v1_21_7_id: 5i32,
    };
    pub const PLAY_BLOCK_ENTITY_DATA: super::PacketId = super::PacketId {
        latest_id: 6i32,
        v1_21_7_id: 6i32,
    };
    pub const PLAY_BLOCK_EVENT: super::PacketId = super::PacketId {
        latest_id: 7i32,
        v1_21_7_id: 7i32,
    };
    pub const PLAY_BLOCK_UPDATE: super::PacketId = super::PacketId {
        latest_id: 8i32,
        v1_21_7_id: 8i32,
    };
    pub const PLAY_BOSS_EVENT: super::PacketId = super::PacketId {
        latest_id: 9i32,
        v1_21_7_id: 9i32,
    };
    pub const PLAY_BUNDLE_DELIMITER: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
    pub const PLAY_CHANGE_DIFFICULTY: super::PacketId = super::PacketId {
        latest_id: 10i32,
        v1_21_7_id: 10i32,
    };
    pub const PLAY_CHUNK_BATCH_FINISHED: super::PacketId = super::PacketId {
        latest_id: 11i32,
        v1_21_7_id: 11i32,
    };
    pub const PLAY_CHUNK_BATCH_START: super::PacketId = super::PacketId {
        latest_id: 12i32,
        v1_21_7_id: 12i32,
    };
    pub const PLAY_CHUNKS_BIOMES: super::PacketId = super::PacketId {
        latest_id: 13i32,
        v1_21_7_id: 13i32,
    };
    pub const PLAY_CLEAR_DIALOG: super::PacketId = super::PacketId {
        latest_id: 137i32,
        v1_21_7_id: 132i32,
    };
    pub const PLAY_CLEAR_TITLES: super::PacketId = super::PacketId {
        latest_id: 14i32,
        v1_21_7_id: 14i32,
    };
    pub const PLAY_COMMAND_SUGGESTIONS: super::PacketId = super::PacketId {
        latest_id: 15i32,
        v1_21_7_id: 15i32,
    };
    pub const PLAY_COMMANDS: super::PacketId = super::PacketId {
        latest_id: 16i32,
        v1_21_7_id: 16i32,
    };
    pub const PLAY_CONTAINER_CLOSE: super::PacketId = super::PacketId {
        latest_id: 17i32,
        v1_21_7_id: 17i32,
    };
    pub const PLAY_CONTAINER_SET_CONTENT: super::PacketId = super::PacketId {
        latest_id: 18i32,
        v1_21_7_id: 18i32,
    };
    pub const PLAY_CONTAINER_SET_DATA: super::PacketId = super::PacketId {
        latest_id: 19i32,
        v1_21_7_id: 19i32,
    };
    pub const PLAY_CONTAINER_SET_SLOT: super::PacketId = super::PacketId {
        latest_id: 20i32,
        v1_21_7_id: 20i32,
    };
    pub const PLAY_COOKIE_REQUEST: super::PacketId = super::PacketId {
        latest_id: 21i32,
        v1_21_7_id: 21i32,
    };
    pub const PLAY_COOLDOWN: super::PacketId = super::PacketId {
        latest_id: 22i32,
        v1_21_7_id: 22i32,
    };
    pub const PLAY_CUSTOM_CHAT_COMPLETIONS: super::PacketId = super::PacketId {
        latest_id: 23i32,
        v1_21_7_id: 23i32,
    };
    pub const PLAY_CUSTOM_PAYLOAD: super::PacketId = super::PacketId {
        latest_id: 24i32,
        v1_21_7_id: 24i32,
    };
    pub const PLAY_CUSTOM_REPORT_DETAILS: super::PacketId = super::PacketId {
        latest_id: 134i32,
        v1_21_7_id: 129i32,
    };
    pub const PLAY_DAMAGE_EVENT: super::PacketId = super::PacketId {
        latest_id: 25i32,
        v1_21_7_id: 25i32,
    };
    pub const PLAY_DEBUG_BLOCK_VALUE: super::PacketId = super::PacketId {
        latest_id: 26i32,
        v1_21_7_id: -1i32,
    };
    pub const PLAY_DEBUG_CHUNK_VALUE: super::PacketId = super::PacketId {
        latest_id: 27i32,
        v1_21_7_id: -1i32,
    };
    pub const PLAY_DEBUG_ENTITY_VALUE: super::PacketId = super::PacketId {
        latest_id: 28i32,
        v1_21_7_id: -1i32,
    };
    pub const PLAY_DEBUG_EVENT: super::PacketId = super::PacketId {
        latest_id: 29i32,
        v1_21_7_id: -1i32,
    };
    pub const PLAY_DEBUG_SAMPLE: super::PacketId = super::PacketId {
        latest_id: 30i32,
        v1_21_7_id: 26i32,
    };
    pub const PLAY_DELETE_CHAT: super::PacketId = super::PacketId {
        latest_id: 31i32,
        v1_21_7_id: 27i32,
    };
    pub const PLAY_DISCONNECT: super::PacketId = super::PacketId {
        latest_id: 32i32,
        v1_21_7_id: 28i32,
    };
    pub const PLAY_DISGUISED_CHAT: super::PacketId = super::PacketId {
        latest_id: 33i32,
        v1_21_7_id: 29i32,
    };
    pub const PLAY_ENTITY_EVENT: super::PacketId = super::PacketId {
        latest_id: 34i32,
        v1_21_7_id: 30i32,
    };
    pub const PLAY_ENTITY_POSITION_SYNC: super::PacketId = super::PacketId {
        latest_id: 35i32,
        v1_21_7_id: 31i32,
    };
    pub const PLAY_EXPLODE: super::PacketId = super::PacketId {
        latest_id: 36i32,
        v1_21_7_id: 32i32,
    };
    pub const PLAY_FORGET_LEVEL_CHUNK: super::PacketId = super::PacketId {
        latest_id: 37i32,
        v1_21_7_id: 33i32,
    };
    pub const PLAY_GAME_EVENT: super::PacketId = super::PacketId {
        latest_id: 38i32,
        v1_21_7_id: 34i32,
    };
    pub const PLAY_GAME_TEST_HIGHLIGHT_POS: super::PacketId = super::PacketId {
        latest_id: 39i32,
        v1_21_7_id: -1i32,
    };
    pub const PLAY_HURT_ANIMATION: super::PacketId = super::PacketId {
        latest_id: 41i32,
        v1_21_7_id: 36i32,
    };
    pub const PLAY_INITIALIZE_BORDER: super::PacketId = super::PacketId {
        latest_id: 42i32,
        v1_21_7_id: 37i32,
    };
    pub const PLAY_KEEP_ALIVE: super::PacketId = super::PacketId {
        latest_id: 43i32,
        v1_21_7_id: 38i32,
    };
    pub const PLAY_LEVEL_CHUNK_WITH_LIGHT: super::PacketId = super::PacketId {
        latest_id: 44i32,
        v1_21_7_id: 39i32,
    };
    pub const PLAY_LEVEL_EVENT: super::PacketId = super::PacketId {
        latest_id: 45i32,
        v1_21_7_id: 40i32,
    };
    pub const PLAY_LEVEL_PARTICLES: super::PacketId = super::PacketId {
        latest_id: 46i32,
        v1_21_7_id: 41i32,
    };
    pub const PLAY_LIGHT_UPDATE: super::PacketId = super::PacketId {
        latest_id: 47i32,
        v1_21_7_id: 42i32,
    };
    pub const PLAY_LOGIN: super::PacketId = super::PacketId {
        latest_id: 48i32,
        v1_21_7_id: 43i32,
    };
    pub const PLAY_MAP_ITEM_DATA: super::PacketId = super::PacketId {
        latest_id: 49i32,
        v1_21_7_id: 44i32,
    };
    pub const PLAY_MERCHANT_OFFERS: super::PacketId = super::PacketId {
        latest_id: 50i32,
        v1_21_7_id: 45i32,
    };
    pub const PLAY_MOUNT_SCREEN_OPEN: super::PacketId = super::PacketId {
        latest_id: 40i32,
        v1_21_7_id: -1i32,
    };
    pub const PLAY_MOVE_ENTITY_POS: super::PacketId = super::PacketId {
        latest_id: 51i32,
        v1_21_7_id: 46i32,
    };
    pub const PLAY_MOVE_ENTITY_POS_ROT: super::PacketId = super::PacketId {
        latest_id: 52i32,
        v1_21_7_id: 47i32,
    };
    pub const PLAY_MOVE_ENTITY_ROT: super::PacketId = super::PacketId {
        latest_id: 54i32,
        v1_21_7_id: 49i32,
    };
    pub const PLAY_MOVE_MINECART_ALONG_TRACK: super::PacketId = super::PacketId {
        latest_id: 53i32,
        v1_21_7_id: 48i32,
    };
    pub const PLAY_MOVE_VEHICLE: super::PacketId = super::PacketId {
        latest_id: 55i32,
        v1_21_7_id: 50i32,
    };
    pub const PLAY_OPEN_BOOK: super::PacketId = super::PacketId {
        latest_id: 56i32,
        v1_21_7_id: 51i32,
    };
    pub const PLAY_OPEN_SCREEN: super::PacketId = super::PacketId {
        latest_id: 57i32,
        v1_21_7_id: 52i32,
    };
    pub const PLAY_OPEN_SIGN_EDITOR: super::PacketId = super::PacketId {
        latest_id: 58i32,
        v1_21_7_id: 53i32,
    };
    pub const PLAY_PING: super::PacketId = super::PacketId {
        latest_id: 59i32,
        v1_21_7_id: 54i32,
    };
    pub const PLAY_PLACE_GHOST_RECIPE: super::PacketId = super::PacketId {
        latest_id: 61i32,
        v1_21_7_id: 56i32,
    };
    pub const PLAY_PLAYER_ABILITIES: super::PacketId = super::PacketId {
        latest_id: 62i32,
        v1_21_7_id: 57i32,
    };
    pub const PLAY_PLAYER_CHAT: super::PacketId = super::PacketId {
        latest_id: 63i32,
        v1_21_7_id: 58i32,
    };
    pub const PLAY_PLAYER_COMBAT_END: super::PacketId = super::PacketId {
        latest_id: 64i32,
        v1_21_7_id: 59i32,
    };
    pub const PLAY_PLAYER_COMBAT_ENTER: super::PacketId = super::PacketId {
        latest_id: 65i32,
        v1_21_7_id: 60i32,
    };
    pub const PLAY_PLAYER_COMBAT_KILL: super::PacketId = super::PacketId {
        latest_id: 66i32,
        v1_21_7_id: 61i32,
    };
    pub const PLAY_PLAYER_INFO_REMOVE: super::PacketId = super::PacketId {
        latest_id: 67i32,
        v1_21_7_id: 62i32,
    };
    pub const PLAY_PLAYER_INFO_UPDATE: super::PacketId = super::PacketId {
        latest_id: 68i32,
        v1_21_7_id: 63i32,
    };
    pub const PLAY_PLAYER_LOOK_AT: super::PacketId = super::PacketId {
        latest_id: 69i32,
        v1_21_7_id: 64i32,
    };
    pub const PLAY_PLAYER_POSITION: super::PacketId = super::PacketId {
        latest_id: 70i32,
        v1_21_7_id: 65i32,
    };
    pub const PLAY_PLAYER_ROTATION: super::PacketId = super::PacketId {
        latest_id: 71i32,
        v1_21_7_id: 66i32,
    };
    pub const PLAY_PONG_RESPONSE: super::PacketId = super::PacketId {
        latest_id: 60i32,
        v1_21_7_id: 55i32,
    };
    pub const PLAY_PROJECTILE_POWER: super::PacketId = super::PacketId {
        latest_id: 133i32,
        v1_21_7_id: 128i32,
    };
    pub const PLAY_RECIPE_BOOK_ADD: super::PacketId = super::PacketId {
        latest_id: 72i32,
        v1_21_7_id: 67i32,
    };
    pub const PLAY_RECIPE_BOOK_REMOVE: super::PacketId = super::PacketId {
        latest_id: 73i32,
        v1_21_7_id: 68i32,
    };
    pub const PLAY_RECIPE_BOOK_SETTINGS: super::PacketId = super::PacketId {
        latest_id: 74i32,
        v1_21_7_id: 69i32,
    };
    pub const PLAY_REMOVE_ENTITIES: super::PacketId = super::PacketId {
        latest_id: 75i32,
        v1_21_7_id: 70i32,
    };
    pub const PLAY_REMOVE_MOB_EFFECT: super::PacketId = super::PacketId {
        latest_id: 76i32,
        v1_21_7_id: 71i32,
    };
    pub const PLAY_RESET_SCORE: super::PacketId = super::PacketId {
        latest_id: 77i32,
        v1_21_7_id: 72i32,
    };
    pub const PLAY_RESOURCE_PACK_POP: super::PacketId = super::PacketId {
        latest_id: 78i32,
        v1_21_7_id: 73i32,
    };
    pub const PLAY_RESOURCE_PACK_PUSH: super::PacketId = super::PacketId {
        latest_id: 79i32,
        v1_21_7_id: 74i32,
    };
    pub const PLAY_RESPAWN: super::PacketId = super::PacketId {
        latest_id: 80i32,
        v1_21_7_id: 75i32,
    };
    pub const PLAY_ROTATE_HEAD: super::PacketId = super::PacketId {
        latest_id: 81i32,
        v1_21_7_id: 76i32,
    };
    pub const PLAY_SECTION_BLOCKS_UPDATE: super::PacketId = super::PacketId {
        latest_id: 82i32,
        v1_21_7_id: 77i32,
    };
    pub const PLAY_SELECT_ADVANCEMENTS_TAB: super::PacketId = super::PacketId {
        latest_id: 83i32,
        v1_21_7_id: 78i32,
    };
    pub const PLAY_SERVER_DATA: super::PacketId = super::PacketId {
        latest_id: 84i32,
        v1_21_7_id: 79i32,
    };
    pub const PLAY_SERVER_LINKS: super::PacketId = super::PacketId {
        latest_id: 135i32,
        v1_21_7_id: 130i32,
    };
    pub const PLAY_SET_ACTION_BAR_TEXT: super::PacketId = super::PacketId {
        latest_id: 85i32,
        v1_21_7_id: 80i32,
    };
    pub const PLAY_SET_BORDER_CENTER: super::PacketId = super::PacketId {
        latest_id: 86i32,
        v1_21_7_id: 81i32,
    };
    pub const PLAY_SET_BORDER_LERP_SIZE: super::PacketId = super::PacketId {
        latest_id: 87i32,
        v1_21_7_id: 82i32,
    };
    pub const PLAY_SET_BORDER_SIZE: super::PacketId = super::PacketId {
        latest_id: 88i32,
        v1_21_7_id: 83i32,
    };
    pub const PLAY_SET_BORDER_WARNING_DELAY: super::PacketId = super::PacketId {
        latest_id: 89i32,
        v1_21_7_id: 84i32,
    };
    pub const PLAY_SET_BORDER_WARNING_DISTANCE: super::PacketId = super::PacketId {
        latest_id: 90i32,
        v1_21_7_id: 85i32,
    };
    pub const PLAY_SET_CAMERA: super::PacketId = super::PacketId {
        latest_id: 91i32,
        v1_21_7_id: 86i32,
    };
    pub const PLAY_SET_CHUNK_CACHE_CENTER: super::PacketId = super::PacketId {
        latest_id: 92i32,
        v1_21_7_id: 87i32,
    };
    pub const PLAY_SET_CHUNK_CACHE_RADIUS: super::PacketId = super::PacketId {
        latest_id: 93i32,
        v1_21_7_id: 88i32,
    };
    pub const PLAY_SET_CURSOR_ITEM: super::PacketId = super::PacketId {
        latest_id: 94i32,
        v1_21_7_id: 89i32,
    };
    pub const PLAY_SET_DEFAULT_SPAWN_POSITION: super::PacketId = super::PacketId {
        latest_id: 95i32,
        v1_21_7_id: 90i32,
    };
    pub const PLAY_SET_DISPLAY_OBJECTIVE: super::PacketId = super::PacketId {
        latest_id: 96i32,
        v1_21_7_id: 91i32,
    };
    pub const PLAY_SET_ENTITY_DATA: super::PacketId = super::PacketId {
        latest_id: 97i32,
        v1_21_7_id: 92i32,
    };
    pub const PLAY_SET_ENTITY_LINK: super::PacketId = super::PacketId {
        latest_id: 98i32,
        v1_21_7_id: 93i32,
    };
    pub const PLAY_SET_ENTITY_MOTION: super::PacketId = super::PacketId {
        latest_id: 99i32,
        v1_21_7_id: 94i32,
    };
    pub const PLAY_SET_EQUIPMENT: super::PacketId = super::PacketId {
        latest_id: 100i32,
        v1_21_7_id: 95i32,
    };
    pub const PLAY_SET_EXPERIENCE: super::PacketId = super::PacketId {
        latest_id: 101i32,
        v1_21_7_id: 96i32,
    };
    pub const PLAY_SET_HEALTH: super::PacketId = super::PacketId {
        latest_id: 102i32,
        v1_21_7_id: 97i32,
    };
    pub const PLAY_SET_HELD_SLOT: super::PacketId = super::PacketId {
        latest_id: 103i32,
        v1_21_7_id: 98i32,
    };
    pub const PLAY_SET_OBJECTIVE: super::PacketId = super::PacketId {
        latest_id: 104i32,
        v1_21_7_id: 99i32,
    };
    pub const PLAY_SET_PASSENGERS: super::PacketId = super::PacketId {
        latest_id: 105i32,
        v1_21_7_id: 100i32,
    };
    pub const PLAY_SET_PLAYER_INVENTORY: super::PacketId = super::PacketId {
        latest_id: 106i32,
        v1_21_7_id: 101i32,
    };
    pub const PLAY_SET_PLAYER_TEAM: super::PacketId = super::PacketId {
        latest_id: 107i32,
        v1_21_7_id: 102i32,
    };
    pub const PLAY_SET_SCORE: super::PacketId = super::PacketId {
        latest_id: 108i32,
        v1_21_7_id: 103i32,
    };
    pub const PLAY_SET_SIMULATION_DISTANCE: super::PacketId = super::PacketId {
        latest_id: 109i32,
        v1_21_7_id: 104i32,
    };
    pub const PLAY_SET_SUBTITLE_TEXT: super::PacketId = super::PacketId {
        latest_id: 110i32,
        v1_21_7_id: 105i32,
    };
    pub const PLAY_SET_TIME: super::PacketId = super::PacketId {
        latest_id: 111i32,
        v1_21_7_id: 106i32,
    };
    pub const PLAY_SET_TITLE_TEXT: super::PacketId = super::PacketId {
        latest_id: 112i32,
        v1_21_7_id: 107i32,
    };
    pub const PLAY_SET_TITLES_ANIMATION: super::PacketId = super::PacketId {
        latest_id: 113i32,
        v1_21_7_id: 108i32,
    };
    pub const PLAY_SHOW_DIALOG: super::PacketId = super::PacketId {
        latest_id: 138i32,
        v1_21_7_id: 133i32,
    };
    pub const PLAY_SOUND: super::PacketId = super::PacketId {
        latest_id: 115i32,
        v1_21_7_id: 110i32,
    };
    pub const PLAY_SOUND_ENTITY: super::PacketId = super::PacketId {
        latest_id: 114i32,
        v1_21_7_id: 109i32,
    };
    pub const PLAY_START_CONFIGURATION: super::PacketId = super::PacketId {
        latest_id: 116i32,
        v1_21_7_id: 111i32,
    };
    pub const PLAY_STOP_SOUND: super::PacketId = super::PacketId {
        latest_id: 117i32,
        v1_21_7_id: 112i32,
    };
    pub const PLAY_STORE_COOKIE: super::PacketId = super::PacketId {
        latest_id: 118i32,
        v1_21_7_id: 113i32,
    };
    pub const PLAY_SYSTEM_CHAT: super::PacketId = super::PacketId {
        latest_id: 119i32,
        v1_21_7_id: 114i32,
    };
    pub const PLAY_TAB_LIST: super::PacketId = super::PacketId {
        latest_id: 120i32,
        v1_21_7_id: 115i32,
    };
    pub const PLAY_TAG_QUERY: super::PacketId = super::PacketId {
        latest_id: 121i32,
        v1_21_7_id: 116i32,
    };
    pub const PLAY_TAKE_ITEM_ENTITY: super::PacketId = super::PacketId {
        latest_id: 122i32,
        v1_21_7_id: 117i32,
    };
    pub const PLAY_TELEPORT_ENTITY: super::PacketId = super::PacketId {
        latest_id: 123i32,
        v1_21_7_id: 118i32,
    };
    pub const PLAY_TEST_INSTANCE_BLOCK_STATUS: super::PacketId = super::PacketId {
        latest_id: 124i32,
        v1_21_7_id: 119i32,
    };
    pub const PLAY_TICKING_STATE: super::PacketId = super::PacketId {
        latest_id: 125i32,
        v1_21_7_id: 120i32,
    };
    pub const PLAY_TICKING_STEP: super::PacketId = super::PacketId {
        latest_id: 126i32,
        v1_21_7_id: 121i32,
    };
    pub const PLAY_TRANSFER: super::PacketId = super::PacketId {
        latest_id: 127i32,
        v1_21_7_id: 122i32,
    };
    pub const PLAY_UPDATE_ADVANCEMENTS: super::PacketId = super::PacketId {
        latest_id: 128i32,
        v1_21_7_id: 123i32,
    };
    pub const PLAY_UPDATE_ATTRIBUTES: super::PacketId = super::PacketId {
        latest_id: 129i32,
        v1_21_7_id: 124i32,
    };
    pub const PLAY_UPDATE_MOB_EFFECT: super::PacketId = super::PacketId {
        latest_id: 130i32,
        v1_21_7_id: 125i32,
    };
    pub const PLAY_UPDATE_RECIPES: super::PacketId = super::PacketId {
        latest_id: 131i32,
        v1_21_7_id: 126i32,
    };
    pub const PLAY_UPDATE_TAGS: super::PacketId = super::PacketId {
        latest_id: 132i32,
        v1_21_7_id: 127i32,
    };
    pub const PLAY_WAYPOINT: super::PacketId = super::PacketId {
        latest_id: 136i32,
        v1_21_7_id: 131i32,
    };
    pub const STATUS_PONG_RESPONSE: super::PacketId = super::PacketId {
        latest_id: 1i32,
        v1_21_7_id: 1i32,
    };
    pub const STATUS_STATUS_RESPONSE: super::PacketId = super::PacketId {
        latest_id: 0i32,
        v1_21_7_id: 0i32,
    };
}
