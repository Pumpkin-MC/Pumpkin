const VERSION: u8 = 6;

pub const TRANSPORT_LAYER_NETHERNET: i32 = 2;
pub const CONNECTION_TYPE_LAN_SIGNALING: i32 = 4;

/// The application payload a `DiscoveryResponsePacket` carries, describing the
/// server on the client's LAN world list.
pub struct ServerData<'a> {
    pub server_name: &'a str,
    pub level_name: &'a str,
    pub game_type: i32,
    pub player_count: i32,
    pub max_player_count: i32,
    pub editor_world: bool,
    pub hardcore: bool,
    pub accepts_online_auth: bool,
    pub accepts_self_signed_auth: bool,
    /// Stable identifier of the world, sent as 16 lowercase hexadecimal digits.
    pub world_id: &'a str,
    pub transport_layer: i32,
    pub connection_type: i32,
}

impl ServerData<'_> {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(VERSION);
        write_string(&mut out, self.server_name);
        write_string(&mut out, self.level_name);
        write_var_int(&mut out, self.game_type);
        out.extend_from_slice(&self.player_count.to_le_bytes());
        out.extend_from_slice(&self.max_player_count.to_le_bytes());
        out.push(u8::from(self.editor_world));
        out.push(u8::from(self.hardcore));
        out.push(u8::from(self.accepts_online_auth));
        out.push(u8::from(self.accepts_self_signed_auth));
        write_string(&mut out, self.world_id);
        write_var_int(&mut out, self.transport_layer);
        write_var_int(&mut out, self.connection_type);
        out
    }
}

fn write_string(out: &mut Vec<u8>, value: &str) {
    write_var_uint(out, value.len() as u32);
    out.extend_from_slice(value.as_bytes());
}

fn write_var_int(out: &mut Vec<u8>, value: i32) {
    write_var_uint(out, ((value << 1) ^ (value >> 31)) as u32);
}

fn write_var_uint(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Byte-for-byte reproduction of a `1.26.40` dedicated server response.
    #[test]
    fn encodes_the_vanilla_layout() {
        let data = ServerData {
            server_name: "Dedicated Server",
            level_name: "Bedrock level",
            game_type: 0,
            player_count: 0,
            max_player_count: 10,
            editor_world: false,
            hardcore: false,
            accepts_online_auth: true,
            accepts_self_signed_auth: false,
            world_id: "b9b3c028566627f7",
            transport_layer: TRANSPORT_LAYER_NETHERNET,
            connection_type: CONNECTION_TYPE_LAN_SIGNALING,
        };
        assert_eq!(
            hex::encode(data.encode()),
            "0610446564696361746564205365727665720d426564726f636b206c6576656c\
             00000000000a0000000000010010623962336330323835363636323766370408"
        );
    }

    #[test]
    fn var_ints_are_zigzag_encoded() {
        let mut out = Vec::new();
        write_var_int(&mut out, -1);
        write_var_int(&mut out, 64);
        assert_eq!(out, vec![1, 128, 1]);
    }
}
