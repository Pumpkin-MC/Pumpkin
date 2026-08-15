use std::fmt::Debug;

/// Interpolates between two values of an environment attribute.
pub trait Lerp<T>: Debug + Send + Sync {
    fn lerp(&self, t: f32, from: T, to: T) -> T;
}

#[derive(Clone, Copy)]
pub struct FnLerp<T> {
    op: fn(f32, T, T) -> T,
}

impl<T> FnLerp<T> {
    #[must_use]
    pub const fn new(op: fn(f32, T, T) -> T) -> Self {
        Self { op }
    }
}

impl<T> Debug for FnLerp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnLerp").finish_non_exhaustive()
    }
}

impl<T> Lerp<T> for FnLerp<T> {
    fn lerp(&self, t: f32, from: T, to: T) -> T {
        (self.op)(t, from, to)
    }
}
