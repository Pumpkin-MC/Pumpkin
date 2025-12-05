use pumpkin_util::math::vector3::Vector3;

use crate::VarInt;
use serde::{Serialize, ser::Serializer};

#[derive(Clone, Copy)]
pub struct Velocity(pub Vector3<f64>);

const MAX_VELOCITY_CLAMP: f64 = 1.7179869183E10;
const MIN_VELOCITY_MAGNITUDE: f64 = 3.051944088384301E-5;
const MAX_15_BIT_VALUE: f64 = 32766.0;

fn clamp_value(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(-MAX_VELOCITY_CLAMP, MAX_VELOCITY_CLAMP)
}

fn abs_max(a: f64, b: f64) -> f64 {
    a.abs().max(b.abs())
}

fn to_long(value: f64) -> i64 {
    ((value * 0.5 + 0.5) * MAX_15_BIT_VALUE).round() as i64
}

impl Serialize for Velocity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        let velocity = self.0;
        let d = clamp_value(velocity.x);
        let e = clamp_value(velocity.y);
        let f = clamp_value(velocity.z);
        let g = abs_max(d, abs_max(e, f));

        if g < MIN_VELOCITY_MAGNITUDE {
            buf.push(0u8);
            return serializer.serialize_bytes(&buf);
        }

        // l is the scale factor
        let l = g.ceil() as i64;
        // bl is true if the scale factor is > 3 and needs VarInt extension
        let bl = l > 3;

        // m is the first byte of the packed data (low 3 bits of l + 4 if bl is true)
        let m = if bl { l & 3 | 4 } else { l };

        // Quantize and shift the velocity components (15 bits each)
        let n: i64 = to_long(d / l as f64) << 3; // X shifted 3
        let o: i64 = to_long(e / l as f64) << 18; // Y shifted 18
        let p: i64 = to_long(f / l as f64) << 33; // Z shifted 33

        // q is the packed 48-bit value (m + X + Y + Z)
        let q: i64 = m | n | o | p;

        // 1. Write first byte (low 8 bits)
        buf.push(q as u8);

        // 2. Write second byte (bits 8-15)
        buf.push((q >> 8) as u8);

        // 3. Write remaining 4 bytes (bits 16-47) as a Big Endian i32 (Java's writeInt)
        buf.extend_from_slice(&((q >> 16) as i32).to_be_bytes());

        // 4. Write VarInt for scale factor tail if needed
        if bl {
            VarInt::encode(&VarInt((l >> 2) as i32), &mut buf).unwrap();
        }

        serializer.serialize_bytes(&buf)
    }
}
