use std::{collections::HashMap, sync::LazyLock};

use crate::text::TextComponentBase;

const EN_US_JSON: &str = include_str!("../../assets/en_us.json");

pub static EN_US: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| serde_json::from_str(EN_US_JSON).expect("Could not parse en_us.json."));

pub fn get_translation_en_us(key: &str, with: Vec<TextComponentBase>) -> Option<String> {
    let mut translation = EN_US.get(key)?.clone();
    for replace in &with {
        translation = translation.replacen("%s", &replace.clone().to_pretty_console(), 1);
    }
    Some(translation)
}
