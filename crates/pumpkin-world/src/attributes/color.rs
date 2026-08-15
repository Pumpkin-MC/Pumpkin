use pumpkin_codecs::{DataResult, Decode, DynamicOps, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgbColor(pub u32);

impl RgbColor {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value & 0x00FF_FFFF)
    }

    #[must_use]
    pub const fn r(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    #[must_use]
    pub const fn g(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    #[must_use]
    pub const fn b(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    #[must_use]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }
}

impl ArgbColor {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn a(self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    #[must_use]
    pub const fn r(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    #[must_use]
    pub const fn g(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    #[must_use]
    pub const fn b(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    #[must_use]
    pub const fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    #[must_use]
    pub const fn rgb(self) -> RgbColor {
        RgbColor::from_rgb(self.r(), self.g(), self.b())
    }
}

fn parse_hex(input: &str, digits: usize) -> Option<u32> {
    let hex = input.strip_prefix('#')?;
    if hex.len() != digits {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()
}

fn component(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Encode for RgbColor {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        ops.merge_into_primitive(prefix, ops.create_string(&format!("#{:06X}", self.0)))
    }
}

impl Decode for RgbColor {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        if let Some(value) = ops
            .get_string(&input)
            .into_result()
            .and_then(|value| parse_hex(&value, 6))
        {
            return DataResult::new_success((Self::new(value), ops.empty()));
        }

        if let Some(number) = ops.get_number(&input).into_result() {
            let value: i64 = number.into();
            if let Ok(value) = u32::try_from(value)
                && value <= 0x00FF_FFFF
            {
                return DataResult::new_success((Self::new(value), ops.empty()));
            }
        }

        if let Some(iter) = ops.get_iter(input).into_result() {
            let values: Vec<f32> = iter
                .filter_map(|value| ops.get_number(&value).into_result().map(Into::into))
                .collect();
            if values.len() == 3 {
                return DataResult::new_success((
                    Self::from_rgb(
                        component(values[0]),
                        component(values[1]),
                        component(values[2]),
                    ),
                    ops.empty(),
                ));
            }
        }

        DataResult::new_error("Invalid RGB color")
    }
}

impl Encode for ArgbColor {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        ops.merge_into_primitive(prefix, ops.create_string(&format!("#{:08X}", self.0)))
    }
}

impl Decode for ArgbColor {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        if let Some(value) = ops
            .get_string(&input)
            .into_result()
            .and_then(|value| parse_hex(&value, 8))
        {
            return DataResult::new_success((Self::new(value), ops.empty()));
        }

        if let Some(number) = ops.get_number(&input).into_result() {
            let value: i64 = number.into();
            if let Ok(value) = u32::try_from(value) {
                return DataResult::new_success((Self::new(value), ops.empty()));
            }
        }

        if let Some(iter) = ops.get_iter(input).into_result() {
            let values: Vec<f32> = iter
                .filter_map(|value| ops.get_number(&value).into_result().map(Into::into))
                .collect();
            if values.len() == 4 {
                return DataResult::new_success((
                    Self::from_argb(
                        component(values[0]),
                        component(values[1]),
                        component(values[2]),
                        component(values[3]),
                    ),
                    ops.empty(),
                ));
            }
        }

        DataResult::new_error("Invalid ARGB color")
    }
}
