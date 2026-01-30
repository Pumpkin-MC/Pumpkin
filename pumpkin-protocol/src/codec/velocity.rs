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

        // Clamp and find the maximum component magnitude
        let clamped_x = clamp_value(velocity.x);
        let clamped_y = clamp_value(velocity.y);
        let clamped_z = clamp_value(velocity.z);
        let max_component = abs_max(clamped_x, abs_max(clamped_y, clamped_z));

        if max_component < MIN_VELOCITY_MAGNITUDE {
            return writer.write_slice(&[0u8]);
        }

        let scale_factor = max_component.ceil() as i64;
        let is_extended = scale_factor > 3;

        // The header byte: bits 0-1 are scale, bit 2 is the extension flag
        let header = if is_extended {
            (scale_factor & 3) | 4
        } else {
            scale_factor
        };

        // Pack the 15-bit quantized components into a 64-bit buffer
        // Quantized components: x (bits 3-17), y (bits 18-32), z (bits 33-47)
        let quantized_x = to_long(clamped_x / scale_factor as f64) << 3;
        let quantized_y = to_long(clamped_y / scale_factor as f64) << 18;
        let quantized_z = to_long(clamped_z / scale_factor as f64) << 33;

        let packed_data: i64 = header | quantized_x | quantized_y | quantized_z;

        // Write low 16 bits (Little Endian)
        writer
            .write_all(&(packed_data as u16).to_le_bytes())
            .unwrap();

        // Write next 32 bits (Big Endian)
        writer
            .write_all(&((packed_data >> 16) as i32).to_be_bytes())
            .unwrap();

        if is_extended {
            let scale_tail = VarInt((scale_factor >> 2) as i32);
            writer.write_var_int(&scale_tail)?;
        }

        Ok(())
    }
}

const MAX_VELOCITY_CLAMP: f64 = 1.717_986_918_3E10;
const MIN_VELOCITY_MAGNITUDE: f64 = 3.051_944_088_384_301E-5;
const MAX_15_BIT_VALUE: f64 = 32766.0;

fn clamp_value(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(-MAX_VELOCITY_CLAMP, MAX_VELOCITY_CLAMP)
}

const fn abs_max(a: f64, b: f64) -> f64 {
    a.abs().max(b.abs())
}

fn to_long(value: f64) -> i64 {
    ((value.mul_add(0.5, 0.5) * MAX_15_BIT_VALUE).round() as i64).clamp(0, 32766)
}

impl Serialize for Velocity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        self.write(&mut buf).unwrap();
        serializer.serialize_bytes(&buf)
    }
}
