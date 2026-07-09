use std::{borrow::Cow, str::FromStr};

/// Supported locales for translations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Locale {
    AfZa,
    ArSa,
    AstEs,
    AzAz,
    BaRu,
    Bar,
    BeBy,
    BeLatn,
    BgBg,
    BrFr,
    Brb,
    BsBa,
    CaEs,
    CsCz,
    CvCu,
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
    FrCh,
    FrFr,
    FraDe,
    FurIt,
    FyNl,
    GaIe,
    GdGb,
    GlEs,
    GoFr,
    HalUa,
    HawUs,
    HeIl,
    HiIn,
    HnNo,
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
    KyKg,
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
    Pls,
    PtBr,
    PtPt,
    QcbEs,
    Qid,
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
    TzoMx,
    UkUa,
    UzUz,
    ValEs,
    VecIt,
    ViVn,
    VpVl,
    Vro,
    YiDe,
    YoNg,
    ZhCn,
    ZhHk,
    ZhTw,
    ZlmArab,
}

impl Locale {
    pub const COUNT: usize = Self::ZlmArab as usize + 1;

    /// Sorted array of locale codes matching enum discriminant order.
    const LOCALE_CODES: [&str; Self::COUNT] = [
        "af_za", "ar_sa", "ast_es", "az_az", "ba_ru", "bar", "be_by", "be_latn", "bg_bg", "br_fr",
        "brb", "bs_ba", "ca_es", "cs_cz", "cv_cu", "cy_gb", "da_dk", "de_at", "de_ch", "de_de",
        "el_gr", "en_au", "en_ca", "en_gb", "en_nz", "en_pt", "en_ud", "en_us", "enp", "enws",
        "eo_uy", "es_ar", "es_cl", "es_ec", "es_es", "es_mx", "es_uy", "es_ve", "esan", "et_ee",
        "eu_es", "fa_ir", "fi_fi", "fil_ph", "fo_fo", "fr_ca", "fr_ch", "fr_fr", "fra_de",
        "fur_it", "fy_nl", "ga_ie", "gd_gb", "gl_es", "go_fr", "hal_ua", "haw_us", "he_il",
        "hi_in", "hn_no", "hr_hr", "hu_hu", "hy_am", "id_id", "ig_ng", "io_en", "is_is", "isv",
        "it_it", "ja_jp", "jbo_en", "ka_ge", "kk_kz", "kn_in", "ko_kr", "ksh", "kw_gb", "ky_kg",
        "la_la", "lb_lu", "li_li", "lmo", "lo_la", "lol_us", "lt_lt", "lv_lv", "lzh", "mk_mk",
        "mn_mn", "ms_my", "mt_mt", "nah", "nds_de", "nl_be", "nl_nl", "nn_no", "no_no", "oc_fr",
        "ovd", "pl_pl", "pls", "pt_br", "pt_pt", "qcb_es", "qid", "qya_aa", "ro_ro", "rpr",
        "ru_ru", "ry_ua", "sah_sah", "se_no", "sk_sk", "sl_si", "so_so", "sq_al", "sr_cs", "sr_sp",
        "sv_se", "sxu", "szl", "ta_in", "th_th", "tl_ph", "tlh_aa", "tok", "tr_tr", "tt_ru",
        "tzo_mx", "uk_ua", "uz_uz", "val_es", "vec_it", "vi_vn", "vp_vl", "vro", "yi_de", "yo_ng",
        "zh_cn", "zh_hk", "zh_tw", "zlm_arab",
    ];

    /// Returns the lowercase underscore-separated locale code (e.g. `"en_us"`, `"zh_cn"`).
    #[must_use]
    pub const fn to_code(self) -> &'static str {
        Self::LOCALE_CODES[self as usize]
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
            "ba_ru" => Ok(Self::BaRu),       // Bashkir
            "bar" => Ok(Self::Bar),          // Bavarian
            "be_by" => Ok(Self::BeBy),       // Belarusian
            "be_latn" => Ok(Self::BeLatn),   // Belarusian (Latin)
            "bg_bg" => Ok(Self::BgBg),       // Bulgarian
            "br_fr" => Ok(Self::BrFr),       // Breton
            "brb" => Ok(Self::Brb),          // Brabantian
            "bs_ba" => Ok(Self::BsBa),       // Bosnian
            "ca_es" => Ok(Self::CaEs),       // Catalan
            "cs_cz" => Ok(Self::CsCz),       // Czech
            "cv_cu" => Ok(Self::CvCu),       // Chuvash
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
            "en_us" => Ok(Self::EnUs),       // English (US)
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
            "fr_ch" => Ok(Self::FrCh),       // Swiss French
            "fr_fr" => Ok(Self::FrFr),       // European French
            "fra_de" => Ok(Self::FraDe),     // East Franconian
            "fur_it" => Ok(Self::FurIt),     // Friulian
            "fy_nl" => Ok(Self::FyNl),       // Frisian
            "ga_ie" => Ok(Self::GaIe),       // Irish
            "gd_gb" => Ok(Self::GdGb),       // Scottish Gaelic
            "gl_es" => Ok(Self::GlEs),       // Galician
            "go_fr" => Ok(Self::GoFr),       // Franco-Provençal
            "hal_ua" => Ok(Self::HalUa),     // Halych
            "haw_us" => Ok(Self::HawUs),     // Hawaiian
            "he_il" => Ok(Self::HeIl),       // Hebrew
            "hi_in" => Ok(Self::HiIn),       // Hindi
            "hn_no" => Ok(Self::HnNo),       // Høgnorsk
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
            "ky_kg" => Ok(Self::KyKg),       // Kyrgyz
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
            "pls" => Ok(Self::Pls),          // Pseudo-Latin
            "pt_br" => Ok(Self::PtBr),       // Brazilian Portuguese
            "pt_pt" => Ok(Self::PtPt),       // European Portuguese
            "qcb_es" => Ok(Self::QcbEs),     // Querétaro Otomi
            "qid" => Ok(Self::Qid),          // Quenya (Sindarin inspired)
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
            "tzo_mx" => Ok(Self::TzoMx),     // Tzotzil
            "uk_ua" => Ok(Self::UkUa),       // Ukrainian
            "uz_uz" => Ok(Self::UzUz),       // Uzbek
            "val_es" => Ok(Self::ValEs),     // Valencian
            "vec_it" => Ok(Self::VecIt),     // Venetian
            "vi_vn" => Ok(Self::ViVn),       // Vietnamese
            "vp_vl" => Ok(Self::VpVl),       // Võro (South Estonian variant)
            "vro" => Ok(Self::Vro),          // Võro
            "yi_de" => Ok(Self::YiDe),       // Yiddish
            "yo_ng" => Ok(Self::YoNg),       // Yoruba
            "zh_cn" => Ok(Self::ZhCn),       // Chinese Simplified (China; Mandarin)
            "zh_hk" => Ok(Self::ZhHk),       // Chinese Traditional (Hong Kong; Mix)
            "zh_tw" => Ok(Self::ZhTw),       // Chinese Traditional (Taiwan; Mandarin)
            "zlm_arab" => Ok(Self::ZlmArab), // Malay (Jawi)
            _ => Err(()),                    // Unrecognized locale code — caller should handle
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
