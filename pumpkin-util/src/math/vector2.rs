use std::ops::{Add, Div, Mul, Neg, Sub};

use num_traits::Float;

use super::vector3::{Vector3, SIZE_BITS_Y};

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, Default)]
pub struct Vector2<T> {
    pub x: T,
    pub z: T,
}

impl<T: Math + Copy> Vector2<T> {
    pub const fn new(x: T, z: T) -> Self {
        Vector2 { x, z }
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.z * self.z
    }

    pub fn add(&self, other: &Vector2<T>) -> Self {
        Vector2 {
            x: self.x + other.x,
            z: self.z + other.z,
        }
    }

    pub fn sub(&self, other: &Vector2<T>) -> Self {
        Vector2 {
            x: self.x - other.x,
            z: self.z - other.z,
        }
    }

    pub fn multiply(self, x: T, z: T) -> Self {
        Self {
            x: self.x * x,
            z: self.z * z,
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
            z: self.z / length,
        }
    }
}

impl<T: Math + Copy> Mul<T> for Vector2<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            z: self.z * scalar,
        }
    }
}

impl<T: Math + Copy> Add for Vector2<T> {
    type Output = Vector2<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            z: self.z + rhs.z,
        }
    }
}

impl<T: Math + Copy> Neg for Vector2<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Vector2 {
            x: -self.x,
            z: -self.z,
        }
    }
}

impl<T> From<(T, T)> for Vector2<T> {
    fn from((x, z): (T, T)) -> Self {
        Vector2 { x, z }
    }
}

impl<T> From<Vector3<T>> for Vector2<T> {
    fn from(value: Vector3<T>) -> Self {
        Self {
            x: value.x,
            z: value.z,
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

pub const MAX_HEIGHT: u32 = (1 << SIZE_BITS_Y) - 32;
pub const MAX_COLUMN_HEIGHT: u32 = (MAX_HEIGHT >> 1) - 1;
pub const MIN_HEIGHT: i32 = MAX_COLUMN_HEIGHT as i32 - MAX_HEIGHT as i32 + 1;
pub const MIN_HEIGHT_CELL: i32 = MIN_HEIGHT << 4;

pub const MARKER: u64 = packed(&Vector2::new(1875066, 1875066));

pub const fn packed(vec: &Vector2<i32>) -> u64 {
    (vec.x as u64 & 4294967295u64) | ((vec.z as u64 & 4294967295u64) << 32)
}

pub const fn unpack_x(packed: u64) -> i32 {
    (packed & 4294967295u64) as i32
}

pub const fn unpack_z(packed: u64) -> i32 {
    ((packed >> 32) & 4294967295u64) as i32
}

pub const fn start_block_x(vec: &Vector2<i32>) -> i32 {
    vec.x << 4
}

pub const fn end_block_x(vec: &Vector2<i32>) -> i32 {
    start_block_x(vec) + 15
}

pub const fn start_block_z(vec: &Vector2<i32>) -> i32 {
    vec.z << 4
}

pub const fn end_block_z(vec: &Vector2<i32>) -> i32 {
    start_block_z(vec) + 15
}

pub const fn to_chunk_pos(vec: &Vector2<i32>) -> Vector2<i32> {
    Vector2::new(vec.x >> 4, vec.z >> 4)
}
