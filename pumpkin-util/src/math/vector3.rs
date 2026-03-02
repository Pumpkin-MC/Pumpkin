use bytes::BufMut;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use num_traits::{Float, Num};

use super::position::BlockPos;
use super::vector2::Vector2;

/// A 3-dimensional vector with components of type `T`.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, Default)]
pub struct Vector3<T> {
    /// The X component of the vector.
    pub x: T,
    /// The Y component of the vector.
    pub y: T,
    /// The Z component of the vector.
    pub z: T,
}

/// Represents a primary axis in 3D space.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Axis {
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
}

impl Axis {
    /// Returns all three axes `[Y, X, Z]`.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Y, Self::X, Self::Z]
    }

    /// Returns horizontal axes `[X, Z]`.
    #[must_use]
    pub const fn horizontal() -> [Self; 2] {
        [Self::X, Self::Z]
    }

    /// Returns the two axes excluding the given `axis`.
    ///
    /// # Arguments
    /// - `axis` – The axis to exclude.
    ///
    /// # Returns
    /// An array containing the two axes that are not the given axis.
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
    /// Gets the component value for the specified axis.
    ///
    /// # Arguments
    /// - `a` – The axis to get the value for.
    ///
    /// # Returns
    /// The component value at the specified axis.
    pub const fn get_axis(&self, a: Axis) -> T {
        match a {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    /// Sets the component value for the specified axis.
    ///
    /// # Arguments
    /// - `a` – The axis to set the value for.
    /// - `value` – The new value for the axis.
    pub const fn set_axis(&mut self, a: Axis, value: T) {
        match a {
            Axis::X => self.x = value,
            Axis::Y => self.y = value,
            Axis::Z => self.z = value,
        }
    }
}

impl<T: Math + PartialOrd + Copy> Vector3<T> {
    /// Creates a new `Vector3` with the given components.
    ///
    /// # Arguments
    /// - `x` – The X component.
    /// - `y` – The Y component.
    /// - `z` – The Z component.
    ///
    /// # Returns
    /// A new `Vector3` with the specified components.
    #[must_use]
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// Calculates the squared length (magnitude) of the vector.
    ///
    /// # Returns
    /// The squared length of the vector.
    #[must_use]
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Calculates the squared horizontal length (XZ components only).
    ///
    /// # Returns
    /// The squared horizontal length of the vector.
    #[must_use]
    pub fn horizontal_length_squared(&self) -> T {
        self.x * self.x + self.z * self.z
    }

    /// Adds another vector to this one and returns the result.
    ///
    /// # Arguments
    /// - `other` – The vector to add.
    ///
    /// # Returns
    /// A new vector representing the component-wise sum.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Adds raw component values to this vector and returns the result.
    ///
    /// # Arguments
    /// - `x` – The X value to add.
    /// - `y` – The Y value to add.
    /// - `z` – The Z value to add.
    ///
    /// # Returns
    /// A new vector with the raw values added to each component.
    #[must_use]
    pub fn add_raw(&self, x: T, y: T, z: T) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            z: self.z + z,
        }
    }

    /// Subtracts another vector from this one and returns the result.
    ///
    /// # Arguments
    /// - `other` – The vector to subtract.
    ///
    /// # Returns
    /// A new vector representing the component-wise difference.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// Subtracts raw component values from this vector and returns the result.
    ///
    /// # Arguments
    /// - `x` – The X value to subtract.
    /// - `y` – The Y value to subtract.
    /// - `z` – The Z value to subtract.
    ///
    /// # Returns
    /// A new vector with the raw values subtracted from each component.
    #[must_use]
    pub fn sub_raw(&self, x: T, y: T, z: T) -> Self {
        Self {
            x: self.x - x,
            y: self.y - y,
            z: self.z - z,
        }
    }

    /// Multiplies this vector by raw component values and returns the result.
    ///
    /// # Arguments
    /// - `x` – The X multiplier.
    /// - `y` – The Y multiplier.
    /// - `z` – The Z multiplier.
    ///
    /// # Returns
    /// A new vector with each component multiplied by its corresponding raw value.
    #[must_use]
    pub fn multiply(self, x: T, y: T, z: T) -> Self {
        Self {
            x: self.x * x,
            y: self.y * y,
            z: self.z * z,
        }
    }

    /// Performs linear interpolation between this vector and another.
    ///
    /// # Arguments
    /// - `other` – The target vector to interpolate toward.
    /// - `t` – The interpolation factor (0.0 = this vector, 1.0 = another vector).
    ///
    /// # Returns
    /// The interpolated vector at the given factor `t`.
    #[must_use]
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    /// Returns a vector with components -1, 0, or 1 indicating the sign of each component.
    ///
    /// # Returns
    /// A `Vector3<i32>` where each component is:
    /// - `1` if the original component is positive.
    /// - `-1` if the original component is negative.
    /// - `0` if the original component is zero.
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

    /// Calculates the squared distance between this vector and another vector.
    ///
    /// # Arguments
    /// - `other` – The other vector.
    ///
    /// # Returns
    /// The squared Euclidean distance between the two vectors.
    #[must_use]
    pub fn squared_distance_to_vec(&self, other: &Self) -> T {
        self.squared_distance_to(other.x, other.y, other.z)
    }

    /// Calculates the squared distance between this vector and the given coordinates.
    ///
    /// # Arguments
    /// - `x` – The X coordinate.
    /// - `y` – The Y coordinate.
    /// - `z` – The Z coordinate.
    ///
    /// # Returns
    /// The squared Euclidean distance to the specified coordinates.
    #[must_use]
    pub fn squared_distance_to(&self, x: T, y: T, z: T) -> T {
        let delta_x = self.x - x;
        let delta_y = self.y - y;
        let delta_z = self.z - z;
        delta_x * delta_x + delta_y * delta_y + delta_z * delta_z
    }

    /// Calculates the squared horizontal distance (XZ only) between this vector and another vector.
    ///
    /// # Arguments
    /// - `other` – The other vector.
    ///
    /// # Returns
    /// The squared horizontal distance between the two vectors.
    pub fn squared_distance_to_vec_xz(&self, other: Self) -> T {
        self.squared_distance_to_xz(other.x, other.z)
    }

    /// Calculates the squared horizontal distance (XZ only) between this vector and the given coordinates.
    ///
    /// # Arguments
    /// - `x` – The X coordinate.
    /// - `z` – The Z coordinate.
    ///
    /// # Returns
    /// The squared horizontal distance to the specified coordinates.
    pub fn squared_distance_to_xz(&self, x: T, z: T) -> T {
        let delta_x = self.x - x;
        let delta_z = self.z - z;
        delta_x * delta_x + delta_z * delta_z
    }

    /// Checks if this vector is within a cuboid region centered at the given position.
    ///
    /// # Arguments
    /// - `block_pos` – The centre position of the cuboid.
    /// - `x` – The half-width in the X direction.
    /// - `y` – The half-height in the Y direction.
    /// - `z` – The half-depth in the Z direction.
    ///
    /// # Returns
    /// `true` if the vector is within the bounds, `false` otherwise.
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
    /// Calculates the length (magnitude) of the vector.
    ///
    /// # Returns
    /// The length of the vector.
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// Calculates the horizontal length (size of XZ components) of the vector.
    ///
    /// # Returns
    /// The horizontal length of the vector.
    pub fn horizontal_length(&self) -> T {
        self.horizontal_length_squared().sqrt()
    }

    /// Returns a normalized version of this vector (unit vector).
    ///
    /// The resulting vector will have the same direction but a length of 1.
    ///
    /// # Returns
    /// A new unit vector pointing in the same direction.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let length = self.length();
        Self {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    /// Creates a direction vector from pitch and yaw angles.
    ///
    /// # Arguments
    /// - `pitch` – The pitch angle in degrees (up/down).
    /// - `yaw` – The yaw angle in degrees (left/right).
    ///
    /// # Returns
    /// A unit vector representing the direction.
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
    /// Converts this vector to a `Vector3<f64>`.
    ///
    /// # Returns
    /// A new `Vector3<f64>` with each component converted to `f64`.
    pub fn to_f64(&self) -> Vector3<f64> {
        Vector3 {
            x: self.x.into(),
            y: self.y.into(),
            z: self.z.into(),
        }
    }
}

impl<T: Math + Copy + Into<f32>> Vector3<T> {
    /// Converts this vector to a `Vector3<f32>`.
    ///
    /// # Returns
    /// A new `Vector3<f32>` with each component converted to `f32`.
    pub fn to_f32(&self) -> Vector3<f32> {
        Vector3 {
            x: self.x.into(),
            y: self.y.into(),
            z: self.z.into(),
        }
    }
}

impl<T: Math + Copy + Into<f64>> Vector3<T> {
    /// Rounds each component to the nearest integer and converts to `Vector3<i32>`.
    ///
    /// # Returns
    /// A new `Vector3<i32>` with each component rounded to the nearest integer.
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

    /// Rounds the X and Z components to the nearest integer and converts to a 2D vector.
    ///
    /// # Returns
    /// A new `Vector2<i32>` with the X and Z components rounded to the nearest integer.
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
    /// Floors each component to the nearest integer and converts to `Vector3<i32>`.
    ///
    /// # Returns
    /// A new `Vector3<i32>` with each component floored to the nearest integer.
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

    /// Floors the X and Z components to the nearest integer and converts to a 2D vector.
    ///
    /// # Returns
    /// A new `Vector2<i32>` with the X and Z components floored to the nearest integer.
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
    /// Converts this vector to a `BlockPos` by rounding each component to the nearest integer.
    ///
    /// # Returns
    /// A new `BlockPos` representing the rounded position.
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos(self.to_i32())
    }
}

/// A trait representing basic mathematical operations required for vector components.
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

/// Packs a chunk position vector into a single 64-bit integer.
///
/// The packing format is:
/// - Bits 42-63: X coordinate (22 bits)
/// - Bits 20-41: Z coordinate (22 bits)
/// - Bits 0-19: Y coordinate (20 bits)
///
/// # Arguments
/// - `vec` – The chunk position vector to pack.
///
/// # Returns
/// A packed 64-bit integer containing the encoded position.
#[inline]
#[must_use]
pub const fn packed_chunk_pos(vec: &Vector3<i32>) -> i64 {
    let mut result = 0i64;
    // NOTE: Need to go to i64 first to conserve a sign.
    result |= (vec.x as i64 & 0x03FF_FFFF) << 42;
    result |= (vec.z as i64 & 0x003F_FFFF) << 20;
    result |= vec.y as i64 & 0xFFFFF;
    result
}

/// Packs a local position within a chunk into a single 16-bit integer.
///
/// The packing format is:
/// - Bits 8-15: X coordinate (4 bits)
/// - Bits 4-7: Z coordinate (4 bits)
/// - Bits 0-3: Y coordinate (4 bits)
///
/// # Arguments
/// - `vec` – The local position vector to pack.
///
/// # Returns
/// A packed 16-bit integer containing the encoded position.
#[inline]
#[must_use]
pub const fn packed_local(vec: &Vector3<i32>) -> i16 {
    let x = vec.x as i16;
    let y = vec.y as i16;
    let z = vec.z as i16;
    (x << 8) | (z << 4) | y
}
