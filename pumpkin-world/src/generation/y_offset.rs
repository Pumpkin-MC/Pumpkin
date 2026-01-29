use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum YOffset {
    Absolute(Absolute),
    AboveBottom(AboveBottom),
    BelowTop(BelowTop),
}

impl YOffset {
    #[must_use]
    pub const fn get_y(&self, min_y: i16, height: u16) -> i32 {
        match self {
            Self::AboveBottom(above_bottom) => min_y as i32 + above_bottom.above_bottom as i32,
            Self::BelowTop(below_top) => {
                height as i32 - 1 + min_y as i32 - below_top.below_top as i32
            }
            Self::Absolute(absolute) => absolute.absolute as i32,
        }
    }
}

#[derive(Deserialize)]
pub struct Absolute {
    absolute: i16,
}

#[derive(Deserialize)]
pub struct AboveBottom {
    above_bottom: i8,
}
#[derive(Deserialize)]
pub struct BelowTop {
    below_top: i8,
}
