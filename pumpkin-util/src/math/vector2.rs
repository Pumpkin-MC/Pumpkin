use std::ops::{Add, Div, Mul, Neg, Sub};

use bytes::BufMut;
use num_traits::Float;

use super::vector3::Vector3;

/// A 2-dimensional vector with generic numeric components.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, Default)]
pub struct Vector2<T> {
    /// The X component of the vector.
    pub x: T,
    /// The Y component of the vector.
    pub y: T,
}

impl<T: Math + Copy> Vector2<T> {
    /// Creates a new vector with the given components.
    ///
    /// # Arguments
    /// * `x` – The X component.
    /// * `y` – The Y component.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Returns the squared length of the vector.
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    /// Returns the sum of this vector and another vector.
    ///
    /// # Arguments
    /// * `other` – The vector to add.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Adds raw component values to this vector.
    ///
    /// # Arguments
    /// * `x` – Value to add to X.
    /// * `y` – Value to add to Y.
    #[must_use]
    pub fn add_raw(&self, x: T, y: T) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
    }

    /// Returns the difference between this vector and another vector.
    ///
    /// # Arguments
    /// * `other` – The vector to subtract.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// Multiplies the vector by component-wise values.
    ///
    /// # Arguments
    /// * `x` – Multiplier for X.
    /// * `y` – Multiplier for Y.
    #[must_use]
    pub fn multiply(self, x: T, y: T) -> Self {
        Self {
            x: self.x * x,
            y: self.y * y,
        }
    }
}

impl<T: Math + Copy + Float> Vector2<T> {
    /// Returns the length (magnitude) of the vector.
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// Returns a normalized version of the vector with length 1.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let length = self.length();
        Self {
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
    type Output = Self;

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
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> From<(T, T)> for Vector2<T> {
    fn from((x, z): (T, T)) -> Self {
        Self { x, y: z }
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

/// Trait representing numeric types that support standard arithmetic operations.
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

/// Converts a block position vector to a chunk position vector.
///
/// # Arguments
/// * `vec` – The block position vector to convert.
///
/// # Returns
/// A `Vector2<i32>` representing the corresponding chunk position.
#[must_use]
pub const fn to_chunk_pos(vec: &Vector2<i32>) -> Vector2<i32> {
    Vector2::new(vec.x >> 4, vec.y >> 4)
}

impl serde::Serialize for Vector2<f32> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        buf.put_f32(self.x);
        buf.put_f32(self.y);
        serializer.serialize_bytes(&buf)
    }
}
