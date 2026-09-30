use std::io::Read;
use std::str::FromStr;
use std::sync::{LazyLock, OnceLock};

use flate2::read::DeflateDecoder;
use pumpkin_protocol::bedrock::client::{
    CResourcePackChunkData, CResourcePackDataInfo,
    resource_packs_info::{PackIdVersion, PackInfoData},
};
use pumpkin_util::translation::{Locale, add_translation};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

pub const ID: Uuid = Uuid::from_u128(0xb762d721269944a1a166b6eeafbc1943);
pub const VERSION: &str = "1.0.9";
const ID_STRING: &str = "b762d721-2699-44a1-a166-b6eeafbc1943";
pub(super) const RESOURCE_NAME: &str = "b762d721-2699-44a1-a166-b6eeafbc1943_1.0.9";
const CHUNK_SIZE: usize = 1_048_576;
const CHUNK_SIZE_U32: u32 = 1_048_576;
const DATA: &[u8] = include_bytes!("../../../../../assets/bedrock/advancement_translations.mcpack");
const RESOURCE_TYPE: u8 = 6;

static HASH: LazyLock<[u8; 32]> = LazyLock::new(|| Sha256::digest(DATA).into());

pub fn info_entry() -> PackInfoData {
    PackInfoData {
        pack_id_version: PackIdVersion::new(ID, VERSION.to_owned()),
        pack_size: u64::try_from(DATA.len()).unwrap_or(u64::MAX),
        content_key: String::new(),
        subpack_name: String::new(),
        content_identity: String::new(),
        has_scripts: false,
        is_addon_pack: false,
        is_ray_tracing_capable: false,
        cdn_url: String::new(),
    }
}

pub fn matches(resource_name: &str) -> bool {
    resource_name == ID_STRING || resource_name == RESOURCE_NAME
}

pub fn data_info() -> CResourcePackDataInfo {
    CResourcePackDataInfo {
        resource_name: RESOURCE_NAME.to_owned(),
        chunk_size: CHUNK_SIZE_U32,
        number_of_chunks: u32::try_from(DATA.len().div_ceil(CHUNK_SIZE)).unwrap_or(u32::MAX),
        file_size: u64::try_from(DATA.len()).unwrap_or(u64::MAX),
        file_hash: HASH.to_vec(),
        is_premium_pack: false,
        pack_type: RESOURCE_TYPE,
    }
}

pub fn chunk(index: i32) -> Option<CResourcePackChunkData<'static>> {
    let index = usize::try_from(index).ok()?;
    let offset = index.checked_mul(CHUNK_SIZE)?;
    let data = DATA.get(offset..offset.saturating_add(CHUNK_SIZE).min(DATA.len()))?;
    Some(CResourcePackChunkData {
        resource_name: RESOURCE_NAME,
        chunk_id: u32::try_from(index).ok()?,
        byte_offset: u64::try_from(offset).ok()?,
        chunk_data: data,
    })
}

const CENTRAL_FILE: usize = 0x0201_4b50;
const END_OF_CENTRAL_DIR: usize = 0x0605_4b50;

/// Registers pack translations for plain-string Bedrock toasts.
pub fn register_translations() {
    static REGISTERED: OnceLock<()> = OnceLock::new();

    REGISTERED.get_or_init(|| {
        for (locale, lang) in lang_files() {
            for line in lang.lines() {
                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                let key = key.trim();
                if key.starts_with("advancements.") {
                    add_translation("minecraft", key, value.trim(), locale);
                }
            }
        }
    });
}

/// Reads the contents of every `texts/<locale>.lang` file in the pack.
fn lang_files() -> Vec<(Locale, String)> {
    let Some((count, mut at)) = central_directory() else {
        warn!("Advancement translation pack has no readable zip directory");
        return Vec::new();
    };

    let mut files = Vec::new();
    for _ in 0..count {
        let Some((entry, next)) = central_file(DATA, at) else {
            break;
        };
        at = next;

        let Some(locale) = entry.locale() else {
            continue;
        };
        if let Some(lang) = entry.contents(DATA) {
            files.push((locale, lang));
        }
    }
    files
}

fn central_directory() -> Option<(usize, usize)> {
    // The ZIP end record can be followed by a comment of up to 65,535 bytes.
    let search = DATA.len().saturating_sub(22 + 65_535);
    let end = (search..DATA.len())
        .rev()
        .find(|&at| le_u32(DATA, at) == Some(END_OF_CENTRAL_DIR))?;
    Some((le_u16(DATA, end + 10)?, le_u32(DATA, end + 16)?))
}

struct Entry<'a> {
    name: &'a [u8],
    method: usize,
    compressed_size: usize,
    local_header: usize,
}

fn central_file(data: &[u8], at: usize) -> Option<(Entry<'_>, usize)> {
    if le_u32(data, at)? != CENTRAL_FILE {
        return None;
    }
    let name_len = le_u16(data, at + 28)?;
    let entry = Entry {
        name: data.get(at + 46..at + 46 + name_len)?,
        method: le_u16(data, at + 10)?,
        compressed_size: le_u32(data, at + 20)?,
        local_header: le_u32(data, at + 42)?,
    };
    let record_len = 46 + name_len + le_u16(data, at + 30)? + le_u16(data, at + 32)?;
    Some((entry, at + record_len))
}

impl Entry<'_> {
    fn locale(&self) -> Option<Locale> {
        let name = std::str::from_utf8(self.name).ok()?;
        let code = name.strip_prefix("texts/")?.strip_suffix(".lang")?;
        let locale = Locale::from_str(code).ok()?;
        if locale == Locale::EnUs && !code.eq_ignore_ascii_case("en_us") {
            // Unknown locale codes must not overwrite English translations.
            warn!("Advancement translations for locale {code} are not supported");
            return None;
        }
        Some(locale)
    }

    fn contents(&self, data: &[u8]) -> Option<String> {
        let start = self.local_header
            + 30
            + le_u16(data, self.local_header + 26)?
            + le_u16(data, self.local_header + 28)?;
        let compressed = data.get(start..start + self.compressed_size)?;

        match self.method {
            0 => String::from_utf8(compressed.to_vec()).ok(),
            8 => {
                let mut text = String::new();
                DeflateDecoder::new(compressed)
                    .read_to_string(&mut text)
                    .ok()?;
                Some(text)
            }
            _ => None,
        }
    }
}

fn le_u16(data: &[u8], at: usize) -> Option<usize> {
    Some(usize::from(u16::from_le_bytes(
        data.get(at..at + 2)?.try_into().ok()?,
    )))
}

fn le_u32(data: &[u8], at: usize) -> Option<usize> {
    usize::try_from(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::translation::get_translation;

    #[test]
    fn every_pack_locale_reaches_the_server() {
        const TITLE: &str = "advancements.adventure.adventuring_time.title";
        const LABEL: &str = "advancements.toast.task";

        let value = |lang: &str, key: &str| {
            lang.lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                .map(|value| value.trim().to_string())
                .expect("every locale translates every advancement key")
        };

        register_translations();

        let (count, mut at) = central_directory().expect("the pack is a readable zip");
        let mut checked = 0;

        for _ in 0..count {
            let (entry, next) = central_file(DATA, at).expect("the pack has readable entries");
            at = next;

            let name = std::str::from_utf8(entry.name).expect("entry names are text");
            let Some(code) = name
                .strip_prefix("texts/")
                .and_then(|name| name.strip_suffix(".lang"))
            else {
                continue;
            };

            let locale = entry.locale().expect("every pack locale maps to a locale");
            let lang = entry.contents(DATA).expect("pack entries decompress");

            for key in [TITLE, LABEL] {
                assert_eq!(
                    get_translation(&format!("minecraft:{key}"), locale),
                    value(&lang, key),
                    "{code} did not resolve {key} to the pack's own translation"
                );
            }
            checked += 1;
        }
        assert!(checked > 0, "the pack must carry locales");
    }
}
