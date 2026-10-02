//! Shared helpers for entities

/// Extra sampling on top of [`rand::RngExt`].
pub trait RandomExt: rand::RngExt {
    /// Triangular distribution centred on `center`.
    fn triangle(&mut self, center: f64, spread: f64) -> f64 {
        spread.mul_add(self.random::<f64>() - self.random::<f64>(), center)
    }
}

impl<T: rand::RngExt + ?Sized> RandomExt for T {}
