pub trait TextComponentExt {
    fn from_bytes(data: &[u8]) -> pumpkin_util::text::TextComponent {
        serde_json::from_slice(data).expect("Failed to deserialize TextComponent")
    }

    fn to_bytes(&self) -> Vec<u8>;
}

impl TextComponentExt for pumpkin_util::text::TextComponent {
    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize TextComponent")
    }
}
