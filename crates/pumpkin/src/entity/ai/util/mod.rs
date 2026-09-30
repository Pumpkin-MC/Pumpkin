//! Helpers used by the goals to pick a random destination the navigation can reach.

pub mod air_and_water_random_pos;
pub mod default_random_pos;
pub mod goal_utils;
pub mod hover_random_pos;
pub mod land_random_pos;
pub mod random_pos;

/// Extra sampling the goals need on top of [`rand::RngExt`].
pub trait RandomExt: rand::RngExt {
    /// Triangular distribution centred on `center`.
    fn triangle(&mut self, center: f64, spread: f64) -> f64 {
        spread.mul_add(self.random::<f64>() - self.random::<f64>(), center)
    }
}

impl<T: rand::RngExt + ?Sized> RandomExt for T {}
