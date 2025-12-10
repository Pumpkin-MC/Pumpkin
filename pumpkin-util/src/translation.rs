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
/// - Try to optimize code of 'to_translated'
use crate::text::{TextComponentBase, TextContent, style::Style};

static VANILLA_EN_US_JSON: &str = include_str!("../../assets/en_us.json");
static PUMPKIN_EN_US_JSON: &str = include_str!("../../assets/translations/en_us.json");
static PUMPKIN_ES_ES_JSON: &str = include_str!("../../assets/translations/es_es.json");
static PUMPKIN_FR_FR_JSON: &str = include_str!("../../assets/translations/fr_fr.json");
static PUMPKIN_ZH_CN_JSON: &str = include_str!("../../assets/translations/zh_cn.json");
static PUMPKIN_TR_TR_JSON: &str = include_str!("../../assets/translations/tr_tr.json");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubstitutionRange {
    pub start: usize,
    pub end: usize,
}
impl SubstitutionRange {
    pub fn len(&self) -> usize {
        (self.end - self.start) + 1
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

pub fn add_translation<P: Into<String>>(namespace: P, key: P, translation: P, locale: Locale) {
    let mut translations = TRANSLATIONS.lock().unwrap();
    let namespaced_key = format!("{}:{}", namespace.into(), key.into()).to_lowercase();
    translations[locale as usize].insert(namespaced_key, translation.into());
}

pub fn add_translation_file<P: Into<String>>(namespace: P, file_path: P, locale: Locale) {
    let translations_map: HashMap<String, String> =
        serde_json::from_str(&file_path.into()).unwrap_or(HashMap::new());
    if translations_map.is_empty() {
        // TODO: Handle the case where the file is empty or not found properly
        return;
    }

    let mut translations = TRANSLATIONS.lock().unwrap();
    let namespace = namespace.into();
    for (key, translation) in translations_map {
        let namespaced_key = format!("{namespace}:{key}").to_lowercase();
        translations[locale as usize].insert(namespaced_key, translation);
    }
}

pub fn get_translation(key: &str, locale: Locale) -> String {
    let translations = TRANSLATIONS.lock().unwrap();
    let key = key.to_lowercase();
    match translations[locale as usize].get(&key) {
        Some(translation) => translation.clone(),
        None => match translations[Locale::EnUs as usize].get(&key) {
            Some(translation) => translation.clone(),
            None => key,
        },
    }
}

pub fn reorder_substitutions(
    translation: &str,
    with: Vec<TextComponentBase>,
) -> (Vec<TextComponentBase>, Vec<SubstitutionRange>) {
    let indices: Vec<usize> = translation
        .match_indices("%")
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
            content: TextContent::Text { text: "".into() },
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
        while bytes[i + pos].is_ascii_digit() {
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

pub fn translation_to_pretty<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    let mut translation = get_translation(&namespaced_key.into(), locale);
    if with.is_empty() || !translation.contains('%') {
        return translation;
    }

    let (substitutions, indices) = reorder_substitutions(&translation, with);
    let mut displacement = 0;
    for (idx, &range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, substitutions.len() - 1);
        let substitution = substitutions[sub_idx].clone().to_pretty_console();
        translation.replace_range(
            range.start + displacement..=range.end + displacement,
            &substitution,
        );
        displacement += substitution.len() - range.len();
    }
    translation
}

pub fn get_translation_text<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    let mut translation = get_translation(&namespaced_key.into(), locale);
    if with.is_empty() || !translation.contains('%') {
        return translation;
    }

    let (substitutions, indices) = reorder_substitutions(&translation, with);
    let mut displacement = 0i32;
    for (idx, &range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, substitutions.len() - 1);
        let substitution = substitutions[sub_idx].clone().get_text(locale);
        translation.replace_range(
            range.start + displacement as usize..=range.end + displacement as usize,
            &substitution,
        );
        displacement += substitution.len() as i32 - range.len() as i32;
    }
    translation
}

pub static TRANSLATIONS: LazyLock<Mutex<[HashMap<String, String>; Locale::last() as usize]>> =
    LazyLock::new(|| {
        let mut array: [HashMap<String, String>; Locale::last() as usize] =
            std::array::from_fn(|_| HashMap::new());
        let vanilla_en_us: HashMap<String, String> =
            serde_json::from_str(VANILLA_EN_US_JSON).expect("Could not parse en_us.json.");
        let pumpkin_en_us: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_US_JSON).expect("Could not parse en_us.json.");
        let pumpkin_es_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_ES_JSON).expect("Could not parse es_es.json.");
        let pumpkin_fr_fr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FR_FR_JSON).expect("Could not parse fr_fr.json.");
        let pumpkin_zh_cn: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZH_CN_JSON).expect("Could not parse zh_cn.json.");
        let pumpkin_tr_tr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TR_TR_JSON).expect("Could not parse tr_tr.json.");

        for (key, value) in vanilla_en_us {
            array[Locale::EnUs as usize].insert(format!("minecraft:{key}"), value);
        }
        for (key, value) in pumpkin_en_us {
            array[Locale::EnUs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_es {
            array[Locale::EsEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fr_fr {
            array[Locale::FrFr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_zh_cn {
            array[Locale::ZhCn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_tr_tr {
            array[Locale::TrTr as usize].insert(format!("pumpkin:{key}"), value);
        }
        Mutex::new(array)
    });

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
    pub const fn last() -> Self {
        Locale::ZlmArab
    }
}

impl FromStr for Locale {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "af_za" => Ok(Locale::AfZa),       // Afrikaans (Suid-Afrika)
            "ar_sa" => Ok(Locale::ArSa),       // Arabic
            "ast_es" => Ok(Locale::AstEs),     // Asturian
            "az_az" => Ok(Locale::AzAz),       // Azerbaijani
            "ba_ru" => Ok(Locale::BaRu),       // Bashkir
            "bar" => Ok(Locale::Bar),          // Bavarian
            "be_by" => Ok(Locale::BeBy),       // Belarusian
            "bg_bg" => Ok(Locale::BgBg),       // Bulgarian
            "br_fr" => Ok(Locale::BrFr),       // Breton
            "brb" => Ok(Locale::Brb),          // Brabantian
            "bs_ba" => Ok(Locale::BsBa),       // Bosnian
            "ca_es" => Ok(Locale::CaEs),       // Catalan
            "cs_cz" => Ok(Locale::CsCz),       // Czech
            "cy_gb" => Ok(Locale::CyGb),       // Welsh
            "da_dk" => Ok(Locale::DaDk),       // Danish
            "de_at" => Ok(Locale::DeAt),       // Austrian German
            "de_ch" => Ok(Locale::DeCh),       // Swiss German
            "de_de" => Ok(Locale::DeDe),       // German
            "el_gr" => Ok(Locale::ElGr),       // Greek
            "en_au" => Ok(Locale::EnAu),       // Australian English
            "en_ca" => Ok(Locale::EnCa),       // Canadian English
            "en_gb" => Ok(Locale::EnGb),       // British English
            "en_nz" => Ok(Locale::EnNz),       // New Zealand English
            "en_pt" => Ok(Locale::EnPt),       // Pirate English
            "en_ud" => Ok(Locale::EnUd),       // Upside down British English
            "en_us" => Ok(Locale::EnUs),       // American English
            "enp" => Ok(Locale::Enp),          // Modern English minus borrowed words
            "enws" => Ok(Locale::Enws),        // Early Modern English
            "eo_uy" => Ok(Locale::EoUy),       // Esperanto
            "es_ar" => Ok(Locale::EsAr),       // Argentinian Spanish
            "es_cl" => Ok(Locale::EsCl),       // Chilean Spanish
            "es_ec" => Ok(Locale::EsEc),       // Ecuadorian Spanish
            "es_es" => Ok(Locale::EsEs),       // European Spanish
            "es_mx" => Ok(Locale::EsMx),       // Mexican Spanish
            "es_uy" => Ok(Locale::EsUy),       // Uruguayan Spanish
            "es_ve" => Ok(Locale::EsVe),       // Venezuelan Spanish
            "esan" => Ok(Locale::Esan),        // Andalusian
            "et_ee" => Ok(Locale::EtEe),       // Estonian
            "eu_es" => Ok(Locale::EuEs),       // Basque
            "fa_ir" => Ok(Locale::FaIr),       // Persian
            "fi_fi" => Ok(Locale::FiFi),       // Finnish
            "fil_ph" => Ok(Locale::FilPh),     // Filipino
            "fo_fo" => Ok(Locale::FoFo),       // Faroese
            "fr_ca" => Ok(Locale::FrCa),       // Canadian French
            "fr_fr" => Ok(Locale::FrFr),       // European French
            "fra_de" => Ok(Locale::FraDe),     // East Franconian
            "fur_it" => Ok(Locale::FurIt),     // Friulian
            "fy_nl" => Ok(Locale::FyNl),       // Frisian
            "ga_ie" => Ok(Locale::GaIe),       // Irish
            "gd_gb" => Ok(Locale::GdGb),       // Scottish Gaelic
            "gl_es" => Ok(Locale::GlEs),       // Galician
            "haw_us" => Ok(Locale::HawUs),     // Hawaiian
            "he_il" => Ok(Locale::HeIl),       // Hebrew
            "hi_in" => Ok(Locale::HiIn),       // Hindi
            "hr_hr" => Ok(Locale::HrHr),       // Croatian
            "hu_hu" => Ok(Locale::HuHu),       // Hungarian
            "hy_am" => Ok(Locale::HyAm),       // Armenian
            "id_id" => Ok(Locale::IdId),       // Indonesian
            "ig_ng" => Ok(Locale::IgNg),       // Igbo
            "io_en" => Ok(Locale::IoEn),       // Ido
            "is_is" => Ok(Locale::IsIs),       // Icelandic
            "isv" => Ok(Locale::Isv),          // Interslavic
            "it_it" => Ok(Locale::ItIt),       // Italian
            "ja_jp" => Ok(Locale::JaJp),       // Japanese
            "jbo_en" => Ok(Locale::JboEn),     // Lojban
            "ka_ge" => Ok(Locale::KaGe),       // Georgian
            "kk_kz" => Ok(Locale::KkKz),       // Kazakh
            "kn_in" => Ok(Locale::KnIn),       // Kannada
            "ko_kr" => Ok(Locale::KoKr),       // Korean
            "ksh" => Ok(Locale::Ksh),          // Kölsch/Ripuarian
            "kw_gb" => Ok(Locale::KwGb),       // Cornish
            "la_la" => Ok(Locale::LaLa),       // Latin
            "lb_lu" => Ok(Locale::LbLu),       // Luxembourgish
            "li_li" => Ok(Locale::LiLi),       // Limburgish
            "lmo" => Ok(Locale::Lmo),          // Lombard
            "lo_la" => Ok(Locale::LoLa),       // Lao
            "lol_us" => Ok(Locale::LolUs),     // LOLCAT
            "lt_lt" => Ok(Locale::LtLt),       // Lithuanian
            "lv_lv" => Ok(Locale::LvLv),       // Latvian
            "lzh" => Ok(Locale::Lzh),          // Classical Chinese
            "mk_mk" => Ok(Locale::MkMk),       // Macedonian
            "mn_mn" => Ok(Locale::MnMn),       // Mongolian
            "ms_my" => Ok(Locale::MsMy),       // Malay
            "mt_mt" => Ok(Locale::MtMt),       // Maltese
            "nah" => Ok(Locale::Nah),          // Nahuatl
            "nds_de" => Ok(Locale::NdsDe),     // Low German
            "nl_be" => Ok(Locale::NlBe),       // Dutch, Flemish
            "nl_nl" => Ok(Locale::NlNl),       // Dutch
            "nn_no" => Ok(Locale::NnNo),       // Norwegian Nynorsk
            "no_no" => Ok(Locale::NoNo),       // Norwegian Bokmål
            "oc_fr" => Ok(Locale::OcFr),       // Occitan
            "ovd" => Ok(Locale::Ovd),          // Elfdalian
            "pl_pl" => Ok(Locale::PlPl),       // Polish
            "pt_br" => Ok(Locale::PtBr),       // Brazilian Portuguese
            "pt_pt" => Ok(Locale::PtPt),       // European Portuguese
            "qya_aa" => Ok(Locale::QyaAa),     // Quenya (Form of Elvish from LOTR)
            "ro_ro" => Ok(Locale::RoRo),       // Romanian
            "rpr" => Ok(Locale::Rpr),          // Russian (Pre-revolutionary)
            "ru_ru" => Ok(Locale::RuRu),       // Russian
            "ry_ua" => Ok(Locale::RyUa),       // Rusyn
            "sah_sah" => Ok(Locale::SahSah),   // Yakut
            "se_no" => Ok(Locale::SeNo),       // Northern Sami
            "sk_sk" => Ok(Locale::SkSk),       // Slovak
            "sl_si" => Ok(Locale::SlSi),       // Slovenian
            "so_so" => Ok(Locale::SoSo),       // Somali
            "sq_al" => Ok(Locale::SqAl),       // Albanian
            "sr_cs" => Ok(Locale::SrCs),       // Serbian (Latin)
            "sr_sp" => Ok(Locale::SrSp),       // Serbian (Cyrillic)
            "sv_se" => Ok(Locale::SvSe),       // Swedish
            "sxu" => Ok(Locale::Sxu),          // Upper Saxon German
            "szl" => Ok(Locale::Szl),          // Silesian
            "ta_in" => Ok(Locale::TaIn),       // Tamil
            "th_th" => Ok(Locale::ThTh),       // Thai
            "tl_ph" => Ok(Locale::TlPh),       // Tagalog
            "tlh_aa" => Ok(Locale::TlhAa),     // Klingon
            "tok" => Ok(Locale::Tok),          // Toki Pona
            "tr_tr" => Ok(Locale::TrTr),       // Turkish
            "tt_ru" => Ok(Locale::TtRu),       // Tatar
            "uk_ua" => Ok(Locale::UkUa),       // Ukrainian
            "val_es" => Ok(Locale::ValEs),     // Valencian
            "vec_it" => Ok(Locale::VecIt),     // Venetian
            "vi_vn" => Ok(Locale::ViVn),       // Vietnamese
            "yi_de" => Ok(Locale::YiDe),       // Yiddish
            "yo_ng" => Ok(Locale::YoNg),       // Yoruba
            "zh_cn" => Ok(Locale::ZhCn),       // Chinese Simplified (China; Mandarin)
            "zh_hk" => Ok(Locale::ZhHk),       // Chinese Traditional (Hong Kong; Mix)
            "zh_tw" => Ok(Locale::ZhTw),       // Chinese Traditional (Taiwan; Mandarin)
            "zlm_arab" => Ok(Locale::ZlmArab), // Malay (Jawi)
            _ => Ok(Locale::EnUs),             // Default to English (US) if not found
        }
    }
}
