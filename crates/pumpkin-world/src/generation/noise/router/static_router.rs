use pumpkin_util::math::vector3::Vector3;

/// Zero-cost static trait for monomorphized density function pipeline evaluation.
pub trait StaticDensityFunction {
    fn sample(&self, pos: &Vector3<i32>) -> f64;
}

#[derive(Clone, Copy, Debug)]
pub struct Constant(pub f64);

impl StaticDensityFunction for Constant {
    #[inline]
    fn sample(&self, _pos: &Vector3<i32>) -> f64 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct Add<A: StaticDensityFunction, B: StaticDensityFunction>(pub A, pub B);

impl<A: StaticDensityFunction, B: StaticDensityFunction> StaticDensityFunction for Add<A, B> {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        self.0.sample(pos) + self.1.sample(pos)
    }
}

#[derive(Clone, Debug)]
pub struct Mul<A: StaticDensityFunction, B: StaticDensityFunction>(pub A, pub B);

impl<A: StaticDensityFunction, B: StaticDensityFunction> StaticDensityFunction for Mul<A, B> {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        let v1 = self.0.sample(pos);
        if v1 == 0.0 {
            0.0
        } else {
            v1 * self.1.sample(pos)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Min<A: StaticDensityFunction, B: StaticDensityFunction>(pub A, pub B);

impl<A: StaticDensityFunction, B: StaticDensityFunction> StaticDensityFunction for Min<A, B> {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        self.0.sample(pos).min(self.1.sample(pos))
    }
}

#[derive(Clone, Debug)]
pub struct Max<A: StaticDensityFunction, B: StaticDensityFunction>(pub A, pub B);

impl<A: StaticDensityFunction, B: StaticDensityFunction> StaticDensityFunction for Max<A, B> {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        self.0.sample(pos).max(self.1.sample(pos))
    }
}

#[derive(Clone, Debug)]
pub struct Clamp<I: StaticDensityFunction> {
    pub input: I,
    pub min: f64,
    pub max: f64,
}

impl<I: StaticDensityFunction> StaticDensityFunction for Clamp<I> {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        self.input.sample(pos).clamp(self.min, self.max)
    }
}

#[derive(Clone, Debug)]
pub struct Linear<I: StaticDensityFunction> {
    pub input: I,
    pub argument: f64,
    pub addend: f64,
}

impl<I: StaticDensityFunction> StaticDensityFunction for Linear<I> {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        self.input.sample(pos) * self.argument + self.addend
    }
}

#[derive(Clone, Debug)]
pub struct ClampedYGradient {
    pub from_y: f64,
    pub to_y: f64,
    pub from_value: f64,
    pub to_value: f64,
}

impl StaticDensityFunction for ClampedYGradient {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        pumpkin_util::math::clamped_map(
            pos.y as f64,
            self.from_y,
            self.to_y,
            self.from_value,
            self.to_value,
        )
    }
}
