use chrono::{Datelike, Local};
use pumpkin_config::advanced_config;
use rand::{seq::SliceRandom, thread_rng};

// In fact Mojang also has some Seasonal Events, so we can use that later to match Vanilla :D

#[must_use]
pub fn is_april() -> bool {
    let data = Local::now();
    data.day() == 1 && data.month() == 4
}

#[must_use]
pub fn modify_chat_message(message: &str) -> Option<String> {
    if !advanced_config().fun.april_fools || !is_april() {
        return None;
    }
    let mut words: Vec<&str> = message.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    let mut rng = thread_rng();
    words.shuffle(&mut rng);
    let result = words.join(" ");
    Some(result)
}
