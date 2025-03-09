pub mod perlin;
pub mod simplex;

struct Gradient {
    x: f64,
    y: f64,
    z: f64,
}

const GRADIENTS: [Gradient; 16] = [
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
    #[inline]
    fn dot(&self, x: f64, y: f64, z: f64) -> f64 {
        self.x * x + self.y * y + self.z * z
    }
}
