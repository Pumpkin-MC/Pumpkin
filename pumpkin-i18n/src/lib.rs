use std::{
    collections::HashMap,
    env,
    str::FromStr,
    sync::{Arc, LazyLock, OnceLock},
};

use arc_swap::ArcSwap;

static VANILLA_EN_US_JSON: &str = include_str!("../../assets/en_us_java.json");
static PUMPKIN_BRB_JSON: &str = include_str!("../../assets/translations/brb.json");
static PUMPKIN_DE_DE_JSON: &str = include_str!("../../assets/translations/de_de.json");
static PUMPKIN_EN_US_JSON: &str = include_str!("../../assets/translations/en_us.json");
static PUMPKIN_ES_ES_JSON: &str = include_str!("../../assets/translations/es_es.json");
static PUMPKIN_FR_FR_JSON: &str = include_str!("../../assets/translations/fr_fr.json");
static PUMPKING_IT_IT_JSON: &str = include_str!("../../assets/translations/it_it.json");
static PUMPKIN_JA_JP_JSON: &str = include_str!("../../assets/translations/ja_jp.json");
static PUMPKIN_KA_GE_JSON: &str = include_str!("../../assets/translations/ka_ge.json");
static PUMPKIN_KO_KR_JSON: &str = include_str!("../../assets/translations/ko_kr.json");
static PUMPKIN_LZH_JSON: &str = include_str!("../../assets/translations/lzh.json");
static PUMPKIN_NDS_DE_JSON: &str = include_str!("../../assets/translations/nds_de.json");
static PUMPKIN_NL_BE_JSON: &str = include_str!("../../assets/translations/nl_be.json");
static PUMPKIN_NL_NL_JSON: &str = include_str!("../../assets/translations/nl_nl.json");
static PUMPKIN_PL_PL_JSON: &str = include_str!("../../assets/translations/pl_pl.json");
static PUMPKIN_PT_BR_JSON: &str = include_str!("../../assets/translations/pt_br.json");
static PUMPKIN_RO_RO_JSON: &str = include_str!("../../assets/translations/ro_ro.json");
static PUMPKIN_RU_RU_JSON: &str = include_str!("../../assets/translations/ru_ru.json");
static PUMPKIN_SQ_AL_JSON: &str = include_str!("../../assets/translations/sq_al.json");
static PUMPKIN_TR_TR_JSON: &str = include_str!("../../assets/translations/tr_tr.json");
static PUMPKIN_UK_UA_JSON: &str = include_str!("../../assets/translations/uk_ua.json");
static PUMPKIN_VI_VN_JSON: &str = include_str!("../../assets/translations/vi_vn.json");
static PUMPKIN_ZH_CN_JSON: &str = include_str!("../../assets/translations/zh_cn.json");
static PUMPKIN_ZH_HK_JSON: &str = include_str!("../../assets/translations/zh_hk.json");
static PUMPKIN_ZH_TW_JSON: &str = include_str!("../../assets/translations/zh_tw.json");

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

// ---------------------------------------------------------------------------
// Pre-parsed translation entry & storage
// ---------------------------------------------------------------------------

/// A stored translation with pre-computed substitution ranges.
///
/// The ranges are parsed once at insertion time so the hot lookup path
/// never needs to scan the translation string for `%` placeholders.
#[derive(Clone, Debug)]
pub struct TranslationEntry {
    /// The localized text (e.g. `"Hello %s!"`).
    pub text: Arc<str>,
    /// Pre‑computed byte ranges of each substitution placeholder.
    ///
    /// Empty if the text contains no substitutions.
    pub ranges: Arc<[SubstitutionRange]>,
    /// `true` when the translation uses positional placeholders (`%1$s`).
    pub has_positional: bool,
}

impl TranslationEntry {
    /// Creates an entry without any substitution placeholders.
    fn simple(text: Arc<str>) -> Self {
        Self {
            text,
            ranges: Arc::new([]),
            has_positional: false,
        }
    }
}

/// All loaded translations for one locale.
type LocaleMap = HashMap<Arc<str>, TranslationEntry>;

/// Global translations table — lock‑free reads via [`ArcSwap`].
///
/// Writes are infrequent (startup + plugin loading), so the
/// copy‑on‑write approach is cheap.  Reads are wait‑free.
static TRANSLATIONS: LazyLock<ArcSwap<Vec<Option<LocaleMap>>>> =
    LazyLock::new(|| ArcSwap::from_pointee(build_initial_translations()));

// ---------------------------------------------------------------------------
// Public mutation API
// ---------------------------------------------------------------------------

/// Adds or overrides a single translation entry.
///
/// Uses [`ArcSwap::rcu`] so concurrent writes are safe (last writer wins).
///
/// # Arguments
/// * `namespace`: The namespace of the translation key.
/// * `key`: The translation key without namespace.
/// * `translation`: The localized translation string.
/// * `locale`: The locale the translation belongs to.
pub fn add_translation<P: Into<String>>(namespace: P, key: P, translation: P, locale: Locale) {
    let namespaced_key: Arc<str> = format!("{}:{}", namespace.into(), key.into())
        .to_lowercase()
        .into();
    let entry = parse_translation_entry(translation.into());

    TRANSLATIONS.rcu(|current| {
        let mut new = (**current).clone();
        let map = new[locale as usize].get_or_insert_with(LocaleMap::new);
        map.insert(Arc::clone(&namespaced_key), entry.clone());
        Arc::new(new)
    });
}

/// Loads translations from a JSON string and registers them under a namespace.
///
/// Uses [`ArcSwap::rcu`] so concurrent writes are safe (last writer wins).
///
/// # Arguments
/// * `namespace`: The namespace applied to all loaded keys.
/// * `file_path`: A JSON string containing a flat key-value translation map.
/// * `locale`: The locale the translations belong to.
pub fn add_translation_file<P: Into<String>>(namespace: P, file_path: P, locale: Locale) {
    let translations_map: HashMap<String, String> =
        serde_json::from_str(&file_path.into()).unwrap_or_default();
    if translations_map.is_empty() {
        return;
    }

    let namespace: Arc<str> = namespace.into().into();

    TRANSLATIONS.rcu(|current| {
        let mut new = (**current).clone();
        let map = new[locale as usize].get_or_insert_with(LocaleMap::new);
        for (key, translation) in &translations_map {
            let namespaced_key: Arc<str> = format!("{namespace}:{key}").to_lowercase().into();
            let entry = parse_translation_entry(translation.clone());
            map.insert(namespaced_key, entry);
        }
        Arc::new(new)
    });
}

/// Cached system locale — detected once and reused.
static SYSTEM_LOCALE: OnceLock<Locale> = OnceLock::new();

/// Retrieves a translation for the given key and locale.
///
/// Lock‑free read via [`ArcSwap`].  Returns a cheaply cloneable [`Arc<str>`]
/// — on a cache hit only an atomic reference‑count increment is performed
/// (no string allocation).
///
/// When the key is not found in the requested locale, falls back to
/// [`Locale::EnUs`]. If the key is missing there as well the raw key
/// itself is returned as a new allocation. A warning is emitted in each
/// fallback case.
///
/// # Arguments
/// * `key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
///
/// # Returns
/// The localized translation text as [`Arc<str>`].
#[must_use]
pub fn get_translation(key: &str, locale: Locale) -> Arc<str> {
    // Helper: look up key in a locale map, trying exact match first,
    // then lowercased fallback.
    fn lookup<'a>(map: &'a LocaleMap, key: &str) -> Option<&'a Arc<str>> {
        map.get(key).map(|e| &e.text).or_else(|| {
            // Only allocate to_lowercase() when the exact match fails.
            let lower = key.to_lowercase();
            if lower == key {
                None
            } else {
                map.get(&*lower).map(|e| &e.text)
            }
        })
    }

    let guard = TRANSLATIONS.load();

    // 1. Try the requested locale
    if let Some(locale_map) = &guard[locale as usize]
        && let Some(value) = lookup(locale_map, key)
    {
        return Arc::clone(value);
    }

    // 2. Fall back to English
    if let Some(en_map) = &guard[Locale::EnUs as usize]
        && let Some(value) = lookup(en_map, key)
    {
        tracing::warn!(
            key = %key,
            locale = ?locale,
            "translation key not found – falling back to English (en_us)"
        );
        return Arc::clone(value);
    }

    // 3. Missing entirely — return the key itself
    tracing::error!(
        key = %key,
        "translation key not found in any locale – returning raw key"
    );
    Arc::from(key.to_owned().into_boxed_str())
}

/// Retrieves the full [`TranslationEntry`] (text + pre‑computed ranges) for
/// the given key and locale.  Used by substitution functions that need the ranges.
///
/// Fallback behaviour mirrors [`get_translation`].
#[must_use]
pub fn get_translation_entry(key: &str, locale: Locale) -> TranslationEntry {
    fn lookup<'a>(map: &'a LocaleMap, key: &str) -> Option<&'a TranslationEntry> {
        map.get(key).or_else(|| {
            let lower = key.to_lowercase();
            if lower == key { None } else { map.get(&*lower) }
        })
    }

    let guard = TRANSLATIONS.load();

    // 1. Requested locale
    if let Some(locale_map) = &guard[locale as usize]
        && let Some(entry) = lookup(locale_map, key)
    {
        return entry.clone();
    }

    // 2. Fall back to English
    if let Some(en_map) = &guard[Locale::EnUs as usize]
        && let Some(entry) = lookup(en_map, key)
    {
        tracing::warn!(
            key = %key,
            locale = ?locale,
            "translation key not found – falling back to English (en_us)"
        );
        return entry.clone();
    }

    // 3. Missing entirely
    tracing::error!(
        key = %key,
        "translation key not found in any locale – returning raw key"
    );
    TranslationEntry::simple(Arc::from(key.to_owned().into_boxed_str()))
}

/// Applies reordered substitutions using pre‑computed ranges.
///
/// This is the optimized version of [`reorder_substitutions`] that uses
/// ranges computed at translation load time.  When positions are already
/// sequential (no `%1$s` reordering), the `with` vector is returned as-is
/// without any allocation.
///
/// # Arguments
/// * `entry`: The pre‑parsed translation entry (from [`get_translation_entry`]).
/// * `with`: Substitution values to insert into the placeholders.
/// * `default`: A fallback value used when a placeholder has no corresponding item.
///
/// # Returns
/// A tuple containing the reordered items and their substitution ranges.
#[must_use]
pub fn reorder_with_entry<T: Clone>(
    entry: &TranslationEntry,
    with: Vec<T>,
    default: T,
) -> (Vec<T>, &[SubstitutionRange]) {
    let ranges = &*entry.ranges;

    if ranges.is_empty() || with.is_empty() {
        // No substitutions or nothing to substitute — return as-is.
        return (with, ranges);
    }

    if !entry.has_positional {
        // Simple %s placeholders — no reordering needed.
        return (with, ranges);
    }

    // Positional placeholders (%1$s, %2$s, …) — need to rebuild.
    let mut substitutions: Vec<T> = ranges.iter().map(|_| default.clone()).collect();

    let text_bytes = entry.text.as_bytes();
    let mut next_idx = 0usize;
    for (idx, range) in ranges.iter().enumerate() {
        // Determine positional index from the placeholder text.
        // We can't access the text directly from ranges alone, so
        // check the bytes at the placeholder position.
        let mut pos = 1; // skip '%'
        let mut num_chars = String::new();
        let start = range.start;
        while start + pos < text_bytes.len() && text_bytes[start + pos].is_ascii_digit() {
            num_chars.push(text_bytes[start + pos] as char);
            pos += 1;
        }

        if num_chars.is_empty() {
            substitutions[idx] = with[next_idx].clone();
            next_idx = (next_idx + 1).min(with.len().saturating_sub(1));
        } else if let Ok(digit) = num_chars.parse::<usize>() {
            let src = digit.saturating_sub(1).min(with.len().saturating_sub(1));
            substitutions[idx] = with[src].clone();
        }
    }
    (substitutions, ranges)
}

/// Parses substitution ranges from a translation string.
///
/// Called once at load time.  Scans for `%s` and `%N$s` patterns.
fn parse_substitution_ranges(translation: &str) -> (Arc<[SubstitutionRange]>, bool) {
    let bytes = translation.as_bytes();
    let mut ranges = Vec::new();
    let mut has_positional = false;
    let mut i = 0;

    while let Some(pos) = translation[i..].find('%') {
        let abs_pos = i + pos;
        // Skip escaped `\%`
        if abs_pos > 0 && bytes[abs_pos - 1] == b'\\' {
            i = abs_pos + 1;
            continue;
        }

        let after = abs_pos + 1;
        if after >= bytes.len() {
            break;
        }

        // Check for digits (positional like %1$s)
        let mut j = after;
        let mut has_digit = false;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            has_digit = true;
            j += 1;
        }

        if has_digit
            && j < bytes.len()
            && bytes[j] == b'$'
            && j + 1 < bytes.len()
            && bytes[j + 1] == b's'
        {
            // Positional: %N$s
            has_positional = true;
            ranges.push(SubstitutionRange {
                start: abs_pos,
                end: j + 1, // includes % through s
            });
            i = j + 2;
        } else if after < bytes.len() && bytes[after] == b's' {
            // Simple: %s
            ranges.push(SubstitutionRange {
                start: abs_pos,
                end: after,
            });
            i = after + 1;
        } else {
            // Standalone '%' or unknown pattern — skip
            i = abs_pos + 1;
        }
    }

    (Arc::from(ranges.into_boxed_slice()), has_positional)
}

/// Creates a [`TranslationEntry`] from a raw translation string, parsing
/// substitution ranges once.
fn parse_translation_entry(translation: String) -> TranslationEntry {
    if !translation.contains('%') {
        return TranslationEntry::simple(Arc::from(translation.into_boxed_str()));
    }

    let (ranges, has_positional) = parse_substitution_ranges(&translation);
    TranslationEntry {
        text: Arc::from(translation.into_boxed_str()),
        ranges,
        has_positional,
    }
}

/// Legacy compatibility wrapper — re-scans the translation string for
/// substitution ranges.  Prefer [`reorder_with_entry`] when you already
/// have a [`TranslationEntry`].
///
/// This still exists for callers that have a raw `&str` translation
/// (e.g. from `to_translated` which modifies the text before substitution).
#[must_use]
pub fn reorder_substitutions<T: Clone>(
    translation: &str,
    with: Vec<T>,
    default: T,
) -> (Vec<T>, Vec<SubstitutionRange>) {
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

    let mut substitutions: Vec<T> = indices.iter().map(|_| default.clone()).collect();
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

// ---------------------------------------------------------------------------
// System-locale detection (cross‑platform: Linux / macOS / Windows / Android)
// ---------------------------------------------------------------------------

/// Detects the system locale from the OS environment.
///
/// Result is cached on first call — subsequent calls return instantly.
///
/// **Linux / macOS / Android:** checks `LC_ALL`, `LC_MESSAGES`, `LANG`.
///
/// **Windows:** first checks the same POSIX-style variables (MSYS2, Git Bash,
/// WSL interop). If none is set, calls `GetUserDefaultLocaleName` from
/// kernel32 to obtain the native Windows locale.
///
/// Falls back to [`Locale::EnUs`] when no valid locale can be determined.
#[must_use]
pub fn detect_system_locale() -> Locale {
    *SYSTEM_LOCALE.get_or_init(|| {
        detect_locale_string()
            .and_then(|s| Locale::from_str(&s).ok())
            .unwrap_or(Locale::EnUs)
    })
}

/// Raw locale string from the environment / OS, e.g. `"zh_CN"` or `"de_DE"`.
fn detect_locale_string() -> Option<String> {
    // POSIX path (works on Linux, macOS, Android, and Windows with MSYS2/WSL)
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(raw) = env::var(var) {
            // Strip encoding suffix: "en_US.UTF-8" → "en_US", "C.UTF-8" → "C"
            let locale = raw.split('.').next().unwrap_or(&raw);
            if locale != "C" && locale != "POSIX" {
                return Some(locale.to_string());
            }
        }
    }

    // Windows native fallback
    #[cfg(target_os = "windows")]
    {
        if let Some(wl) = windows_user_locale() {
            return Some(wl);
        }
    }

    None
}

/// Calls `GetUserDefaultLocaleName` on Windows and converts the BCP‑47 tag
/// (e.g. `"zh-CN"`) into the POSIX form (`"zh_CN"`).
#[cfg(target_os = "windows")]
fn windows_user_locale() -> Option<String> {
    unsafe extern "system" {
        fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
    }
    let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
    let len = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
    if len > 0 {
        let s = String::from_utf16_lossy(&buf[..len as usize - 1]);
        Some(s.replace('-', "_"))
    } else {
        None
    }
}

pub mod client;
pub mod server;

// ---------------------------------------------------------------------------
// Initial translation table builder (called once at startup)
// ---------------------------------------------------------------------------

/// Builds the initial sparse translation table (only locales that have data).
fn build_initial_translations() -> Vec<Option<LocaleMap>> {
    let mut locales: Vec<Option<LocaleMap>> = std::iter::repeat_with(|| None)
        .take(Locale::COUNT)
        .collect();

    // -- Vanilla Minecraft English (Java Edition) --------------------------
    let vanilla_en_us: HashMap<String, String> =
        serde_json::from_str(VANILLA_EN_US_JSON).expect("Could not parse en_us_java.json.");
    let mut map = LocaleMap::with_capacity(vanilla_en_us.len());
    for (key, value) in vanilla_en_us {
        let namespaced_key: Arc<str> = Arc::from(format!("minecraft:{key}").into_boxed_str());
        let entry = parse_translation_entry(value);
        map.insert(namespaced_key, entry);
    }
    locales[Locale::EnUs as usize] = Some(map);

    // -- Pumpkin translations (one JSON per shipped locale) -----------------
    // Macro to reduce repetition
    macro_rules! load_pumpkin_locale {
        ($json:expr, $locale:ident) => {
            let data: HashMap<String, String> = serde_json::from_str($json).expect(concat!(
                "Could not parse ",
                stringify!($locale),
                ".json"
            ));
            let mut map = LocaleMap::with_capacity(data.len());
            for (key, value) in data {
                let namespaced_key: Arc<str> = Arc::from(format!("pumpkin:{key}").into_boxed_str());
                let entry = parse_translation_entry(value);
                map.insert(namespaced_key, entry);
            }
            locales[Locale::$locale as usize] = Some(map);
        };
    }

    load_pumpkin_locale!(PUMPKIN_BRB_JSON, Brb);
    load_pumpkin_locale!(PUMPKIN_DE_DE_JSON, DeDe);
    // EnUs is handled separately below (merged into vanilla English)
    load_pumpkin_locale!(PUMPKIN_ES_ES_JSON, EsEs);
    load_pumpkin_locale!(PUMPKIN_FR_FR_JSON, FrFr);
    load_pumpkin_locale!(PUMPKING_IT_IT_JSON, ItIt);
    load_pumpkin_locale!(PUMPKIN_JA_JP_JSON, JaJp);
    load_pumpkin_locale!(PUMPKIN_KA_GE_JSON, KaGe);
    load_pumpkin_locale!(PUMPKIN_KO_KR_JSON, KoKr);
    load_pumpkin_locale!(PUMPKIN_LZH_JSON, Lzh);
    load_pumpkin_locale!(PUMPKIN_NDS_DE_JSON, NdsDe);
    load_pumpkin_locale!(PUMPKIN_NL_BE_JSON, NlBe);
    load_pumpkin_locale!(PUMPKIN_NL_NL_JSON, NlNl);
    load_pumpkin_locale!(PUMPKIN_PL_PL_JSON, PlPl);
    load_pumpkin_locale!(PUMPKIN_PT_BR_JSON, PtBr);
    load_pumpkin_locale!(PUMPKIN_RO_RO_JSON, RoRo);
    load_pumpkin_locale!(PUMPKIN_RU_RU_JSON, RuRu);
    load_pumpkin_locale!(PUMPKIN_SQ_AL_JSON, SqAl);
    load_pumpkin_locale!(PUMPKIN_TR_TR_JSON, TrTr);
    load_pumpkin_locale!(PUMPKIN_UK_UA_JSON, UkUa);
    load_pumpkin_locale!(PUMPKIN_VI_VN_JSON, ViVn);
    load_pumpkin_locale!(PUMPKIN_ZH_CN_JSON, ZhCn);
    load_pumpkin_locale!(PUMPKIN_ZH_HK_JSON, ZhHk);
    load_pumpkin_locale!(PUMPKIN_ZH_TW_JSON, ZhTw);

    // -- Pumpkin English merged into existing EnUs map ----------------------
    {
        let pumpkin_en_us: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_US_JSON).expect("Could not parse en_us.json.");
        let map =
            locales[Locale::EnUs as usize].get_or_insert_with(|| LocaleMap::with_capacity(16));
        for (key, value) in pumpkin_en_us {
            let namespaced_key: Arc<str> = Arc::from(format!("pumpkin:{key}").into_boxed_str());
            let entry = parse_translation_entry(value);
            map.insert(namespaced_key, entry);
        }
    }

    locales
}

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
