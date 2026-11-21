use super::RandomImpl;

/// A trait extending `RandomImpl` with Gaussian (normal) distribution generation capabilities.
pub trait GaussianGenerator: RandomImpl {
    /// Returns the stored Gaussian value from a previous calculation, if available.
    ///
    /// # Returns
    /// The previously stored Gaussian value, or `None` if no value is stored.
    fn stored_next_gaussian(&self) -> Option<f64>;

    /// Sets the stored Gaussian value for the next call.
    ///
    /// # Arguments
    /// - `value` – The Gaussian value to store, or `None` to clear the storage.
    fn set_stored_next_gaussian(&mut self, value: Option<f64>);

    /// Generates the next Gaussian-distributed random value.
    ///
    /// # Returns
    /// A random value from a standard Gaussian (normal) distribution.
    fn calculate_gaussian(&mut self) -> f64 {
        if let Some(gaussian) = self.stored_next_gaussian() {
            self.set_stored_next_gaussian(None);
            gaussian
        } else {
            let (first, second) = sample_gaussian_pair(|| self.next_f64());
            self.set_stored_next_gaussian(Some(second));
            first
        }
    }
}

/// Samples two standard normal values from uniform draws in [0, 1).
#[must_use]
pub fn sample_gaussian_pair(mut uniform: impl FnMut() -> f64) -> (f64, f64) {
    loop {
        let d = uniform().mul_add(2.0, -1.0);
        let e = uniform().mul_add(2.0, -1.0);
        let f = d * d + e * e;

        if f < 1.0 && f != 0.0 {
            let g = (-2.0 * f.ln() / f).sqrt();
            return (d * g, e * g);
        }
    }
}
