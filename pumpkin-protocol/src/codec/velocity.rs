use std::fmt::Formatter;

use pumpkin_util::math::vector3::Vector3;

use crate::{
    VarInt,
    ser::{NetworkWriteExt, ReadingError, WritingError},
};
use serde::{
    Deserializer, Serialize,
    de::{self, Visitor},
    ser::Serializer,
};

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

    pub fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, ReadingError> {
        let mut low_16 = [0u8; 2];
        reader
            .read_exact(&mut low_16)
            .map_err(|e| ReadingError::Message(e.to_string()))?;

        if low_16[0] == 0 && low_16[1] == 0 {
            return Ok(Self(Vector3::new(0.0, 0.0, 0.0)));
        }

        let mut mid_32 = [0u8; 4];
        reader
            .read_exact(&mut mid_32)
            .map_err(|e| ReadingError::Message(e.to_string()))?;

        let low = u16::from_le_bytes(low_16) as i64;
        let mid = i32::from_be_bytes(mid_32) as i64;
        let packed_data = low | (mid << 16);

        let header = packed_data & 0x07;
        let is_extended = (header & 4) != 0;

        let scale_factor = if is_extended {
            let scale_tail = VarInt::decode(reader)?;
            ((scale_tail.0 as i64) << 2) | (header & 3)
        } else {
            header & 3
        };

        if scale_factor == 0 && !is_extended {
            return Ok(Self(Vector3::new(0.0, 0.0, 0.0)));
        }

        let q_x = (packed_data >> 3) & 0x7FFF;
        let q_y = (packed_data >> 18) & 0x7FFF;
        let q_z = (packed_data >> 33) & 0x7FFF;

        let scale = scale_factor as f64;

        Ok(Self(Vector3::new(
            from_long(q_x, scale),
            from_long(q_y, scale),
            from_long(q_z, scale),
        )))
    }

    pub fn write_legacy<W: std::io::Write>(&self, writer: &mut W) -> Result<(), WritingError> {
        writer.write_i16_be(encode_legacy_velocity_component(self.0.x))?;
        writer.write_i16_be(encode_legacy_velocity_component(self.0.y))?;
        writer.write_i16_be(encode_legacy_velocity_component(self.0.z))?;
        Ok(())
    }
}

const MAX_VELOCITY_CLAMP: f64 = 1.717_986_918_3E10;
const LEGACY_COMPONENT_CLAMP: f64 = 3.9;
const LEGACY_COMPONENT_SCALE: f64 = 8000.0;
const MIN_VELOCITY_MAGNITUDE: f64 = 3.051_944_088_384_301E-5;
const MAX_15_BIT_VALUE: f64 = 32766.0;

#[must_use]
pub fn encode_legacy_velocity_component(component: f64) -> i16 {
    // Legacy clients (<= 1.21.8 / protocol 772) encode velocity as clamped component * 8000.
    (component.clamp(-LEGACY_COMPONENT_CLAMP, LEGACY_COMPONENT_CLAMP) * LEGACY_COMPONENT_SCALE)
        as i16
}

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

fn from_long(quantized: i64, scale: f64) -> f64 {
    // Reverse: ((v * 0.5 + 0.5) * 32766) -> v
    let normalized = (quantized as f64 / MAX_15_BIT_VALUE) - 0.5;
    (normalized / 0.5) * scale
}

impl Serialize for Velocity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        self.write(&mut buf).unwrap();
        serializer.serialize_bytes(&buf)
    }
}

impl<'de> de::Deserialize<'de> for Velocity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VelocityVisitor;

        impl Visitor<'_> for VelocityVisitor {
            type Value = Velocity;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array representing bit-packed velocity")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut cursor = std::io::Cursor::new(v);
                Velocity::read(&mut cursor).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_bytes(VelocityVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{Velocity, encode_legacy_velocity_component};
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn legacy_component_is_clamped_and_scaled() {
        assert_eq!(encode_legacy_velocity_component(0.5), 4000);
        assert_eq!(encode_legacy_velocity_component(-0.5), -4000);
        assert_eq!(encode_legacy_velocity_component(4.0), 31200);
        assert_eq!(encode_legacy_velocity_component(-4.0), -31200);
    }

    #[test]
    fn write_legacy_writes_three_i16_be_components() {
        let velocity = Velocity(Vector3::new(0.5, -0.5, 0.0));
        let mut buf = Vec::new();
        velocity.write_legacy(&mut buf).unwrap();
        assert_eq!(buf, vec![0x0F, 0xA0, 0xF0, 0x60, 0x00, 0x00]);
    }
}
