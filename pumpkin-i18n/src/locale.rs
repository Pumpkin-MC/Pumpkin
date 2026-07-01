use std::{borrow::Cow, str::FromStr};

// Include auto-generated locale codes array from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_locale_codes.rs"));

/// Supported locales for translations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Locale {
    AfZa,
    ArSa,
    AstEs,
    AzAz,
    Bar,
    BaRu,
    BeBy,
    BgBg,
    Brb,
    BrFr,
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
    Enp,
    EnPt,
    EnUd,
    EnUs,
    Enws,
    EoUy,
    Esan,
    EsAr,
    EsCl,
    EsEc,
    EsEs,
    EsMx,
    EsUy,
    EsVe,
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
    TlhAa,
    TlPh,
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

    /// Returns the lowercase underscore-separated locale code (e.g. `"en_us"`, `"zh_cn"`).
    #[must_use]
    pub fn to_code(self) -> &'static str {
        locale_code(self as usize)
    }
}

impl FromStr for Locale {
    type Err = ();

    #[expect(clippy::too_many_lines)]
    #[allow(clippy::match_same_arms)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match normalize_locale_code(s).as_ref() {
            "af_za" => Ok(Self::AfZa),       // Afrikaans (Suid-Afrika)
            "ar_sa" => Ok(Self::ArSa),       // Arabic
            "ast_es" => Ok(Self::AstEs),     // Asturian
            "az_az" => Ok(Self::AzAz),       // Azerbaijani
            "bar" => Ok(Self::Bar),          // Bavarian
            "ba_ru" => Ok(Self::BaRu),       // Bashkir
            "be_by" => Ok(Self::BeBy),       // Belarusian
            "bg_bg" => Ok(Self::BgBg),       // Bulgarian
            "brb" => Ok(Self::Brb),          // Brabantian
            "br_fr" => Ok(Self::BrFr),       // Breton
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
            "enp" => Ok(Self::Enp),          // Modern English minus borrowed words
            "en_pt" => Ok(Self::EnPt),       // Pirate English
            "en_ud" => Ok(Self::EnUd),       // Upside down British English
            "en_us" => Ok(Self::EnUs),       // English (US)
            "enws" => Ok(Self::Enws),        // Early Modern English
            "eo_uy" => Ok(Self::EoUy),       // Esperanto
            "esan" => Ok(Self::Esan),        // Andalusian
            "es_ar" => Ok(Self::EsAr),       // Argentinian Spanish
            "es_cl" => Ok(Self::EsCl),       // Chilean Spanish
            "es_ec" => Ok(Self::EsEc),       // Ecuadorian Spanish
            "es_es" => Ok(Self::EsEs),       // European Spanish
            "es_mx" => Ok(Self::EsMx),       // Mexican Spanish
            "es_uy" => Ok(Self::EsUy),       // Uruguayan Spanish
            "es_ve" => Ok(Self::EsVe),       // Venezuelan Spanish
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
            "tlh_aa" => Ok(Self::TlhAa),     // Klingon
            "tl_ph" => Ok(Self::TlPh),       // Tagalog
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

fn normalize_locale_code(raw: &str) -> Cow<'_, str> {
    if !raw
        .bytes()
        .any(|byte| byte == b'-' || byte.is_ascii_uppercase())
    {
        return Cow::Borrowed(raw);
    }

    let mut normalized = String::with_capacity(raw.len() + 2);
    let mut previous_was_separator = true;
    let mut previous_was_uppercase = false;

    for (idx, byte) in raw.bytes().enumerate() {
        match byte {
            b'-' => {
                normalized.push('_');
                previous_was_separator = true;
                previous_was_uppercase = false;
            }
            b'A'..=b'Z' => {
                if idx > 0 && !previous_was_separator && !previous_was_uppercase {
                    normalized.push('_');
                }
                normalized.push(byte.to_ascii_lowercase() as char);
                previous_was_separator = false;
                previous_was_uppercase = true;
            }
            _ => {
                normalized.push(byte as char);
                previous_was_separator = byte == b'_';
                previous_was_uppercase = false;
            }
        }
    }

    Cow::Owned(normalized)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Locale;

    #[test]
    fn parses_common_locale_spellings() {
        assert_eq!(Locale::from_str("en_us"), Ok(Locale::EnUs));
        assert_eq!(Locale::from_str("en-US"), Ok(Locale::EnUs));
        assert_eq!(Locale::from_str("EnUs"), Ok(Locale::EnUs));
        assert_eq!(Locale::from_str("ZH_CN"), Ok(Locale::ZhCn));
        assert_eq!(Locale::from_str("zlm-Arab"), Ok(Locale::ZlmArab));
    }

    #[test]
    fn to_code_returns_lowercase_underscore() {
        assert_eq!(Locale::EnUs.to_code(), "en_us");
        assert_eq!(Locale::ZhCn.to_code(), "zh_cn");
        assert_eq!(Locale::ZlmArab.to_code(), "zlm_arab");
        assert_eq!(Locale::Bar.to_code(), "bar");
        assert_eq!(Locale::BaRu.to_code(), "ba_ru");
        assert_eq!(Locale::Enp.to_code(), "enp");
        assert_eq!(Locale::DeDe.to_code(), "de_de");
    }

    #[test]
    fn to_code_round_trips_with_from_str() {
        // Every Locale variant's code should parse back to the same variant.
        for variant in 0..Locale::COUNT {
            let locale: Locale = unsafe { std::mem::transmute(variant as u8) };
            let code = locale.to_code();
            let parsed = Locale::from_str(code).unwrap();
            assert_eq!(
                parsed, locale,
                "round-trip failed for variant {variant}: code={code:?}"
            );
        }
    }
}
