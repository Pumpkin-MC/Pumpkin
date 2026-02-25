use std::{
    borrow::Cow,
    collections::HashMap,
    str::FromStr,
    sync::{LazyLock, Mutex},
};

use crate::text::{TextComponentBase, TextContent, style::Style};

static VANILLA_EN_US_JSON: &str = include_str!("../../assets/en_us.json");
static PUMPKIN_AF_ZA_JSON: &str = include_str!("../../assets/translations/af_za.json");
static PUMPKIN_AR_SA_JSON: &str = include_str!("../../assets/translations/ar_sa.json");
static PUMPKIN_AST_ES_JSON: &str = include_str!("../../assets/translations/ast_es.json");
static PUMPKIN_AZ_AZ_JSON: &str = include_str!("../../assets/translations/az_az.json");
static PUMPKIN_BA_RU_JSON: &str = include_str!("../../assets/translations/ba_ru.json");
static PUMPKIN_BAR_JSON: &str = include_str!("../../assets/translations/bar.json");
static PUMPKIN_BE_BY_JSON: &str = include_str!("../../assets/translations/be_by.json");
static PUMPKIN_BG_BG_JSON: &str = include_str!("../../assets/translations/bg_bg.json");
static PUMPKIN_BR_FR_JSON: &str = include_str!("../../assets/translations/br_fr.json");
static PUMPKIN_BRB_JSON: &str = include_str!("../../assets/translations/brb.json");
static PUMPKIN_BS_BA_JSON: &str = include_str!("../../assets/translations/bs_ba.json");
static PUMPKIN_CA_ES_JSON: &str = include_str!("../../assets/translations/ca_es.json");
static PUMPKIN_CS_CZ_JSON: &str = include_str!("../../assets/translations/cs_cz.json");
static PUMPKIN_CY_GB_JSON: &str = include_str!("../../assets/translations/cy_gb.json");
static PUMPKIN_DA_DK_JSON: &str = include_str!("../../assets/translations/da_dk.json");
static PUMPKIN_DE_AT_JSON: &str = include_str!("../../assets/translations/de_at.json");
static PUMPKIN_DE_CH_JSON: &str = include_str!("../../assets/translations/de_ch.json");
static PUMPKIN_DE_DE_JSON: &str = include_str!("../../assets/translations/de_de.json");
static PUMPKIN_EL_GR_JSON: &str = include_str!("../../assets/translations/el_gr.json");
static PUMPKIN_EN_AU_JSON: &str = include_str!("../../assets/translations/en_au.json");
static PUMPKIN_EN_CA_JSON: &str = include_str!("../../assets/translations/en_ca.json");
static PUMPKIN_EN_GB_JSON: &str = include_str!("../../assets/translations/en_gb.json");
static PUMPKIN_EN_NZ_JSON: &str = include_str!("../../assets/translations/en_nz.json");
static PUMPKIN_EN_PT_JSON: &str = include_str!("../../assets/translations/en_pt.json");
static PUMPKIN_EN_UD_JSON: &str = include_str!("../../assets/translations/en_ud.json");
static PUMPKIN_EN_US_JSON: &str = include_str!("../../assets/translations/en_us.json");
static PUMPKIN_ENP_JSON: &str = include_str!("../../assets/translations/enp.json");
static PUMPKIN_ENWS_JSON: &str = include_str!("../../assets/translations/enws.json");
static PUMPKIN_EO_UY_JSON: &str = include_str!("../../assets/translations/eo_uy.json");
static PUMPKIN_ES_AR_JSON: &str = include_str!("../../assets/translations/es_ar.json");
static PUMPKIN_ES_CL_JSON: &str = include_str!("../../assets/translations/es_cl.json");
static PUMPKIN_ES_EC_JSON: &str = include_str!("../../assets/translations/es_ec.json");
static PUMPKIN_ES_ES_JSON: &str = include_str!("../../assets/translations/es_es.json");
static PUMPKIN_ES_MX_JSON: &str = include_str!("../../assets/translations/es_mx.json");
static PUMPKIN_ES_UY_JSON: &str = include_str!("../../assets/translations/es_uy.json");
static PUMPKIN_ES_VE_JSON: &str = include_str!("../../assets/translations/es_ve.json");
static PUMPKIN_ESAN_JSON: &str = include_str!("../../assets/translations/esan.json");
static PUMPKIN_ET_EE_JSON: &str = include_str!("../../assets/translations/et_ee.json");
static PUMPKIN_EU_ES_JSON: &str = include_str!("../../assets/translations/eu_es.json");
static PUMPKIN_FA_IR_JSON: &str = include_str!("../../assets/translations/fa_ir.json");
static PUMPKIN_FI_FI_JSON: &str = include_str!("../../assets/translations/fi_fi.json");
static PUMPKIN_FIL_PH_JSON: &str = include_str!("../../assets/translations/fil_ph.json");
static PUMPKIN_FO_FO_JSON: &str = include_str!("../../assets/translations/fo_fo.json");
static PUMPKIN_FR_CA_JSON: &str = include_str!("../../assets/translations/fr_ca.json");
static PUMPKIN_FR_FR_JSON: &str = include_str!("../../assets/translations/fr_fr.json");
static PUMPKIN_FRA_DE_JSON: &str = include_str!("../../assets/translations/fra_de.json");
static PUMPKIN_FUR_IT_JSON: &str = include_str!("../../assets/translations/fur_it.json");
static PUMPKIN_FY_NL_JSON: &str = include_str!("../../assets/translations/fy_nl.json");
static PUMPKIN_GA_IE_JSON: &str = include_str!("../../assets/translations/ga_ie.json");
static PUMPKIN_GD_GB_JSON: &str = include_str!("../../assets/translations/gd_gb.json");
static PUMPKIN_GL_ES_JSON: &str = include_str!("../../assets/translations/gl_es.json");
static PUMPKIN_HAW_US_JSON: &str = include_str!("../../assets/translations/haw_us.json");
static PUMPKIN_HE_IL_JSON: &str = include_str!("../../assets/translations/he_il.json");
static PUMPKIN_HI_IN_JSON: &str = include_str!("../../assets/translations/hi_in.json");
static PUMPKIN_HR_HR_JSON: &str = include_str!("../../assets/translations/hr_hr.json");
static PUMPKIN_HU_HU_JSON: &str = include_str!("../../assets/translations/hu_hu.json");
static PUMPKIN_HY_AM_JSON: &str = include_str!("../../assets/translations/hy_am.json");
static PUMPKIN_ID_ID_JSON: &str = include_str!("../../assets/translations/id_id.json");
static PUMPKIN_IG_NG_JSON: &str = include_str!("../../assets/translations/ig_ng.json");
static PUMPKIN_IO_EN_JSON: &str = include_str!("../../assets/translations/io_en.json");
static PUMPKIN_IS_IS_JSON: &str = include_str!("../../assets/translations/is_is.json");
static PUMPKIN_ISV_JSON: &str = include_str!("../../assets/translations/isv.json");
static PUMPKIN_IT_IT_JSON: &str = include_str!("../../assets/translations/it_it.json");
static PUMPKIN_JA_JP_JSON: &str = include_str!("../../assets/translations/ja_jp.json");
static PUMPKIN_JBO_EN_JSON: &str = include_str!("../../assets/translations/jbo_en.json");
static PUMPKIN_KA_GE_JSON: &str = include_str!("../../assets/translations/ka_ge.json");
static PUMPKIN_KK_KZ_JSON: &str = include_str!("../../assets/translations/kk_kz.json");
static PUMPKIN_KN_IN_JSON: &str = include_str!("../../assets/translations/kn_in.json");
static PUMPKIN_KO_KR_JSON: &str = include_str!("../../assets/translations/ko_kr.json");
static PUMPKIN_KSH_JSON: &str = include_str!("../../assets/translations/ksh.json");
static PUMPKIN_KW_GB_JSON: &str = include_str!("../../assets/translations/kw_gb.json");
static PUMPKIN_LA_LA_JSON: &str = include_str!("../../assets/translations/la_la.json");
static PUMPKIN_LB_LU_JSON: &str = include_str!("../../assets/translations/lb_lu.json");
static PUMPKIN_LI_LI_JSON: &str = include_str!("../../assets/translations/li_li.json");
static PUMPKIN_LMO_JSON: &str = include_str!("../../assets/translations/lmo.json");
static PUMPKIN_LO_LA_JSON: &str = include_str!("../../assets/translations/lo_la.json");
static PUMPKIN_LOL_US_JSON: &str = include_str!("../../assets/translations/lol_us.json");
static PUMPKIN_LT_LT_JSON: &str = include_str!("../../assets/translations/lt_lt.json");
static PUMPKIN_LV_LV_JSON: &str = include_str!("../../assets/translations/lv_lv.json");
static PUMPKIN_LZH_JSON: &str = include_str!("../../assets/translations/lzh.json");
static PUMPKIN_MK_MK_JSON: &str = include_str!("../../assets/translations/mk_mk.json");
static PUMPKIN_MN_MN_JSON: &str = include_str!("../../assets/translations/mn_mn.json");
static PUMPKIN_MS_MY_JSON: &str = include_str!("../../assets/translations/ms_my.json");
static PUMPKIN_MT_MT_JSON: &str = include_str!("../../assets/translations/mt_mt.json");
static PUMPKIN_NAH_JSON: &str = include_str!("../../assets/translations/nah.json");
static PUMPKIN_NDS_DE_JSON: &str = include_str!("../../assets/translations/nds_de.json");
static PUMPKIN_NL_BE_JSON: &str = include_str!("../../assets/translations/nl_be.json");
static PUMPKIN_NL_NL_JSON: &str = include_str!("../../assets/translations/nl_nl.json");
static PUMPKIN_NN_NO_JSON: &str = include_str!("../../assets/translations/nn_no.json");
static PUMPKIN_NO_NO_JSON: &str = include_str!("../../assets/translations/no_no.json");
static PUMPKIN_OC_FR_JSON: &str = include_str!("../../assets/translations/oc_fr.json");
static PUMPKIN_OVD_JSON: &str = include_str!("../../assets/translations/ovd.json");
static PUMPKIN_PL_PL_JSON: &str = include_str!("../../assets/translations/pl_pl.json");
static PUMPKIN_PT_BR_JSON: &str = include_str!("../../assets/translations/pt_br.json");
static PUMPKIN_PT_PT_JSON: &str = include_str!("../../assets/translations/pt_pt.json");
static PUMPKIN_QYA_AA_JSON: &str = include_str!("../../assets/translations/qya_aa.json");
static PUMPKIN_RO_RO_JSON: &str = include_str!("../../assets/translations/ro_ro.json");
static PUMPKIN_RPR_JSON: &str = include_str!("../../assets/translations/rpr.json");
static PUMPKIN_RU_RU_JSON: &str = include_str!("../../assets/translations/ru_ru.json");
static PUMPKIN_RY_UA_JSON: &str = include_str!("../../assets/translations/ry_ua.json");
static PUMPKIN_SAH_SAH_JSON: &str = include_str!("../../assets/translations/sah_sah.json");
static PUMPKIN_SE_NO_JSON: &str = include_str!("../../assets/translations/se_no.json");
static PUMPKIN_SK_SK_JSON: &str = include_str!("../../assets/translations/sk_sk.json");
static PUMPKIN_SL_SI_JSON: &str = include_str!("../../assets/translations/sl_si.json");
static PUMPKIN_SO_SO_JSON: &str = include_str!("../../assets/translations/so_so.json");
static PUMPKIN_SQ_AL_JSON: &str = include_str!("../../assets/translations/sq_al.json");
static PUMPKIN_SR_CS_JSON: &str = include_str!("../../assets/translations/sr_cs.json");
static PUMPKIN_SR_SP_JSON: &str = include_str!("../../assets/translations/sr_sp.json");
static PUMPKIN_SV_SE_JSON: &str = include_str!("../../assets/translations/sv_se.json");
static PUMPKIN_SXU_JSON: &str = include_str!("../../assets/translations/sxu.json");
static PUMPKIN_SZL_JSON: &str = include_str!("../../assets/translations/szl.json");
static PUMPKIN_TA_IN_JSON: &str = include_str!("../../assets/translations/ta_in.json");
static PUMPKIN_TH_TH_JSON: &str = include_str!("../../assets/translations/th_th.json");
static PUMPKIN_TL_PH_JSON: &str = include_str!("../../assets/translations/tl_ph.json");
static PUMPKIN_TLH_AA_JSON: &str = include_str!("../../assets/translations/tlh_aa.json");
static PUMPKIN_TOK_JSON: &str = include_str!("../../assets/translations/tok.json");
static PUMPKIN_TR_TR_JSON: &str = include_str!("../../assets/translations/tr_tr.json");
static PUMPKIN_TT_RU_JSON: &str = include_str!("../../assets/translations/tt_ru.json");
static PUMPKIN_UK_UA_JSON: &str = include_str!("../../assets/translations/uk_ua.json");
static PUMPKIN_VAL_ES_JSON: &str = include_str!("../../assets/translations/val_es.json");
static PUMPKIN_VEC_IT_JSON: &str = include_str!("../../assets/translations/vec_it.json");
static PUMPKIN_VI_VN_JSON: &str = include_str!("../../assets/translations/vi_vn.json");
static PUMPKIN_YI_DE_JSON: &str = include_str!("../../assets/translations/yi_de.json");
static PUMPKIN_YO_NG_JSON: &str = include_str!("../../assets/translations/yo_ng.json");
static PUMPKIN_ZH_CN_JSON: &str = include_str!("../../assets/translations/zh_cn.json");
static PUMPKIN_ZH_HK_JSON: &str = include_str!("../../assets/translations/zh_hk.json");
static PUMPKIN_ZH_TW_JSON: &str = include_str!("../../assets/translations/zh_tw.json");
static PUMPKIN_ZLM_ARAB_JSON: &str = include_str!("../../assets/translations/zlm_arab.json");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubstitutionRange {
    pub start: usize,
    pub end: usize,
}
impl SubstitutionRange {
    #[must_use]
    pub const fn len(&self) -> usize {
        (self.end - self.start) + 1
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
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
    translations[locale as usize].get(&key).map_or_else(
        || {
            translations[Locale::EnUs as usize]
                .get(&key)
                .map_or(key, Clone::clone)
        },
        Clone::clone,
    )
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

pub static TRANSLATIONS: LazyLock<Mutex<[HashMap<String, String>; Locale::last() as usize + 1]>> =
    LazyLock::new(|| {
        let mut array: [HashMap<String, String>; Locale::last() as usize + 1] =
            std::array::from_fn(|_| HashMap::new());
        let vanilla_en_us: HashMap<String, String> =
            serde_json::from_str(VANILLA_EN_US_JSON).expect("Could not parse en_us.json.");
        let pumpkin_af_za: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_AF_ZA_JSON).expect("Could not parse af_za.json.");
        let pumpkin_ar_sa: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_AR_SA_JSON).expect("Could not parse ar_sa.json.");
        let pumpkin_ast_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_AST_ES_JSON).expect("Could not parse ast_es.json.");
        let pumpkin_az_az: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_AZ_AZ_JSON).expect("Could not parse az_az.json.");
        let pumpkin_ba_ru: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BA_RU_JSON).expect("Could not parse ba_ru.json.");
        let pumpkin_bar: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BAR_JSON).expect("Could not parse bar.json.");
        let pumpkin_be_by: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BE_BY_JSON).expect("Could not parse be_by.json.");
        let pumpkin_bg_bg: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BG_BG_JSON).expect("Could not parse bg_bg.json.");
        let pumpkin_br_fr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BR_FR_JSON).expect("Could not parse br_fr.json.");
        let pumpkin_brb: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BRB_JSON).expect("Could not parse brb.json.");
        let pumpkin_bs_ba: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_BS_BA_JSON).expect("Could not parse bs_ba.json.");
        let pumpkin_ca_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_CA_ES_JSON).expect("Could not parse ca_es.json.");
        let pumpkin_cs_cz: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_CS_CZ_JSON).expect("Could not parse cs_cz.json.");
        let pumpkin_cy_gb: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_CY_GB_JSON).expect("Could not parse cy_gb.json.");
        let pumpkin_da_dk: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_DA_DK_JSON).expect("Could not parse da_dk.json.");
        let pumpkin_de_at: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_DE_AT_JSON).expect("Could not parse de_at.json.");
        let pumpkin_de_ch: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_DE_CH_JSON).expect("Could not parse de_ch.json.");
        let pumpkin_de_de: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_DE_DE_JSON).expect("Could not parse de_de.json.");
        let pumpkin_el_gr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EL_GR_JSON).expect("Could not parse el_gr.json.");
        let pumpkin_en_au: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_AU_JSON).expect("Could not parse en_au.json.");
        let pumpkin_en_ca: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_CA_JSON).expect("Could not parse en_ca.json.");
        let pumpkin_en_gb: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_GB_JSON).expect("Could not parse en_gb.json.");
        let pumpkin_en_nz: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_NZ_JSON).expect("Could not parse en_nz.json.");
        let pumpkin_en_pt: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_PT_JSON).expect("Could not parse en_pt.json.");
        let pumpkin_en_ud: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_UD_JSON).expect("Could not parse en_ud.json.");
        let pumpkin_en_us: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EN_US_JSON).expect("Could not parse en_us.json.");
        let pumpkin_enp: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ENP_JSON).expect("Could not parse enp.json.");
        let pumpkin_enws: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ENWS_JSON).expect("Could not parse enws.json.");
        let pumpkin_eo_uy: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EO_UY_JSON).expect("Could not parse eo_uy.json.");
        let pumpkin_es_ar: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_AR_JSON).expect("Could not parse es_ar.json.");
        let pumpkin_es_cl: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_CL_JSON).expect("Could not parse es_cl.json.");
        let pumpkin_es_ec: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_EC_JSON).expect("Could not parse es_ec.json.");
        let pumpkin_es_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_ES_JSON).expect("Could not parse es_es.json.");
        let pumpkin_es_mx: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_MX_JSON).expect("Could not parse es_mx.json.");
        let pumpkin_es_uy: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_UY_JSON).expect("Could not parse es_uy.json.");
        let pumpkin_es_ve: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ES_VE_JSON).expect("Could not parse es_ve.json.");
        let pumpkin_esan: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ESAN_JSON).expect("Could not parse esan.json.");
        let pumpkin_et_ee: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ET_EE_JSON).expect("Could not parse et_ee.json.");
        let pumpkin_eu_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_EU_ES_JSON).expect("Could not parse eu_es.json.");
        let pumpkin_fa_ir: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FA_IR_JSON).expect("Could not parse fa_ir.json.");
        let pumpkin_fi_fi: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FI_FI_JSON).expect("Could not parse fi_fi.json.");
        let pumpkin_fil_ph: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FIL_PH_JSON).expect("Could not parse fil_ph.json.");
        let pumpkin_fo_fo: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FO_FO_JSON).expect("Could not parse fo_fo.json.");
        let pumpkin_fr_ca: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FR_CA_JSON).expect("Could not parse fr_ca.json.");
        let pumpkin_fr_fr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FR_FR_JSON).expect("Could not parse fr_fr.json.");
        let pumpkin_fra_de: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FRA_DE_JSON).expect("Could not parse fra_de.json.");
        let pumpkin_fur_it: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FUR_IT_JSON).expect("Could not parse fur_it.json.");
        let pumpkin_fy_nl: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_FY_NL_JSON).expect("Could not parse fy_nl.json.");
        let pumpkin_ga_ie: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_GA_IE_JSON).expect("Could not parse ga_ie.json.");
        let pumpkin_gd_gb: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_GD_GB_JSON).expect("Could not parse gd_gb.json.");
        let pumpkin_gl_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_GL_ES_JSON).expect("Could not parse gl_es.json.");
        let pumpkin_haw_us: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_HAW_US_JSON).expect("Could not parse haw_us.json.");
        let pumpkin_he_il: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_HE_IL_JSON).expect("Could not parse he_il.json.");
        let pumpkin_hi_in: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_HI_IN_JSON).expect("Could not parse hi_in.json.");
        let pumpkin_hr_hr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_HR_HR_JSON).expect("Could not parse hr_hr.json.");
        let pumpkin_hu_hu: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_HU_HU_JSON).expect("Could not parse hu_hu.json.");
        let pumpkin_hy_am: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_HY_AM_JSON).expect("Could not parse hy_am.json.");
        let pumpkin_id_id: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ID_ID_JSON).expect("Could not parse id_id.json.");
        let pumpkin_ig_ng: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_IG_NG_JSON).expect("Could not parse ig_ng.json.");
        let pumpkin_io_en: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_IO_EN_JSON).expect("Could not parse io_en.json.");
        let pumpkin_is_is: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_IS_IS_JSON).expect("Could not parse is_is.json.");
        let pumpkin_isv: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ISV_JSON).expect("Could not parse isv.json.");
        let pumpkin_it_it: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_IT_IT_JSON).expect("Could not parse it_it.json.");
        let pumpkin_ja_jp: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_JA_JP_JSON).expect("Could not parse ja_jp.json.");
        let pumpkin_jbo_en: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_JBO_EN_JSON).expect("Could not parse jbo_en.json.");
        let pumpkin_ka_ge: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KA_GE_JSON).expect("Could not parse ka_ge.json.");
        let pumpkin_kk_kz: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KK_KZ_JSON).expect("Could not parse kk_kz.json.");
        let pumpkin_kn_in: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KN_IN_JSON).expect("Could not parse kn_in.json.");
        let pumpkin_ko_kr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KO_KR_JSON).expect("Could not parse ko_kr.json.");
        let pumpkin_ksh: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KSH_JSON).expect("Could not parse ksh.json.");
        let pumpkin_kw_gb: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_KW_GB_JSON).expect("Could not parse kw_gb.json.");
        let pumpkin_la_la: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LA_LA_JSON).expect("Could not parse la_la.json.");
        let pumpkin_lb_lu: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LB_LU_JSON).expect("Could not parse lb_lu.json.");
        let pumpkin_li_li: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LI_LI_JSON).expect("Could not parse li_li.json.");
        let pumpkin_lmo: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LMO_JSON).expect("Could not parse lmo.json.");
        let pumpkin_lo_la: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LO_LA_JSON).expect("Could not parse lo_la.json.");
        let pumpkin_lol_us: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LOL_US_JSON).expect("Could not parse lol_us.json.");
        let pumpkin_lt_lt: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LT_LT_JSON).expect("Could not parse lt_lt.json.");
        let pumpkin_lv_lv: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LV_LV_JSON).expect("Could not parse lv_lv.json.");
        let pumpkin_lzh: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_LZH_JSON).expect("Could not parse lzh.json.");
        let pumpkin_mk_mk: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_MK_MK_JSON).expect("Could not parse mk_mk.json.");
        let pumpkin_mn_mn: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_MN_MN_JSON).expect("Could not parse mn_mn.json.");
        let pumpkin_ms_my: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_MS_MY_JSON).expect("Could not parse ms_my.json.");
        let pumpkin_mt_mt: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_MT_MT_JSON).expect("Could not parse mt_mt.json.");
        let pumpkin_nah: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NAH_JSON).expect("Could not parse nah.json.");
        let pumpkin_nds_de: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NDS_DE_JSON).expect("Could not parse nds_de.json.");
        let pumpkin_nl_be: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NL_BE_JSON).expect("Could not parse nl_be.json.");
        let pumpkin_nl_nl: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NL_NL_JSON).expect("Could not parse nl_nl.json.");
        let pumpkin_nn_no: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NN_NO_JSON).expect("Could not parse nn_no.json.");
        let pumpkin_no_no: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_NO_NO_JSON).expect("Could not parse no_no.json.");
        let pumpkin_oc_fr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_OC_FR_JSON).expect("Could not parse oc_fr.json.");
        let pumpkin_ovd: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_OVD_JSON).expect("Could not parse ovd.json.");
        let pumpkin_pl_pl: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_PL_PL_JSON).expect("Could not parse pl_pl.json.");
        let pumpkin_pt_br: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_PT_BR_JSON).expect("Could not parse pt_br.json.");
        let pumpkin_pt_pt: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_PT_PT_JSON).expect("Could not parse pt_pt.json.");
        let pumpkin_qya_aa: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_QYA_AA_JSON).expect("Could not parse qya_aa.json.");
        let pumpkin_ro_ro: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_RO_RO_JSON).expect("Could not parse ro_ro.json.");
        let pumpkin_rpr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_RPR_JSON).expect("Could not parse rpr.json.");
        let pumpkin_ru_ru: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_RU_RU_JSON).expect("Could not parse ru_ru.json.");
        let pumpkin_ry_ua: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_RY_UA_JSON).expect("Could not parse ry_ua.json.");
        let pumpkin_sah_sah: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SAH_SAH_JSON).expect("Could not parse sah_sah.json.");
        let pumpkin_se_no: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SE_NO_JSON).expect("Could not parse se_no.json.");
        let pumpkin_sk_sk: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SK_SK_JSON).expect("Could not parse sk_sk.json.");
        let pumpkin_sl_si: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SL_SI_JSON).expect("Could not parse sl_si.json.");
        let pumpkin_so_so: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SO_SO_JSON).expect("Could not parse so_so.json.");
        let pumpkin_sq_al: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SQ_AL_JSON).expect("Could not parse sq_al.json.");
        let pumpkin_sr_cs: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SR_CS_JSON).expect("Could not parse sr_cs.json.");
        let pumpkin_sr_sp: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SR_SP_JSON).expect("Could not parse sr_sp.json.");
        let pumpkin_sv_se: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SV_SE_JSON).expect("Could not parse sv_se.json.");
        let pumpkin_sxu: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SXU_JSON).expect("Could not parse sxu.json.");
        let pumpkin_szl: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_SZL_JSON).expect("Could not parse szl.json.");
        let pumpkin_ta_in: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TA_IN_JSON).expect("Could not parse ta_in.json.");
        let pumpkin_th_th: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TH_TH_JSON).expect("Could not parse th_th.json.");
        let pumpkin_tl_ph: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TL_PH_JSON).expect("Could not parse tl_ph.json.");
        let pumpkin_tlh_aa: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TLH_AA_JSON).expect("Could not parse tlh_aa.json.");
        let pumpkin_tok: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TOK_JSON).expect("Could not parse tok.json.");
        let pumpkin_tr_tr: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TR_TR_JSON).expect("Could not parse tr_tr.json.");
        let pumpkin_tt_ru: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_TT_RU_JSON).expect("Could not parse tt_ru.json.");
        let pumpkin_uk_ua: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_UK_UA_JSON).expect("Could not parse uk_ua.json.");
        let pumpkin_val_es: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_VAL_ES_JSON).expect("Could not parse val_es.json.");
        let pumpkin_vec_it: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_VEC_IT_JSON).expect("Could not parse vec_it.json.");
        let pumpkin_vi_vn: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_VI_VN_JSON).expect("Could not parse vi_vn.json.");
        let pumpkin_yi_de: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_YI_DE_JSON).expect("Could not parse yi_de.json.");
        let pumpkin_yo_ng: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_YO_NG_JSON).expect("Could not parse yo_ng.json.");
        let pumpkin_zh_cn: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZH_CN_JSON).expect("Could not parse zh_cn.json.");
        let pumpkin_zh_hk: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZH_HK_JSON).expect("Could not parse zh_hk.json.");
        let pumpkin_zh_tw: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZH_TW_JSON).expect("Could not parse zh_tw.json.");
        let pumpkin_zlm_arab: HashMap<String, String> =
            serde_json::from_str(PUMPKIN_ZLM_ARAB_JSON).expect("Could not parse zlm_arab.json.");

        for (key, value) in vanilla_en_us {
            array[Locale::EnUs as usize].insert(format!("minecraft:{key}"), value);
        }
        for (key, value) in pumpkin_af_za {
            array[Locale::AfZa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ar_sa {
            array[Locale::ArSa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ast_es {
            array[Locale::AstEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_az_az {
            array[Locale::AzAz as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ba_ru {
            array[Locale::BaRu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_bar {
            array[Locale::Bar as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_be_by {
            array[Locale::BeBy as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_bg_bg {
            array[Locale::BgBg as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_br_fr {
            array[Locale::BrFr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_brb {
            array[Locale::Brb as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_bs_ba {
            array[Locale::BsBa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ca_es {
            array[Locale::CaEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_cs_cz {
            array[Locale::CsCz as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_cy_gb {
            array[Locale::CyGb as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_da_dk {
            array[Locale::DaDk as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_de_at {
            array[Locale::DeAt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_de_ch {
            array[Locale::DeCh as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_de_de {
            array[Locale::DeDe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_el_gr {
            array[Locale::ElGr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_en_au {
            array[Locale::EnAu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_en_ca {
            array[Locale::EnCa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_en_gb {
            array[Locale::EnGb as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_en_nz {
            array[Locale::EnNz as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_en_pt {
            array[Locale::EnPt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_en_ud {
            array[Locale::EnUd as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_en_us {
            array[Locale::EnUs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_enp {
            array[Locale::Enp as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_enws {
            array[Locale::Enws as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_eo_uy {
            array[Locale::EoUy as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_ar {
            array[Locale::EsAr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_cl {
            array[Locale::EsCl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_ec {
            array[Locale::EsEc as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_es {
            array[Locale::EsEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_mx {
            array[Locale::EsMx as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_uy {
            array[Locale::EsUy as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_es_ve {
            array[Locale::EsVe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_esan {
            array[Locale::Esan as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_et_ee {
            array[Locale::EtEe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_eu_es {
            array[Locale::EuEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fa_ir {
            array[Locale::FaIr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fi_fi {
            array[Locale::FiFi as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fil_ph {
            array[Locale::FilPh as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fo_fo {
            array[Locale::FoFo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fr_ca {
            array[Locale::FrCa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fr_fr {
            array[Locale::FrFr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fra_de {
            array[Locale::FraDe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fur_it {
            array[Locale::FurIt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_fy_nl {
            array[Locale::FyNl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ga_ie {
            array[Locale::GaIe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_gd_gb {
            array[Locale::GdGb as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_gl_es {
            array[Locale::GlEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_haw_us {
            array[Locale::HawUs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_he_il {
            array[Locale::HeIl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_hi_in {
            array[Locale::HiIn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_hr_hr {
            array[Locale::HrHr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_hu_hu {
            array[Locale::HuHu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_hy_am {
            array[Locale::HyAm as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_id_id {
            array[Locale::IdId as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ig_ng {
            array[Locale::IgNg as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_io_en {
            array[Locale::IoEn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_is_is {
            array[Locale::IsIs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_isv {
            array[Locale::Isv as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_it_it {
            array[Locale::ItIt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ja_jp {
            array[Locale::JaJp as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_jbo_en {
            array[Locale::JboEn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ka_ge {
            array[Locale::KaGe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_kk_kz {
            array[Locale::KkKz as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_kn_in {
            array[Locale::KnIn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ko_kr {
            array[Locale::KoKr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ksh {
            array[Locale::Ksh as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_kw_gb {
            array[Locale::KwGb as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_la_la {
            array[Locale::LaLa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lb_lu {
            array[Locale::LbLu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_li_li {
            array[Locale::LiLi as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lmo {
            array[Locale::Lmo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lo_la {
            array[Locale::LoLa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lol_us {
            array[Locale::LolUs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lt_lt {
            array[Locale::LtLt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lv_lv {
            array[Locale::LvLv as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_lzh {
            array[Locale::Lzh as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_mk_mk {
            array[Locale::MkMk as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_mn_mn {
            array[Locale::MnMn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ms_my {
            array[Locale::MsMy as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_mt_mt {
            array[Locale::MtMt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_nah {
            array[Locale::Nah as usize].insert(format!("pumpkin:{key}"), value);
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
        for (key, value) in pumpkin_nn_no {
            array[Locale::NnNo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_no_no {
            array[Locale::NoNo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_oc_fr {
            array[Locale::OcFr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ovd {
            array[Locale::Ovd as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_pl_pl {
            array[Locale::PlPl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_pt_br {
            array[Locale::PtBr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_pt_pt {
            array[Locale::PtPt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_qya_aa {
            array[Locale::QyaAa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ro_ro {
            array[Locale::RoRo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_rpr {
            array[Locale::Rpr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ru_ru {
            array[Locale::RuRu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ry_ua {
            array[Locale::RyUa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sah_sah {
            array[Locale::SahSah as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_se_no {
            array[Locale::SeNo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sk_sk {
            array[Locale::SkSk as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sl_si {
            array[Locale::SlSi as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_so_so {
            array[Locale::SoSo as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sq_al {
            array[Locale::SqAl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sr_cs {
            array[Locale::SrCs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sr_sp {
            array[Locale::SrSp as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sv_se {
            array[Locale::SvSe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_sxu {
            array[Locale::Sxu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_szl {
            array[Locale::Szl as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_ta_in {
            array[Locale::TaIn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_th_th {
            array[Locale::ThTh as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_tl_ph {
            array[Locale::TlPh as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_tlh_aa {
            array[Locale::TlhAa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_tok {
            array[Locale::Tok as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_tr_tr {
            array[Locale::TrTr as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_tt_ru {
            array[Locale::TtRu as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_uk_ua {
            array[Locale::UkUa as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_val_es {
            array[Locale::ValEs as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_vec_it {
            array[Locale::VecIt as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_vi_vn {
            array[Locale::ViVn as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_yi_de {
            array[Locale::YiDe as usize].insert(format!("pumpkin:{key}"), value);
        }
        for (key, value) in pumpkin_yo_ng {
            array[Locale::YoNg as usize].insert(format!("pumpkin:{key}"), value);
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
        for (key, value) in pumpkin_zlm_arab {
            array[Locale::ZlmArab as usize].insert(format!("pumpkin:{key}"), value);
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
    #[must_use]
    pub const fn last() -> Self {
        Self::ZlmArab
    }
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
