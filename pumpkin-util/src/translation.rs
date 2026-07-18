use pumpkin_i18n::{
    Locale, PUMPKIN_NAMESPACE, format_translation, get_translation, pumpkin_translation_key,
    server_global_locale,
};

use crate::text::TextComponent;

static VANILLA_EN_US_JSON: &str = include_str!("../../assets/en_us_java.json");
static PUMPKIN_EN_US_JSON: &str = include_str!("../../assets/translations/en_us.json");
static PUMPKIN_BRB_JSON: &str = include_str!("../../assets/translations/brb.json");
static PUMPKIN_DE_DE_JSON: &str = include_str!("../../assets/translations/de_de.json");
static PUMPKIN_ES_ES_JSON: &str = include_str!("../../assets/translations/es_es.json");
static PUMPKIN_FR_FR_JSON: &str = include_str!("../../assets/translations/fr_fr.json");
static PUMPKING_IT_IT_JSON: &str = include_str!("../../assets/translations/it_it.json");
static PUMPKIN_JA_JP_JSON: &str = include_str!("../../assets/translations/ja_jp.json");
static PUMPKIN_KA_GE_JSON: &str = include_str!("../../assets/translations/ka_ge.json");
static PUMPKIN_KO_KR_JSON: &str = include_str!("../../assets/translations/ko_kr.json");
static PUMPKIN_NDS_DE_JSON: &str = include_str!("../../assets/translations/nds_de.json");
static PUMPKIN_NL_BE_JSON: &str = include_str!("../../assets/translations/nl_be.json");
static PUMPKIN_NL_NL_JSON: &str = include_str!("../../assets/translations/nl_nl.json");
static PUMPKIN_RO_RO_JSON: &str = include_str!("../../assets/translations/ro_ro.json");
static PUMPKIN_RU_RU_JSON: &str = include_str!("../../assets/translations/ru_ru.json");
static PUMPKIN_SQ_AL_JSON: &str = include_str!("../../assets/translations/sq_al.json");
static PUMPKIN_ZH_CN_JSON: &str = include_str!("../../assets/translations/zh_cn.json");
static PUMPKIN_ZH_HK_JSON: &str = include_str!("../../assets/translations/zh_hk.json");
static PUMPKIN_ZH_TW_JSON: &str = include_str!("../../assets/translations/zh_tw.json");
static PUMPKIN_LZH_JSON: &str = include_str!("../../assets/translations/lzh.json");
static PUMPKIN_TR_TR_JSON: &str = include_str!("../../assets/translations/tr_tr.json");
static PUMPKIN_UK_UA_JSON: &str = include_str!("../../assets/translations/uk_ua.json");
static PUMPKIN_VI_VN_JSON: &str = include_str!("../../assets/translations/vi_vn.json");
static PUMPKIN_PT_BR_JSON: &str = include_str!("../../assets/translations/pt_br.json");
static PUMPKIN_PL_PL_JSON: &str = include_str!("../../assets/translations/pl_pl.json");
static BEDROCK_EN_US_LANG: &str = include_str!("../../assets/en_us_bedrock.lang");

/// Translate a pumpkin‑namespaced key for a specific locale (no formatting).
#[must_use]
pub fn translate_plain(key: &str, locale: Locale) -> String {
    get_translation(&pumpkin_translation_key(key), locale)
}

/// Translate a pumpkin‑namespaced key for a specific locale with format args.
#[must_use]
pub fn translate_format(key: &str, locale: Locale, args: &[String]) -> String {
    format_translation(&pumpkin_translation_key(key), locale, args)
}

// ---------------------------------------------------------------------------
// Server‑global‑locale convenience wrappers (log / console output).
// ---------------------------------------------------------------------------

#[must_use]
pub fn localized_log(key: &str) -> String {
    translate_plain(key, server_global_locale())
}

#[must_use]
pub fn reorder_substitutions(
    translation: &str,
    with: Vec<TextComponentBase>,
) -> (Vec<TextComponentBase>, Vec<SubstitutionRange>) {
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
                .map(|&i| SubstitutionRange {
                    start: i,
                    end: i + 1,
                })
                .collect(),
        );
    }

    let mut substitutions: Vec<TextComponentBase> = indices
        .iter()
        .map(|_| TextComponentBase {
            content: Box::new(TextContent::Text { text: "".into() }),
            style: Box::new(Style::default()),
            extra: vec![],
        })
        .collect();
    let mut ranges: Vec<SubstitutionRange> = vec![];

    let bytes = translation.as_bytes();
    let mut next_idx = 0usize;
    for (idx, &i) in indices.iter().enumerate() {
        let mut num_chars = String::new();
        let mut pos = 1;
        while i + pos < bytes.len() && bytes[i + pos].is_ascii_digit() {
            num_chars.push(bytes[i + pos] as char);
            pos += 1;
        }

        if num_chars.is_empty() {
            ranges.push(SubstitutionRange {
                start: i,
                end: i + 1,
            });
            substitutions[idx] = with[next_idx].clone();
            next_idx = (next_idx + 1).clamp(0, with.len() - 1);
            continue;
        }

        ranges.push(SubstitutionRange {
            start: i,
            end: i + pos + 1,
        });
        if let Ok(digit) = num_chars.parse::<usize>() {
            substitutions[idx] = with[digit.clamp(1, with.len()) - 1].clone();
        }
    }
    (substitutions, ranges)
}

/// Resolves a translation into formatted console output.
///
/// # Arguments
/// * `namespaced_key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
/// * `with`: Substitution components used to replace placeholders.
///
/// # Returns
/// The resolved and formatted translation string.
pub fn translation_to_pretty<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    let translation = get_translation(&namespaced_key.into(), locale);
    if with.is_empty() || !translation.contains('%') {
        return translation;
    }

    let (substitutions, indices) = reorder_substitutions(&translation, with);
    let mut result = String::new();
    let mut pos = 0;

    for (idx, &range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, substitutions.len() - 1);
        let substitution = substitutions[sub_idx].clone().to_pretty_console();

        result.push_str(&translation[pos..range.start]);
        result.push_str(&substitution);
        pos = range.end + 1;
    }

    result.push_str(&translation[pos..]);
    result
}

/// Resolves a translation into plain text.
///
/// # Arguments
/// * `namespaced_key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
/// * `with`: Substitution components used to replace placeholders.
///
/// # Returns
/// The resolved translation as plain text.
pub fn get_translation_text<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    let translation = get_translation(&namespaced_key.into(), locale);
    if with.is_empty() || !translation.contains('%') {
        return translation;
    }

    let (substitutions, indices) = reorder_substitutions(&translation, with);
    let mut result = String::new();
    let mut pos = 0;

    for (idx, &range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, substitutions.len() - 1);
        let substitution = substitutions[sub_idx].clone().get_text(locale);

        result.push_str(&translation[pos..range.start]);
        result.push_str(&substitution);
        pos = range.end + 1;
    }

    result.push_str(&translation[pos..]);
    result
}

pub static TRANSLATIONS: LazyLock<Mutex<[HashMap<String, String>; Locale::COUNT]>> =
    LazyLock::new(|| {
        let mut array: [HashMap<String, String>; Locale::COUNT] =
            std::array::from_fn(|_| HashMap::new());
        let vanilla_en_us: HashMap<String, String> =
            serde_json::from_str(VANILLA_EN_US_JSON).expect("Could not parse en_us_java.json.");
        let pumpkin_en_us: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_US_JSON).expect("Could not parse en_us.json.");
        let pumpkin_brb: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BRB_JSON).expect("Could not parse brb.json.");
        let pumpkin_de_de: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_DE_DE_JSON).expect("Could not parse de_de.json.");
        let pumpkin_es_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_ES_JSON).expect("Could not parse es_es.json.");
        let pumpkin_fr_fr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FR_FR_JSON).expect("Could not parse fr_fr.json.");
        let pumpkin_it_it: HashMap<String, String> =
            serde_json::from_str(PUMPKING_IT_IT_JSON).expect("Could not parse it_it.json.");
        let pumpkin_ja_jp: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_JA_JP_JSON).expect("Could not parse ja_jp.json.");
        let pumpkin_ka_ge: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KA_GE_JSON).expect("Could not parse ka_ge.json.");
        let pumpkin_ko_kr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KO_KR_JSON).expect("Could not parse ko_kr.json.");
        let pumpkin_nds_de: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NDS_DE_JSON).expect("Could not parse nds_de.json.");
        let pumpkin_nl_be: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NL_BE_JSON).expect("Could not parse nl_be.json.");
        let pumpkin_nl_nl: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NL_NL_JSON).expect("Could not parse nl_nl.json.");
        let pumpkin_ro_ro: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_RO_RO_JSON).expect("Could not parse ro_ro.json.");
        let pumpkin_ru_ru: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_RU_RU_JSON).expect("Could not parse ru_ru.json.");
        let pumpkin_sq_al: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SQ_AL_JSON).expect("Could not parse sq_al.json.");
        let pumpkin_zh_cn: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZH_CN_JSON).expect("Could not parse zh_cn.json.");
        let pumpkin_zh_hk: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZH_HK_JSON).expect("Could not parse zh_hk.json.");
        let pumpkin_zh_tw: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZH_TW_JSON).expect("Could not parse zh_tw.json.");
        let pumpkin_lzh: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LZH_JSON).expect("Could not parse lzh.json.");
        let pumpkin_tr_tr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TR_TR_JSON).expect("Could not parse tr_tr.json.");
        let pumpkin_uk_ua: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_UK_UA_JSON).expect("Could not parse uk_ua.json.");
        let pumpkin_vi_vn: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_VI_VN_JSON).expect("Could not parse vi_vn.json.");
        let pumpkin_pt_br: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_PT_BR_JSON).expect("Could not parse pt_br.json.");
        let pumpkin_pl_pl: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_PL_PL_JSON).expect("Could not parse pl_pl.json.");

        for (key, value) in vanilla_en_us {
            array[Locale::EnUs as usize].insert(format!("minecraft:{key}"), value);
        }
        for (key, value) in pumpkin_en_us {
            array[Locale::EnUs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_brb {
            array[Locale::Brb as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_de_de {
            array[Locale::DeDe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_es {
            array[Locale::EsEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fr_fr {
            array[Locale::FrFr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_it_it {
            array[Locale::ItIt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ja_jp {
            array[Locale::JaJp as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ka_ge {
            array[Locale::KaGe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ko_kr {
            array[Locale::KoKr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_nds_de {
            array[Locale::NdsDe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_nl_be {
            array[Locale::NlBe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_nl_nl {
            array[Locale::NlNl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ro_ro {
            array[Locale::RoRo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ru_ru {
            array[Locale::RuRu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sq_al {
            array[Locale::SqAl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_zh_cn {
            array[Locale::ZhCn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_zh_hk {
            array[Locale::ZhHk as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_zh_tw {
            array[Locale::ZhTw as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lzh {
            array[Locale::Lzh as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_tr_tr {
            array[Locale::TrTr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_uk_ua {
            array[Locale::UkUa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_vi_vn {
            array[Locale::ViVn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_pt_br {
            array[Locale::PtBr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_pl_pl {
            array[Locale::PlPl as usize].insert(format!("pumpkin:{key}"), value);
        }

        for line in BEDROCK_EN_US_LANG.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('/') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();
                array[Locale::EnUs as usize].insert(key.clone(), value.clone());
                array[Locale::EnUs as usize].insert(format!("minecraft:{key}"), value.clone());
                array[Locale::EnUs as usize].insert(format!("pumpkin:{key}"), value);
            }
        }

        Mutex::new(array)
    });

#[must_use]
pub fn localized_text<W>(key: &'static str, with: W) -> TextComponent
where
    W: Into<Vec<TextComponent>>,
{
    TextComponent::custom(PUMPKIN_NAMESPACE, key, server_global_locale(), with)
}
