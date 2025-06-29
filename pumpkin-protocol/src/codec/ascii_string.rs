use std::io::Write;

use bytes::BufMut;
use serde::{Serialize, Serializer};

pub struct AsciiString(pub String);

impl Serialize for AsciiString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = Vec::new();

        // Prefixed by a short
        buf.put_u16(self.0.len() as u16);
        buf.write_all(self.0.as_bytes()).unwrap();

        serializer.serialize_bytes(&buf)
    }
}
