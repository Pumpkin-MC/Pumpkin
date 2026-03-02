pub mod perlin;
pub mod simplex;

/// A 3D gradient vector used for noise calculations.
pub struct Gradient {
    /// The X component of the gradient vector.
    x: f64,
    /// The Y component of the gradient vector.
    y: f64,
    /// The Z component of the gradient vector.
    z: f64,
}

/// A pre-computed set of 16 gradient vectors for 3D noise generation.
pub const GRADIENTS: [Gradient; 16] = [
    Gradient {
        x: 1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: -1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: 1f64,
        y: -1f64,
        z: 0f64,
    },
    Gradient {
        x: -1f64,
        y: -1f64,
        z: 0f64,
    },
    Gradient {
        x: 1f64,
        y: 0f64,
        z: 1f64,
    },
    Gradient {
        x: -1f64,
        y: 0f64,
        z: 1f64,
    },
    Gradient {
        x: 1f64,
        y: 0f64,
        z: -1f64,
    },
    Gradient {
        x: -1f64,
        y: 0f64,
        z: -1f64,
    },
    Gradient {
        x: 0f64,
        y: 1f64,
        z: 1f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: 1f64,
    },
    Gradient {
        x: 0f64,
        y: 1f64,
        z: -1f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: -1f64,
    },
    Gradient {
        x: 1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: 1f64,
    },
    Gradient {
        x: -1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: -1f64,
    },
];

impl Gradient {
    /// Computes the dot product of this gradient vector with the given coordinates.
    ///
    /// # Arguments
    /// - `x` – The X coordinate to dot with.
    /// - `y` – The Y coordinate to dot with.
    /// - `z` – The Z coordinate to dot with.
    ///
    /// # Returns
    /// The dot product `self.x * x + self.y * y + self.z * z`.
    #[inline]
    #[must_use]
    pub fn dot(&self, x: f64, y: f64, z: f64) -> f64 {
        self.z.mul_add(z, self.x.mul_add(x, self.y * y))
    }
}
