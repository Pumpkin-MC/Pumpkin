pub trait TextComponentExt {
    fn from_bytes(data: &[u8]) -> pumpkin_util::text::TextComponent {
        postcard::from_bytes(data).expect("Failed to deserialize TextComponent")
    }

    fn to_bytes(&self) -> Vec<u8>;
}

impl TextComponentExt for pumpkin_util::text::TextComponent {
    fn to_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("Failed to serialize TextComponent")
    }
}
