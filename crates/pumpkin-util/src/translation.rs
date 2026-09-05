use std::{
    borrow::Cow,
    collections::HashMap,
    str::FromStr,
    sync::{LazyLock, Mutex},
};

/// TODO List
/// - Add server locale support
/// - Use translations in the logs
/// - Open a public translation system, maybe a Crowdin like Minecraft?
/// - Add support for translations on commands descriptions
/// - Integrate custom translations with the plugins API
/// - Try to optimize code of '`to_translated`'
use crate::text::{TextComponentBase, TextContent, style::Style};

static VANILLA_EN_US_JSON: &str = include_str!("../../../assets/en_us_java.json");
static PUMPKIN_EN_US_JSON: &str = include_str!("../../../assets/translations/en_us.json");
static PUMPKIN_BRB_JSON: &str = include_str!("../../../assets/translations/brb.json");
static PUMPKIN_DE_DE_JSON: &str = include_str!("../../../assets/translations/de_de.json");
static PUMPKIN_ES_ES_JSON: &str = include_str!("../../../assets/translations/es_es.json");
static PUMPKIN_FR_FR_JSON: &str = include_str!("../../../assets/translations/fr_fr.json");
static PUMPKIN_HR_HR_JSON: &str = include_str!("../../../assets/translations/hr_hr.json");
static PUMPKING_IT_IT_JSON: &str = include_str!("../../../assets/translations/it_it.json");
static PUMPKIN_JA_JP_JSON: &str = include_str!("../../../assets/translations/ja_jp.json");
static PUMPKIN_KA_GE_JSON: &str = include_str!("../../../assets/translations/ka_ge.json");
static PUMPKIN_KO_KR_JSON: &str = include_str!("../../../assets/translations/ko_kr.json");
static PUMPKIN_NDS_DE_JSON: &str = include_str!("../../../assets/translations/nds_de.json");
static PUMPKIN_NL_BE_JSON: &str = include_str!("../../../assets/translations/nl_be.json");
static PUMPKIN_NL_NL_JSON: &str = include_str!("../../../assets/translations/nl_nl.json");
static PUMPKIN_RO_RO_JSON: &str = include_str!("../../../assets/translations/ro_ro.json");
static PUMPKIN_RU_RU_JSON: &str = include_str!("../../../assets/translations/ru_ru.json");
static PUMPKIN_SQ_AL_JSON: &str = include_str!("../../../assets/translations/sq_al.json");
static PUMPKIN_ZH_CN_JSON: &str = include_str!("../../../assets/translations/zh_cn.json");
static PUMPKIN_ZH_HK_JSON: &str = include_str!("../../../assets/translations/zh_hk.json");
static PUMPKIN_ZH_TW_JSON: &str = include_str!("../../../assets/translations/zh_tw.json");
static PUMPKIN_LZH_JSON: &str = include_str!("../../../assets/translations/lzh.json");
static PUMPKIN_TR_TR_JSON: &str = include_str!("../../../assets/translations/tr_tr.json");
static PUMPKIN_UK_UA_JSON: &str = include_str!("../../../assets/translations/uk_ua.json");
static PUMPKIN_VI_VN_JSON: &str = include_str!("../../../assets/translations/vi_vn.json");
static PUMPKIN_PT_BR_JSON: &str = include_str!("../../../assets/translations/pt_br.json");
static PUMPKIN_PL_PL_JSON: &str = include_str!("../../../assets/translations/pl_pl.json");
static BEDROCK_EN_US_LANG: &str = include_str!("../../../assets/en_us_bedrock.lang");

/// A character range representing a substitution placeholder within a translation string.
///
/// The range is inclusive and corresponds to the full placeholder span
/// (for example `%s` or `%1$s`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubstitutionRange {
    /// Start byte index (inclusive).
    pub start: usize,
    /// End byte index (inclusive).
    pub end: usize,
}
impl SubstitutionRange {
    /// Returns the length of the range.
    #[must_use]
    pub const fn len(&self) -> usize {
        (self.end - self.start) + 1
    }
    /// Returns `true` if the range contains no characters.
    ///
    /// A range is considered empty when `start == end`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Adds or overrides a single translation entry.
///
/// # Arguments
/// * `namespace`: The namespace of the translation key.
/// * `key`: The translation key without namespace.
/// * `translation`: The localized translation string.
/// * `locale`: The locale the translation belongs to.
pub fn add_translation<P: Into<String>>(namespace: P, key: P, translation: P, locale: Locale) {
    let mut translations = TRANSLATIONS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let namespaced_key = format!("{}:{}", namespace.into(), key.into()).to_lowercase();
    translations[locale as usize].insert(namespaced_key, translation.into());
}

/// Loads translations from a JSON string and registers them under a namespace.
///
/// # Arguments
/// * `namespace`: The namespace applied to all loaded keys.
/// * `file_path`: A JSON string containing a flat key-value translation map.
/// * `locale`: The locale the translations belong to.
pub fn add_translation_file<P: Into<String>>(namespace: P, file_path: P, locale: Locale) {
    let translations_map: HashMap<String, String> =
        serde_json::from_str(&file_path.into()).unwrap_or_default();
    if translations_map.is_empty() {
        // TODO: Handle the case where the file is empty or not found properly
        return;
    }

    let mut translations = TRANSLATIONS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let namespace = namespace.into();
    for (key, translation) in translations_map {
        let namespaced_key = format!("{namespace}:{key}").to_lowercase();
        translations[locale as usize].insert(namespaced_key, translation);
    }
}

/// Retrieves a translation for the given key and locale.
///
/// # Arguments
/// * `key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
///
/// # Returns
/// The localized translation. Falls back to `en_us` or the key itself if not found.
pub fn get_translation(key: &str, locale: Locale) -> String {
    let translations = TRANSLATIONS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let key = key.to_lowercase();
    translations[locale as usize].get(&key).map_or_else(
        || {
            translations[Locale::EnUs as usize]
                .get(&key)
                .map_or(key, Clone::clone)
        },
        Clone::clone,
    )
}

/// What a `%` in a translation string introduces.
enum Placeholder {
    /// `%%`, which stands for a single literal `%`.
    Escape,
    /// `%<conversion>`, which takes the next substitution in order.
    Next,
    /// `%<digits>$<conversion>`, which takes the substitution at that one-based
    /// index.
    Indexed(usize),
}

/// Whether `byte` closes a placeholder. The shipped tables use `s` and `d`
/// (`en_us_bedrock.lang` is where the `%d` forms come from), and every
/// conversion is substituted the same way here, so the letter only marks the
/// end of the token.
const fn is_conversion(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

/// Classifies the `%` at `start`, returning what it introduces together with the
/// inclusive byte index it ends at.
///
/// Returns `None` when the `%` starts nothing this module understands, such as a
/// trailing `%` or digits with no conversion after them; those are literal text.
/// The end is reported rather than assumed because a [`SubstitutionRange`] has to
/// span the whole placeholder for the callers that resume reading at `end + 1`.
fn placeholder_at(bytes: &[u8], start: usize) -> Option<(Placeholder, usize)> {
    let after = start + 1;
    match bytes.get(after) {
        Some(&b'%') => return Some((Placeholder::Escape, after)),
        Some(&byte) if is_conversion(byte) => return Some((Placeholder::Next, after)),
        _ => {}
    }

    let mut end = after;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    if end == after
        || bytes.get(end) != Some(&b'$')
        || !bytes.get(end + 1).copied().is_some_and(is_conversion)
    {
        return None;
    }

    // Saturating so that an absurdly long run of digits still lands on the last
    // substitution instead of failing to parse.
    let index = bytes[after..end].iter().fold(0usize, |index, digit| {
        index
            .saturating_mul(10)
            .saturating_add(usize::from(digit - b'0'))
    });
    Some((Placeholder::Indexed(index), end + 1))
}

/// Reorders substitution placeholders within a translation string.
///
/// # Arguments
/// * `translation`: The raw translation string containing placeholders.
/// * `with`: Substitution components to insert into the placeholders.
///
/// # Returns
/// A tuple containing the reordered components and their substitution ranges.
#[must_use]
pub fn reorder_substitutions(
    translation: &str,
    with: Vec<TextComponentBase>,
) -> (Vec<TextComponentBase>, Vec<SubstitutionRange>) {
    fn literal(text: &'static str) -> TextComponentBase {
        TextComponentBase {
            content: Box::new(TextContent::Text { text: text.into() }),
            style: Box::new(Style::default()),
            extra: vec![],
        }
    }

    // A placeholder may repeat an index (`%1$s ... %1$s`), so the arguments are
    // read from rather than drained. Freeze them for that read-only access.
    let with = with.into_boxed_slice();

    let bytes = translation.as_bytes();
    let mut substitutions: Vec<TextComponentBase> = vec![];
    let mut ranges: Vec<SubstitutionRange> = vec![];
    let mut next_idx = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] != b'%' || (i > 0 && bytes[i - 1] == b'\\') {
            i += 1;
            continue;
        }
        let Some((placeholder, end)) = placeholder_at(bytes, i) else {
            i += 1;
            continue;
        };

        substitutions.push(match placeholder {
            Placeholder::Escape => literal("%"),
            Placeholder::Next => {
                let taken = with.get(next_idx).cloned().unwrap_or_else(|| literal(""));
                next_idx = (next_idx + 1).min(with.len().saturating_sub(1));
                taken
            }
            Placeholder::Indexed(index) => with
                .get(index.clamp(1, with.len().max(1)) - 1)
                .cloned()
                .unwrap_or_else(|| literal("")),
        });
        ranges.push(SubstitutionRange { start: i, end });
        i = end + 1;
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
        let parse_json = |json: &str| -> HashMap<String, String> {
            serde_json::from_str(json).unwrap_or_default()
        };
        let vanilla_en_us = parse_json(VANILLA_EN_US_JSON);
        let pumpkin_en_us = parse_json(PUMPKIN_EN_US_JSON);
        let pumpkin_brb = parse_json(PUMPKIN_BRB_JSON);
        let pumpkin_de_de = parse_json(PUMPKIN_DE_DE_JSON);
        let pumpkin_es_es = parse_json(PUMPKIN_ES_ES_JSON);
        let pumpkin_fr_fr = parse_json(PUMPKIN_FR_FR_JSON);
        let pumpkin_hr_hr = parse_json(PUMPKIN_HR_HR_JSON);
        let pumpkin_it_it = parse_json(PUMPKING_IT_IT_JSON);
        let pumpkin_ja_jp = parse_json(PUMPKIN_JA_JP_JSON);
        let pumpkin_ka_ge = parse_json(PUMPKIN_KA_GE_JSON);
        let pumpkin_ko_kr = parse_json(PUMPKIN_KO_KR_JSON);
        let pumpkin_nds_de = parse_json(PUMPKIN_NDS_DE_JSON);
        let pumpkin_nl_be = parse_json(PUMPKIN_NL_BE_JSON);
        let pumpkin_nl_nl = parse_json(PUMPKIN_NL_NL_JSON);
        let pumpkin_ro_ro = parse_json(PUMPKIN_RO_RO_JSON);
        let pumpkin_ru_ru = parse_json(PUMPKIN_RU_RU_JSON);
        let pumpkin_sq_al = parse_json(PUMPKIN_SQ_AL_JSON);
        let pumpkin_zh_cn = parse_json(PUMPKIN_ZH_CN_JSON);
        let pumpkin_zh_hk = parse_json(PUMPKIN_ZH_HK_JSON);
        let pumpkin_zh_tw = parse_json(PUMPKIN_ZH_TW_JSON);
        let pumpkin_lzh = parse_json(PUMPKIN_LZH_JSON);
        let pumpkin_tr_tr = parse_json(PUMPKIN_TR_TR_JSON);
        let pumpkin_uk_ua = parse_json(PUMPKIN_UK_UA_JSON);
        let pumpkin_vi_vn = parse_json(PUMPKIN_VI_VN_JSON);
        let pumpkin_pt_br = parse_json(PUMPKIN_PT_BR_JSON);
        let pumpkin_pl_pl = parse_json(PUMPKIN_PL_PL_JSON);

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
        for (key, value) in pumpkin_hr_hr {
            array[Locale::HrHr as usize].insert(format!("pumpkin:{key}"), value);
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

/// Supported locales for translations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locale {
    AfZa,
    ArSa,
    AstEs,
    AzAz,
    BaRu,
    Bar,
    BeBy,
    BgBg,
    BrFr,
    Brb,
    BsBa,
    CaEs,
    CsCz,
    CyGb,
    DaDk,
    DeAt,
    DeCh,
    DeDe,
    ElGr,
    EnAu,
    EnCa,
    EnGb,
    EnNz,
    EnPt,
    EnUd,
    EnUs,
    Enp,
    Enws,
    EoUy,
    EsAr,
    EsCl,
    EsEc,
    EsEs,
    EsMx,
    EsUy,
    EsVe,
    Esan,
    EtEe,
    EuEs,
    FaIr,
    FiFi,
    FilPh,
    FoFo,
    FrCa,
    FrFr,
    FraDe,
    FurIt,
    FyNl,
    GaIe,
    GdGb,
    GlEs,
    HawUs,
    HeIl,
    HiIn,
    HrHr,
    HuHu,
    HyAm,
    IdId,
    IgNg,
    IoEn,
    IsIs,
    Isv,
    ItIt,
    JaJp,
    JboEn,
    KaGe,
    KkKz,
    KnIn,
    KoKr,
    Ksh,
    KwGb,
    LaLa,
    LbLu,
    LiLi,
    Lmo,
    LoLa,
    LolUs,
    LtLt,
    LvLv,
    Lzh,
    MkMk,
    MnMn,
    MsMy,
    MtMt,
    Nah,
    NdsDe,
    NlBe,
    NlNl,
    NnNo,
    NoNo,
    OcFr,
    Ovd,
    PlPl,
    PtBr,
    PtPt,
    QyaAa,
    RoRo,
    Rpr,
    RuRu,
    RyUa,
    SahSah,
    SeNo,
    SkSk,
    SlSi,
    SoSo,
    SqAl,
    SrCs,
    SrSp,
    SvSe,
    Sxu,
    Szl,
    TaIn,
    ThTh,
    TlPh,
    TlhAa,
    Tok,
    TrTr,
    TtRu,
    UkUa,
    ValEs,
    VecIt,
    ViVn,
    YiDe,
    YoNg,
    ZhCn,
    ZhHk,
    ZhTw,
    ZlmArab,
}

impl Locale {
    pub const COUNT: usize = Self::ZlmArab as usize + 1;
}

impl FromStr for Locale {
    type Err = ();

    #[expect(clippy::too_many_lines)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "af_za" => Ok(Self::AfZa),       // Afrikaans (Suid-Afrika)
            "ar_sa" => Ok(Self::ArSa),       // Arabic
            "ast_es" => Ok(Self::AstEs),     // Asturian
            "az_az" => Ok(Self::AzAz),       // Azerbaijani
            "ba_ru" => Ok(Self::BaRu),       // Bashkir
            "bar" => Ok(Self::Bar),          // Bavarian
            "be_by" => Ok(Self::BeBy),       // Belarusian
            "bg_bg" => Ok(Self::BgBg),       // Bulgarian
            "br_fr" => Ok(Self::BrFr),       // Breton
            "brb" => Ok(Self::Brb),          // Brabantian
            "bs_ba" => Ok(Self::BsBa),       // Bosnian
            "ca_es" => Ok(Self::CaEs),       // Catalan
            "cs_cz" => Ok(Self::CsCz),       // Czech
            "cy_gb" => Ok(Self::CyGb),       // Welsh
            "da_dk" => Ok(Self::DaDk),       // Danish
            "de_at" => Ok(Self::DeAt),       // Austrian German
            "de_ch" => Ok(Self::DeCh),       // Swiss German
            "de_de" => Ok(Self::DeDe),       // German
            "el_gr" => Ok(Self::ElGr),       // Greek
            "en_au" => Ok(Self::EnAu),       // Australian English
            "en_ca" => Ok(Self::EnCa),       // Canadian English
            "en_gb" => Ok(Self::EnGb),       // British English
            "en_nz" => Ok(Self::EnNz),       // New Zealand English
            "en_pt" => Ok(Self::EnPt),       // Pirate English
            "en_ud" => Ok(Self::EnUd),       // Upside down British English
            "enp" => Ok(Self::Enp),          // Modern English minus borrowed words
            "enws" => Ok(Self::Enws),        // Early Modern English
            "eo_uy" => Ok(Self::EoUy),       // Esperanto
            "es_ar" => Ok(Self::EsAr),       // Argentinian Spanish
            "es_cl" => Ok(Self::EsCl),       // Chilean Spanish
            "es_ec" => Ok(Self::EsEc),       // Ecuadorian Spanish
            "es_es" => Ok(Self::EsEs),       // European Spanish
            "es_mx" => Ok(Self::EsMx),       // Mexican Spanish
            "es_uy" => Ok(Self::EsUy),       // Uruguayan Spanish
            "es_ve" => Ok(Self::EsVe),       // Venezuelan Spanish
            "esan" => Ok(Self::Esan),        // Andalusian
            "et_ee" => Ok(Self::EtEe),       // Estonian
            "eu_es" => Ok(Self::EuEs),       // Basque
            "fa_ir" => Ok(Self::FaIr),       // Persian
            "fi_fi" => Ok(Self::FiFi),       // Finnish
            "fil_ph" => Ok(Self::FilPh),     // Filipino
            "fo_fo" => Ok(Self::FoFo),       // Faroese
            "fr_ca" => Ok(Self::FrCa),       // Canadian French
            "fr_fr" => Ok(Self::FrFr),       // European French
            "fra_de" => Ok(Self::FraDe),     // East Franconian
            "fur_it" => Ok(Self::FurIt),     // Friulian
            "fy_nl" => Ok(Self::FyNl),       // Frisian
            "ga_ie" => Ok(Self::GaIe),       // Irish
            "gd_gb" => Ok(Self::GdGb),       // Scottish Gaelic
            "gl_es" => Ok(Self::GlEs),       // Galician
            "haw_us" => Ok(Self::HawUs),     // Hawaiian
            "he_il" => Ok(Self::HeIl),       // Hebrew
            "hi_in" => Ok(Self::HiIn),       // Hindi
            "hr_hr" => Ok(Self::HrHr),       // Croatian
            "hu_hu" => Ok(Self::HuHu),       // Hungarian
            "hy_am" => Ok(Self::HyAm),       // Armenian
            "id_id" => Ok(Self::IdId),       // Indonesian
            "ig_ng" => Ok(Self::IgNg),       // Igbo
            "io_en" => Ok(Self::IoEn),       // Ido
            "is_is" => Ok(Self::IsIs),       // Icelandic
            "isv" => Ok(Self::Isv),          // Interslavic
            "it_it" => Ok(Self::ItIt),       // Italian
            "ja_jp" => Ok(Self::JaJp),       // Japanese
            "jbo_en" => Ok(Self::JboEn),     // Lojban
            "ka_ge" => Ok(Self::KaGe),       // Georgian
            "kk_kz" => Ok(Self::KkKz),       // Kazakh
            "kn_in" => Ok(Self::KnIn),       // Kannada
            "ko_kr" => Ok(Self::KoKr),       // Korean
            "ksh" => Ok(Self::Ksh),          // Kölsch/Ripuarian
            "kw_gb" => Ok(Self::KwGb),       // Cornish
            "la_la" => Ok(Self::LaLa),       // Latin
            "lb_lu" => Ok(Self::LbLu),       // Luxembourgish
            "li_li" => Ok(Self::LiLi),       // Limburgish
            "lmo" => Ok(Self::Lmo),          // Lombard
            "lo_la" => Ok(Self::LoLa),       // Lao
            "lol_us" => Ok(Self::LolUs),     // LOLCAT
            "lt_lt" => Ok(Self::LtLt),       // Lithuanian
            "lv_lv" => Ok(Self::LvLv),       // Latvian
            "lzh" => Ok(Self::Lzh),          // Classical Chinese
            "mk_mk" => Ok(Self::MkMk),       // Macedonian
            "mn_mn" => Ok(Self::MnMn),       // Mongolian
            "ms_my" => Ok(Self::MsMy),       // Malay
            "mt_mt" => Ok(Self::MtMt),       // Maltese
            "nah" => Ok(Self::Nah),          // Nahuatl
            "nds_de" => Ok(Self::NdsDe),     // Low German
            "nl_be" => Ok(Self::NlBe),       // Dutch, Flemish
            "nl_nl" => Ok(Self::NlNl),       // Dutch
            "nn_no" => Ok(Self::NnNo),       // Norwegian Nynorsk
            "no_no" => Ok(Self::NoNo),       // Norwegian Bokmål
            "oc_fr" => Ok(Self::OcFr),       // Occitan
            "ovd" => Ok(Self::Ovd),          // Elfdalian
            "pl_pl" => Ok(Self::PlPl),       // Polish
            "pt_br" => Ok(Self::PtBr),       // Brazilian Portuguese
            "pt_pt" => Ok(Self::PtPt),       // European Portuguese
            "qya_aa" => Ok(Self::QyaAa),     // Quenya (Form of Elvish from LOTR)
            "ro_ro" => Ok(Self::RoRo),       // Romanian
            "rpr" => Ok(Self::Rpr),          // Russian (Pre-revolutionary)
            "ru_ru" => Ok(Self::RuRu),       // Russian
            "ry_ua" => Ok(Self::RyUa),       // Rusyn
            "sah_sah" => Ok(Self::SahSah),   // Yakut
            "se_no" => Ok(Self::SeNo),       // Northern Sami
            "sk_sk" => Ok(Self::SkSk),       // Slovak
            "sl_si" => Ok(Self::SlSi),       // Slovenian
            "so_so" => Ok(Self::SoSo),       // Somali
            "sq_al" => Ok(Self::SqAl),       // Albanian
            "sr_cs" => Ok(Self::SrCs),       // Serbian (Latin)
            "sr_sp" => Ok(Self::SrSp),       // Serbian (Cyrillic)
            "sv_se" => Ok(Self::SvSe),       // Swedish
            "sxu" => Ok(Self::Sxu),          // Upper Saxon German
            "szl" => Ok(Self::Szl),          // Silesian
            "ta_in" => Ok(Self::TaIn),       // Tamil
            "th_th" => Ok(Self::ThTh),       // Thai
            "tl_ph" => Ok(Self::TlPh),       // Tagalog
            "tlh_aa" => Ok(Self::TlhAa),     // Klingon
            "tok" => Ok(Self::Tok),          // Toki Pona
            "tr_tr" => Ok(Self::TrTr),       // Turkish
            "tt_ru" => Ok(Self::TtRu),       // Tatar
            "uk_ua" => Ok(Self::UkUa),       // Ukrainian
            "val_es" => Ok(Self::ValEs),     // Valencian
            "vec_it" => Ok(Self::VecIt),     // Venetian
            "vi_vn" => Ok(Self::ViVn),       // Vietnamese
            "yi_de" => Ok(Self::YiDe),       // Yiddish
            "yo_ng" => Ok(Self::YoNg),       // Yoruba
            "zh_cn" => Ok(Self::ZhCn),       // Chinese Simplified (China; Mandarin)
            "zh_hk" => Ok(Self::ZhHk),       // Chinese Traditional (Hong Kong; Mix)
            "zh_tw" => Ok(Self::ZhTw),       // Chinese Traditional (Taiwan; Mandarin)
            "zlm_arab" => Ok(Self::ZlmArab), // Malay (Jawi)
            _ => Ok(Self::EnUs),             // Default to English (US) if not found
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Locale, TRANSLATIONS, get_translation_text, reorder_substitutions};
    use crate::text::{TextComponentBase, TextContent, style::Style};

    fn arg(text: &str) -> TextComponentBase {
        TextComponentBase {
            content: Box::new(TextContent::Text {
                text: text.to_string().into(),
            }),
            style: Box::new(Style::default()),
            extra: vec![],
        }
    }

    /// `SubstitutionRange` documents itself as spanning the full placeholder, and
    /// the renderer resumes at `end + 1` on that promise. A lone `%` is one byte
    /// wide, so it is not a placeholder and must not claim a range.
    #[test]
    fn a_lone_percent_is_not_a_placeholder() {
        let (substitutions, ranges) = reorder_substitutions("100% of %s", vec![arg("A")]);

        assert_eq!(ranges.len(), 1, "only the %s is a placeholder");
        assert_eq!(ranges[0].start, 8);
        assert_eq!(ranges[0].end, 9);
        assert_eq!(substitutions.len(), 1);
    }

    /// `assets/en_us_java.json` ships this verbatim as `attribute.modifier.plus.1`.
    /// It also reaches the renderer as a format string in its own right, because
    /// `get_translation` returns the key when the key is unknown.
    #[test]
    fn an_escaped_percent_renders_as_one_percent() {
        assert_eq!(
            get_translation_text("+%s%% %s", Locale::EnUs, vec![arg("10"), arg("Speed")]),
            "+10% Speed"
        );
        // Shipped as `options.languageWarning`. Passed here as an unknown key, so
        // `get_translation` hands it back as its own format string, lowercased.
        assert_eq!(
            get_translation_text(
                "Language translations may not be 100%% accurate",
                Locale::EnUs,
                vec![arg("unused")]
            ),
            "language translations may not be 100% accurate"
        );
    }

    /// Shipped as `translation.test.escape`, which is the escape rule stated as a
    /// test by the game's own translation data: `%%` is one literal `%`, and only
    /// a `%` left over after the escapes introduces a substitution.
    #[test]
    fn runs_of_percents_escape_pairwise() {
        assert_eq!(
            get_translation_text(
                "%%s %%%s %%%%s %%%%%s",
                Locale::EnUs,
                vec![arg("A"), arg("B")]
            ),
            "%s %A %%s %%B"
        );
    }

    /// Callers index `substitutions` by the position of the range they are on,
    /// so the two must always line up. They used to diverge whenever `with` was
    /// not exactly as long as the placeholder count.
    #[test]
    fn a_substitution_is_produced_for_every_range() {
        for (translation, count) in [
            ("%s %s %s", 3),
            ("%s", 1),
            ("%1$s %1$s", 2),
            ("%% %s", 2),
            ("no placeholders", 0),
        ] {
            for args in 0..4 {
                let with = (0..args).map(|_| arg("x")).collect();
                let (substitutions, ranges) = reorder_substitutions(translation, with);

                assert_eq!(ranges.len(), count, "{translation:?} with {args} args");
                assert_eq!(
                    substitutions.len(),
                    ranges.len(),
                    "{translation:?} with {args} args"
                );
            }
        }
    }

    /// Shipped as `translation.test.invalid`.
    #[test]
    fn a_trailing_percent_renders_instead_of_panicking() {
        assert_eq!(
            get_translation_text("hi %", Locale::EnUs, vec![arg("A")]),
            "hi %"
        );
    }

    /// Digits with no `$s` after them are not a placeholder either, and used to
    /// run the cursor off the end of the string.
    #[test]
    fn trailing_digits_without_a_conversion_render_instead_of_panicking() {
        assert_eq!(
            get_translation_text("%5", Locale::EnUs, vec![arg("A")]),
            "%5"
        );
    }

    /// The renderer has to survive the data it ships with. `en_us_java.json`
    /// alone holds 19 strings that used to take the whole server down, so walk
    /// the loaded table and render every one of them.
    #[test]
    fn every_shipped_translation_renders() {
        // Collect first: `get_translation` takes the same lock.
        let keys: Vec<String> = {
            let translations = TRANSLATIONS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            translations[Locale::EnUs as usize]
                .keys()
                .cloned()
                .collect()
        };

        assert!(keys.len() > 1000, "the vanilla table should be loaded");
        for key in keys {
            // A non-empty `with` is what sends a string down the substituting path.
            get_translation_text(key, Locale::EnUs, vec![arg("A"), arg("B")]);
        }
    }

    /// The control: the forms this module substitutes must be untouched.
    #[test]
    fn supported_placeholders_still_substitute() {
        assert_eq!(
            get_translation_text("%s joined", Locale::EnUs, vec![arg("Steve")]),
            "Steve joined"
        );
        assert_eq!(
            get_translation_text(
                "%1$s was slain by %2$s",
                Locale::EnUs,
                vec![arg("Steve"), arg("Alex")]
            ),
            "Steve was slain by Alex"
        );
    }

    /// `assets/en_us_bedrock.lang` is loaded into the same table and closes its
    /// placeholders with `d`, so a conversion is not always `s`. Reading only `s`
    /// would leave these rendering as literal text.
    #[test]
    fn a_numeric_conversion_substitutes_too() {
        assert_eq!(
            get_translation_text("+%d %s", Locale::EnUs, vec![arg("10"), arg("Speed")]),
            "+10 Speed"
        );
        assert_eq!(
            get_translation_text("%1$d blocks cloned", Locale::EnUs, vec![arg("42")]),
            "42 blocks cloned"
        );
    }

    /// Digits that never reach a conversion are not a placeholder. The renderer
    /// used to consume the character after them, turning `(%5 blocks away)` into
    /// `(Alocks away)`.
    #[test]
    fn digits_without_a_conversion_do_not_eat_the_next_character() {
        assert_eq!(
            get_translation_text("%1$s (%5 blocks away)", Locale::EnUs, vec![arg("Village")]),
            "Village (%5 blocks away)"
        );
        assert_eq!(
            get_translation_text("entered (%.2f)", Locale::EnUs, vec![arg("x")]),
            "entered (%.2f)"
        );
    }
}
