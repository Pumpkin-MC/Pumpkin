use std::{collections::HashMap, fs, sync::LazyLock};

use crate::text::TextComponentBase;

// The path is different for the build script!
pub static EN_US: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let content = fs::read_to_string("assets/en_us.json")
        .or_else(|_| fs::read_to_string("../assets/en_us.json"))
        .expect("Could not find en_us.json!");
    serde_json::from_str(&content).expect("Could not parse en_us.json.")
});

pub fn get_translation_en_us(key: &str, with: Vec<TextComponentBase>) -> Option<String> {
    let mut translation = EN_US.get(key)?.clone();
    for replace in &with {
        translation = translation.replacen("%s", &replace.clone().to_pretty_console(), 1);
    }
    Some(translation)
}
