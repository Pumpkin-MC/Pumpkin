use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum YOffset {
    Absolute(Absolute),
    AboveBottom(AboveBottom),
    BelowTop(BelowTop),
}

impl YOffset {
    pub fn get_y(&self, min_y: i8, height: u16) -> i16 {
        match self {
            YOffset::AboveBottom(above_bottom) => min_y as i16 + above_bottom.above_bottom as i16,
            YOffset::BelowTop(below_top) => {
                height as i16 - 1 + min_y as i16 - below_top.below_top as i16
            }
            YOffset::Absolute(absolute) => absolute.absolute,
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
