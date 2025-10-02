use pumpkin_util::math::vector3::Vector3;

use crate::VarInt;
use serde::{Serialize, ser::Serializer};

#[derive(Clone, Copy)]
pub struct Velocity(pub Vector3<f64>);

fn clamp_value(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(-1.7179869183E10, 1.7179869183E10)
}

fn abs_max(a: f64, b: f64) -> f64 {
    a.abs().max(b.abs())
}

fn to_long(value: f64) -> i64 {
    ((value * 0.5 + 0.5) * 32766.0).round() as i64
}

impl Serialize for Velocity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        let velocity = self.0;
        let d = clamp_value(velocity.x);
        let e = clamp_value(velocity.y);
        let f = clamp_value(velocity.z);
        let g = abs_max(d, abs_max(e, f));

        if g < 0.00003051944088384301 {
            buf.push(0u8);
            return serializer.serialize_bytes(&buf);
        }

        let l = g.ceil() as i64;

        let bl = l > 3;

        let m = if bl { l & 3 | 4 } else { l };

        let n: i64 = to_long(d / l as f64) << 3;
        let o: i64 = to_long(e / l as f64) << 18;
        let p: i64 = to_long(f / l as f64) << 33;

        let q: i64 = m | n | o | p;

        buf.push(q as u8);
        buf.push((q >> 8) as u8);

        // This writes 4 bytes (an i32) for the remaining 48 bits of the i64 (q)
        // Note: The Java logic is slightly unusual here, using only 6 bytes total (1+1+4)
        // We write the remaining 32 bits of the 64-bit value 'q' as an i32 (4 bytes).
        // Since we already used 2 bytes (0-15), we need 4 more bytes (16-47)
        buf.extend_from_slice(&((q >> 16) as i32).to_le_bytes());

        if bl {
            VarInt::encode(&VarInt((l >> 2) as i32), &mut buf).unwrap();
        }
        serializer.serialize_bytes(&buf)
    }
}
