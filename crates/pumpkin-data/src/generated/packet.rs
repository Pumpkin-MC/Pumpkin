/* This file is generated. Do not edit manually. */
use pumpkin_util::version::JavaMinecraftVersion;
pub const CURRENT_MC_VERSION: JavaMinecraftVersion =
    pumpkin_util::version::JavaMinecraftVersion::V_26_3;
pub const LOWEST_SUPPORTED_MC_VERSION: JavaMinecraftVersion =
    pumpkin_util::version::JavaMinecraftVersion::V_26_3;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketId(pub i32);
impl PacketId {
    #[doc = r" Converts the requested protocol version into the corresponding packet ID."]
    #[must_use]
    pub const fn to_id(&self, _version: JavaMinecraftVersion) -> i32 {
        self.0
    }
}
impl PartialEq<i32> for PacketId {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}
impl PartialEq<PacketId> for i32 {
    fn eq(&self, other: &PacketId) -> bool {
        *self == other.0
    }
}
impl From<PacketId> for i32 {
    fn from(id: PacketId) -> Self {
        id.0
    }
}
impl From<i32> for PacketId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}
pub mod serverbound {
    pub mod handshake {
        pub const INTENTION: super::super::PacketId = super::super::PacketId(0i32);
        pub const HANDSHAKE: super::super::PacketId = INTENTION;
        pub const HANDSHAKING: super::super::PacketId = INTENTION;
    }
    pub mod status {
        pub const PING_REQUEST: super::super::PacketId = super::super::PacketId(1i32);
        pub const STATUS_REQUEST: super::super::PacketId = super::super::PacketId(0i32);
    }
    pub mod login {
        pub const COOKIE_RESPONSE: super::super::PacketId = super::super::PacketId(4i32);
        pub const CUSTOM_QUERY_ANSWER: super::super::PacketId = super::super::PacketId(2i32);
        pub const HELLO: super::super::PacketId = super::super::PacketId(0i32);
        pub const KEY: super::super::PacketId = super::super::PacketId(1i32);
        pub const LOGIN_ACKNOWLEDGED: super::super::PacketId = super::super::PacketId(3i32);
        pub const LOGIN_START: super::super::PacketId = HELLO;
        pub const ENCRYPTION_RESPONSE: super::super::PacketId = KEY;
        pub const LOGIN_PLUGIN_RESPONSE: super::super::PacketId = CUSTOM_QUERY_ANSWER;
    }
    pub mod config {
        pub const ACCEPT_CODE_OF_CONDUCT: super::super::PacketId = super::super::PacketId(9i32);
        pub const CLIENT_INFORMATION: super::super::PacketId = super::super::PacketId(0i32);
        pub const COOKIE_RESPONSE: super::super::PacketId = super::super::PacketId(1i32);
        pub const CUSTOM_CLICK_ACTION: super::super::PacketId = super::super::PacketId(8i32);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId(2i32);
        pub const FINISH_CONFIGURATION: super::super::PacketId = super::super::PacketId(3i32);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId(4i32);
        pub const PONG: super::super::PacketId = super::super::PacketId(5i32);
        pub const RESOURCE_PACK: super::super::PacketId = super::super::PacketId(6i32);
        pub const SELECT_KNOWN_PACKS: super::super::PacketId = super::super::PacketId(7i32);
    }
    pub mod play {
        pub const ACCEPT_TELEPORTATION: super::super::PacketId = super::super::PacketId(0i32);
        pub const ATTACK: super::super::PacketId = super::super::PacketId(1i32);
        pub const BLOCK_ENTITY_TAG_QUERY: super::super::PacketId = super::super::PacketId(2i32);
        pub const BUNDLE_ITEM_SELECTED: super::super::PacketId = super::super::PacketId(3i32);
        pub const CHANGE_DIFFICULTY: super::super::PacketId = super::super::PacketId(4i32);
        pub const CHANGE_GAME_MODE: super::super::PacketId = super::super::PacketId(5i32);
        pub const CHAT: super::super::PacketId = super::super::PacketId(9i32);
        pub const CHAT_ACK: super::super::PacketId = super::super::PacketId(6i32);
        pub const CHAT_COMMAND: super::super::PacketId = super::super::PacketId(7i32);
        pub const CHAT_COMMAND_SIGNED: super::super::PacketId = super::super::PacketId(8i32);
        pub const CHAT_SESSION_UPDATE: super::super::PacketId = super::super::PacketId(10i32);
        pub const CHUNK_BATCH_RECEIVED: super::super::PacketId = super::super::PacketId(11i32);
        pub const CLIENT_COMMAND: super::super::PacketId = super::super::PacketId(12i32);
        pub const CLIENT_INFORMATION: super::super::PacketId = super::super::PacketId(14i32);
        pub const CLIENT_TICK_END: super::super::PacketId = super::super::PacketId(13i32);
        pub const COMMAND_SUGGESTION: super::super::PacketId = super::super::PacketId(15i32);
        pub const CONFIGURATION_ACKNOWLEDGED: super::super::PacketId =
            super::super::PacketId(16i32);
        pub const CONTAINER_BUTTON_CLICK: super::super::PacketId = super::super::PacketId(17i32);
        pub const CONTAINER_CLICK: super::super::PacketId = super::super::PacketId(18i32);
        pub const CONTAINER_CLOSE: super::super::PacketId = super::super::PacketId(19i32);
        pub const CONTAINER_SLOT_STATE_CHANGED: super::super::PacketId =
            super::super::PacketId(20i32);
        pub const COOKIE_RESPONSE: super::super::PacketId = super::super::PacketId(21i32);
        pub const CUSTOM_CLICK_ACTION: super::super::PacketId = super::super::PacketId(68i32);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId(22i32);
        pub const DEBUG_SUBSCRIPTION_REQUEST: super::super::PacketId =
            super::super::PacketId(23i32);
        pub const EDIT_BOOK: super::super::PacketId = super::super::PacketId(24i32);
        pub const ENTITY_TAG_QUERY: super::super::PacketId = super::super::PacketId(25i32);
        pub const INTERACT: super::super::PacketId = super::super::PacketId(26i32);
        pub const JIGSAW_GENERATE: super::super::PacketId = super::super::PacketId(27i32);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId(28i32);
        pub const LOCK_DIFFICULTY: super::super::PacketId = super::super::PacketId(29i32);
        pub const MOVE_PLAYER_POS: super::super::PacketId = super::super::PacketId(30i32);
        pub const MOVE_PLAYER_POS_ROT: super::super::PacketId = super::super::PacketId(31i32);
        pub const MOVE_PLAYER_ROT: super::super::PacketId = super::super::PacketId(32i32);
        pub const MOVE_PLAYER_STATUS_ONLY: super::super::PacketId = super::super::PacketId(33i32);
        pub const MOVE_VEHICLE: super::super::PacketId = super::super::PacketId(34i32);
        pub const PADDLE_BOAT: super::super::PacketId = super::super::PacketId(35i32);
        pub const PICK_ITEM_FROM_BLOCK: super::super::PacketId = super::super::PacketId(36i32);
        pub const PICK_ITEM_FROM_ENTITY: super::super::PacketId = super::super::PacketId(37i32);
        pub const PING_REQUEST: super::super::PacketId = super::super::PacketId(38i32);
        pub const PLACE_RECIPE: super::super::PacketId = super::super::PacketId(39i32);
        pub const PLAYER_ABILITIES: super::super::PacketId = super::super::PacketId(40i32);
        pub const PLAYER_ACTION: super::super::PacketId = super::super::PacketId(41i32);
        pub const PLAYER_COMMAND: super::super::PacketId = super::super::PacketId(42i32);
        pub const PLAYER_INPUT: super::super::PacketId = super::super::PacketId(43i32);
        pub const PLAYER_LOADED: super::super::PacketId = super::super::PacketId(44i32);
        pub const PONG: super::super::PacketId = super::super::PacketId(45i32);
        pub const PUNCH: super::super::PacketId = super::super::PacketId(46i32);
        pub const RECIPE_BOOK_CHANGE_SETTINGS: super::super::PacketId =
            super::super::PacketId(47i32);
        pub const RECIPE_BOOK_SEEN_RECIPE: super::super::PacketId = super::super::PacketId(48i32);
        pub const RENAME_ITEM: super::super::PacketId = super::super::PacketId(49i32);
        pub const RESOURCE_PACK: super::super::PacketId = super::super::PacketId(50i32);
        pub const SEEN_ADVANCEMENTS: super::super::PacketId = super::super::PacketId(51i32);
        pub const SELECT_TRADE: super::super::PacketId = super::super::PacketId(52i32);
        pub const SET_BEACON: super::super::PacketId = super::super::PacketId(53i32);
        pub const SET_CARRIED_ITEM: super::super::PacketId = super::super::PacketId(54i32);
        pub const SET_COMMAND_BLOCK: super::super::PacketId = super::super::PacketId(55i32);
        pub const SET_COMMAND_MINECART: super::super::PacketId = super::super::PacketId(56i32);
        pub const SET_CREATIVE_MODE_SLOT: super::super::PacketId = super::super::PacketId(57i32);
        pub const SET_GAME_RULE: super::super::PacketId = super::super::PacketId(58i32);
        pub const SET_JIGSAW_BLOCK: super::super::PacketId = super::super::PacketId(59i32);
        pub const SET_STRUCTURE_BLOCK: super::super::PacketId = super::super::PacketId(60i32);
        pub const SET_TEST_BLOCK: super::super::PacketId = super::super::PacketId(61i32);
        pub const SIGN_UPDATE: super::super::PacketId = super::super::PacketId(62i32);
        pub const SPECTATOR_ACTION: super::super::PacketId = super::super::PacketId(63i32);
        pub const TELEPORT_TO_ENTITY: super::super::PacketId = super::super::PacketId(64i32);
        pub const TEST_INSTANCE_BLOCK_ACTION: super::super::PacketId =
            super::super::PacketId(65i32);
        pub const USE_ITEM: super::super::PacketId = super::super::PacketId(67i32);
        pub const USE_ITEM_ON: super::super::PacketId = super::super::PacketId(66i32);
        pub const CHAT_MESSAGE: super::super::PacketId = CHAT;
        pub const TELEPORT_CONFIRM: super::super::PacketId = ACCEPT_TELEPORTATION;
        pub const SELECT_BUNDLE_ITEM: super::super::PacketId = BUNDLE_ITEM_SELECTED;
        pub const SET_DIFFICULTY: super::super::PacketId = CHANGE_DIFFICULTY;
        pub const CHUNK_BATCH_ACK: super::super::PacketId = CHUNK_BATCH_RECEIVED;
        pub const CLICK_CONTAINER_BUTTON: super::super::PacketId = CONTAINER_BUTTON_CLICK;
        pub const CLICK_CONTAINER: super::super::PacketId = CONTAINER_CLICK;
        pub const SLOT_STATE_CHANGE: super::super::PacketId = CONTAINER_SLOT_STATE_CHANGED;
        pub const INTERACT_ENTITY: super::super::PacketId = INTERACT;
        pub const GENERATE_STRUCTURE: super::super::PacketId = JIGSAW_GENERATE;
        pub const PLAYER_POSITION: super::super::PacketId = MOVE_PLAYER_POS;
        pub const PLAYER_POSITION_ROTATION: super::super::PacketId = MOVE_PLAYER_POS_ROT;
        pub const PLAYER_POSITION_AND_ROTATION: super::super::PacketId = MOVE_PLAYER_POS_ROT;
        pub const PLAYER_ROTATION: super::super::PacketId = MOVE_PLAYER_ROT;
        pub const PLAYER_FLYING: super::super::PacketId = MOVE_PLAYER_STATUS_ONLY;
        pub const STEER_BOAT: super::super::PacketId = PADDLE_BOAT;
        pub const PLAYER_DIGGING: super::super::PacketId = PLAYER_ACTION;
        pub const ENTITY_ACTION: super::super::PacketId = PLAYER_COMMAND;
        pub const SWING_ARM: super::super::PacketId = PUNCH;
        pub const ANIMATION: super::super::PacketId = PUNCH;
        pub const SWING: super::super::PacketId = PUNCH;
        pub const DEBUG_SAMPLE_SUBSCRIPTION: super::super::PacketId = DEBUG_SUBSCRIPTION_REQUEST;
        pub const PLAYER_BLOCK_PLACEMENT: super::super::PacketId = USE_ITEM_ON;
        pub const SPECTATE: super::super::PacketId = SPECTATOR_ACTION;
        pub const SPECTATE_ENTITY: super::super::PacketId = SPECTATOR_ACTION;
    }
}
pub mod clientbound {
    pub mod handshake {}
    pub mod status {
        pub const PONG_RESPONSE: super::super::PacketId = super::super::PacketId(1i32);
        pub const STATUS_RESPONSE: super::super::PacketId = super::super::PacketId(0i32);
    }
    pub mod login {
        pub const COOKIE_REQUEST: super::super::PacketId = super::super::PacketId(5i32);
        pub const CUSTOM_QUERY: super::super::PacketId = super::super::PacketId(4i32);
        pub const HELLO: super::super::PacketId = super::super::PacketId(1i32);
        pub const LOGIN_COMPRESSION: super::super::PacketId = super::super::PacketId(3i32);
        pub const LOGIN_DISCONNECT: super::super::PacketId = super::super::PacketId(0i32);
        pub const LOGIN_FINISHED: super::super::PacketId = super::super::PacketId(2i32);
        pub const LOGIN_SUCCESS: super::super::PacketId = LOGIN_FINISHED;
        pub const GAME_PROFILE: super::super::PacketId = LOGIN_FINISHED;
        pub const SET_COMPRESSION: super::super::PacketId = LOGIN_COMPRESSION;
        pub const ENCRYPTION_REQUEST: super::super::PacketId = HELLO;
        pub const LOGIN_PLUGIN_REQUEST: super::super::PacketId = CUSTOM_QUERY;
    }
    pub mod config {
        pub const CLEAR_DIALOG: super::super::PacketId = super::super::PacketId(18i32);
        pub const CODE_OF_CONDUCT: super::super::PacketId = super::super::PacketId(20i32);
        pub const COOKIE_REQUEST: super::super::PacketId = super::super::PacketId(0i32);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId(1i32);
        pub const CUSTOM_REPORT_DETAILS: super::super::PacketId = super::super::PacketId(16i32);
        pub const DISCONNECT: super::super::PacketId = super::super::PacketId(2i32);
        pub const FINISH_CONFIGURATION: super::super::PacketId = super::super::PacketId(3i32);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId(4i32);
        pub const PING: super::super::PacketId = super::super::PacketId(5i32);
        pub const POST_EFFECTS: super::super::PacketId = super::super::PacketId(10i32);
        pub const REGISTRY_DATA: super::super::PacketId = super::super::PacketId(7i32);
        pub const RESET_CHAT: super::super::PacketId = super::super::PacketId(6i32);
        pub const RESOURCE_PACK_POP: super::super::PacketId = super::super::PacketId(8i32);
        pub const RESOURCE_PACK_PUSH: super::super::PacketId = super::super::PacketId(9i32);
        pub const SELECT_KNOWN_PACKS: super::super::PacketId = super::super::PacketId(15i32);
        pub const SERVER_LINKS: super::super::PacketId = super::super::PacketId(17i32);
        pub const SHOW_DIALOG: super::super::PacketId = super::super::PacketId(19i32);
        pub const STORE_COOKIE: super::super::PacketId = super::super::PacketId(11i32);
        pub const TRANSFER: super::super::PacketId = super::super::PacketId(12i32);
        pub const UPDATE_ENABLED_FEATURES: super::super::PacketId = super::super::PacketId(13i32);
        pub const UPDATE_TAGS: super::super::PacketId = super::super::PacketId(14i32);
    }
    pub mod play {
        pub const ADD_ENTITY: super::super::PacketId = super::super::PacketId(1i32);
        pub const ADD_TRANSIENT_BLOCK: super::super::PacketId = super::super::PacketId(37i32);
        pub const ANIMATE: super::super::PacketId = super::super::PacketId(2i32);
        pub const AWARD_STATS: super::super::PacketId = super::super::PacketId(3i32);
        pub const BLOCK_CHANGED_ACK: super::super::PacketId = super::super::PacketId(4i32);
        pub const BLOCK_DESTRUCTION: super::super::PacketId = super::super::PacketId(5i32);
        pub const BLOCK_ENTITY_DATA: super::super::PacketId = super::super::PacketId(6i32);
        pub const BLOCK_EVENT: super::super::PacketId = super::super::PacketId(7i32);
        pub const BLOCK_UPDATE: super::super::PacketId = super::super::PacketId(8i32);
        pub const BOSS_EVENT: super::super::PacketId = super::super::PacketId(9i32);
        pub const BUNDLE_DELIMITER: super::super::PacketId = super::super::PacketId(0i32);
        pub const CHANGE_DIFFICULTY: super::super::PacketId = super::super::PacketId(10i32);
        pub const CHUNKS_BIOMES: super::super::PacketId = super::super::PacketId(13i32);
        pub const CHUNK_BATCH_FINISHED: super::super::PacketId = super::super::PacketId(11i32);
        pub const CHUNK_BATCH_START: super::super::PacketId = super::super::PacketId(12i32);
        pub const CLEAR_DIALOG: super::super::PacketId = super::super::PacketId(142i32);
        pub const CLEAR_TITLES: super::super::PacketId = super::super::PacketId(14i32);
        pub const COMMANDS: super::super::PacketId = super::super::PacketId(16i32);
        pub const COMMAND_SUGGESTIONS: super::super::PacketId = super::super::PacketId(15i32);
        pub const CONTAINER_CLOSE: super::super::PacketId = super::super::PacketId(17i32);
        pub const CONTAINER_SET_CONTENT: super::super::PacketId = super::super::PacketId(18i32);
        pub const CONTAINER_SET_DATA: super::super::PacketId = super::super::PacketId(19i32);
        pub const CONTAINER_SET_SLOT: super::super::PacketId = super::super::PacketId(20i32);
        pub const COOKIE_REQUEST: super::super::PacketId = super::super::PacketId(21i32);
        pub const COOLDOWN: super::super::PacketId = super::super::PacketId(22i32);
        pub const CUSTOM_CHAT_COMPLETIONS: super::super::PacketId = super::super::PacketId(23i32);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId(24i32);
        pub const CUSTOM_REPORT_DETAILS: super::super::PacketId = super::super::PacketId(139i32);
        pub const DAMAGE_EVENT: super::super::PacketId = super::super::PacketId(25i32);
        pub const DEBUG_BLOCK_VALUE: super::super::PacketId = super::super::PacketId(26i32);
        pub const DEBUG_CHUNK_VALUE: super::super::PacketId = super::super::PacketId(27i32);
        pub const DEBUG_ENTITY_VALUE: super::super::PacketId = super::super::PacketId(28i32);
        pub const DEBUG_EVENT: super::super::PacketId = super::super::PacketId(29i32);
        pub const DEBUG_SAMPLE: super::super::PacketId = super::super::PacketId(30i32);
        pub const DELETE_CHAT: super::super::PacketId = super::super::PacketId(31i32);
        pub const DISCONNECT: super::super::PacketId = super::super::PacketId(32i32);
        pub const DISGUISED_CHAT: super::super::PacketId = super::super::PacketId(33i32);
        pub const ENTITY_EVENT: super::super::PacketId = super::super::PacketId(34i32);
        pub const ENTITY_POSITION_SYNC: super::super::PacketId = super::super::PacketId(35i32);
        pub const EXPLODE: super::super::PacketId = super::super::PacketId(36i32);
        pub const FORGET_LEVEL_CHUNK: super::super::PacketId = super::super::PacketId(38i32);
        pub const GAME_EVENT: super::super::PacketId = super::super::PacketId(39i32);
        pub const GAME_RULE_VALUES: super::super::PacketId = super::super::PacketId(40i32);
        pub const GAME_TEST_HIGHLIGHT_POS: super::super::PacketId = super::super::PacketId(41i32);
        pub const HURT_ANIMATION: super::super::PacketId = super::super::PacketId(43i32);
        pub const INITIALIZE_BORDER: super::super::PacketId = super::super::PacketId(44i32);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId(45i32);
        pub const LEVEL_CHUNK_WITH_LIGHT: super::super::PacketId = super::super::PacketId(46i32);
        pub const LEVEL_EVENT: super::super::PacketId = super::super::PacketId(47i32);
        pub const LEVEL_PARTICLES: super::super::PacketId = super::super::PacketId(48i32);
        pub const LIGHT_UPDATE: super::super::PacketId = super::super::PacketId(49i32);
        pub const LOGIN: super::super::PacketId = super::super::PacketId(50i32);
        pub const LOW_DISK_SPACE_WARNING: super::super::PacketId = super::super::PacketId(51i32);
        pub const MAP_ITEM_DATA: super::super::PacketId = super::super::PacketId(52i32);
        pub const MERCHANT_OFFERS: super::super::PacketId = super::super::PacketId(53i32);
        pub const MOUNT_SCREEN_OPEN: super::super::PacketId = super::super::PacketId(42i32);
        pub const MOVE_ENTITY_POS: super::super::PacketId = super::super::PacketId(54i32);
        pub const MOVE_ENTITY_POS_ROT: super::super::PacketId = super::super::PacketId(55i32);
        pub const MOVE_ENTITY_ROT: super::super::PacketId = super::super::PacketId(57i32);
        pub const MOVE_MINECART_ALONG_TRACK: super::super::PacketId = super::super::PacketId(56i32);
        pub const MOVE_VEHICLE: super::super::PacketId = super::super::PacketId(58i32);
        pub const OPEN_BOOK: super::super::PacketId = super::super::PacketId(59i32);
        pub const OPEN_SCREEN: super::super::PacketId = super::super::PacketId(60i32);
        pub const OPEN_SIGN_EDITOR: super::super::PacketId = super::super::PacketId(61i32);
        pub const PING: super::super::PacketId = super::super::PacketId(62i32);
        pub const PLACE_GHOST_RECIPE: super::super::PacketId = super::super::PacketId(64i32);
        pub const PLAYER_ABILITIES: super::super::PacketId = super::super::PacketId(65i32);
        pub const PLAYER_CHAT: super::super::PacketId = super::super::PacketId(66i32);
        pub const PLAYER_COMBAT_END: super::super::PacketId = super::super::PacketId(67i32);
        pub const PLAYER_COMBAT_ENTER: super::super::PacketId = super::super::PacketId(68i32);
        pub const PLAYER_COMBAT_KILL: super::super::PacketId = super::super::PacketId(69i32);
        pub const PLAYER_INFO_REMOVE: super::super::PacketId = super::super::PacketId(70i32);
        pub const PLAYER_INFO_UPDATE: super::super::PacketId = super::super::PacketId(71i32);
        pub const PLAYER_LOOK_AT: super::super::PacketId = super::super::PacketId(72i32);
        pub const PLAYER_POSITION: super::super::PacketId = super::super::PacketId(73i32);
        pub const PLAYER_ROTATION: super::super::PacketId = super::super::PacketId(74i32);
        pub const PONG_RESPONSE: super::super::PacketId = super::super::PacketId(63i32);
        pub const POST_EFFECTS: super::super::PacketId = super::super::PacketId(83i32);
        pub const PROJECTILE_POWER: super::super::PacketId = super::super::PacketId(138i32);
        pub const RECIPE_BOOK_ADD: super::super::PacketId = super::super::PacketId(75i32);
        pub const RECIPE_BOOK_REMOVE: super::super::PacketId = super::super::PacketId(76i32);
        pub const RECIPE_BOOK_SETTINGS: super::super::PacketId = super::super::PacketId(77i32);
        pub const REMOVE_ENTITIES: super::super::PacketId = super::super::PacketId(78i32);
        pub const REMOVE_MOB_EFFECT: super::super::PacketId = super::super::PacketId(79i32);
        pub const RESET_SCORE: super::super::PacketId = super::super::PacketId(80i32);
        pub const RESOURCE_PACK_POP: super::super::PacketId = super::super::PacketId(81i32);
        pub const RESOURCE_PACK_PUSH: super::super::PacketId = super::super::PacketId(82i32);
        pub const RESPAWN: super::super::PacketId = super::super::PacketId(84i32);
        pub const ROTATE_HEAD: super::super::PacketId = super::super::PacketId(85i32);
        pub const SECTION_BLOCKS_UPDATE: super::super::PacketId = super::super::PacketId(86i32);
        pub const SELECT_ADVANCEMENTS_TAB: super::super::PacketId = super::super::PacketId(87i32);
        pub const SERVER_DATA: super::super::PacketId = super::super::PacketId(88i32);
        pub const SERVER_LINKS: super::super::PacketId = super::super::PacketId(140i32);
        pub const SET_ACTION_BAR_TEXT: super::super::PacketId = super::super::PacketId(89i32);
        pub const SET_BORDER_CENTER: super::super::PacketId = super::super::PacketId(90i32);
        pub const SET_BORDER_LERP_SIZE: super::super::PacketId = super::super::PacketId(91i32);
        pub const SET_BORDER_SIZE: super::super::PacketId = super::super::PacketId(92i32);
        pub const SET_BORDER_WARNING_DELAY: super::super::PacketId = super::super::PacketId(93i32);
        pub const SET_BORDER_WARNING_DISTANCE: super::super::PacketId =
            super::super::PacketId(94i32);
        pub const SET_CAMERA: super::super::PacketId = super::super::PacketId(95i32);
        pub const SET_CHUNK_CACHE_CENTER: super::super::PacketId = super::super::PacketId(96i32);
        pub const SET_CHUNK_CACHE_RADIUS: super::super::PacketId = super::super::PacketId(97i32);
        pub const SET_CURSOR_ITEM: super::super::PacketId = super::super::PacketId(98i32);
        pub const SET_DEFAULT_SPAWN_POSITION: super::super::PacketId =
            super::super::PacketId(99i32);
        pub const SET_DISPLAY_OBJECTIVE: super::super::PacketId = super::super::PacketId(100i32);
        pub const SET_ENTITY_DATA: super::super::PacketId = super::super::PacketId(101i32);
        pub const SET_ENTITY_LINK: super::super::PacketId = super::super::PacketId(102i32);
        pub const SET_ENTITY_MOTION: super::super::PacketId = super::super::PacketId(103i32);
        pub const SET_EQUIPMENT: super::super::PacketId = super::super::PacketId(104i32);
        pub const SET_EXPERIENCE: super::super::PacketId = super::super::PacketId(105i32);
        pub const SET_HEALTH: super::super::PacketId = super::super::PacketId(106i32);
        pub const SET_HELD_SLOT: super::super::PacketId = super::super::PacketId(107i32);
        pub const SET_OBJECTIVE: super::super::PacketId = super::super::PacketId(108i32);
        pub const SET_PASSENGERS: super::super::PacketId = super::super::PacketId(109i32);
        pub const SET_PLAYER_INVENTORY: super::super::PacketId = super::super::PacketId(110i32);
        pub const SET_PLAYER_TEAM: super::super::PacketId = super::super::PacketId(111i32);
        pub const SET_SCORE: super::super::PacketId = super::super::PacketId(112i32);
        pub const SET_SIMULATION_DISTANCE: super::super::PacketId = super::super::PacketId(113i32);
        pub const SET_SUBTITLE_TEXT: super::super::PacketId = super::super::PacketId(114i32);
        pub const SET_TIME: super::super::PacketId = super::super::PacketId(115i32);
        pub const SET_TITLES_ANIMATION: super::super::PacketId = super::super::PacketId(117i32);
        pub const SET_TITLE_TEXT: super::super::PacketId = super::super::PacketId(116i32);
        pub const SHOW_DIALOG: super::super::PacketId = super::super::PacketId(143i32);
        pub const SOUND: super::super::PacketId = super::super::PacketId(119i32);
        pub const SOUND_ENTITY: super::super::PacketId = super::super::PacketId(118i32);
        pub const START_CONFIGURATION: super::super::PacketId = super::super::PacketId(120i32);
        pub const STOP_SOUND: super::super::PacketId = super::super::PacketId(121i32);
        pub const STORE_COOKIE: super::super::PacketId = super::super::PacketId(122i32);
        pub const SWING_ANIMATION: super::super::PacketId = super::super::PacketId(123i32);
        pub const SYSTEM_CHAT: super::super::PacketId = super::super::PacketId(124i32);
        pub const TAB_LIST: super::super::PacketId = super::super::PacketId(125i32);
        pub const TAG_QUERY: super::super::PacketId = super::super::PacketId(126i32);
        pub const TAKE_ITEM_ENTITY: super::super::PacketId = super::super::PacketId(127i32);
        pub const TELEPORT_ENTITY: super::super::PacketId = super::super::PacketId(128i32);
        pub const TEST_INSTANCE_BLOCK_STATUS: super::super::PacketId =
            super::super::PacketId(129i32);
        pub const TICKING_STATE: super::super::PacketId = super::super::PacketId(130i32);
        pub const TICKING_STEP: super::super::PacketId = super::super::PacketId(131i32);
        pub const TRANSFER: super::super::PacketId = super::super::PacketId(132i32);
        pub const UPDATE_ADVANCEMENTS: super::super::PacketId = super::super::PacketId(133i32);
        pub const UPDATE_ATTRIBUTES: super::super::PacketId = super::super::PacketId(134i32);
        pub const UPDATE_MOB_EFFECT: super::super::PacketId = super::super::PacketId(135i32);
        pub const UPDATE_RECIPES: super::super::PacketId = super::super::PacketId(136i32);
        pub const UPDATE_TAGS: super::super::PacketId = super::super::PacketId(137i32);
        pub const WAYPOINT: super::super::PacketId = super::super::PacketId(141i32);
        pub const SET_CARRIED_ITEM: super::super::PacketId = SET_CURSOR_ITEM;
        pub const CHAT: super::super::PacketId = SYSTEM_CHAT;
        pub const BUNDLE: super::super::PacketId = BUNDLE_DELIMITER;
        pub const SPAWN_ENTITY: super::super::PacketId = ADD_ENTITY;
        pub const ENTITY_ANIMATION: super::super::PacketId = ANIMATE;
        pub const STATISTICS: super::super::PacketId = AWARD_STATS;
        pub const ACKNOWLEDGE_BLOCK_CHANGES: super::super::PacketId = BLOCK_CHANGED_ACK;
        pub const BLOCK_BREAK_ANIMATION: super::super::PacketId = BLOCK_DESTRUCTION;
        pub const BLOCK_ACTION: super::super::PacketId = BLOCK_EVENT;
        pub const BLOCK_CHANGE: super::super::PacketId = BLOCK_UPDATE;
        pub const BOSS_BAR: super::super::PacketId = BOSS_EVENT;
        pub const SERVER_DIFFICULTY: super::super::PacketId = CHANGE_DIFFICULTY;
        pub const CHUNK_BATCH_END: super::super::PacketId = CHUNK_BATCH_FINISHED;
        pub const CHUNK_BATCH_BEGIN: super::super::PacketId = CHUNK_BATCH_START;
        pub const TAB_COMPLETE: super::super::PacketId = COMMAND_SUGGESTIONS;
        pub const DECLARE_COMMANDS: super::super::PacketId = COMMANDS;
        pub const CLOSE_WINDOW: super::super::PacketId = CONTAINER_CLOSE;
        pub const WINDOW_ITEMS: super::super::PacketId = CONTAINER_SET_CONTENT;
        pub const WINDOW_PROPERTY: super::super::PacketId = CONTAINER_SET_DATA;
        pub const SET_SLOT: super::super::PacketId = CONTAINER_SET_SLOT;
        pub const PLUGIN_MESSAGE: super::super::PacketId = CUSTOM_PAYLOAD;
        pub const ENTITY_STATUS: super::super::PacketId = ENTITY_EVENT;
        pub const EXPLOSION: super::super::PacketId = EXPLODE;
        pub const UNLOAD_CHUNK: super::super::PacketId = FORGET_LEVEL_CHUNK;
        pub const CHANGE_GAME_STATE: super::super::PacketId = GAME_EVENT;
        pub const OPEN_HORSE_WINDOW: super::super::PacketId = MOUNT_SCREEN_OPEN;
        pub const INITIALIZE_WORLD_BORDER: super::super::PacketId = INITIALIZE_BORDER;
        pub const CHUNK_DATA: super::super::PacketId = LEVEL_CHUNK_WITH_LIGHT;
        pub const EFFECT: super::super::PacketId = LEVEL_EVENT;
        pub const PARTICLE: super::super::PacketId = LEVEL_PARTICLES;
        pub const JOIN_GAME: super::super::PacketId = LOGIN;
        pub const MAP_DATA: super::super::PacketId = MAP_ITEM_DATA;
        pub const ENTITY_RELATIVE_MOVE: super::super::PacketId = MOVE_ENTITY_POS;
        pub const ENTITY_RELATIVE_MOVE_AND_ROTATION: super::super::PacketId = MOVE_ENTITY_POS_ROT;
        pub const MOVE_MINECART: super::super::PacketId = MOVE_MINECART_ALONG_TRACK;
        pub const ENTITY_ROTATION: super::super::PacketId = MOVE_ENTITY_ROT;
        pub const OPEN_WINDOW: super::super::PacketId = OPEN_SCREEN;
        pub const DEBUG_PONG: super::super::PacketId = PONG_RESPONSE;
        pub const CRAFT_RECIPE_RESPONSE: super::super::PacketId = PLACE_GHOST_RECIPE;
        pub const CHAT_MESSAGE: super::super::PacketId = PLAYER_CHAT;
        pub const FACE_PLAYER: super::super::PacketId = PLAYER_LOOK_AT;
        pub const PLAYER_POSITION_AND_LOOK: super::super::PacketId = PLAYER_POSITION;
        pub const DESTROY_ENTITIES: super::super::PacketId = REMOVE_ENTITIES;
        pub const REMOVE_ENTITY_EFFECT: super::super::PacketId = REMOVE_MOB_EFFECT;
        pub const RESOURCE_PACK_REMOVE: super::super::PacketId = RESOURCE_PACK_POP;
        pub const RESOURCE_PACK_SEND: super::super::PacketId = RESOURCE_PACK_PUSH;
        pub const ENTITY_HEAD_LOOK: super::super::PacketId = ROTATE_HEAD;
        pub const MULTI_BLOCK_CHANGE: super::super::PacketId = SECTION_BLOCKS_UPDATE;
        pub const ACTION_BAR: super::super::PacketId = SET_ACTION_BAR_TEXT;
        pub const WORLD_BORDER_CENTER: super::super::PacketId = SET_BORDER_CENTER;
        pub const WORLD_BORDER_LERP_SIZE: super::super::PacketId = SET_BORDER_LERP_SIZE;
        pub const WORLD_BORDER_SIZE: super::super::PacketId = SET_BORDER_SIZE;
        pub const WORLD_BORDER_WARNING_DELAY: super::super::PacketId = SET_BORDER_WARNING_DELAY;
        pub const WORLD_BORDER_WARNING_REACH: super::super::PacketId = SET_BORDER_WARNING_DISTANCE;
        pub const UPDATE_VIEW_POSITION: super::super::PacketId = SET_CHUNK_CACHE_CENTER;
        pub const UPDATE_VIEW_DISTANCE: super::super::PacketId = SET_CHUNK_CACHE_RADIUS;
        pub const SPAWN_POSITION: super::super::PacketId = SET_DEFAULT_SPAWN_POSITION;
        pub const DISPLAY_SCOREBOARD: super::super::PacketId = SET_DISPLAY_OBJECTIVE;
        pub const ENTITY_METADATA: super::super::PacketId = SET_ENTITY_DATA;
        pub const ATTACH_ENTITY: super::super::PacketId = SET_ENTITY_LINK;
        pub const ENTITY_VELOCITY: super::super::PacketId = SET_ENTITY_MOTION;
        pub const ENTITY_EQUIPMENT: super::super::PacketId = SET_EQUIPMENT;
        pub const UPDATE_HEALTH: super::super::PacketId = SET_HEALTH;
        pub const HELD_ITEM_CHANGE: super::super::PacketId = SET_HELD_SLOT;
        pub const SCOREBOARD_OBJECTIVE: super::super::PacketId = SET_OBJECTIVE;
        pub const UPDATE_SCORE: super::super::PacketId = SET_SCORE;
        pub const UPDATE_SIMULATION_DISTANCE: super::super::PacketId = SET_SIMULATION_DISTANCE;
        pub const SET_TITLE_SUBTITLE: super::super::PacketId = SET_SUBTITLE_TEXT;
        pub const TIME_UPDATE: super::super::PacketId = SET_TIME;
        pub const SET_TITLE_TIMES: super::super::PacketId = SET_TITLES_ANIMATION;
        pub const ENTITY_SOUND_EFFECT: super::super::PacketId = SOUND_ENTITY;
        pub const SOUND_EFFECT: super::super::PacketId = SOUND;
        pub const CONFIGURATION_START: super::super::PacketId = START_CONFIGURATION;
        pub const SYSTEM_CHAT_MESSAGE: super::super::PacketId = SYSTEM_CHAT;
        pub const PLAYER_LIST_HEADER_AND_FOOTER: super::super::PacketId = TAB_LIST;
        pub const NBT_QUERY_RESPONSE: super::super::PacketId = TAG_QUERY;
        pub const COLLECT_ITEM: super::super::PacketId = TAKE_ITEM_ENTITY;
        pub const ENTITY_EFFECT: super::super::PacketId = UPDATE_MOB_EFFECT;
        pub const DECLARE_RECIPES: super::super::PacketId = UPDATE_RECIPES;
        pub const TAGS: super::super::PacketId = UPDATE_TAGS;
    }
}
