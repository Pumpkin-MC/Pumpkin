use pumpkin_util::math::vector3::Vector3;

use crate::{
    VarInt,
    ser::{NetworkWriteExt, WritingError},
};
use serde::{Serialize, ser::Serializer};

#[derive(Clone, Copy)]
pub struct Velocity(pub Vector3<f64>);

impl Velocity {
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), WritingError> {
        let velocity = self.0;
        let d = clamp_value(velocity.x);
        let e = clamp_value(velocity.y);
        let f = clamp_value(velocity.z);
        let g = abs_max(d, abs_max(e, f));

        if g < MIN_VELOCITY_MAGNITUDE {
            return writer.write_slice(&[0u8]);
        }

        let l = g.ceil() as i64;
        let bl = l > 3;

        // The header byte: bits 0-1 are scale, bit 2 is the extension flag
        let m = if bl { (l & 3) | 4 } else { l };

        // Pack the 15-bit quantized components into a 64-bit long
        // n (x): bits 3-17 | o (y): bits 18-32 | p (z): bits 33-47
        let n = to_long(d / l as f64) << 3;
        let o = to_long(e / l as f64) << 18;
        let p = to_long(f / l as f64) << 33;

        let packed_data: i64 = m | n | o | p;

        writer
            .write_all(&(packed_data as u16).to_le_bytes())
            .unwrap(); // Write low 16 bits
        writer
            .write_all(&((packed_data >> 16) as i32).to_be_bytes())
            .unwrap(); // Write next 32 bits

        if bl {
            let scale_tail = VarInt((l >> 2) as i32);
            writer.write_var_int(&scale_tail)?;
        }

        Ok(())
    }
}

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
    (((value * 0.5 + 0.5) * MAX_15_BIT_VALUE).round() as i64).clamp(0, 32766)
}

impl Serialize for Velocity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        self.write(&mut buf).unwrap();
        serializer.serialize_bytes(&buf)
    }
}
