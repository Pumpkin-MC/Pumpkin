/* This file is generated. Do not edit manually. */
#[doc = r" Number of version columns, oldest first; the last one is the current version."]
pub const VERSIONS: usize = 52;
#[doc = r" A packet's id in every version, `-1` where the version doesn't have it."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketId(pub [i16; VERSIONS]);
pub mod serverbound {
    pub mod handshake {
        pub const INTENTION: super::super::PacketId = super::super::PacketId([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        pub const HANDSHAKE: super::super::PacketId = INTENTION;
        pub const HANDSHAKING: super::super::PacketId = INTENTION;
    }
    pub mod status {
        pub const PING_REQUEST: super::super::PacketId = super::super::PacketId([
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]);
        pub const STATUS_REQUEST: super::super::PacketId = super::super::PacketId([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
    }
    pub mod login {
        pub const COOKIE_RESPONSE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4,
        ]);
        pub const CUSTOM_QUERY_ANSWER: super::super::PacketId = super::super::PacketId([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ]);
        pub const HELLO: super::super::PacketId = super::super::PacketId([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        pub const KEY: super::super::PacketId = super::super::PacketId([
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]);
        pub const LOGIN_ACKNOWLEDGED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3,
        ]);
        pub const LOGIN_START: super::super::PacketId = HELLO;
        pub const ENCRYPTION_RESPONSE: super::super::PacketId = KEY;
        pub const LOGIN_PLUGIN_RESPONSE: super::super::PacketId = CUSTOM_QUERY_ANSWER;
    }
    pub mod config {
        pub const ACCEPT_CODE_OF_CONDUCT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 9, 9, 9, 9, 9,
        ]);
        pub const CLIENT_INFORMATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ]);
        pub const COOKIE_RESPONSE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
        ]);
        pub const CUSTOM_CLICK_ACTION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 8, 8, 8, 8, 8, 8, 8,
        ]);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2,
        ]);
        pub const FINISH_CONFIGURATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, 2, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3,
        ]);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4,
        ]);
        pub const PONG: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 4, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5,
        ]);
        pub const RESOURCE_PACK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 5, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6,
        ]);
        pub const SELECT_KNOWN_PACKS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
        ]);
    }
    pub mod play {
        pub const ACCEPT_TELEPORTATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        pub const ATTACK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, 1, 1, 1,
        ]);
        pub const BLOCK_ENTITY_TAG_QUERY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
        ]);
        pub const BUNDLE_ITEM_SELECTED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, 2,
            2, 2, 2, 2, 2, 3, 3, 3,
        ]);
        pub const CHANGE_DIFFICULTY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4,
        ]);
        pub const CHANGE_GAME_MODE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 4, 4, 4, 4, 5, 5, 5,
        ]);
        pub const CHAT: super::super::PacketId = super::super::PacketId([
            1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 4, 5, 5, 5, 5, 5, 5, 6, 6, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9,
        ]);
        pub const CHAT_ACK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 5, 5,
            5, 5, 6, 6, 6,
        ]);
        pub const CHAT_COMMAND: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 4, 4, 4, 4, 4, 4, 5, 4, 5, 5, 5, 6, 6,
            6, 6, 7, 7, 7,
        ]);
        pub const CHAT_COMMAND_SIGNED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 6, 6, 6,
            7, 7, 7, 7, 8, 8, 8,
        ]);
        pub const CHAT_PREVIEW: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const CHAT_SESSION_UPDATE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 32, 6, 6, 6, 6, 7, 7, 8, 8, 8, 9,
            9, 9, 9, 10, 10, 10,
        ]);
        pub const CHUNK_BATCH_RECEIVED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 7, 7, 8, 8, 9, 9, 9,
            10, 10, 10, 10, 11, 11, 11,
        ]);
        pub const CLIENT_COMMAND: super::super::PacketId = super::super::PacketId([
            22, 22, 22, 3, 3, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 6, 7, 6, 7, 7, 8, 8, 9, 9, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12,
        ]);
        pub const CLIENT_INFORMATION: super::super::PacketId = super::super::PacketId([
            21, 21, 21, 4, 4, 4, 4, 4, 4, 4, 5, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 7, 8, 7, 8, 8, 9, 9, 10, 10, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14,
        ]);
        pub const CLIENT_TICK_END: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 11,
            11, 12, 12, 12, 12, 13, 13, 13,
        ]);
        pub const COMMAND_SUGGESTION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, 15, 15,
        ]);
        pub const COMMAND_SUGGESTIONS: super::super::PacketId = super::super::PacketId([
            20, 20, 20, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 8, 9, 8, 9, 9, 10, 10, 11, 11, 13, 13, 13, 14, 14, 14, 14, 15, -1, -1,
        ]);
        pub const CONFIGURATION_ACKNOWLEDGED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 11, 12, 12, 14, 14,
            14, 15, 15, 15, 15, 16, 16, 16,
        ]);
        pub const CONTAINER_BUTTON_CLICK: super::super::PacketId = super::super::PacketId([
            17, 17, 17, 6, 6, 6, 6, 6, 6, 6, 7, 6, 6, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 7, 7, 7, 7, 9, 10, 9, 10, 10, 12, 12, 13, 13, 15, 15, 15, 16, 16, 16, 16, 17, 17,
            17,
        ]);
        pub const CONTAINER_CLICK: super::super::PacketId = super::super::PacketId([
            14, 14, 14, 7, 7, 7, 7, 7, 7, 7, 8, 7, 7, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 8, 8, 8, 8, 10, 11, 10, 11, 11, 13, 13, 14, 14, 16, 16, 16, 17, 17, 17, 17, 18, 18,
            18,
        ]);
        pub const CONTAINER_CLOSE: super::super::PacketId = super::super::PacketId([
            13, 13, 13, 8, 8, 8, 8, 8, 8, 8, 9, 8, 8, 9, 9, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 9, 9, 9, 9, 11, 12, 11, 12, 12, 14, 14, 15, 15, 17, 17, 17, 18, 18, 18,
            18, 19, 19, 19,
        ]);
        pub const CONTAINER_SLOT_STATE_CHANGED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 15, 16, 16, 18, 18,
            18, 19, 19, 19, 19, 20, 20, 20,
        ]);
        pub const COOKIE_RESPONSE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 17, 17, 19, 19,
            19, 20, 20, 20, 20, 21, 21, 21,
        ]);
        pub const CUSTOM_CLICK_ACTION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 65, 65, 65, 65, 68, 68, 68,
        ]);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId([
            23, 23, 23, 9, 9, 9, 9, 9, 9, 9, 10, 9, 9, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 11,
            11, 11, 11, 11, 11, 10, 10, 10, 10, 12, 13, 12, 13, 13, 15, 16, 18, 18, 20, 20, 20, 21,
            21, 21, 21, 22, 22, 22,
        ]);
        pub const DEBUG_SAMPLE_SUBSCRIPTION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 19, 19, 21, 21,
            21, 22, 22, -1, -1, -1, -1, -1,
        ]);
        pub const DEBUG_SUBSCRIPTION_REQUEST: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 22, 22, 23, 23, 23,
        ]);
        pub const EDIT_BOOK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 11, 11, 12, 12, 12, 12, 12, 12,
            12, 12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 13, 14, 13, 14, 14, 16, 17, 20, 20, 22, 22,
            22, 23, 23, 23, 23, 24, 24, 24,
        ]);
        pub const ENTITY_TAG_QUERY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 12, 12, 12, 13, 13, 13, 13, 13, 13,
            13, 13, 13, 13, 13, 13, 13, 12, 12, 12, 12, 14, 15, 14, 15, 15, 17, 18, 21, 21, 23, 23,
            23, 24, 24, 24, 24, 25, 25, 25,
        ]);
        pub const INTERACT: super::super::PacketId = super::super::PacketId([
            2, 2, 2, 10, 10, 10, 10, 10, 10, 10, 11, 10, 10, 13, 13, 13, 14, 14, 14, 14, 14, 14,
            14, 14, 14, 14, 14, 14, 14, 13, 13, 13, 13, 15, 16, 15, 16, 16, 18, 19, 22, 22, 24, 24,
            24, 25, 25, 25, 25, 26, 26, 26,
        ]);
        pub const JIGSAW_GENERATE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, 15, 15, 15, 15, 15, 14, 14, 14, 14, 16, 17, 16, 17, 17, 19, 20, 23, 23, 25, 25,
            25, 26, 26, 26, 26, 27, 27, 27,
        ]);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId([
            0, 0, 0, 11, 11, 11, 11, 11, 11, 11, 12, 11, 11, 14, 14, 14, 15, 15, 15, 15, 15, 15,
            15, 15, 16, 16, 16, 16, 16, 15, 15, 15, 15, 17, 18, 17, 18, 18, 20, 21, 24, 24, 26, 26,
            26, 27, 27, 27, 27, 28, 28, 28,
        ]);
        pub const LOCK_DIFFICULTY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 16, 16, 16, 16, 16, 16,
            16, 16, 17, 17, 17, 17, 17, 16, 16, 16, 16, 18, 19, 18, 19, 19, 21, 22, 25, 25, 27, 27,
            27, 28, 28, 28, 28, 29, 29, 29,
        ]);
        pub const MOVE_PLAYER_POS: super::super::PacketId = super::super::PacketId([
            4, 4, 4, 12, 12, 12, 12, 12, 12, 12, 14, 13, 13, 16, 16, 16, 17, 17, 17, 17, 17, 17,
            17, 17, 18, 18, 18, 18, 18, 17, 17, 17, 17, 19, 20, 19, 20, 20, 22, 23, 26, 26, 28, 28,
            28, 29, 29, 29, 29, 30, 30, 30,
        ]);
        pub const MOVE_PLAYER_POS_ROT: super::super::PacketId = super::super::PacketId([
            6, 6, 6, 13, 13, 13, 13, 13, 13, 13, 15, 14, 14, 17, 17, 17, 18, 18, 18, 18, 18, 18,
            18, 18, 19, 19, 19, 19, 19, 18, 18, 18, 18, 20, 21, 20, 21, 21, 23, 24, 27, 27, 29, 29,
            29, 30, 30, 30, 30, 31, 31, 31,
        ]);
        pub const MOVE_PLAYER_ROT: super::super::PacketId = super::super::PacketId([
            5, 5, 5, 14, 14, 14, 14, 14, 14, 14, 16, 15, 15, 18, 18, 18, 19, 19, 19, 19, 19, 19,
            19, 19, 20, 20, 20, 20, 20, 19, 19, 19, 19, 21, 22, 21, 22, 22, 24, 25, 28, 28, 30, 30,
            30, 31, 31, 31, 31, 32, 32, 32,
        ]);
        pub const MOVE_PLAYER_STATUS_ONLY: super::super::PacketId = super::super::PacketId([
            3, 3, 3, 15, 15, 15, 15, 15, 15, 15, 13, 12, 12, 15, 15, 15, 20, 20, 20, 20, 20, 20,
            20, 20, 21, 21, 21, 21, 21, 20, 20, 20, 20, 22, 23, 22, 23, 23, 25, 26, 29, 29, 31, 31,
            31, 32, 32, 32, 32, 33, 33, 33,
        ]);
        pub const MOVE_VEHICLE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 16, 16, 16, 16, 16, 16, 16, 17, 16, 16, 19, 19, 19, 21, 21, 21, 21, 21, 21,
            21, 21, 22, 22, 22, 22, 22, 21, 21, 21, 21, 23, 24, 23, 24, 24, 26, 27, 30, 30, 32, 32,
            32, 33, 33, 33, 33, 34, 34, 34,
        ]);
        pub const PADDLE_BOAT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 17, 17, 17, 17, 17, 17, 17, 18, 17, 17, 20, 20, 20, 22, 22, 22, 22, 22, 22,
            22, 22, 23, 23, 23, 23, 23, 22, 22, 22, 22, 24, 25, 24, 25, 25, 27, 28, 31, 31, 33, 33,
            33, 34, 34, 34, 34, 35, 35, 35,
        ]);
        pub const PICK_ITEM: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 21, 21, 21, 23, 23, 23, 23, 23, 23,
            23, 23, 24, 24, 24, 24, 24, 23, 23, 23, 23, 25, 26, 25, 26, 26, 28, 29, 32, 32, 34, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const PICK_ITEM_FROM_BLOCK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 34,
            34, 35, 35, 35, 35, 36, 36, 36,
        ]);
        pub const PICK_ITEM_FROM_ENTITY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 35,
            35, 36, 36, 36, 36, 37, 37, 37,
        ]);
        pub const PING_REQUEST: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 29, 30, 33, 33, 35, 36,
            36, 37, 37, 37, 37, 38, 38, 38,
        ]);
        pub const PLACE_RECIPE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 18, 18, 22, 22, 22, 24, 24, 24, 24, 24, 24,
            24, 24, 25, 25, 25, 25, 25, 24, 24, 24, 24, 26, 27, 26, 27, 27, 30, 31, 34, 34, 36, 37,
            37, 38, 38, 38, 38, 39, 39, 39,
        ]);
        pub const PLAYER_ABILITIES: super::super::PacketId = super::super::PacketId([
            19, 19, 19, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 23, 23, 23, 25, 25, 25, 25, 25, 25,
            25, 25, 26, 26, 26, 26, 26, 25, 25, 25, 25, 27, 28, 27, 28, 28, 31, 32, 35, 35, 37, 38,
            38, 39, 39, 39, 39, 40, 40, 40,
        ]);
        pub const PLAYER_ACTION: super::super::PacketId = super::super::PacketId([
            7, 7, 7, 19, 19, 19, 19, 19, 19, 19, 20, 20, 20, 24, 24, 24, 26, 26, 26, 26, 26, 26,
            26, 26, 27, 27, 27, 27, 27, 26, 26, 26, 26, 28, 29, 28, 29, 29, 32, 33, 36, 36, 38, 39,
            39, 40, 40, 40, 40, 41, 41, 41,
        ]);
        pub const PLAYER_COMMAND: super::super::PacketId = super::super::PacketId([
            11, 11, 11, 20, 20, 20, 20, 20, 20, 20, 21, 21, 21, 25, 25, 25, 27, 27, 27, 27, 27, 27,
            27, 27, 28, 28, 28, 28, 28, 27, 27, 27, 27, 29, 30, 29, 30, 30, 33, 34, 37, 37, 39, 40,
            40, 41, 41, 41, 41, 42, 42, 42,
        ]);
        pub const PLAYER_INPUT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 40, 41,
            41, 42, 42, 42, 42, 43, 43, 43,
        ]);
        pub const PLAYER_LOADED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 42,
            42, 43, 43, 43, 43, 44, 44, 44,
        ]);
        pub const PONG: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 29, 29, 29, 29, 31, 32, 31, 32, 32, 35, 36, 39, 39, 41, 43,
            43, 44, 44, 44, 44, 45, 45, 45,
        ]);
        pub const PUNCH: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 46,
        ]);
        pub const RECIPE_BOOK_CHANGE_SETTINGS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, 30, 30, 30, 30, 30, 30, 30, 32, 33, 33, 33, 33, 36, 37, 40, 40, 42, 44,
            44, 45, 45, 45, 45, 46, 46, 47,
        ]);
        pub const RECIPE_BOOK_DATA: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 23, 23, 23, 27, 27, 27, 29, 29, 29, 29, 29, 29,
            29, 29, 30, 30, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const RECIPE_BOOK_SEEN_RECIPE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, 31, 31, 31, 31, 31, 31, 31, 33, 34, 34, 34, 34, 37, 38, 41, 41, 43, 45,
            45, 46, 46, 46, 46, 47, 47, 48,
        ]);
        pub const RENAME_ITEM: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 28, 28, 28, 30, 30, 30, 30, 30, 30,
            30, 30, 31, 31, 32, 32, 32, 32, 32, 32, 32, 34, 35, 35, 35, 35, 38, 39, 42, 42, 44, 46,
            46, 47, 47, 47, 47, 48, 48, 49,
        ]);
        pub const RESOURCE_PACK: super::super::PacketId = super::super::PacketId([
            -1, -1, 25, 22, 22, 22, 22, 22, 22, 22, 24, 24, 24, 29, 29, 29, 31, 31, 31, 31, 31, 31,
            31, 31, 32, 32, 33, 33, 33, 33, 33, 33, 33, 35, 36, 36, 36, 36, 39, 40, 43, 43, 45, 47,
            47, 48, 48, 48, 48, 49, 49, 50,
        ]);
        pub const SEEN_ADVANCEMENTS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 25, 25, 25, 30, 30, 30, 32, 32, 32, 32, 32, 32,
            32, 32, 33, 33, 34, 34, 34, 34, 34, 34, 34, 36, 37, 37, 37, 37, 40, 41, 44, 44, 46, 48,
            48, 49, 49, 49, 49, 50, 50, 51,
        ]);
        pub const SELECT_TRADE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 31, 31, 31, 33, 33, 33, 33, 33, 33,
            33, 33, 34, 34, 35, 35, 35, 35, 35, 35, 35, 37, 38, 38, 38, 38, 41, 42, 45, 45, 47, 49,
            49, 50, 50, 50, 50, 51, 51, 52,
        ]);
        pub const SET_BEACON: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 32, 32, 32, 34, 34, 34, 34, 34, 34,
            34, 34, 35, 35, 36, 36, 36, 36, 36, 36, 36, 38, 39, 39, 39, 39, 42, 43, 46, 46, 48, 50,
            50, 51, 51, 51, 51, 52, 52, 53,
        ]);
        pub const SET_CARRIED_ITEM: super::super::PacketId = super::super::PacketId([
            9, 9, 9, 23, 23, 23, 23, 23, 23, 23, 26, 26, 26, 33, 33, 33, 35, 35, 35, 35, 35, 35,
            35, 35, 36, 36, 37, 37, 37, 37, 37, 37, 37, 39, 40, 40, 40, 40, 43, 44, 47, 47, 49, 51,
            51, 52, 52, 52, 52, 53, 53, 54,
        ]);
        pub const SET_COMMAND_BLOCK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 34, 34, 34, 36, 36, 36, 36, 36, 36,
            36, 36, 37, 37, 38, 38, 38, 38, 38, 38, 38, 40, 41, 41, 41, 41, 44, 45, 48, 48, 50, 52,
            52, 53, 53, 53, 53, 54, 54, 55,
        ]);
        pub const SET_COMMAND_MINECART: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 35, 35, 35, 37, 37, 37, 37, 37, 37,
            37, 37, 38, 38, 39, 39, 39, 39, 39, 39, 39, 41, 42, 42, 42, 42, 45, 46, 49, 49, 51, 53,
            53, 54, 54, 54, 54, 55, 55, 56,
        ]);
        pub const SET_CREATIVE_MODE_SLOT: super::super::PacketId = super::super::PacketId([
            16, 16, 16, 24, 24, 24, 24, 24, 24, 24, 27, 27, 27, 36, 36, 36, 38, 38, 38, 38, 38, 38,
            38, 38, 39, 39, 40, 40, 40, 40, 40, 40, 40, 42, 43, 43, 43, 43, 46, 47, 50, 50, 52, 54,
            54, 55, 55, 55, 55, 56, 56, 57,
        ]);
        pub const SET_GAME_RULE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, 57, 57, 58,
        ]);
        pub const SET_JIGSAW_BLOCK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 39, 39, 39, 39, 39, 39,
            39, 39, 40, 40, 41, 41, 41, 41, 41, 41, 41, 43, 44, 44, 44, 44, 47, 48, 51, 51, 53, 55,
            55, 56, 56, 56, 56, 58, 58, 59,
        ]);
        pub const SET_STRUCTURE_BLOCK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 37, 37, 37, 40, 40, 40, 40, 40, 40,
            40, 40, 41, 41, 42, 42, 42, 42, 42, 42, 42, 44, 45, 45, 45, 45, 48, 49, 52, 52, 54, 56,
            56, 57, 57, 57, 57, 59, 59, 60,
        ]);
        pub const SET_TEST_BLOCK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            57, 58, 58, 58, 58, 60, 60, 61,
        ]);
        pub const SIGN_UPDATE: super::super::PacketId = super::super::PacketId([
            18, 18, 18, 25, 25, 25, 25, 25, 25, 25, 28, 28, 28, 38, 38, 38, 41, 41, 41, 41, 41, 41,
            41, 41, 42, 42, 43, 43, 43, 43, 43, 43, 43, 45, 46, 46, 46, 46, 49, 50, 53, 53, 55, 57,
            58, 59, 59, 59, 59, 61, 61, 62,
        ]);
        pub const SPECTATE_ENTITY: super::super::PacketId = super::super::PacketId([
            -1, -1, 24, 27, 27, 27, 27, 27, 27, 27, 30, 30, 30, 40, 40, 40, 43, 43, 43, 43, 43, 43,
            43, 43, 44, 44, 45, 45, 45, 45, 45, 45, 45, 47, 48, 48, 48, 48, 51, 52, 55, 55, 57, 59,
            60, 61, 61, 61, 61, 64, -1, -1,
        ]);
        pub const SPECTATOR_ACTION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, 62, 62, 63,
        ]);
        pub const STEER_VEHICLE: super::super::PacketId = super::super::PacketId([
            12, 12, 12, 21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 26, 26, 26, 28, 28, 28, 28, 28, 28,
            28, 28, 29, 29, 29, 29, 29, 28, 28, 28, 28, 30, 31, 30, 31, 31, 34, 35, 38, 38, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const SWING: super::super::PacketId = super::super::PacketId([
            10, 10, 10, 26, 26, 26, 26, 26, 26, 26, 29, 29, 29, 39, 39, 39, 42, 42, 42, 42, 42, 42,
            42, 42, 43, 43, 44, 44, 44, 44, 44, 44, 44, 46, 47, 47, 47, 47, 50, 51, 54, 54, 56, 58,
            59, 60, 60, 60, 60, 63, 63, -1,
        ]);
        pub const TELEPORT_TO_ENTITY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, 64, 64,
        ]);
        pub const TEST_INSTANCE_BLOCK_ACTION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            61, 62, 62, 62, 62, 65, 65, 65,
        ]);
        pub const USE_ITEM: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 29, 29, 29, 29, 29, 29, 29, 32, 32, 32, 42, 42, 42, 45, 45, 45, 45, 45, 45,
            45, 45, 46, 46, 47, 47, 47, 47, 47, 47, 47, 49, 50, 50, 50, 50, 53, 54, 57, 57, 59, 61,
            63, 64, 64, 64, 64, 67, 67, 67,
        ]);
        pub const USE_ITEM_ON: super::super::PacketId = super::super::PacketId([
            8, 8, 8, 28, 28, 28, 28, 28, 28, 28, 31, 31, 31, 41, 41, 41, 44, 44, 44, 44, 44, 44,
            44, 44, 45, 45, 46, 46, 46, 46, 46, 46, 46, 48, 49, 49, 49, 49, 52, 53, 56, 56, 58, 60,
            62, 63, 63, 63, 63, 66, 66, 66,
        ]);
        pub const WINDOW_CONFIRMATION: super::super::PacketId = super::super::PacketId([
            15, 15, 15, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1,
        ]);
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
        pub const PLAYER_BLOCK_PLACEMENT: super::super::PacketId = USE_ITEM_ON;
        pub const SPECTATE: super::super::PacketId = SPECTATOR_ACTION;
    }
}
pub mod clientbound {
    pub mod handshake {}
    pub mod status {
        pub const PONG_RESPONSE: super::super::PacketId = super::super::PacketId([
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]);
        pub const STATUS_RESPONSE: super::super::PacketId = super::super::PacketId([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
    }
    pub mod login {
        pub const COOKIE_REQUEST: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5,
        ]);
        pub const CUSTOM_QUERY: super::super::PacketId = super::super::PacketId([
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ]);
        pub const GAME_PROFILE: super::super::PacketId = super::super::PacketId([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, -1, -1,
        ]);
        pub const HELLO: super::super::PacketId = super::super::PacketId([
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]);
        pub const LOGIN_COMPRESSION: super::super::PacketId = super::super::PacketId([
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        ]);
        pub const LOGIN_DISCONNECT: super::super::PacketId = super::super::PacketId([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        pub const LOGIN_FINISHED: super::super::PacketId = super::super::PacketId([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ]);
        pub const LOGIN_SUCCESS: super::super::PacketId = LOGIN_FINISHED;
        pub const SET_COMPRESSION: super::super::PacketId = LOGIN_COMPRESSION;
        pub const ENCRYPTION_REQUEST: super::super::PacketId = HELLO;
        pub const LOGIN_PLUGIN_REQUEST: super::super::PacketId = CUSTOM_QUERY;
    }
    pub mod config {
        pub const CLEAR_DIALOG: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 17, 17, 17, 17, 17, 17, 18,
        ]);
        pub const CODE_OF_CONDUCT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 19, 19, 19, 19, 20,
        ]);
        pub const COOKIE_REQUEST: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ]);
        pub const CUSTOM_REPORT_DETAILS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 15, 15, 15,
            15, 15, 15, 15, 15, 15, 15, 16,
        ]);
        pub const DISCONNECT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2,
        ]);
        pub const FINISH_CONFIGURATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, 2, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3,
        ]);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4,
        ]);
        pub const PING: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 4, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5,
        ]);
        pub const POST_EFFECTS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 10,
        ]);
        pub const REGISTRY_DATA: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 5, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7,
        ]);
        pub const RESET_CHAT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6,
        ]);
        pub const RESOURCE_PACK_POP: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 6, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8,
        ]);
        pub const RESOURCE_PACK_PUSH: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 6, 7, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
        ]);
        pub const SELECT_KNOWN_PACKS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 14, 14, 14, 14,
            14, 14, 14, 14, 14, 14, 14, 15,
        ]);
        pub const SERVER_LINKS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 16, 16, 16,
            16, 16, 16, 16, 16, 16, 16, 17,
        ]);
        pub const SHOW_DIALOG: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 18, 18, 18, 18, 18, 18, 19,
        ]);
        pub const STORE_COOKIE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 11,
        ]);
        pub const TRANSFER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 11, 11, 11,
            11, 11, 11, 11, 11, 11, 11, 12,
        ]);
        pub const UPDATE_ENABLED_FEATURES: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 7, 8, 12, 12, 12, 12,
            12, 12, 12, 12, 12, 12, 12, 13,
        ]);
        pub const UPDATE_TAGS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 8, 9, 13, 13, 13, 13,
            13, 13, 13, 13, 13, 13, 13, 14,
        ]);
    }
    pub mod play {
        pub const ACKNOWLEDGE_PLAYER_DIGGING: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 92, 8,
            8, 8, 7, 7, 7, 7, 7, 8, 8, 8, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1,
        ]);
        pub const ADD_ENTITY: super::super::PacketId = super::super::PacketId([
            14, 14, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]);
        pub const ADD_TRANSIENT_BLOCK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 37,
        ]);
        pub const ANIMATE: super::super::PacketId = super::super::PacketId([
            11, 11, 11, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5,
            5, 6, 6, 6, 6, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2,
        ]);
        pub const AWARD_STATS: super::super::PacketId = super::super::PacketId([
            55, 55, 55, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6,
            6, 7, 7, 7, 7, 4, 4, 4, 5, 5, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3,
        ]);
        pub const BLOCK_CHANGED_ACK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 5, 5, 6, 6, 5, 5, 5, 5, 5, 5, 4, 4, 4,
            4, 4, 4, 4, 4,
        ]);
        pub const BLOCK_DESTRUCTION: super::super::PacketId = super::super::PacketId([
            37, 37, 37, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 8, 8, 8, 8,
            8, 9, 9, 9, 9, 6, 6, 6, 7, 7, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5,
        ]);
        pub const BLOCK_ENTITY_DATA: super::super::PacketId = super::super::PacketId([
            53, 53, 53, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 9, 9, 9,
            9, 9, 10, 10, 10, 10, 7, 7, 7, 8, 8, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 6,
        ]);
        pub const BLOCK_EVENT: super::super::PacketId = super::super::PacketId([
            36, 36, 36, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 11,
            11, 11, 10, 10, 10, 10, 10, 11, 11, 11, 11, 8, 8, 8, 9, 9, 8, 8, 8, 8, 8, 8, 7, 7, 7,
            7, 7, 7, 7, 7,
        ]);
        pub const BLOCK_UPDATE: super::super::PacketId = super::super::PacketId([
            35, 35, 35, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 12,
            12, 12, 11, 11, 11, 11, 11, 12, 12, 12, 12, 9, 9, 9, 10, 10, 9, 9, 9, 9, 9, 9, 8, 8, 8,
            8, 8, 8, 8, 8,
        ]);
        pub const BOSS_EVENT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 13,
            13, 13, 12, 12, 12, 12, 12, 13, 13, 13, 13, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10,
            9, 9, 9, 9, 9, 9, 9, 9,
        ]);
        pub const BUNDLE_DELIMITER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ]);
        pub const CHANGE_DIFFICULTY: super::super::PacketId = super::super::PacketId([
            -1, -1, 65, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 14,
            14, 14, 13, 13, 13, 13, 13, 14, 14, 14, 14, 11, 11, 11, 12, 12, 11, 11, 11, 11, 11, 11,
            10, 10, 10, 10, 10, 10, 10, 10,
        ]);
        pub const CHAT: super::super::PacketId = super::super::PacketId([
            2, 2, 2, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 14, 14, 14, 14, 14, 14, 14, 14, 15,
            15, 15, 14, 14, 14, 14, 14, 15, 15, 15, 15, 48, 51, 49, 53, 53, 55, 55, 57, 57, 59, 59,
            58, 58, 58, 63, 63, 65, -1, -1,
        ]);
        pub const CHAT_PREVIEW_PACKET: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 12, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const CHUNKS_BIOMES: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 13, 13, 14, 14, 14, 14, 14, 14,
            13, 13, 13, 13, 13, 13, 13, 13,
        ]);
        pub const CHUNK_BATCH_FINISHED: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 12, 12, 12, 12, 12, 12,
            11, 11, 11, 11, 11, 11, 11, 11,
        ]);
        pub const CHUNK_BATCH_START: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 13, 13, 13, 13, 13, 13,
            12, 12, 12, 12, 12, 12, 12, 12,
        ]);
        pub const CLEAR_DIALOG: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 132, 132, 137, 137, 139, 139, 142,
        ]);
        pub const CLEAR_TITLES: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 16, 16, 16, 16, 13, 13, 12, 14, 14, 15, 15, 15, 15, 15, 15,
            14, 14, 14, 14, 14, 14, 14, 14,
        ]);
        pub const COMBAT_EVENT: super::super::PacketId = super::super::PacketId([
            -1, -1, 66, 44, 44, 44, 44, 44, 44, 44, 44, 45, 45, 47, 47, 47, 50, 50, 50, 50, 50, 51,
            51, 51, 50, 50, 49, 49, 49, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const COMMANDS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 17, 17, 17, 17, 17, 17, 17, 17, 18,
            18, 18, 17, 17, 16, 16, 16, 18, 18, 18, 18, 15, 15, 14, 16, 16, 17, 17, 17, 17, 17, 17,
            16, 16, 16, 16, 16, 16, 16, 16,
        ]);
        pub const COMMAND_SUGGESTIONS: super::super::PacketId = super::super::PacketId([
            58, 58, 58, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 16, 16, 16, 16, 16, 16, 16, 16, 17,
            17, 17, 16, 16, 15, 15, 15, 17, 17, 17, 17, 14, 14, 13, 15, 15, 16, 16, 16, 16, 16, 16,
            15, 15, 15, 15, 15, 15, 15, 15,
        ]);
        pub const CONTAINER_CLOSE: super::super::PacketId = super::super::PacketId([
            46, 46, 46, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 20,
            20, 20, 19, 19, 18, 18, 18, 19, 19, 19, 19, 16, 16, 15, 17, 17, 18, 18, 18, 18, 18, 18,
            17, 17, 17, 17, 17, 17, 17, 17,
        ]);
        pub const CONTAINER_SET_CONTENT: super::super::PacketId = super::super::PacketId([
            48, 48, 48, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 21, 21, 21, 20, 20, 20, 20, 20, 21,
            21, 21, 20, 20, 19, 19, 19, 20, 20, 20, 20, 17, 17, 16, 18, 18, 19, 19, 19, 19, 19, 19,
            18, 18, 18, 18, 18, 18, 18, 18,
        ]);
        pub const CONTAINER_SET_DATA: super::super::PacketId = super::super::PacketId([
            49, 49, 49, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 21, 21, 21, 21, 21, 22,
            22, 22, 21, 21, 20, 20, 20, 21, 21, 21, 21, 18, 18, 17, 19, 19, 20, 20, 20, 20, 20, 20,
            19, 19, 19, 19, 19, 19, 19, 19,
        ]);
        pub const CONTAINER_SET_SLOT: super::super::PacketId = super::super::PacketId([
            47, 47, 47, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 23, 23, 23, 22, 22, 22, 22, 22, 23,
            23, 23, 22, 22, 21, 21, 21, 22, 22, 22, 22, 19, 19, 18, 20, 20, 21, 21, 21, 21, 21, 21,
            20, 20, 20, 20, 20, 20, 20, 20,
        ]);
        pub const COOKIE_REQUEST: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 22, 22, 22, 22,
            21, 21, 21, 21, 21, 21, 21, 21,
        ]);
        pub const COOLDOWN: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 24, 24, 24, 23, 23, 23, 23, 23, 24,
            24, 24, 23, 23, 22, 22, 22, 23, 23, 23, 23, 20, 20, 19, 21, 21, 22, 22, 23, 23, 23, 23,
            22, 22, 22, 22, 22, 22, 22, 22,
        ]);
        pub const CUSTOM_CHAT_COMPLETIONS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 21, 20, 22, 22, 23, 23, 24, 24, 24, 24,
            23, 23, 23, 23, 23, 23, 23, 23,
        ]);
        pub const CUSTOM_PAYLOAD: super::super::PacketId = super::super::PacketId([
            63, 63, 63, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 25, 25, 25, 24, 24, 24, 24, 24, 25,
            25, 25, 24, 24, 23, 23, 23, 24, 24, 24, 24, 21, 22, 21, 23, 23, 24, 24, 25, 25, 25, 25,
            24, 24, 24, 24, 24, 24, 24, 24,
        ]);
        pub const CUSTOM_REPORT_DETAILS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 122, 129,
            129, 129, 129, 129, 134, 134, 136, 136, 139,
        ]);
        pub const DAMAGE_EVENT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 24, 24, 25, 25, 26, 26, 26, 26,
            25, 25, 25, 25, 25, 25, 25, 25,
        ]);
        pub const DEBUG_BLOCK_VALUE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 26, 26, 26, 26, 26,
        ]);
        pub const DEBUG_CHUNK_VALUE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 27, 27, 27, 27, 27,
        ]);
        pub const DEBUG_ENTITY_VALUE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 28, 28, 28, 28, 28,
        ]);
        pub const DEBUG_EVENT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 29, 29, 29, 29, 29,
        ]);
        pub const DEBUG_SAMPLE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 27, 27, 27, 27,
            26, 26, 26, 30, 30, 30, 30, 30,
        ]);
        pub const DELETE_CHAT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 24, 22, 25, 25, 26, 26, 28, 28, 28, 28,
            27, 27, 27, 31, 31, 31, 31, 31,
        ]);
        pub const DISCONNECT: super::super::PacketId = super::super::PacketId([
            64, 64, 64, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 27, 27, 27, 26, 26, 26, 26, 26, 27,
            27, 27, 26, 26, 25, 25, 25, 26, 26, 26, 26, 23, 25, 23, 26, 26, 27, 27, 29, 29, 29, 29,
            28, 28, 28, 32, 32, 32, 32, 32,
        ]);
        pub const DISGUISED_CHAT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 24, 27, 27, 28, 28, 30, 30, 30, 30,
            29, 29, 29, 33, 33, 33, 33, 33,
        ]);
        pub const DISPLAY_CHAT_PREVIEW: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 75, 78, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const ENTITY_EVENT: super::super::PacketId = super::super::PacketId([
            26, 26, 26, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 28, 28, 28, 27, 27, 27, 27, 27, 28,
            28, 28, 27, 27, 26, 26, 26, 27, 27, 27, 27, 24, 26, 25, 28, 28, 29, 29, 31, 31, 31, 31,
            30, 30, 30, 34, 34, 34, 34, 34,
        ]);
        pub const ENTITY_MOVEMENT: super::super::PacketId = super::super::PacketId([
            20, 20, 20, 40, 40, 40, 40, 40, 40, 40, 37, 37, 37, 39, 39, 39, 43, 43, 43, 43, 43, 44,
            44, 44, 43, 43, 42, 42, 42, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const ENTITY_POSITION_SYNC: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 32, 32,
            31, 31, 31, 35, 35, 35, 35, 35,
        ]);
        pub const EXPLODE: super::super::PacketId = super::super::PacketId([
            39, 39, 39, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 30, 30, 30, 28, 28, 28, 28, 28, 29,
            29, 29, 28, 28, 27, 27, 27, 28, 28, 28, 28, 25, 27, 26, 29, 29, 30, 30, 32, 32, 33, 33,
            32, 32, 32, 36, 36, 36, 36, 36,
        ]);
        pub const FORGET_LEVEL_CHUNK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 31, 31, 31, 29, 29, 29, 29, 29, 30,
            30, 30, 29, 29, 28, 28, 28, 29, 29, 29, 29, 26, 28, 27, 30, 30, 31, 31, 33, 33, 34, 34,
            33, 33, 33, 37, 37, 37, 37, 38,
        ]);
        pub const GAME_EVENT: super::super::PacketId = super::super::PacketId([
            43, 43, 43, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 32, 32, 32, 30, 30, 30, 30, 30, 31,
            31, 31, 30, 30, 29, 29, 29, 30, 30, 30, 30, 27, 29, 28, 31, 31, 32, 32, 34, 34, 35, 35,
            34, 34, 34, 38, 38, 38, 38, 39,
        ]);
        pub const GAME_RULE_VALUES: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, 39, 39, 40,
        ]);
        pub const GAME_TEST_HIGHLIGHT_POS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, 39, 39, 40, 40, 41,
        ]);
        pub const HURT_ANIMATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 33, 33, 34, 34, 36, 36, 37, 37,
            36, 36, 36, 41, 41, 42, 42, 43,
        ]);
        pub const INITIALIZE_BORDER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 32, 32, 32, 32, 29, 31, 30, 34, 34, 35, 35, 37, 37, 38, 38,
            37, 37, 37, 42, 42, 43, 43, 44,
        ]);
        pub const KEEP_ALIVE: super::super::PacketId = super::super::PacketId([
            0, 0, 0, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 33, 33, 33, 32, 32, 32, 32, 32, 33,
            33, 33, 32, 32, 31, 31, 31, 33, 33, 33, 33, 30, 32, 31, 35, 35, 36, 36, 38, 38, 39, 39,
            38, 38, 38, 43, 43, 44, 44, 45,
        ]);
        pub const LEVEL_CHUNK_WITH_LIGHT: super::super::PacketId = super::super::PacketId([
            33, 33, 33, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 34, 34, 34, 33, 33, 33, 33, 33, 34,
            34, 34, 33, 33, 32, 32, 32, 34, 34, 34, 34, 31, 33, 32, 36, 36, 37, 37, 39, 39, 40, 40,
            39, 39, 39, 44, 44, 45, 45, 46,
        ]);
        pub const LEVEL_EVENT: super::super::PacketId = super::super::PacketId([
            40, 40, 40, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 35, 35, 35, 34, 34, 34, 34, 34, 35,
            35, 35, 34, 34, 33, 33, 33, 35, 35, 35, 35, 32, 34, 33, 37, 37, 38, 38, 40, 40, 41, 41,
            40, 40, 40, 45, 45, 46, 46, 47,
        ]);
        pub const LEVEL_PARTICLES: super::super::PacketId = super::super::PacketId([
            42, 42, 42, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 36, 36, 36, 35, 35, 35, 35, 35, 36,
            36, 36, 35, 35, 34, 34, 34, 36, 36, 36, 36, 33, 35, 34, 38, 38, 39, 39, 41, 41, 42, 42,
            41, 41, 41, 46, 46, 47, 47, 48,
        ]);
        pub const LIGHT_UPDATE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 36, 36, 36, 36, 36, 37,
            37, 37, 36, 36, 35, 35, 35, 37, 37, 37, 37, 34, 36, 35, 39, 39, 40, 40, 42, 42, 43, 43,
            42, 42, 42, 47, 47, 48, 48, 49,
        ]);
        pub const LOGIN: super::super::PacketId = super::super::PacketId([
            1, 1, 1, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 37, 37, 37, 37, 37, 37, 37, 37, 38,
            38, 38, 37, 37, 36, 36, 36, 38, 38, 38, 38, 35, 37, 36, 40, 40, 41, 41, 43, 43, 44, 44,
            43, 43, 43, 48, 48, 49, 49, 50,
        ]);
        pub const LOW_DISK_SPACE_WARNING: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, 50, 50, 51,
        ]);
        pub const MAP_CHUNK_BULK: super::super::PacketId = super::super::PacketId([
            38, 38, 38, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const MAP_ITEM_DATA: super::super::PacketId = super::super::PacketId([
            52, 52, 52, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 38, 38, 38, 38, 38, 38, 38, 38, 39,
            39, 39, 38, 38, 37, 37, 37, 39, 39, 39, 39, 36, 38, 37, 41, 41, 42, 42, 44, 44, 45, 45,
            44, 44, 44, 49, 49, 51, 51, 52,
        ]);
        pub const MERCHANT_OFFERS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 39, 39, 39, 39, 39, 40,
            40, 40, 39, 39, 38, 38, 38, 40, 40, 40, 40, 37, 39, 38, 42, 42, 43, 43, 45, 45, 46, 46,
            45, 45, 45, 50, 50, 52, 52, 53,
        ]);
        pub const MOUNT_SCREEN_OPEN: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 31, 31, 31, 31, 31, 32,
            32, 32, 31, 31, 30, 30, 30, 31, 31, 31, 31, 28, 30, 29, 32, 32, 33, 33, 35, 35, 36, 36,
            35, 35, 35, 40, 40, 41, 41, 42,
        ]);
        pub const MOVE_ENTITY_POS: super::super::PacketId = super::super::PacketId([
            21, 21, 21, 37, 37, 37, 37, 37, 37, 37, 38, 38, 38, 40, 40, 40, 40, 40, 40, 40, 40, 41,
            41, 41, 40, 40, 39, 39, 39, 41, 41, 41, 41, 38, 40, 39, 43, 43, 44, 44, 46, 46, 47, 47,
            46, 46, 46, 51, 51, 53, 53, 54,
        ]);
        pub const MOVE_ENTITY_POS_ROT: super::super::PacketId = super::super::PacketId([
            23, 23, 23, 38, 38, 38, 38, 38, 38, 38, 39, 39, 39, 41, 41, 41, 41, 41, 41, 41, 41, 42,
            42, 42, 41, 41, 40, 40, 40, 42, 42, 42, 42, 39, 41, 40, 44, 44, 45, 45, 47, 47, 48, 48,
            47, 47, 47, 52, 52, 54, 54, 55,
        ]);
        pub const MOVE_ENTITY_ROT: super::super::PacketId = super::super::PacketId([
            22, 22, 22, 39, 39, 39, 39, 39, 39, 39, 40, 40, 40, 42, 42, 42, 42, 42, 42, 42, 42, 43,
            43, 43, 42, 42, 41, 41, 41, 43, 43, 43, 43, 40, 42, 41, 45, 45, 46, 46, 48, 48, 50, 50,
            49, 49, 49, 54, 54, 56, 56, 57,
        ]);
        pub const MOVE_MINECART_ALONG_TRACK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 49, 49,
            48, 48, 48, 53, 53, 55, 55, 56,
        ]);
        pub const MOVE_PLAYER_ROT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 67, 67,
            66, 66, 66, 71, 71, 73, -1, -1,
        ]);
        pub const MOVE_VEHICLE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 43, 43, 43, 44, 44, 44, 44, 44, 45,
            45, 45, 44, 44, 43, 43, 43, 44, 44, 44, 44, 41, 43, 42, 46, 46, 47, 47, 49, 49, 51, 51,
            50, 50, 50, 55, 55, 57, 57, 58,
        ]);
        pub const NAMED_SOUND_EFFECT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 26, 26, 26, 25, 25, 25, 25, 25, 26,
            26, 26, 25, 25, 24, 24, 24, 25, 25, 25, 25, 22, 23, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const OPEN_BOOK: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 45, 45, 45, 45, 45, 46,
            46, 46, 45, 45, 44, 44, 44, 45, 45, 45, 45, 42, 44, 43, 47, 47, 48, 48, 50, 50, 52, 52,
            51, 51, 51, 56, 56, 58, 58, 59,
        ]);
        pub const OPEN_SCREEN: super::super::PacketId = super::super::PacketId([
            45, 45, 45, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 20, 20, 20, 46, 46, 46, 46, 46, 47,
            47, 47, 46, 46, 45, 45, 45, 46, 46, 46, 46, 43, 45, 44, 48, 48, 49, 49, 51, 51, 53, 53,
            52, 52, 52, 57, 57, 59, 59, 60,
        ]);
        pub const OPEN_SIGN_EDITOR: super::super::PacketId = super::super::PacketId([
            54, 54, 54, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 44, 44, 44, 47, 47, 47, 47, 47, 48,
            48, 48, 47, 47, 46, 46, 46, 47, 47, 47, 47, 44, 46, 45, 49, 49, 50, 50, 52, 52, 54, 54,
            53, 53, 53, 58, 58, 60, 60, 61,
        ]);
        pub const PING: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 48, 48, 48, 48, 45, 47, 46, 50, 50, 51, 51, 53, 53, 55, 55,
            54, 54, 54, 59, 59, 61, 61, 62,
        ]);
        pub const PLACE_GHOST_RECIPE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 43, 43, 45, 45, 45, 48, 48, 48, 48, 48, 49,
            49, 49, 48, 48, 47, 47, 47, 49, 49, 49, 49, 46, 48, 47, 51, 51, 53, 53, 55, 55, 57, 57,
            56, 56, 56, 61, 61, 63, 63, 64,
        ]);
        pub const PLAYER_ABILITIES: super::super::PacketId = super::super::PacketId([
            57, 57, 57, 43, 43, 43, 43, 43, 43, 43, 43, 44, 44, 46, 46, 46, 49, 49, 49, 49, 49, 50,
            50, 50, 49, 49, 48, 48, 48, 50, 50, 50, 50, 47, 49, 48, 52, 52, 54, 54, 56, 56, 58, 58,
            57, 57, 57, 62, 62, 64, 64, 65,
        ]);
        pub const PLAYER_CHAT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, 65, 66,
        ]);
        pub const PLAYER_CHAT_HEADER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 50, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const PLAYER_COMBAT_END: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 51, 51, 51, 51, 49, 52, 50, 54, 54, 56, 56, 58, 58, 60, 60,
            59, 59, 59, 64, 64, 66, 66, 67,
        ]);
        pub const PLAYER_COMBAT_ENTER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 52, 52, 52, 52, 50, 53, 51, 55, 55, 57, 57, 59, 59, 61, 61,
            60, 60, 60, 65, 65, 67, 67, 68,
        ]);
        pub const PLAYER_COMBAT_KILL: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 53, 53, 53, 53, 51, 54, 52, 56, 56, 58, 58, 60, 60, 62, 62,
            61, 61, 61, 66, 66, 68, 68, 69,
        ]);
        pub const PLAYER_INFO: super::super::PacketId = super::super::PacketId([
            56, 56, 56, 45, 45, 45, 45, 45, 45, 45, 45, 46, 46, 48, 48, 48, 51, 51, 51, 51, 51, 52,
            52, 52, 51, 51, 50, 50, 50, 54, 54, 54, 54, 52, 55, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const PLAYER_INFO_REMOVE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 53, 57, 57, 59, 59, 61, 61, 63, 63,
            62, 62, 62, 67, 67, 69, 69, 70,
        ]);
        pub const PLAYER_INFO_UPDATE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 54, 58, 58, 60, 60, 62, 62, 64, 64,
            63, 63, 63, 68, 68, 70, 70, 71,
        ]);
        pub const PLAYER_LOOK_AT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 49, 49, 49, 52, 52, 52, 52, 52, 53,
            53, 53, 52, 52, 51, 51, 51, 55, 55, 55, 55, 53, 56, 55, 59, 59, 61, 61, 63, 63, 65, 65,
            64, 64, 64, 69, 69, 71, 71, 72,
        ]);
        pub const PLAYER_POSITION: super::super::PacketId = super::super::PacketId([
            8, 8, 8, 46, 46, 46, 46, 46, 46, 46, 46, 47, 47, 50, 50, 50, 53, 53, 53, 53, 53, 54,
            54, 54, 53, 53, 52, 52, 52, 56, 56, 56, 56, 54, 57, 56, 60, 60, 62, 62, 64, 64, 66, 66,
            65, 65, 65, 70, 70, 72, 72, 73,
        ]);
        pub const PLAYER_ROTATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, 73, 74,
        ]);
        pub const PONG_RESPONSE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 52, 52, 54, 54, 56, 56,
            55, 55, 55, 60, 60, 62, 62, 63,
        ]);
        pub const POST_EFFECTS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 83,
        ]);
        pub const PROJECTILE_POWER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 121, 121, 128,
            128, 128, 128, 128, 133, 133, 135, 135, 138,
        ]);
        pub const RECIPE_BOOK_ADD: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 68, 68,
            67, 67, 67, 72, 72, 74, 74, 75,
        ]);
        pub const RECIPE_BOOK_REMOVE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 69, 69,
            68, 68, 68, 73, 73, 75, 75, 76,
        ]);
        pub const RECIPE_BOOK_SETTINGS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 70, 70,
            69, 69, 69, 74, 74, 76, 76, 77,
        ]);
        pub const REMOVE_ENTITIES: super::super::PacketId = super::super::PacketId([
            19, 19, 19, 48, 48, 48, 48, 48, 48, 48, 49, 50, 50, 53, 53, 53, 55, 55, 55, 55, 55, 56,
            56, 56, 55, 55, 54, 54, 54, 58, 58, 58, 58, 56, 59, 58, 62, 62, 64, 64, 66, 66, 71, 71,
            70, 70, 70, 75, 75, 77, 77, 78,
        ]);
        pub const REMOVE_MOB_EFFECT: super::super::PacketId = super::super::PacketId([
            30, 30, 30, 49, 49, 49, 49, 49, 49, 49, 50, 51, 51, 54, 54, 54, 56, 56, 56, 56, 56, 57,
            57, 57, 56, 56, 55, 55, 55, 59, 59, 59, 59, 57, 60, 59, 63, 63, 65, 65, 67, 67, 72, 72,
            71, 71, 71, 76, 76, 78, 78, 79,
        ]);
        pub const RESET_SCORE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 66, 68, 68, 73, 73,
            72, 72, 72, 77, 77, 79, 79, 80,
        ]);
        pub const RESOURCE_PACK_POP: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 67, 69, 69, 74, 74,
            73, 73, 73, 78, 78, 80, 80, 81,
        ]);
        pub const RESOURCE_PACK_PUSH: super::super::PacketId = super::super::PacketId([
            -1, -1, 72, 50, 50, 50, 50, 50, 50, 50, 51, 52, 52, 55, 55, 55, 57, 57, 57, 57, 57, 58,
            58, 58, 57, 57, 56, 56, 56, 60, 60, 60, 60, 58, 61, 60, 64, 64, 66, 68, 70, 70, 75, 75,
            74, 74, 74, 79, 79, 81, 81, 82,
        ]);
        pub const RESPAWN: super::super::PacketId = super::super::PacketId([
            7, 7, 7, 51, 51, 51, 51, 51, 51, 51, 52, 53, 53, 56, 56, 56, 58, 58, 58, 58, 58, 59,
            59, 59, 58, 58, 57, 57, 57, 61, 61, 61, 61, 59, 62, 61, 65, 65, 67, 69, 71, 71, 76, 76,
            75, 75, 75, 80, 80, 82, 82, 84,
        ]);
        pub const ROTATE_HEAD: super::super::PacketId = super::super::PacketId([
            25, 25, 25, 52, 52, 52, 52, 52, 52, 52, 53, 54, 54, 57, 57, 57, 59, 59, 59, 59, 59, 60,
            60, 60, 59, 59, 58, 58, 58, 62, 62, 62, 62, 60, 63, 62, 66, 66, 68, 70, 72, 72, 77, 77,
            76, 76, 76, 81, 81, 83, 83, 85,
        ]);
        pub const SCULK_VIBRATION_SIGNAL: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 5, 5, 5, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const SECTION_BLOCKS_UPDATE: super::super::PacketId = super::super::PacketId([
            34, 34, 34, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 15, 15, 15, 15, 15, 15, 15, 15, 16,
            16, 16, 15, 15, 59, 59, 59, 63, 63, 63, 63, 61, 64, 63, 67, 67, 69, 71, 73, 73, 78, 78,
            77, 77, 77, 82, 82, 84, 84, 86,
        ]);
        pub const SELECT_ADVANCEMENTS_TAB: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 54, 55, 55, 58, 58, 58, 60, 60, 60, 60, 60, 61,
            61, 61, 60, 60, 60, 60, 60, 64, 64, 64, 64, 62, 65, 64, 68, 68, 70, 72, 74, 74, 79, 79,
            78, 78, 78, 83, 83, 85, 85, 87,
        ]);
        pub const SERVER_DATA: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 63, 66, 65, 69, 69, 71, 73, 75, 75, 80, 80,
            79, 79, 79, 84, 84, 86, 86, 88,
        ]);
        pub const SERVER_LINKS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 123, 130,
            130, 130, 130, 130, 135, 135, 137, 137, 140,
        ]);
        pub const SET_ACTION_BAR_TEXT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 65, 65, 65, 65, 64, 67, 66, 70, 70, 72, 74, 76, 76, 81, 81,
            80, 80, 80, 85, 85, 87, 87, 89,
        ]);
        pub const SET_BORDER_CENTER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 66, 66, 66, 66, 65, 68, 67, 71, 71, 73, 75, 77, 77, 82, 82,
            81, 81, 81, 86, 86, 88, 88, 90,
        ]);
        pub const SET_BORDER_LERP_SIZE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 67, 67, 67, 67, 66, 69, 68, 72, 72, 74, 76, 78, 78, 83, 83,
            82, 82, 82, 87, 87, 89, 89, 91,
        ]);
        pub const SET_BORDER_SIZE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 68, 68, 68, 68, 67, 70, 69, 73, 73, 75, 77, 79, 79, 84, 84,
            83, 83, 83, 88, 88, 90, 90, 92,
        ]);
        pub const SET_BORDER_WARNING_DELAY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 69, 69, 69, 69, 68, 71, 70, 74, 74, 76, 78, 80, 80, 85, 85,
            84, 84, 84, 89, 89, 91, 91, 93,
        ]);
        pub const SET_BORDER_WARNING_DISTANCE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 70, 70, 70, 70, 69, 72, 71, 75, 75, 77, 79, 81, 81, 86, 86,
            85, 85, 85, 90, 90, 92, 92, 94,
        ]);
        pub const SET_CAMERA: super::super::PacketId = super::super::PacketId([
            -1, -1, 67, 54, 54, 54, 54, 54, 54, 54, 56, 57, 57, 60, 60, 60, 62, 62, 62, 62, 62, 63,
            63, 63, 62, 62, 62, 62, 62, 71, 71, 71, 71, 70, 73, 72, 76, 76, 78, 80, 82, 82, 87, 87,
            86, 86, 86, 91, 91, 93, 93, 95,
        ]);
        pub const SET_CARRIED_ITEM: super::super::PacketId = super::super::PacketId([
            9, 9, 9, 55, 55, 55, 55, 55, 55, 55, 57, 58, 58, 61, 61, 61, 63, 63, 63, 63, 63, 64,
            64, 64, 63, 63, 63, 63, 63, 72, 72, 72, 72, 71, 74, 73, 77, 77, 79, 81, 83, 83, 99, 99,
            98, 98, 98, 103, 103, 105, -1, -1,
        ]);
        pub const SET_CHUNK_CACHE_CENTER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 64, 64, 64, 64, 64, 65,
            65, 65, 64, 64, 64, 64, 64, 73, 73, 73, 73, 72, 75, 74, 78, 78, 80, 82, 84, 84, 88, 88,
            87, 87, 87, 92, 92, 94, 94, 96,
        ]);
        pub const SET_CHUNK_CACHE_RADIUS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 65, 65, 65, 65, 65, 66,
            66, 66, 65, 65, 65, 65, 65, 74, 74, 74, 74, 73, 76, 75, 79, 79, 81, 83, 85, 85, 89, 89,
            88, 88, 88, 93, 93, 95, 95, 97,
        ]);
        pub const SET_COMPRESSION: super::super::PacketId = super::super::PacketId([
            -1, -1, 70, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const SET_CURSOR_ITEM: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 90, 90,
            89, 89, 89, 94, 94, 96, 96, 98,
        ]);
        pub const SET_DEFAULT_SPAWN_POSITION: super::super::PacketId = super::super::PacketId([
            5, 5, 5, 67, 67, 67, 67, 67, 67, 67, 69, 70, 70, 73, 73, 73, 77, 77, 77, 77, 77, 78,
            78, 78, 77, 77, 66, 66, 66, 75, 75, 75, 75, 74, 77, 76, 80, 80, 82, 84, 86, 86, 91, 91,
            90, 90, 90, 95, 95, 97, 97, 99,
        ]);
        pub const SET_DISPLAY_OBJECTIVE: super::super::PacketId = super::super::PacketId([
            61, 61, 61, 56, 56, 56, 56, 56, 56, 56, 58, 59, 59, 62, 62, 62, 66, 66, 66, 66, 66, 67,
            67, 67, 66, 66, 67, 67, 67, 76, 76, 76, 76, 76, 79, 77, 81, 81, 83, 85, 87, 87, 92, 92,
            91, 91, 91, 96, 96, 98, 98, 100,
        ]);
        pub const SET_ENTITY_DATA: super::super::PacketId = super::super::PacketId([
            28, 28, 28, 57, 57, 57, 57, 57, 57, 57, 59, 60, 60, 63, 63, 63, 67, 67, 67, 67, 67, 68,
            68, 68, 67, 67, 68, 68, 68, 77, 77, 77, 77, 77, 80, 78, 82, 82, 84, 86, 88, 88, 93, 93,
            92, 92, 92, 97, 97, 99, 99, 101,
        ]);
        pub const SET_ENTITY_LINK: super::super::PacketId = super::super::PacketId([
            27, 27, 27, 58, 58, 58, 58, 58, 58, 58, 60, 61, 61, 64, 64, 64, 68, 68, 68, 68, 68, 69,
            69, 69, 68, 68, 69, 69, 69, 78, 78, 78, 78, 78, 81, 79, 83, 83, 85, 87, 89, 89, 94, 94,
            93, 93, 93, 98, 98, 100, 100, 102,
        ]);
        pub const SET_ENTITY_MOTION: super::super::PacketId = super::super::PacketId([
            18, 18, 18, 59, 59, 59, 59, 59, 59, 59, 61, 62, 62, 65, 65, 65, 69, 69, 69, 69, 69, 70,
            70, 70, 69, 69, 70, 70, 70, 79, 79, 79, 79, 79, 82, 80, 84, 84, 86, 88, 90, 90, 95, 95,
            94, 94, 94, 99, 99, 101, 101, 103,
        ]);
        pub const SET_EQUIPMENT: super::super::PacketId = super::super::PacketId([
            4, 4, 4, 60, 60, 60, 60, 60, 60, 60, 62, 63, 63, 66, 66, 66, 70, 70, 70, 70, 70, 71,
            71, 71, 70, 70, 71, 71, 71, 80, 80, 80, 80, 80, 83, 81, 85, 85, 87, 89, 91, 91, 96, 96,
            95, 95, 95, 100, 100, 102, 102, 104,
        ]);
        pub const SET_EXPERIENCE: super::super::PacketId = super::super::PacketId([
            31, 31, 31, 61, 61, 61, 61, 61, 61, 61, 63, 64, 64, 67, 67, 67, 71, 71, 71, 71, 71, 72,
            72, 72, 71, 71, 72, 72, 72, 81, 81, 81, 81, 81, 84, 82, 86, 86, 88, 90, 92, 92, 97, 97,
            96, 96, 96, 101, 101, 103, 103, 105,
        ]);
        pub const SET_HEALTH: super::super::PacketId = super::super::PacketId([
            6, 6, 6, 62, 62, 62, 62, 62, 62, 62, 64, 65, 65, 68, 68, 68, 72, 72, 72, 72, 72, 73,
            73, 73, 72, 72, 73, 73, 73, 82, 82, 82, 82, 82, 85, 83, 87, 87, 89, 91, 93, 93, 98, 98,
            97, 97, 97, 102, 102, 104, 104, 106,
        ]);
        pub const SET_HELD_SLOT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, 105, 107,
        ]);
        pub const SET_OBJECTIVE: super::super::PacketId = super::super::PacketId([
            59, 59, 59, 63, 63, 63, 63, 63, 63, 63, 65, 66, 66, 69, 69, 69, 73, 73, 73, 73, 73, 74,
            74, 74, 73, 73, 74, 74, 74, 83, 83, 83, 83, 83, 86, 84, 88, 88, 90, 92, 94, 94, 100,
            100, 99, 99, 99, 104, 104, 106, 106, 108,
        ]);
        pub const SET_PASSENGERS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, 64, 64, 64, 64, 64, 64, 64, 66, 67, 67, 70, 70, 70, 74, 74, 74, 74, 74, 75,
            75, 75, 74, 74, 75, 75, 75, 84, 84, 84, 84, 84, 87, 85, 89, 89, 91, 93, 95, 95, 101,
            101, 100, 100, 100, 105, 105, 107, 107, 109,
        ]);
        pub const SET_PLAYER_INVENTORY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 102,
            102, 101, 101, 101, 106, 106, 108, 108, 110,
        ]);
        pub const SET_PLAYER_TEAM: super::super::PacketId = super::super::PacketId([
            62, 62, 62, 65, 65, 65, 65, 65, 65, 65, 67, 68, 68, 71, 71, 71, 75, 75, 75, 75, 75, 76,
            76, 76, 75, 75, 76, 76, 76, 85, 85, 85, 85, 85, 88, 86, 90, 90, 92, 94, 96, 96, 103,
            103, 102, 102, 102, 107, 107, 109, 109, 111,
        ]);
        pub const SET_SCORE: super::super::PacketId = super::super::PacketId([
            60, 60, 60, 66, 66, 66, 66, 66, 66, 66, 68, 69, 69, 72, 72, 72, 76, 76, 76, 76, 76, 77,
            77, 77, 76, 76, 77, 77, 77, 86, 86, 86, 86, 86, 89, 87, 91, 91, 93, 95, 97, 97, 104,
            104, 103, 103, 103, 108, 108, 110, 110, 112,
        ]);
        pub const SET_SIMULATION_DISTANCE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, 87, 87, 87, 90, 88, 92, 92, 94, 96, 98, 98, 105,
            105, 104, 104, 104, 109, 109, 111, 111, 113,
        ]);
        pub const SET_SUBTITLE_TEXT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 87, 87, 88, 88, 88, 91, 89, 93, 93, 95, 97, 99, 99, 106,
            106, 105, 105, 105, 110, 110, 112, 112, 114,
        ]);
        pub const SET_TIME: super::super::PacketId = super::super::PacketId([
            3, 3, 3, 68, 68, 68, 68, 68, 68, 68, 70, 71, 71, 74, 74, 74, 78, 78, 78, 78, 78, 79,
            79, 79, 78, 78, 78, 78, 78, 88, 88, 89, 89, 89, 92, 90, 94, 94, 96, 98, 100, 100, 107,
            107, 106, 106, 106, 111, 111, 113, 113, 115,
        ]);
        pub const SET_TITLES_ANIMATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 90, 90, 91, 91, 91, 94, 92, 96, 96, 98, 100, 102, 102, 109,
            109, 108, 108, 108, 113, 113, 115, 115, 117,
        ]);
        pub const SET_TITLE_TEXT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 89, 89, 90, 90, 90, 93, 91, 95, 95, 97, 99, 101, 101, 108,
            108, 107, 107, 107, 112, 112, 114, 114, 116,
        ]);
        pub const SHOW_DIALOG: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 133, 133, 138, 138, 140, 140, 143,
        ]);
        pub const SIGN_UPDATE: super::super::PacketId = super::super::PacketId([
            51, 51, 51, 70, 70, 70, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const SOUND: super::super::PacketId = super::super::PacketId([
            41, 41, 41, 71, 71, 71, 70, 70, 70, 70, 72, 73, 73, 77, 77, 77, 81, 81, 81, 81, 81, 82,
            82, 82, 81, 81, 81, 81, 81, 92, 92, 93, 93, 93, 96, 94, 98, 98, 100, 102, 104, 104,
            111, 111, 110, 110, 110, 115, 115, 117, 117, 119,
        ]);
        pub const SOUND_ENTITY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 80, 80, 80, 80, 80, 81,
            81, 81, 80, 80, 80, 80, 80, 91, 91, 92, 92, 92, 95, 93, 97, 97, 99, 101, 103, 103, 110,
            110, 109, 109, 109, 114, 114, 116, 116, 118,
        ]);
        pub const SPAWN_EXPERIENCE_ORB: super::super::PacketId = super::super::PacketId([
            17, 17, 17, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const SPAWN_LIVING_ENTITY: super::super::PacketId = super::super::PacketId([
            15, 15, 15, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2,
            2, 2, 2, 2, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1,
        ]);
        pub const SPAWN_PAINTING: super::super::PacketId = super::super::PacketId([
            16, 16, 16, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3,
            3, 3, 3, 3, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1,
        ]);
        pub const SPAWN_PLAYER: super::super::PacketId = super::super::PacketId([
            12, 12, 12, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 2, 2, 2, 3, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const SPAWN_WEATHER_ENTITY: super::super::PacketId = super::super::PacketId([
            44, 44, 44, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1,
        ]);
        pub const START_CONFIGURATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 101, 103, 105, 105,
            112, 112, 111, 111, 111, 116, 116, 118, 118, 120,
        ]);
        pub const STOP_SOUND: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 76, 76, 76, 82, 82, 82, 82, 82, 83,
            83, 83, 82, 82, 82, 82, 82, 93, 93, 94, 94, 94, 97, 95, 99, 99, 102, 104, 106, 106,
            113, 113, 112, 112, 112, 117, 117, 119, 119, 121,
        ]);
        pub const STORE_COOKIE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 107, 107, 114,
            114, 113, 113, 113, 118, 118, 120, 120, 122,
        ]);
        pub const SWING_ANIMATION: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, 123,
        ]);
        pub const SYSTEM_CHAT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 95, 98, 96, 100, 100, 103, 105, 108, 108,
            115, 115, 114, 114, 114, 119, 119, 121, 121, 124,
        ]);
        pub const TAB_LIST: super::super::PacketId = super::super::PacketId([
            -1, -1, 71, 72, 72, 72, 71, 71, 71, 71, 73, 74, 74, 78, 78, 78, 83, 83, 83, 83, 83, 84,
            84, 84, 83, 83, 83, 83, 83, 94, 94, 95, 95, 96, 99, 97, 101, 101, 104, 106, 109, 109,
            116, 116, 115, 115, 115, 120, 120, 122, 122, 125,
        ]);
        pub const TAG_QUERY: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 29, 29, 29, 84, 84, 84, 84, 84, 85,
            85, 85, 84, 84, 84, 84, 84, 95, 95, 96, 96, 97, 100, 98, 102, 102, 105, 107, 110, 110,
            117, 117, 116, 116, 116, 121, 121, 123, 123, 126,
        ]);
        pub const TAKE_ITEM_ENTITY: super::super::PacketId = super::super::PacketId([
            13, 13, 13, 73, 73, 73, 72, 72, 72, 72, 74, 75, 75, 79, 79, 79, 85, 85, 85, 85, 85, 86,
            86, 86, 85, 85, 85, 85, 85, 96, 96, 97, 97, 98, 101, 99, 103, 103, 106, 108, 111, 111,
            118, 118, 117, 117, 117, 122, 122, 124, 124, 127,
        ]);
        pub const TELEPORT_ENTITY: super::super::PacketId = super::super::PacketId([
            24, 24, 24, 74, 74, 74, 73, 73, 73, 73, 75, 76, 76, 80, 80, 80, 86, 86, 86, 86, 86, 87,
            87, 87, 86, 86, 86, 86, 86, 97, 97, 98, 98, 99, 102, 100, 104, 104, 107, 109, 112, 112,
            119, 119, 118, 118, 118, 123, 123, 125, 125, 128,
        ]);
        pub const TEST_INSTANCE_BLOCK_STATUS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            119, 119, 119, 124, 124, 126, 126, 129,
        ]);
        pub const TICKING_STATE: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 110, 113, 113, 120,
            120, 120, 120, 120, 125, 125, 127, 127, 130,
        ]);
        pub const TICKING_STEP: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 111, 114, 114, 121,
            121, 121, 121, 121, 126, 126, 128, 128, 131,
        ]);
        pub const TITLE: super::super::PacketId = super::super::PacketId([
            -1, -1, 69, 69, 69, 69, 69, 69, 69, 69, 71, 72, 72, 75, 75, 75, 79, 79, 79, 79, 79, 80,
            80, 80, 79, 79, 79, 79, 79, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const TRANSFER: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 115, 115, 122,
            122, 122, 122, 122, 127, 127, 129, 129, 132,
        ]);
        pub const UNLOCK_RECIPES: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 48, 49, 49, 52, 52, 52, 54, 54, 54, 54, 54, 55,
            55, 55, 54, 54, 53, 53, 53, 57, 57, 57, 57, 55, 58, 57, 61, 61, 63, 63, 65, 65, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const UPDATE_ADVANCEMENTS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 76, 77, 77, 81, 81, 81, 87, 87, 87, 87, 87, 88,
            88, 88, 87, 87, 87, 87, 87, 98, 98, 99, 99, 100, 103, 101, 105, 105, 108, 112, 116,
            116, 123, 123, 123, 123, 123, 128, 128, 130, 130, 133,
        ]);
        pub const UPDATE_ATTRIBUTES: super::super::PacketId = super::super::PacketId([
            32, 32, 32, 75, 75, 75, 74, 74, 74, 74, 77, 78, 78, 82, 82, 82, 88, 88, 88, 88, 88, 89,
            89, 89, 88, 88, 88, 88, 88, 99, 99, 100, 100, 101, 104, 102, 106, 106, 109, 113, 117,
            117, 124, 124, 124, 124, 124, 129, 129, 131, 131, 134,
        ]);
        pub const UPDATE_ENABLED_FEATURES: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 103, 107, 107, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const UPDATE_ENTITY_NBT: super::super::PacketId = super::super::PacketId([
            -1, -1, 73, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const UPDATE_MOB_EFFECT: super::super::PacketId = super::super::PacketId([
            29, 29, 29, 76, 76, 76, 75, 75, 75, 75, 78, 79, 79, 83, 83, 83, 89, 89, 89, 89, 89, 90,
            90, 90, 89, 89, 89, 89, 89, 100, 100, 101, 101, 102, 105, 104, 108, 108, 110, 114, 118,
            118, 125, 125, 125, 125, 125, 130, 130, 132, 132, 135,
        ]);
        pub const UPDATE_RECIPES: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 84, 84, 84, 90, 90, 90, 90, 90, 91,
            91, 91, 90, 90, 90, 90, 90, 101, 101, 102, 102, 103, 106, 105, 109, 109, 111, 115, 119,
            119, 126, 126, 126, 126, 126, 131, 131, 133, 133, 136,
        ]);
        pub const UPDATE_TAGS: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 85, 85, 85, 91, 91, 91, 91, 91, 92,
            92, 92, 91, 91, 91, 91, 91, 102, 102, 103, 103, 104, 107, 106, 110, 110, 112, 116, 120,
            120, 127, 127, 127, 127, 127, 132, 132, 134, 134, 137,
        ]);
        pub const USE_BED: super::super::PacketId = super::super::PacketId([
            10, 10, 10, 47, 47, 47, 47, 47, 47, 47, 47, 48, 48, 51, 51, 51, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const WAYPOINT: super::super::PacketId = super::super::PacketId([
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 131, 131, 136, 136, 138, 138, 141,
        ]);
        pub const WINDOW_CONFIRMATION: super::super::PacketId = super::super::PacketId([
            50, 50, 50, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 18, 19,
            19, 19, 18, 18, 17, 17, 17, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
        pub const WORLD_BORDER: super::super::PacketId = super::super::PacketId([
            -1, -1, 68, 53, 53, 53, 53, 53, 53, 53, 55, 56, 56, 59, 59, 59, 61, 61, 61, 61, 61, 62,
            62, 62, 61, 61, 61, 61, 61, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ]);
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
static SERVERBOUND_0: [i16; 1] = [0];
static SERVERBOUND_1: [i16; 2] = [0, 1];
static SERVERBOUND_2: [i16; 3] = [0, 1, 2];
static SERVERBOUND_3: [i16; 5] = [0, 1, 2, 3, 4];
static SERVERBOUND_4: [i16; 0] = [];
static SERVERBOUND_5: [i16; 6] = [0, 2, 3, 4, 5, 6];
static SERVERBOUND_6: [i16; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
static SERVERBOUND_7: [i16; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
static SERVERBOUND_8: [i16; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
static SERVERBOUND_9: [i16; 24] = [
    28, 9, 26, 33, 30, 32, 31, 41, 66, 54, 46, 42, -1, 19, 18, -1, 57, 17, 62, 40, 15, 14, 12, 22,
];
static SERVERBOUND_10: [i16; 26] = [
    28, 9, 26, 33, 30, 32, 31, 41, 66, 54, 46, 42, -1, 19, 18, -1, 57, 17, 62, 40, 15, 14, 12, 22,
    64, 50,
];
static SERVERBOUND_11: [i16; 30] = [
    0, 15, 9, 12, 14, -1, 17, 18, 19, 22, 26, 28, 30, 31, 32, 33, 34, 35, 40, 41, 42, -1, 50, 54,
    57, 62, 46, 64, 66, 67,
];
static SERVERBOUND_12: [i16; 33] = [
    0, 39, 15, 9, 12, 14, -1, 17, 18, 19, 22, 26, 28, 33, 30, 31, 32, 34, 35, 40, 41, 42, -1, -1,
    50, 51, 54, 57, 62, 46, 64, 66, 67,
];
static SERVERBOUND_13: [i16; 33] = [
    0, 15, 9, 12, 14, -1, 17, 18, 19, 22, 26, 28, 33, 30, 31, 32, 34, 35, 39, 40, 41, 42, -1, -1,
    50, 51, 54, 57, 62, 46, 64, 66, 67,
];
static SERVERBOUND_14: [i16; 43] = [
    0, 2, 9, 12, 14, 15, -1, 17, 18, 19, 22, 24, 25, 26, 28, 33, 30, 31, 32, 34, 35, -1, 39, 40,
    41, 42, -1, -1, 49, 50, 51, 52, 53, 54, 55, 56, 57, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_15: [i16; 46] = [
    0, 2, 4, 9, 12, 14, 15, -1, 17, 18, 19, 22, 24, 25, 26, 28, 29, 30, 31, 32, 33, 34, 35, -1, 39,
    40, 41, 42, -1, -1, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_16: [i16; 47] = [
    0, 2, 4, 9, 12, 14, 15, -1, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, -1,
    39, 40, 41, 42, -1, -1, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_17: [i16; 48] = [
    0, 2, 4, 9, 12, 14, 15, -1, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, -1,
    39, 40, 41, 42, -1, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_18: [i16; 48] = [
    0, 2, 4, 9, 12, 14, 15, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, -1, 39,
    40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_19: [i16; 50] = [
    0, 2, 4, 7, 9, -1, 12, 14, 15, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
    -1, 39, 40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64, 66,
    67,
];
static SERVERBOUND_20: [i16; 51] = [
    0, 2, 4, 6, 7, 9, -1, 12, 14, 15, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
    35, -1, 39, 40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64,
    66, 67,
];
static SERVERBOUND_21: [i16; 51] = [
    0, 2, 4, 6, 7, 9, 12, 14, 15, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
    -1, 39, 40, 41, 42, -1, 45, 10, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64,
    66, 67,
];
static SERVERBOUND_22: [i16; 51] = [
    0, 2, 4, 6, 7, 9, 10, 12, 14, 15, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
    35, -1, 39, 40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 62, 46, 64,
    66, 67,
];
static SERVERBOUND_23: [i16; 54] = [
    0, 2, 4, 6, 7, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    33, 34, 35, -1, 38, 39, 40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60,
    62, 46, 64, 66, 67,
];
static SERVERBOUND_24: [i16; 55] = [
    0, 2, 4, 6, 7, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 20, 22, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, -1, 38, 39, 40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59,
    60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_25: [i16; 58] = [
    0, 2, 4, 6, -1, 7, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, -1, 24, 25, 26, 27, 28,
    29, 30, 31, 32, 33, 34, 35, -1, 38, 39, 40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_26: [i16; 58] = [
    0, 2, 4, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, -1, 24, 25, 26, 27, 28,
    29, 30, 31, 32, 33, 34, 35, -1, 38, 39, 40, 41, 42, -1, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_27: [i16; 60] = [
    0, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, -1, 24, 25, 26, 27,
    28, 29, 30, 31, 32, 33, 34, 35, -1, 38, 39, 40, 41, 42, 43, 45, 47, 48, 49, 50, 51, 52, 53, 54,
    55, 56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_28: [i16; 62] = [
    0, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, -1, 24, 25, 26, 27,
    28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 47, 48, 49, 50, 51, 52,
    53, 54, 55, 56, 57, 59, 60, 62, 46, 64, 66, 67,
];
static SERVERBOUND_29: [i16; 64] = [
    0, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, -1, 24, 25, 26, 27,
    28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 47, 48, 49, 50, 51, 52,
    53, 54, 55, 56, 57, 59, 60, 61, 62, 46, 64, 65, 66, 67,
];
static SERVERBOUND_30: [i16; 66] = [
    0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, -1, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 46, 64, 65, 66, 67, 68,
];
static SERVERBOUND_31: [i16; 66] = [
    0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 46, 64, 65, 66, 67, 68,
];
static SERVERBOUND_32: [i16; 69] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 46, 64, 65, 66, 67, 68,
];
static SERVERBOUND_33: [i16; 69] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68,
];
static CLIENTBOUND_0: [i16; 0] = [];
static CLIENTBOUND_1: [i16; 2] = [0, 1];
static CLIENTBOUND_2: [i16; 5] = [0, 1, 2, 3, 4];
static CLIENTBOUND_3: [i16; 6] = [0, 1, 2, 3, 4, 5];
static CLIENTBOUND_4: [i16; 15] = [-1, 0, 1, 2, 3, 4, -1, 5, -1, 6, -1, -1, -1, 7, 8];
static CLIENTBOUND_5: [i16; 15] = [-1, 0, 1, 2, 3, 4, -1, 5, 6, 7, -1, -1, -1, 8, 9];
static CLIENTBOUND_6: [i16; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, 10, 11, 12, 13, 14];
static CLIENTBOUND_7: [i16; 18] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, 10, 11, 12, 13, 14, 15, 16];
static CLIENTBOUND_8: [i16; 20] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, 10, 11, 12, 13, 14, 15, 16, 17, 18,
];
static CLIENTBOUND_9: [i16; 21] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
];
static CLIENTBOUND_10: [i16; 21] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];
static CLIENTBOUND_11: [i16; 136] = [
    -1, 14, 11, 55, -1, 37, 53, 36, 35, -1, -1, -1, -1, -1, -1, 58, -1, 46, 48, 49, 47, -1, -1, -1,
    63, -1, -1, -1, -1, -1, -1, -1, 64, -1, 26, -1, 39, -1, -1, 43, -1, -1, -1, -1, -1, 0, 33, 40,
    42, -1, 1, -1, 52, -1, 21, 23, -1, 22, -1, -1, 45, 54, -1, -1, -1, 57, 2, -1, -1, -1, -1, -1,
    -1, 8, -1, -1, -1, -1, 19, 30, -1, -1, -1, -1, 7, 25, 34, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 5, 61, 28, 27, 18, 4, 31, 6, 9, 59, -1, -1, 62, 60, -1, -1, 3, -1, -1, -1, 41, -1,
    -1, -1, -1, -1, -1, -1, 13, 24, -1, -1, -1, -1, -1, 32, 29,
];
static CLIENTBOUND_12: [i16; 136] = [
    -1, 14, 11, 55, -1, 37, 53, 36, 35, -1, 65, -1, -1, -1, -1, 58, -1, 46, 48, 49, 47, -1, -1, -1,
    63, -1, -1, -1, -1, -1, -1, -1, 64, -1, 26, -1, 39, -1, -1, 43, -1, -1, -1, -1, -1, 0, 33, 40,
    42, -1, 1, -1, 52, -1, 21, 23, -1, 22, -1, -1, 45, 54, -1, -1, -1, 57, 2, -1, -1, -1, -1, -1,
    -1, 8, -1, -1, -1, -1, 19, 30, -1, -1, 72, -1, 7, 25, 34, -1, -1, -1, -1, -1, -1, -1, -1, 67,
    -1, -1, -1, 5, 61, 28, 27, 18, 4, 31, 6, 9, 59, -1, -1, 62, 60, -1, -1, 3, -1, -1, -1, 41, -1,
    -1, -1, -1, -1, 71, -1, 13, 24, -1, -1, -1, -1, -1, 32, 29,
];
static CLIENTBOUND_13: [i16; 136] = [
    -1, 0, 6, 7, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 14, -1, 18, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, -1, -1, -1, 31, 32, 33, 34,
    -1, 35, -1, 36, -1, 37, 38, -1, 39, 41, -1, 19, 42, -1, -1, -1, 43, 15, -1, -1, -1, -1, -1, -1,
    46, -1, -1, -1, -1, 48, 49, -1, -1, 50, -1, 51, 52, 16, -1, -1, -1, -1, -1, -1, -1, -1, 54, -1,
    -1, -1, 67, 56, 57, 58, 59, 60, 61, 62, 55, 63, 64, -1, 65, 66, -1, -1, 68, -1, -1, -1, 71, -1,
    -1, -1, -1, -1, 72, -1, 73, 74, -1, -1, -1, -1, -1, 75, 76,
];
static CLIENTBOUND_14: [i16; 136] = [
    -1, 0, 6, 7, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 14, -1, 18, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, -1, -1, -1, 31, 32, 33, 34,
    -1, 35, -1, 36, -1, 37, 38, -1, 39, 41, -1, 19, 42, -1, -1, -1, 43, 15, -1, -1, -1, -1, -1, -1,
    46, -1, -1, -1, -1, 48, 49, -1, -1, 50, -1, 51, 52, 16, -1, -1, -1, -1, -1, -1, -1, -1, 54, -1,
    -1, -1, 67, 56, 57, 58, 59, 60, 61, 62, 55, 63, 64, -1, 65, 66, -1, -1, 68, -1, -1, -1, 70, -1,
    -1, -1, -1, -1, 71, -1, 72, 73, -1, -1, -1, -1, -1, 74, 75,
];
static CLIENTBOUND_15: [i16; 136] = [
    -1, 0, 6, 7, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 14, -1, 18, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, -1, -1, -1, 31, 32, 33, 34,
    -1, 35, -1, 36, -1, 38, 39, -1, 40, 41, -1, 19, 42, -1, -1, -1, 43, 15, -1, -1, -1, -1, -1, -1,
    46, -1, -1, -1, -1, 49, 50, -1, -1, 51, -1, 52, 53, 16, 54, -1, -1, -1, -1, -1, -1, -1, 56, -1,
    -1, -1, 69, 58, 59, 60, 61, 62, 63, 64, 57, 65, 66, -1, 67, 68, -1, -1, 70, -1, -1, -1, 72, -1,
    -1, -1, -1, -1, 73, -1, 74, 75, -1, -1, -1, -1, 76, 77, 78,
];
static CLIENTBOUND_16: [i16; 136] = [
    -1, 0, 6, 7, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 14, -1, 18, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, -1, -1, -1, 31, 32, 33, 34,
    -1, 35, -1, 36, -1, 38, 39, -1, 40, 41, -1, 19, 42, -1, -1, 43, 44, 15, -1, -1, -1, -1, -1, -1,
    47, -1, -1, -1, -1, 50, 51, -1, -1, 52, -1, 53, 54, 16, 55, -1, -1, -1, -1, -1, -1, -1, 57, -1,
    -1, -1, 70, 59, 60, 61, 62, 63, 64, 65, 58, 66, 67, -1, 68, 69, -1, -1, 71, -1, -1, -1, 73, -1,
    -1, -1, -1, -1, 74, -1, 75, 76, -1, -1, -1, -1, 77, 78, 79,
];
static CLIENTBOUND_17: [i16; 138] = [
    -1, 0, 6, 7, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 16, 17, 19, 21, 22, 23, -1, 24, -1, 25,
    -1, -1, -1, -1, -1, -1, -1, 27, -1, 28, -1, 30, -1, 31, 32, -1, -1, -1, -1, -1, 33, 34, 35, 36,
    -1, 37, -1, 38, -1, 40, 41, -1, 42, 43, -1, 20, 44, -1, -1, 45, 46, 14, -1, -1, -1, -1, -1, 49,
    50, -1, -1, -1, -1, 53, 54, -1, -1, 55, -1, 56, 57, 15, 58, -1, -1, -1, -1, -1, -1, -1, 60, -1,
    -1, -1, 73, 62, 63, 64, 65, 66, 67, 68, 61, 69, 70, -1, 71, 72, -1, -1, 74, -1, -1, -1, 77, -1,
    76, -1, -1, -1, 78, 29, 79, 80, -1, -1, -1, -1, 81, 82, 83, 84, 85,
];
static CLIENTBOUND_18: [i16; 138] = [
    -1, 0, 6, 7, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 16, 17, 19, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, 31, -1, -1, 32, 33, 34, 35,
    36, 37, -1, 38, 39, 40, 41, -1, 42, 44, 45, 46, 47, -1, -1, 48, 49, 14, -1, -1, -1, -1, -1, 52,
    53, -1, -1, -1, -1, 55, 56, -1, -1, 57, -1, 58, 59, 15, 60, -1, -1, -1, -1, -1, -1, -1, 62, 64,
    65, -1, 77, 66, 67, 68, 69, 70, 71, 72, 63, 73, 74, -1, 75, 76, -1, -1, 78, -1, -1, 80, 81, -1,
    82, -1, -1, -1, 83, 84, 85, 86, -1, -1, -1, -1, 87, 88, 89, 90, 91,
];
static CLIENTBOUND_19: [i16; 138] = [
    -1, 0, 6, 7, -1, 9, 10, 11, 12, 13, 14, -1, -1, -1, -1, 17, 18, 20, 21, 22, 23, -1, 24, -1, 25,
    -1, -1, -1, -1, -1, -1, -1, 27, -1, 28, -1, 29, -1, 30, 31, -1, -1, 32, -1, -1, 33, 34, 35, 36,
    37, 38, -1, 39, 40, 41, 42, -1, 43, 45, 46, 47, 48, -1, -1, 49, 50, 15, -1, -1, -1, -1, -1, 53,
    54, -1, -1, -1, -1, 56, 57, -1, -1, 58, -1, 59, 60, 16, 61, -1, -1, -1, -1, -1, -1, -1, 63, 65,
    66, -1, 78, 67, 68, 69, 70, 71, 72, 73, 64, 74, 75, -1, 76, 77, -1, -1, 79, -1, -1, 81, 82, -1,
    83, -1, -1, -1, 84, 85, 86, 87, -1, -1, -1, -1, 88, 89, 90, 91, 92,
];
static CLIENTBOUND_20: [i16; 138] = [
    -1, 0, 5, 6, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 16, 17, 19, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, 31, -1, -1, 32, 33, 34, 35,
    36, 37, -1, 38, 39, 40, 41, -1, 42, 44, 45, 46, 47, -1, -1, 48, 49, 14, -1, -1, -1, -1, -1, 52,
    53, -1, -1, -1, -1, 55, 56, -1, -1, 57, -1, 58, 59, 15, 60, -1, -1, -1, -1, -1, -1, -1, 62, 64,
    65, -1, 77, 66, 67, 68, 69, 70, 71, 72, 63, 73, 74, -1, 75, 76, -1, -1, 78, -1, -1, 80, 81, -1,
    82, -1, -1, -1, 83, 84, 85, 86, -1, -1, -1, -1, 87, 88, 89, 90, 91,
];
static CLIENTBOUND_21: [i16; 138] = [
    -1, 0, 5, 6, -1, 8, 9, 10, 11, 12, 13, -1, -1, -1, -1, 15, 16, 18, 19, 20, 21, -1, 22, -1, 23,
    -1, -1, -1, -1, -1, -1, -1, 25, -1, 26, -1, 27, -1, 28, 29, -1, -1, 30, -1, -1, 31, 32, 33, 34,
    35, 36, -1, 37, 38, 39, 40, -1, 41, 43, 44, 45, 46, -1, -1, 47, 48, 14, -1, -1, -1, -1, -1, 51,
    52, -1, -1, -1, -1, 54, 55, -1, -1, 56, -1, 57, 58, 59, 60, -1, -1, -1, -1, -1, -1, -1, 62, 64,
    65, -1, 66, 67, 68, 69, 70, 71, 72, 73, 63, 74, 75, -1, 76, 77, -1, -1, 78, -1, -1, 80, 81, -1,
    82, -1, -1, -1, 83, 84, 85, 86, -1, -1, -1, -1, 87, 88, 89, 90, 91,
];
static CLIENTBOUND_22: [i16; 138] = [
    -1, 0, 6, 7, -1, 9, 10, 11, 12, 13, 14, -1, -1, -1, 16, 17, 18, 19, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, 31, -1, 32, 33, 34, 35, 36,
    37, 38, -1, 39, 40, 41, 42, -1, 43, 44, 45, 46, 47, 48, -1, 49, 50, 15, 51, 52, 53, -1, -1, 55,
    56, -1, -1, -1, -1, 58, 59, -1, -1, 60, -1, 61, 62, 63, 64, -1, 65, 66, 67, 68, 69, 70, 71, 73,
    74, -1, 75, 76, 77, 78, 79, 80, 81, 82, 72, 83, 84, -1, 85, 86, -1, 87, 88, 89, 90, 91, 92, -1,
    93, -1, -1, -1, 94, 95, 96, 97, -1, -1, -1, -1, 98, 99, 100, 101, 102,
];
static CLIENTBOUND_23: [i16; 138] = [
    -1, 0, 6, 7, -1, 9, 10, 11, 12, 13, 14, -1, -1, -1, 16, 17, 18, 19, 20, 21, 22, -1, 23, -1, 24,
    -1, -1, -1, -1, -1, -1, -1, 26, -1, 27, -1, 28, -1, 29, 30, -1, -1, 31, -1, 32, 33, 34, 35, 36,
    37, 38, -1, 39, 40, 41, 42, -1, 43, 44, 45, 46, 47, 48, -1, 49, 50, 15, 51, 52, 53, -1, -1, 55,
    56, -1, -1, -1, -1, 58, 59, -1, -1, 60, -1, 61, 62, 63, 64, -1, 65, 66, 67, 68, 69, 70, 71, 73,
    74, -1, 75, 76, 77, 78, 79, 80, 81, 82, 72, 83, 84, -1, 85, 86, 87, 88, 89, 90, 91, 92, 93, -1,
    94, -1, -1, -1, 95, 96, 97, 98, -1, -1, -1, -1, 99, 100, 101, 102, 103,
];
static CLIENTBOUND_24: [i16; 138] = [
    -1, 0, 3, 4, 5, 6, 7, 8, 9, 10, 11, -1, -1, -1, 13, 14, 15, 16, 17, 18, 19, -1, 20, -1, 21, -1,
    -1, -1, -1, -1, -1, -1, 23, -1, 24, -1, 25, -1, 26, 27, -1, -1, 28, -1, 29, 30, 31, 32, 33, 34,
    35, -1, 36, 37, 38, 39, -1, 40, 41, 42, 43, 44, 45, -1, 46, 47, 48, 49, 50, 51, -1, -1, 53, 54,
    -1, -1, -1, -1, 56, 57, -1, -1, 58, -1, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 72, 73,
    -1, 74, 76, 77, 78, 79, 80, 81, 82, 71, 83, 84, -1, 85, 86, 87, 88, 89, 90, 91, 92, 93, -1, 94,
    -1, -1, 95, 96, 97, 98, 99, -1, -1, -1, -1, 100, 101, 102, 103, 104,
];
static CLIENTBOUND_25: [i16; 138] = [
    -1, 0, 3, 4, 5, 6, 7, 8, 9, 10, 11, -1, -1, -1, 13, 14, 15, 16, 17, 18, 19, -1, 20, 21, 22, -1,
    -1, -1, -1, -1, -1, 24, 25, -1, 26, -1, 27, -1, 28, 29, -1, -1, 30, -1, 31, 32, 33, 34, 35, 36,
    37, -1, 38, 39, 40, 41, -1, 42, 43, 44, 45, 46, 47, -1, 48, 49, 51, 52, 53, 54, -1, -1, 56, 57,
    -1, -1, -1, -1, 59, 60, -1, -1, 61, -1, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 75, 76,
    -1, 77, 79, 80, 81, 82, 83, 84, 85, 74, 86, 87, -1, 88, 89, 90, 91, 92, 93, 94, 95, 96, -1, 97,
    -1, -1, 98, 99, 100, 101, 102, -1, -1, -1, -1, 103, 104, 105, 106, 107,
];
static CLIENTBOUND_26: [i16; 138] = [
    -1, 0, 3, 4, 5, 6, 7, 8, 9, 10, 11, -1, -1, -1, 12, 13, 14, 15, 16, 17, 18, -1, 19, 20, 21, -1,
    -1, -1, -1, -1, -1, 22, 23, 24, 25, -1, 26, -1, 27, 28, -1, -1, 29, -1, 30, 31, 32, 33, 34, 35,
    36, -1, 37, 38, 39, 40, -1, 41, 42, 43, 44, 45, 46, -1, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
    -1, -1, -1, -1, 58, 59, -1, -1, 60, -1, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 74, 75,
    -1, 76, 77, 78, 79, 80, 81, 82, 83, 73, 84, 85, -1, 86, 87, 88, 89, 90, 91, 92, 93, 94, -1, 95,
    -1, -1, 96, 97, 98, 99, 100, -1, -1, -1, -1, 101, 102, 104, 105, 106,
];
static CLIENTBOUND_27: [i16; 138] = [
    0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, -1, -1, 13, 14, 15, 16, 17, 18, 19, 20, -1, 21, 22, 23, 24,
    -1, -1, -1, -1, -1, 25, 26, 27, 28, -1, 29, -1, 30, 31, -1, -1, 32, 33, 34, 35, 36, 37, 38, 39,
    40, -1, 41, 42, 43, 44, -1, 45, 46, 47, 48, 49, 50, -1, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60,
    -1, -1, -1, -1, 62, 63, -1, -1, 64, -1, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 78, 79,
    -1, 80, 81, 82, 83, 84, 85, 86, 87, 77, 88, 89, -1, 90, 91, 92, 93, 94, 95, 96, 97, 98, -1, 99,
    -1, -1, 100, 101, 102, 103, 104, -1, -1, -1, -1, 105, 106, 108, 109, 110,
];
static CLIENTBOUND_28: [i16; 138] = [
    0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, -1, 22, 23, 24, 25,
    -1, -1, -1, -1, -1, 26, 27, 28, 29, -1, 30, -1, 31, 32, -1, -1, 33, 34, 35, 36, 37, 38, 39, 40,
    41, -1, 42, 43, 44, 45, -1, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
    -1, -1, -1, -1, 64, 65, -1, -1, 66, -1, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 80, 81,
    -1, 82, 83, 84, 85, 86, 87, 88, 89, 79, 90, 91, -1, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101,
    102, -1, -1, 103, 104, 105, 106, 107, -1, -1, -1, -1, 108, 109, 110, 111, 112,
];
static CLIENTBOUND_29: [i16; 138] = [
    0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, -1, 22, 23, 24, 25,
    -1, -1, -1, -1, -1, 26, 27, 28, 29, -1, 30, -1, 31, 32, -1, -1, 33, 34, 35, 36, 37, 38, 39, 40,
    41, -1, 42, 43, 44, 45, -1, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
    -1, -1, -1, -1, 64, 65, 66, 67, 68, -1, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 82, 83,
    -1, 84, 85, 86, 87, 88, 89, 90, 91, 81, 92, 93, -1, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103,
    104, -1, -1, 105, 106, 107, 108, 109, -1, 110, 111, -1, 112, 113, 114, 115, 116,
];
static CLIENTBOUND_30: [i16; 139] = [
    0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    -1, -1, -1, -1, 27, 28, 29, 30, 31, -1, 32, -1, 33, 34, -1, -1, 35, 36, 37, 38, 39, 40, 41, 42,
    43, -1, 44, 45, 46, 47, -1, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
    -1, -1, -1, -1, 66, 67, 68, 69, 70, -1, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 84, 85,
    -1, 86, 87, 88, 89, 90, 91, 92, 93, 83, 94, 95, -1, 96, 97, 98, 99, 100, 101, 102, 103, 104,
    105, 106, 107, -1, 108, 109, 110, 111, 112, -1, 113, 114, 115, 116, 117, 118, 119, 120, 121,
];
static CLIENTBOUND_31: [i16; 141] = [
    0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    -1, -1, -1, -1, 27, 28, 29, 30, 31, -1, 32, -1, 33, 34, -1, -1, 35, 36, 37, 38, 39, 40, 41, 42,
    43, -1, 44, 45, 46, 47, -1, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
    -1, -1, -1, -1, 66, 67, 68, 69, 70, -1, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 84, 85,
    -1, 86, 87, 88, 89, 90, 91, 92, 93, 83, 94, 95, -1, 96, 97, 98, 99, 100, 101, 102, 103, 104,
    105, 106, 107, -1, 108, 109, 110, 111, 112, -1, 113, 114, 115, 116, 117, 118, 119, 120, 121,
    122, 123,
];
static CLIENTBOUND_32: [i16; 141] = [
    0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    -1, -1, -1, -1, 27, 28, 29, 30, 31, 32, 33, -1, 34, 35, -1, -1, 36, 37, 38, 39, 40, 41, 42, 43,
    44, -1, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, -1, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
    90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110,
    111, 112, 113, 114, -1, 115, 116, 117, 118, 119, -1, 120, 121, 122, 123, 124, 125, 126, 127,
    128, 129, 130,
];
static CLIENTBOUND_33: [i16; 141] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    -1, -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, -1, 33, 34, -1, -1, 35, 36, 37, 38, 39, 40, 41, 42,
    43, -1, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65,
    66, 67, 68, 69, 70, 71, 72, 73, 74, -1, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109,
    110, 111, 112, 113, -1, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
    128, 129, 130,
];
static CLIENTBOUND_34: [i16; 144] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    -1, -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, -1, 33, 34, -1, -1, 35, 36, 37, 38, 39, 40, 41, 42,
    43, -1, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65,
    66, 67, 68, 69, 70, 71, 72, 73, 74, -1, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109,
    110, 111, 112, 113, -1, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
    128, 129, 130, 131, 132, 133,
];
static CLIENTBOUND_35: [i16; 144] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, -1, 37, 38, -1, 39, 40, 41, 42, 43, 44, 45, 46, 47,
    48, -1, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, -1, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
    94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113,
    114, 115, 116, 117, 118, -1, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131,
    132, 133, 134, 135, 136, 137, 138,
];
static CLIENTBOUND_36: [i16; 144] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, -1, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
    49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72,
    73, 74, 75, 76, 77, 78, 79, 80, 81, -1, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, -1, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133,
    134, 135, 136, 137, 138, 139, 140,
];
static CLIENTBOUND_37: [i16; 144] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    136, 137, 138, 139, 140, 141, 142, 143,
];
#[doc = r" Client id to current id, per state and version."]
pub(super) static SERVERBOUND_TO_CURRENT: [[&[i16]; VERSIONS]; 5] = [
    [
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
        &SERVERBOUND_0,
    ],
    [
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
        &SERVERBOUND_1,
    ],
    [
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_2,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
        &SERVERBOUND_3,
    ],
    [
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_4,
        &SERVERBOUND_5,
        &SERVERBOUND_5,
        &SERVERBOUND_6,
        &SERVERBOUND_6,
        &SERVERBOUND_6,
        &SERVERBOUND_6,
        &SERVERBOUND_6,
        &SERVERBOUND_7,
        &SERVERBOUND_7,
        &SERVERBOUND_8,
        &SERVERBOUND_8,
        &SERVERBOUND_8,
        &SERVERBOUND_8,
        &SERVERBOUND_8,
    ],
    [
        &SERVERBOUND_9,
        &SERVERBOUND_9,
        &SERVERBOUND_10,
        &SERVERBOUND_11,
        &SERVERBOUND_11,
        &SERVERBOUND_11,
        &SERVERBOUND_11,
        &SERVERBOUND_11,
        &SERVERBOUND_11,
        &SERVERBOUND_11,
        &SERVERBOUND_12,
        &SERVERBOUND_13,
        &SERVERBOUND_13,
        &SERVERBOUND_14,
        &SERVERBOUND_14,
        &SERVERBOUND_14,
        &SERVERBOUND_15,
        &SERVERBOUND_15,
        &SERVERBOUND_15,
        &SERVERBOUND_15,
        &SERVERBOUND_15,
        &SERVERBOUND_15,
        &SERVERBOUND_15,
        &SERVERBOUND_15,
        &SERVERBOUND_16,
        &SERVERBOUND_16,
        &SERVERBOUND_17,
        &SERVERBOUND_17,
        &SERVERBOUND_17,
        &SERVERBOUND_18,
        &SERVERBOUND_18,
        &SERVERBOUND_18,
        &SERVERBOUND_18,
        &SERVERBOUND_19,
        &SERVERBOUND_20,
        &SERVERBOUND_21,
        &SERVERBOUND_22,
        &SERVERBOUND_22,
        &SERVERBOUND_23,
        &SERVERBOUND_24,
        &SERVERBOUND_25,
        &SERVERBOUND_26,
        &SERVERBOUND_27,
        &SERVERBOUND_28,
        &SERVERBOUND_29,
        &SERVERBOUND_30,
        &SERVERBOUND_30,
        &SERVERBOUND_31,
        &SERVERBOUND_31,
        &SERVERBOUND_32,
        &SERVERBOUND_32,
        &SERVERBOUND_33,
    ],
];
#[doc = r" Current id to client id, per state and version."]
pub(super) static CLIENTBOUND_FROM_CURRENT: [[&[i16]; VERSIONS]; 5] = [
    [
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
    ],
    [
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
        &CLIENTBOUND_1,
    ],
    [
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_2,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
        &CLIENTBOUND_3,
    ],
    [
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_0,
        &CLIENTBOUND_4,
        &CLIENTBOUND_5,
        &CLIENTBOUND_6,
        &CLIENTBOUND_7,
        &CLIENTBOUND_7,
        &CLIENTBOUND_7,
        &CLIENTBOUND_7,
        &CLIENTBOUND_8,
        &CLIENTBOUND_8,
        &CLIENTBOUND_9,
        &CLIENTBOUND_9,
        &CLIENTBOUND_9,
        &CLIENTBOUND_9,
        &CLIENTBOUND_10,
    ],
    [
        &CLIENTBOUND_11,
        &CLIENTBOUND_11,
        &CLIENTBOUND_12,
        &CLIENTBOUND_13,
        &CLIENTBOUND_13,
        &CLIENTBOUND_13,
        &CLIENTBOUND_14,
        &CLIENTBOUND_14,
        &CLIENTBOUND_14,
        &CLIENTBOUND_14,
        &CLIENTBOUND_15,
        &CLIENTBOUND_16,
        &CLIENTBOUND_16,
        &CLIENTBOUND_17,
        &CLIENTBOUND_17,
        &CLIENTBOUND_17,
        &CLIENTBOUND_18,
        &CLIENTBOUND_18,
        &CLIENTBOUND_18,
        &CLIENTBOUND_18,
        &CLIENTBOUND_18,
        &CLIENTBOUND_19,
        &CLIENTBOUND_19,
        &CLIENTBOUND_19,
        &CLIENTBOUND_20,
        &CLIENTBOUND_20,
        &CLIENTBOUND_21,
        &CLIENTBOUND_21,
        &CLIENTBOUND_21,
        &CLIENTBOUND_22,
        &CLIENTBOUND_22,
        &CLIENTBOUND_23,
        &CLIENTBOUND_23,
        &CLIENTBOUND_24,
        &CLIENTBOUND_25,
        &CLIENTBOUND_26,
        &CLIENTBOUND_27,
        &CLIENTBOUND_27,
        &CLIENTBOUND_28,
        &CLIENTBOUND_29,
        &CLIENTBOUND_30,
        &CLIENTBOUND_31,
        &CLIENTBOUND_32,
        &CLIENTBOUND_32,
        &CLIENTBOUND_33,
        &CLIENTBOUND_34,
        &CLIENTBOUND_34,
        &CLIENTBOUND_35,
        &CLIENTBOUND_35,
        &CLIENTBOUND_36,
        &CLIENTBOUND_36,
        &CLIENTBOUND_37,
    ],
];
