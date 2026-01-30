use bytes::BufMut;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use num_traits::{Float, Num};

use super::position::BlockPos;
use super::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, Default)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]

pub enum Axis {
    X,

    Y,

    Z,
}

impl Axis {
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Y, Self::X, Self::Z]
    }

    #[must_use]
    pub const fn horizontal() -> [Self; 2] {
        [Self::X, Self::Z]
    }

    #[must_use]
    pub const fn excluding(axis: Self) -> [Self; 2] {
        match axis {
            Self::X => [Self::Y, Self::Z],

            Self::Y => [Self::X, Self::Z],

            Self::Z => [Self::X, Self::Y],
        }
    }
}

impl<T: Copy> Vector3<T> {
    pub const fn get_axis(&self, a: Axis) -> T {
        match a {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub const fn set_axis(&mut self, a: Axis, value: T) {
        match a {
            Axis::X => self.x = value,
            Axis::Y => self.y = value,
            Axis::Z => self.z = value,
        }
    }
}

impl<T: Math + PartialOrd + Copy> Vector3<T> {
    #[must_use]
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[must_use]
    pub fn horizontal_length_squared(&self) -> T {
        self.x * self.x + self.z * self.z
    }

    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    #[must_use]
    pub fn add_raw(&self, x: T, y: T, z: T) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            z: self.z + z,
        }
    }

    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    #[must_use]
    pub fn sub_raw(&self, x: T, y: T, z: T) -> Self {
        Self {
            x: self.x - x,
            y: self.y - y,
            z: self.z - z,
        }
    }

    #[must_use]
    pub fn multiply(self, x: T, y: T, z: T) -> Self {
        Self {
            x: self.x * x,
            y: self.y * y,
            z: self.z * z,
        }
    }

    #[must_use]
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    #[must_use]
    pub fn sign(&self) -> Vector3<i32>
    where
        T: Num + PartialOrd + Copy,
    {
        Vector3 {
            x: if self.x > T::zero() {
                1
            } else if self.x < T::zero() {
                -1
            } else {
                0
            },
            y: if self.y > T::zero() {
                1
            } else if self.y < T::zero() {
                -1
            } else {
                0
            },
            z: if self.z > T::zero() {
                1
            } else if self.z < T::zero() {
                -1
            } else {
                0
            },
        }
    }

    #[must_use]
    pub fn squared_distance_to_vec(&self, other: &Self) -> T {
        self.squared_distance_to(other.x, other.y, other.z)
    }

    #[must_use]
    pub fn squared_distance_to(&self, x: T, y: T, z: T) -> T {
        let delta_x = self.x - x;
        let delta_y = self.y - y;
        let delta_z = self.z - z;
        delta_x * delta_x + delta_y * delta_y + delta_z * delta_z
    }

    #[must_use]
    pub fn is_within_bounds(&self, block_pos: Self, x: T, y: T, z: T) -> bool {
        let min_x = block_pos.x - x;
        let max_x = block_pos.x + x;
        let min_y = block_pos.y - y;
        let max_y = block_pos.y + y;
        let min_z = block_pos.z - z;
        let max_z = block_pos.z + z;

        self.x >= min_x
            && self.x <= max_x
            && self.y >= min_y
            && self.y <= max_y
            && self.z >= min_z
            && self.z <= max_z
    }
}

impl<T: Math + Copy + Float> Vector3<T> {
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn horizontal_length(&self) -> T {
        self.horizontal_length_squared().sqrt()
    }

    #[must_use]
    pub fn normalize(&self) -> Self {
        let length = self.length();
        Self {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    pub fn rotation_vector(pitch: T, yaw: T) -> Self {
        let h = pitch.to_radians();
        let i = (-yaw).to_radians();

        let l = h.cos();
        Self {
            x: i.sin() * l,
            y: -h.sin(),
            z: i.cos() * l,
        }
    }
}

impl<T: Math + Copy> Mul<T> for Vector3<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl<T: Math + Copy> Add for Vector3<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: Math + Copy> Sub for Vector3<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: Math + Copy> AddAssign for Vector3<T> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

/*
impl<T: Math + Copy> Neg for Vector3<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
*/

impl<T> From<(T, T, T)> for Vector3<T> {
    fn from((x, y, z): (T, T, T)) -> Self {
        Self { x, y, z }
    }
}

impl<T> From<Vector3<T>> for (T, T, T) {
    fn from(vector: Vector3<T>) -> Self {
        (vector.x, vector.y, vector.z)
    }
}

impl<T: Math + Copy + Into<f64>> Vector3<T> {
    pub fn to_f64(&self) -> Vector3<f64> {
        Vector3 {
            x: self.x.into(),
            y: self.y.into(),
            z: self.z.into(),
        }
    }
}

impl<T: Math + Copy + Into<f32>> Vector3<T> {
    pub fn to_f32(&self) -> Vector3<f32> {
        Vector3 {
            x: self.x.into(),
            y: self.y.into(),
            z: self.z.into(),
        }
    }
}

impl<T: Math + Copy + Into<f64>> Vector3<T> {
    pub fn to_i32(&self) -> Vector3<i32> {
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();
        let z: f64 = self.z.into();
        Vector3 {
            x: x.round() as i32,
            y: y.round() as i32,
            z: z.round() as i32,
        }
    }

    pub fn to_vec2_i32(&self) -> Vector2<i32> {
        let x: f64 = self.x.into();
        let z: f64 = self.z.into();
        Vector2 {
            x: x.round() as i32,
            y: z.round() as i32,
        }
    }
}

impl<T: Math + Copy + Into<f64>> Vector3<T> {
    pub fn floor_to_i32(&self) -> Vector3<i32> {
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();
        let z: f64 = self.z.into();
        Vector3 {
            x: x.floor() as i32,
            y: y.floor() as i32,
            z: z.floor() as i32,
        }
    }

    pub fn floor_to_vec2_i32(&self) -> Vector2<i32> {
        let x: f64 = self.x.into();
        let z: f64 = self.z.into();
        Vector2 {
            x: x.floor() as i32,
            y: z.floor() as i32,
        }
    }
}

impl<T: Math + Copy + Into<f64>> Vector3<T> {
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos(self.to_i32())
    }
}

pub trait Math:
    Mul<Output = Self>
    //+ Neg<Output = Self>
    + Add<Output = Self>
    + AddAssign<>
    + Div<Output = Self>
    + Sub<Output = Self>
    + Sized
{
}
impl Math for i16 {}
impl Math for f64 {}
impl Math for f32 {}
impl Math for i32 {}
impl Math for i64 {}
impl Math for u8 {}

impl<'de> serde::Deserialize<'de> for Vector3<i32> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Vector3Visitor;

        impl<'de> serde::de::Visitor<'de> for Vector3Visitor {
            type Value = Vector3<i32>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Vector<i32>")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                if let Some(x) = seq.next_element::<i32>()?
                    && let Some(y) = seq.next_element::<i32>()?
                    && let Some(z) = seq.next_element::<i32>()?
                {
                    return Ok(Vector3::new(x, y, z));
                }
                Err(serde::de::Error::custom("Failed to read Vector<i32>"))
            }
        }

        deserializer.deserialize_seq(Vector3Visitor)
    }
}

impl<'de> serde::Deserialize<'de> for Vector3<f32> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Vector3Visitor;

        impl<'de> serde::de::Visitor<'de> for Vector3Visitor {
            type Value = Vector3<f32>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Vector<32>")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                if let Some(x) = seq.next_element::<f32>()?
                    && let Some(y) = seq.next_element::<f32>()?
                    && let Some(z) = seq.next_element::<f32>()?
                {
                    return Ok(Vector3::new(x, y, z));
                }
                Err(serde::de::Error::custom("Failed to read Vector<f32>"))
            }
        }

        deserializer.deserialize_seq(Vector3Visitor)
    }
}

impl<'de> serde::Deserialize<'de> for Vector3<f64> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Vector3Visitor;

        impl<'de> serde::de::Visitor<'de> for Vector3Visitor {
            type Value = Vector3<f64>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Vector<f64>")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                if let Some(x) = seq.next_element::<f64>()?
                    && let Some(y) = seq.next_element::<f64>()?
                    && let Some(z) = seq.next_element::<f64>()?
                {
                    return Ok(Vector3::new(x, y, z));
                }
                Err(serde::de::Error::custom("Failed to read Vector<f64>"))
            }
        }

        deserializer.deserialize_seq(Vector3Visitor)
    }
}

impl serde::Serialize for Vector3<f32> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        buf.put_f32(self.x);
        buf.put_f32(self.y);
        buf.put_f32(self.z);
        serializer.serialize_bytes(&buf)
    }
}

impl serde::Serialize for Vector3<f64> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        buf.put_f64(self.x);
        buf.put_f64(self.y);
        buf.put_f64(self.z);
        serializer.serialize_bytes(&buf)
    }
}

impl serde::Serialize for Vector3<i16> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        buf.put_i16(self.x);
        buf.put_i16(self.y);
        buf.put_i16(self.z);
        serializer.serialize_bytes(&buf)
    }
}

impl serde::Serialize for Vector3<i32> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        buf.put_i32(self.x);
        buf.put_i32(self.y);
        buf.put_i32(self.z);
        serializer.serialize_bytes(&buf)
    }
}

#[inline]
#[must_use]
pub const fn packed_chunk_pos(vec: &Vector3<i32>) -> i64 {
    let mut result = 0i64;
    // Need to go to i64 first to conserve sign
    result |= (vec.x as i64 & 0x03FF_FFFF) << 42;
    result |= (vec.z as i64 & 0x003F_FFFF) << 20;
    result |= vec.y as i64 & 0xFFFFF;
    result
}

#[inline]
#[must_use]
pub const fn packed_local(vec: &Vector3<i32>) -> i16 {
    let x = vec.x as i16;
    let y = vec.y as i16;
    let z = vec.z as i16;
    (x << 8) | (z << 4) | y
}
