use std::{
    borrow::Cow,
    env,
    fs::{self, File},
    io,
    ops::Range,
    sync::{Arc, LazyLock, Mutex},
};

use pumpkin_config::BASIC_CONFIG;
use pumpkin_data::translations::TranslationManager;
use pumpkin_util::text::TextComponentBase;

use crate::{entity::player::Player, text::TextResolution};

pub static TRANSLATION_MANAGER: LazyLock<Arc<Mutex<TranslationManager>>> =
    LazyLock::new(|| Arc::new(Mutex::new(TranslationManager::new())));

pub async fn init_translations() {
    if BASIC_CONFIG.language == "en_us" {
        return;
    }
    if let Some((hash, size)) = TranslationManager::locale_hash(&BASIC_CONFIG.language) {
        let data_dir = env::current_dir().unwrap().join("data/");
        if !data_dir.exists() {
            fs::create_dir(&data_dir).expect("Failed to create data root dir");
        }
        let translations_dir = data_dir.join("translations/");
        if !translations_dir.exists() {
            fs::create_dir(&translations_dir).expect("Failed to create translations folder");
        }
        let vanilla_dir = translations_dir.join(hash);
        if !vanilla_dir.exists() {
            if let Ok(data) = reqwest::get(format!(
                "https://resources.download.minecraft.net/{}/{}",
                &hash[0..2],
                hash
            ))
            .await
            {
                let mut vanilla = File::create(&vanilla_dir).unwrap();
                let _ = io::copy(&mut data.text().await.unwrap().as_bytes(), &mut vanilla);
            } else {
                log::warn!(
                    "A translation could not be downloaded; the server will remain in English."
                );
                return;
            }
        }
        if let Ok(mut manager) = TRANSLATION_MANAGER.lock() {
            let _ = manager.add_file(BASIC_CONFIG.language.as_str(), "minecraft", &vanilla_dir);
        }

        log::info!("Info of {}: {} - {}", BASIC_CONFIG.language, hash, size);
    }
}

pub async fn translated<P: Into<String>>(
    namespaced_key: P,
    player: Option<&Player>,
    fallback: Option<Cow<'static, str>>,
    with: Vec<TextComponentBase>,
    stylized: bool,
) -> String {
    let locale = match player {
        Some(player) => player.locale().await,
        None => BASIC_CONFIG.language.clone(),
    };
    let mut translation =
        TRANSLATION_MANAGER
            .lock()
            .unwrap()
            .get(locale, namespaced_key.into(), fallback);
    if with.is_empty() || !translation.contains('%') {
        return translation;
    }

    let (substitutions, indices) = reorder_substitutions(&translation, with);
    let mut translated_substitutions = Vec::new();
    for substitution in substitutions {
        translated_substitutions.push(substitution.to_string(player, stylized).await);
    }
    let mut displacement = 0i32;
    for (idx, range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, translated_substitutions.len() - 1);
        let substitution = &translated_substitutions[sub_idx];
        translation.replace_range(
            range.start + displacement as usize..=range.end + displacement as usize,
            substitution,
        );
        displacement += substitution.len() as i32 - range.len() as i32;
    }
    translation
}

#[must_use]
pub fn reorder_substitutions(
    translation: &str,
    with: Vec<TextComponentBase>,
) -> (Vec<TextComponentBase>, Vec<Range<usize>>) {
    let indices: Vec<usize> = translation
        .match_indices('%')
        .filter(|(i, _)| *i == 0 || translation.as_bytes()[i - 1] != b'\\')
        .map(|(i, _)| i)
        .collect();
    if translation.matches("%s").count() == indices.len() {
        return (
            with,
            indices
                .iter()
                .map(|&i| Range {
                    start: i,
                    end: i + 1,
                })
                .collect(),
        );
    }

    let mut substitutions: Vec<TextComponentBase> =
        vec![TextComponentBase::default(); indices.len()];
    let mut ranges: Vec<Range<usize>> = vec![];

    let bytes = translation.as_bytes();
    let mut next_idx = 0usize;
    for (idx, &i) in indices.iter().enumerate() {
        let mut num_chars = String::new();
        let mut pos = 1;
        while bytes[i + pos].is_ascii_digit() {
            num_chars.push(bytes[i + pos] as char);
            pos += 1;
        }

        if num_chars.is_empty() {
            ranges.push(Range {
                start: i,
                end: i + 1,
            });
            substitutions[idx] = with[next_idx].clone();
            next_idx = (next_idx + 1).clamp(0, with.len() - 1);
            continue;
        }

        ranges.push(Range {
            start: i,
            end: i + pos + 1,
        });
        if let Ok(digit) = num_chars.parse::<usize>() {
            substitutions[idx] = with[digit.clamp(1, with.len()) - 1].clone();
        }
    }
    (substitutions, ranges)
}
