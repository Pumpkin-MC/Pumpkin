use std::ops::{Add, Div, Mul, Neg, Sub};

use bytes::BufMut;
use num_traits::Float;

use super::vector3::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, Default)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Math + Copy> Vector2<T> {
    pub const fn new(x: T, z: T) -> Self {
        Vector2 { x, y: z }
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn add(&self, other: &Vector2<T>) -> Self {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(&self, other: &Vector2<T>) -> Self {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn multiply(self, x: T, z: T) -> Self {
        Self {
            x: self.x * x,
            y: self.y * z,
        }
    }
}

impl<T: Math + Copy + Float> Vector2<T> {
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }
    pub fn normalize(&self) -> Self {
        let length = self.length();
        Vector2 {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

impl<T: Math + Copy> Mul<T> for Vector2<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T: Math + Copy> Add for Vector2<T> {
    type Output = Vector2<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Math + Copy> Neg for Vector2<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Vector2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> From<(T, T)> for Vector2<T> {
    fn from((x, z): (T, T)) -> Self {
        Vector2 { x, y: z }
    }
}

impl<T> From<Vector3<T>> for Vector2<T> {
    fn from(value: Vector3<T>) -> Self {
        Self {
            x: value.x,
            y: value.z,
        }
    }
}

pub trait Math:
    Mul<Output = Self>
    + Neg<Output = Self>
    + Add<Output = Self>
    + Div<Output = Self>
    + Sub<Output = Self>
    + Sized
{
}
impl Math for f64 {}
impl Math for f32 {}
impl Math for i32 {}
impl Math for i64 {}
impl Math for i8 {}

pub const fn to_chunk_pos(vec: &Vector2<i32>) -> Vector2<i32> {
    Vector2::new(vec.x >> 4, vec.y >> 4)
}

impl serde::Serialize for Vector2<f32> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Vec::with_capacity(size_of::<Vector2<f32>>());
        buf.put_f32(self.x);
        buf.put_f32(self.y);
        serializer.serialize_bytes(&buf)
    }
}
