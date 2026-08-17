use std::f32::consts::PI;

use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode, MapLike, struct_builder::StructBuilder as _,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Constant,
    Linear,
    InBack,
    InBounce,
    InCirc,
    InCubic,
    InElastic,
    InExpo,
    InQuad,
    InQuart,
    InQuint,
    InSine,
    InOutBack,
    InOutBounce,
    InOutCirc,
    InOutCubic,
    InOutElastic,
    InOutExpo,
    InOutQuad,
    InOutQuart,
    InOutQuint,
    InOutSine,
    OutBack,
    OutBounce,
    OutCirc,
    OutCubic,
    OutElastic,
    OutExpo,
    OutQuad,
    OutQuart,
    OutQuint,
    OutSine,
    CubicBezier([f32; 4]),
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}

impl Easing {
    #[must_use]
    pub fn apply(self, value: f32) -> f32 {
        let x = value.clamp(0.0, 1.0);
        match self {
            Self::Constant => 0.0,
            Self::Linear => x,
            Self::InQuad => x * x,
            Self::OutQuad => 1.0 - (1.0 - x).powi(2),
            Self::InOutQuad => {
                if x < 0.5 {
                    2.0 * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
                }
            }
            Self::InCubic => x.powi(3),
            Self::OutCubic => 1.0 - (1.0 - x).powi(3),
            Self::InOutCubic => {
                if x < 0.5 {
                    4.0 * x.powi(3)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
                }
            }
            Self::InQuart => x.powi(4),
            Self::OutQuart => 1.0 - (1.0 - x).powi(4),
            Self::InOutQuart => {
                if x < 0.5 {
                    8.0 * x.powi(4)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(4) / 2.0
                }
            }
            Self::InQuint => x.powi(5),
            Self::OutQuint => 1.0 - (1.0 - x).powi(5),
            Self::InOutQuint => {
                if x < 0.5 {
                    16.0 * x.powi(5)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(5) / 2.0
                }
            }
            Self::InSine => 1.0 - (x * PI / 2.0).cos(),
            Self::OutSine => (x * PI / 2.0).sin(),
            Self::InOutSine => -((PI * x).cos() - 1.0) / 2.0,
            Self::InExpo => {
                if x == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * x - 10.0)
                }
            }
            Self::OutExpo => {
                if x == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * x)
                }
            }
            Self::InOutExpo => {
                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else if x < 0.5 {
                    2.0_f32.powf(20.0 * x - 10.0) / 2.0
                } else {
                    (2.0 - 2.0_f32.powf(-20.0 * x + 10.0)) / 2.0
                }
            }
            Self::InCirc => 1.0 - (1.0 - x * x).sqrt(),
            Self::OutCirc => (1.0 - (x - 1.0).powi(2)).sqrt(),
            Self::InOutCirc => {
                if x < 0.5 {
                    (1.0 - (1.0 - (2.0 * x).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * x + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }
            Self::InBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * x.powi(3) - c1 * x * x
            }
            Self::OutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
            }
            Self::InOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if x < 0.5 {
                    (2.0 * x).powi(2) * ((c2 + 1.0) * 2.0 * x - c2) / 2.0
                } else {
                    ((2.0 * x - 2.0).powi(2) * ((c2 + 1.0) * (x * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }
            Self::OutBounce => out_bounce(x),
            Self::InBounce => 1.0 - out_bounce(1.0 - x),
            Self::InOutBounce => {
                if x < 0.5 {
                    (1.0 - out_bounce(1.0 - 2.0 * x)) / 2.0
                } else {
                    (1.0 + out_bounce(2.0 * x - 1.0)) / 2.0
                }
            }
            Self::InElastic => in_elastic(x),
            Self::OutElastic => out_elastic(x),
            Self::InOutElastic => in_out_elastic(x),
            Self::CubicBezier(points) => cubic_bezier(x, points),
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "constant" => Self::Constant,
            "linear" => Self::Linear,
            "in_back" => Self::InBack,
            "in_bounce" => Self::InBounce,
            "in_circ" => Self::InCirc,
            "in_cubic" => Self::InCubic,
            "in_elastic" => Self::InElastic,
            "in_expo" => Self::InExpo,
            "in_quad" => Self::InQuad,
            "in_quart" => Self::InQuart,
            "in_quint" => Self::InQuint,
            "in_sine" => Self::InSine,
            "in_out_back" => Self::InOutBack,
            "in_out_bounce" => Self::InOutBounce,
            "in_out_circ" => Self::InOutCirc,
            "in_out_cubic" => Self::InOutCubic,
            "in_out_elastic" => Self::InOutElastic,
            "in_out_expo" => Self::InOutExpo,
            "in_out_quad" => Self::InOutQuad,
            "in_out_quart" => Self::InOutQuart,
            "in_out_quint" => Self::InOutQuint,
            "in_out_sine" => Self::InOutSine,
            "out_back" => Self::OutBack,
            "out_bounce" => Self::OutBounce,
            "out_circ" => Self::OutCirc,
            "out_cubic" => Self::OutCubic,
            "out_elastic" => Self::OutElastic,
            "out_expo" => Self::OutExpo,
            "out_quad" => Self::OutQuad,
            "out_quart" => Self::OutQuart,
            "out_quint" => Self::OutQuint,
            "out_sine" => Self::OutSine,
            _ => return None,
        })
    }

    fn name(self) -> Option<&'static str> {
        Some(match self {
            Self::Constant => "constant",
            Self::Linear => "linear",
            Self::InBack => "in_back",
            Self::InBounce => "in_bounce",
            Self::InCirc => "in_circ",
            Self::InCubic => "in_cubic",
            Self::InElastic => "in_elastic",
            Self::InExpo => "in_expo",
            Self::InQuad => "in_quad",
            Self::InQuart => "in_quart",
            Self::InQuint => "in_quint",
            Self::InSine => "in_sine",
            Self::InOutBack => "in_out_back",
            Self::InOutBounce => "in_out_bounce",
            Self::InOutCirc => "in_out_circ",
            Self::InOutCubic => "in_out_cubic",
            Self::InOutElastic => "in_out_elastic",
            Self::InOutExpo => "in_out_expo",
            Self::InOutQuad => "in_out_quad",
            Self::InOutQuart => "in_out_quart",
            Self::InOutQuint => "in_out_quint",
            Self::InOutSine => "in_out_sine",
            Self::OutBack => "out_back",
            Self::OutBounce => "out_bounce",
            Self::OutCirc => "out_circ",
            Self::OutCubic => "out_cubic",
            Self::OutElastic => "out_elastic",
            Self::OutExpo => "out_expo",
            Self::OutQuad => "out_quad",
            Self::OutQuart => "out_quart",
            Self::OutQuint => "out_quint",
            Self::OutSine => "out_sine",
            Self::CubicBezier(_) => return None,
        })
    }
}

impl Encode for Easing {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        if let Some(name) = self.name() {
            return name.to_string().encode(ops, prefix);
        }

        match self {
            Self::CubicBezier(points) => {
                let value = ops.create_list(points.iter().map(|value| ops.create_float(*value)));
                ops.map_builder()
                    .add_key_result_value_result(
                        DataResult::new_success(ops.create_string("cubic_bezier")),
                        DataResult::new_success(value),
                    )
                    .build(prefix)
            }
            _ => DataResult::new_error("timeline easing has no serialized name"),
        }
    }
}

impl Decode for Easing {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        let string = String::parse(input.clone(), ops);
        if let Some(name) = string.into_result_or_partial() {
            return Self::from_name(&name).map_or_else(
                || DataResult::new_error(format!("unknown timeline easing type: {name}")),
                |easing| DataResult::new_success((easing, ops.empty())),
            );
        }

        ops.get_map(&input).flat_map(|map| {
            let Some(value) = map.get(&ops.create_string("cubic_bezier")) else {
                return DataResult::new_error("timeline easing object is missing cubic_bezier");
            };
            Vec::<f32>::parse(value.clone(), ops).flat_map(|values| {
                let Ok(points) = <[f32; 4]>::try_from(values) else {
                    return DataResult::new_error("cubic_bezier must contain exactly four values");
                };
                if !(0.0..=1.0).contains(&points[0]) || !(0.0..=1.0).contains(&points[2]) {
                    return DataResult::new_error(
                        "cubic_bezier x control points must be between 0 and 1",
                    );
                }
                DataResult::new_success((Self::CubicBezier(points), ops.empty()))
            })
        })
    }
}
fn out_bounce(x: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if x < 1.0 / D1 {
        N1 * x * x
    } else if x < 2.0 / D1 {
        let x = x - 1.5 / D1;
        N1 * x * x + 0.75
    } else if x < 2.5 / D1 {
        let x = x - 2.25 / D1;
        N1 * x * x + 0.9375
    } else {
        let x = x - 2.625 / D1;
        N1 * x * x + 0.984375
    }
}

fn in_elastic(x: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    let c4 = 2.0 * PI / 3.0;
    -(2.0_f32.powf(10.0 * x - 10.0)) * ((x * 10.0 - 10.75) * c4).sin()
}

fn out_elastic(x: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    let c4 = 2.0 * PI / 3.0;
    2.0_f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c4).sin() + 1.0
}

fn in_out_elastic(x: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    let c5 = 2.0 * PI / 4.5;
    if x < 0.5 {
        -(2.0_f32.powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * c5).sin()) / 2.0
    } else {
        (2.0_f32.powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * c5).sin()) / 2.0 + 1.0
    }
}

fn cubic_bezier(x: f32, [x1, y1, x2, y2]: [f32; 4]) -> f32 {
    fn sample(t: f32, a: f32, b: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * t * a + 3.0 * mt * t * t * b + t * t * t
    }

    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..16 {
        let t = (low + high) * 0.5;
        if sample(t, x1, x2) < x {
            low = t;
        } else {
            high = t;
        }
    }
    sample((low + high) * 0.5, y1, y2)
}
