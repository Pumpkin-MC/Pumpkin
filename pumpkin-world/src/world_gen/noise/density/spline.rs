use std::sync::Arc;

use crate::world_gen::noise::lerp;

use super::{
    Applier, ApplierImpl, DensityFunction, DensityFunctionImpl, NoisePos, Visitor, VisitorImpl,
};

pub enum SplineValue<'a> {
    Spline(Spline<'a>),
    Fixed(f32),
}

impl<'a> SplineValue<'a> {
    fn max(&self) -> f32 {
        match self {
            Self::Fixed(value) => *value,
            Self::Spline(spline) => spline.max,
        }
    }

    fn min(&self) -> f32 {
        match self {
            Self::Fixed(value) => *value,
            Self::Spline(spline) => spline.min,
        }
    }

    fn apply(&self, pos: &NoisePos) -> f32 {
        match self {
            Self::Fixed(value) => *value,
            Self::Spline(spline) => spline.apply(pos),
        }
    }

    fn visit(&self, visitor: &Visitor<'a>) -> SplineValue<'a> {
        match self {
            Self::Fixed(val) => Self::Fixed(*val),
            Self::Spline(spline) => Self::Spline(spline.visit(visitor)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct SplinePoint<'a> {
    location: f32,
    value: Arc<SplineValue<'a>>,
    derivative: f32,
}

#[derive(Clone)]
pub struct Spline<'a> {
    function: Arc<DensityFunction<'a>>,
    points: Vec<SplinePoint<'a>>,
    min: f32,
    max: f32,
}

enum Range {
    In(usize),
    Below,
}

impl<'a> Spline<'a> {
    fn sample_outside_range(point: f32, value: f32, points: &[SplinePoint], i: usize) -> f32 {
        let f = points[i].derivative;
        if f == 0f32 {
            value
        } else {
            f.mul_add(point - points[i].location, value)
        }
    }

    fn binary_walk(min: usize, max: usize, pred: impl Fn(usize) -> bool) -> usize {
        let mut i = max - min;
        let mut min = min;
        while i > 0 {
            let j = i / 2;
            let k = min + j;
            if pred(k) {
                i = j;
            } else {
                min = k + 1;
                i -= j + 1;
            }
        }
        min
    }

    fn find_range_for_location(points: &[SplinePoint], x: f32) -> Range {
        let index_greater_than_x = Self::binary_walk(0, points.len(), |i| x < points[i].location);
        if index_greater_than_x == 0 {
            Range::Below
        } else {
            Range::In(index_greater_than_x - 1)
        }
    }

    pub fn new(function: Arc<DensityFunction<'a>>, points: &[SplinePoint<'a>]) -> Self {
        let i = points.len() - 1;
        let mut f = f32::INFINITY;
        let mut g = f32::NEG_INFINITY;

        let h = function.min() as f32;
        let j = function.max() as f32;

        if h < points[0].location {
            let k = Self::sample_outside_range(h, points[0].value.min(), points, 0);
            let l = Self::sample_outside_range(h, points[0].value.max(), points, 0);

            f = f.min(k.min(l));
            g = f.max(k.max(l));
        }

        if j > points[i].location {
            let k = Self::sample_outside_range(j, points[i].value.min(), points, i);
            let l = Self::sample_outside_range(j, points[i].value.max(), points, i);

            f = f.min(k.min(l));
            g = g.max(k.max(l));
        }

        for point in points {
            f = f.min(point.value.min());
            g = g.max(point.value.max());
        }

        for m in 0..i {
            let l = points[m].location;
            let n = points[m + 1].location;
            let o = n - l;

            let spline2 = &points[m].value;
            let spline3 = &points[m + 1].value;

            let p = spline2.min();
            let q = spline2.max();
            let r = spline3.min();
            let s = spline3.max();
            let t = points[m].derivative;
            let u = points[m + 1].derivative;

            if t != 0f32 || u != 0f32 {
                let v = t * o;
                let w = u * o;

                let x = p.min(r);
                let y = q.max(s);

                let z = v - s + p;
                let aa = v - r + q;
                let ab = -w + r - q;
                let ac = -w + s - p;
                let ad = z.min(ab);
                let ae = aa.max(ac);

                f = f.min(0.25f32.mul_add(ad, x));
                g = g.max(0.25f32.mul_add(ae, y));
            }
        }

        Self {
            function,
            points: points.to_vec(),
            min: f,
            max: g,
        }
    }

    pub fn apply(&self, pos: &NoisePos) -> f32 {
        let f = self.function.sample(pos) as f32;
        let i = Self::find_range_for_location(&self.points, f);

        match i {
            Range::In(index) => {
                let last_index = self.points.len() - 1;
                if index == last_index {
                    Self::sample_outside_range(
                        f,
                        self.points[last_index].value.apply(pos),
                        &self.points,
                        last_index,
                    )
                } else {
                    let point_1 = &self.points[index];
                    let point_2 = &self.points[index + 1];
                    let k = (f - point_1.location) / (point_2.location - point_1.location);

                    let n = point_1.value.apply(pos);
                    let o = point_2.value.apply(pos);

                    let p = point_1
                        .derivative
                        .mul_add(point_2.location - point_1.location, -(o - n));
                    let q =
                        (-point_2.derivative).mul_add(point_2.location - point_1.location, o - n);
                    (k * (1f32 - k)).mul_add(lerp(k, p, q), lerp(k, n, o))
                }
            }
            Range::Below => {
                Self::sample_outside_range(f, self.points[0].value.apply(pos), &self.points, 0)
            }
        }
    }

    pub fn visit(&self, visitor: &Visitor<'a>) -> Spline<'a> {
        let new_function = visitor.apply(self.function.clone());
        let new_points = self
            .points
            .iter()
            .map(|point| SplinePoint {
                location: point.location,
                derivative: point.derivative,
                value: Arc::new(point.value.visit(visitor)),
            })
            .collect::<Vec<SplinePoint>>();
        Self::new(new_function, &new_points)
    }
}

#[derive(Clone)]
pub struct SplineFunction<'a> {
    spline: Arc<Spline<'a>>,
}

impl<'a> SplineFunction<'a> {
    pub fn new(spline: Arc<Spline<'a>>) -> Self {
        Self { spline }
    }
}

impl<'a> DensityFunctionImpl<'a> for SplineFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.spline.apply(pos) as f64
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        applier.fill(densities, &DensityFunction::Spline(self.clone()))
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        let new_spline = self.spline.visit(visitor);
        Arc::new(DensityFunction::Spline(SplineFunction {
            spline: Arc::new(new_spline),
        }))
    }

    fn max(&self) -> f64 {
        self.spline.max as f64
    }

    fn min(&self) -> f64 {
        self.spline.min as f64
    }
}

#[derive(Clone)]
pub enum FloatAmplifier {
    Identity,
    OffsetAmplifier,
    FactorAmplifier,
    JaggednessAmplifier,
}

impl FloatAmplifier {
    #[inline]
    pub fn apply(&self, f: f32) -> f32 {
        match self {
            Self::Identity => f,
            Self::OffsetAmplifier => {
                if f < 0f32 {
                    f
                } else {
                    f * 2f32
                }
            }
            Self::FactorAmplifier => 1.25f32 - 6.25f32 / (f + 5f32),
            Self::JaggednessAmplifier => f * 2f32,
        }
    }
}
pub struct SplineBuilder<'a> {
    function: Arc<DensityFunction<'a>>,
    amplifier: FloatAmplifier,
    points: Vec<SplinePoint<'a>>,
}

impl<'a> SplineBuilder<'a> {
    pub fn new(function: Arc<DensityFunction<'a>>, amplifier: FloatAmplifier) -> Self {
        Self {
            function,
            amplifier,
            points: Vec::new(),
        }
    }

    #[must_use]
    pub fn add_value(&mut self, location: f32, value: f32, derivative: f32) -> &mut Self {
        self.add_spline(
            location,
            SplineValue::Fixed(self.amplifier.apply(value)),
            derivative,
        )
    }

    #[must_use]
    pub fn add_spline(
        &mut self,
        location: f32,
        value: SplineValue<'a>,
        derivative: f32,
    ) -> &mut Self {
        if let Some(last) = self.points.last() {
            if location <= last.location {
                panic!("Points must be in asscending order");
            }
        }

        self.points.push(SplinePoint {
            location,
            value: Arc::new(value),
            derivative,
        });

        self
    }

    pub fn build(&self) -> Spline<'a> {
        Spline::new(self.function.clone(), &self.points)
    }
}

#[cfg(test)]
mod test {

    use crate::world_gen::noise::{
        density::{BuiltInNoiseFunctions, NoisePos, UnblendedNoisePos},
        BuiltInNoiseParams,
    };

    use super::{FloatAmplifier, SplineBuilder};

    #[test]
    fn test_correctness() {
        let noise_params = BuiltInNoiseParams::new();
        let noise_functions = BuiltInNoiseFunctions::new(&noise_params);
        let pos = NoisePos::Unblended(UnblendedNoisePos { x: 0, y: 0, z: 0 });

        let spline = SplineBuilder::new(
            noise_functions.continents_overworld,
            FloatAmplifier::Identity,
        )
        .add_value(-1.1f32, 0.044f32, 0f32)
        .add_value(-1.02f32, -0.2222f32, 0f32)
        .add_value(-0.51f32, -0.2222f32, 0f32)
        .add_value(-0.44f32, -0.12f32, 0f32)
        .add_value(-0.18f32, -0.12f32, 0f32)
        .build();

        assert_eq!(spline.apply(&pos), -0.12f32);
    }
}
