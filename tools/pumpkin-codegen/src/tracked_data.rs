use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

use crate::version::JavaMinecraftVersion;

/// The newest protocol version used as the fallback for unknown versions in `TrackedId::get`.
const LATEST_VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_26_2;

#[derive(Deserialize)]
struct RawTrackedField {
    id: u8,
    r#type: String,
    #[allow(dead_code)]
    type_id: u8,
}

/// Generates the `TokenStream` for `TrackedId`, `TrackedData`, and all per-entity tracking modules.
pub(crate) fn build() -> TokenStream {
    let assets = [
        (JavaMinecraftVersion::V_1_21, "1_21_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_2, "1_21_2_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_4, "1_21_4_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_5, "1_21_5_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_6, "1_21_6_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_7, "1_21_7_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_9, "1_21_9_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_11, "1_21_11_tracked_data.json"),
        (JavaMinecraftVersion::V_26_1, "26_1_tracked_data.json"),
        (JavaMinecraftVersion::V_26_2, "26_2_tracked_data.json"),
    ];

    let mut raw_versions = BTreeMap::new();
    for (ver, file) in assets {
        let path = format!("../../assets/tracked_data/{file}");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(parsed) = serde_json::from_str::<
                BTreeMap<String, BTreeMap<String, RawTrackedField>>,
            >(&content)
            {
                raw_versions.insert(ver, parsed);
            }
        }
    }

    if raw_versions.is_empty() {
        panic!("No tracked data asset files found in assets/tracked_data");
    }

    let mojang_names: BTreeSet<String> = raw_versions
        .get(&LATEST_VERSION)
        .or_else(|| raw_versions.values().next_back())
        .map(|entities| entities.keys().cloned().collect())
        .unwrap_or_default();

    let mut versions = BTreeMap::new();
    let mut entity_aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (ver, parsed) in raw_versions {
        let mut merged = BTreeMap::new();
        for (entity, fields) in parsed {
            let canonical = canonicalize_entity_name(&entity, &mojang_names);
            if canonical != entity {
                entity_aliases
                    .entry(canonical.clone())
                    .or_default()
                    .insert(entity);
            }
            let dest: &mut BTreeMap<String, RawTrackedField> = merged.entry(canonical).or_default();
            for (name, info) in fields {
                dest.insert(name, info);
            }
        }
        versions.insert(ver, merged);
    }

    if versions.is_empty() {
        panic!("No tracked data asset files found in assets/tracked_data");
    }

    let tracked_id_struct = generate_tracked_id_struct(&versions);
    let tracked_data_struct = generate_tracked_data_struct();
    let entity_modules = generate_entity_modules(&versions, &entity_aliases);

    quote! {
        use crate::meta_data_type::MetaDataType;
        use pumpkin_util::version::JavaMinecraftVersion;

        #tracked_id_struct

        #tracked_data_struct

        #entity_modules
    }
}

/// Generates the `TrackedId` struct definition with one `u8` field per supported version.
fn generate_tracked_id_struct<T>(versions: &BTreeMap<JavaMinecraftVersion, T>) -> TokenStream {
    let mut struct_fields = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        struct_fields.extend(quote! {
            pub #ident: u8,
        });
    }

    let latest_field_ident = if versions.contains_key(&LATEST_VERSION) {
        LATEST_VERSION.to_field_ident()
    } else {
        versions.keys().last().unwrap().to_field_ident()
    };

    let mut match_arms = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        match_arms.extend(quote! {
            #ver => self.#ident,
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct TrackedId {
            #struct_fields
        }

        impl TrackedId {
            #[must_use]
            pub const fn get(&self, version: &JavaMinecraftVersion) -> u8 {
                match version {
                    #match_arms
                    _ => self.#latest_field_ident,
                }
            }
        }

        impl From<TrackedId> for u8 {
            fn from(id: TrackedId) -> u8 {
                id.#latest_field_ident
            }
        }
    }
}

/// Generates the `TrackedData` struct with `id` and `type` fields.
fn generate_tracked_data_struct() -> TokenStream {
    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct TrackedData {
            pub id: TrackedId,
            pub r#type: MetaDataType,
        }

        impl TrackedData {
            #[must_use]
            pub const fn new(id: TrackedId, r#type: MetaDataType) -> Self {
                Self { id, r#type }
            }

            #[must_use]
            pub const fn get(&self, version: &JavaMinecraftVersion) -> u8 {
                self.id.get(version)
            }
        }
    }
}

/// Generates entity-specific modules containing constants for all tracked fields.
fn generate_entity_modules(
    versions: &BTreeMap<JavaMinecraftVersion, BTreeMap<String, BTreeMap<String, RawTrackedField>>>,
    entity_aliases: &BTreeMap<String, BTreeSet<String>>,
) -> TokenStream {
    let mut modules = TokenStream::new();

    let all_entities: BTreeSet<String> = versions
        .values()
        .flat_map(|entities| entities.keys().cloned())
        .collect();

    for entity in &all_entities {
        let entity_ident = format_ident!("{}", entity);

        let all_fields: BTreeSet<String> = versions
            .values()
            .filter_map(|entities| entities.get(entity))
            .flat_map(|fields| fields.keys().cloned())
            .map(|name| canonicalize_tracked_field_name(&name))
            .collect();

        let mut field_consts = TokenStream::new();
        let mut defined_idents = BTreeSet::new();

        // 1. Generate base field constants
        for field in &all_fields {
            let field_upper = field.to_uppercase();
            let field_ident = format_ident!("{}", field_upper);
            defined_idents.insert(field_upper.clone());

            let mut id_fields = TokenStream::new();
            let mut latest_type = String::new();

            for (ver, entities) in versions {
                let ver_ident = ver.to_field_ident();
                let field_info = entities
                    .get(entity)
                    .and_then(|f| lookup_tracked_field(f, field));
                let id = field_info.map_or(255u8, |info| info.id);
                if let Some(info) = field_info {
                    latest_type = canonicalize_field_type(&info.r#type);
                }
                id_fields.extend(quote! {
                    #ver_ident: #id,
                });
            }

            let type_const_ident = format_ident!("{}", latest_type.to_uppercase());

            field_consts.extend(quote! {
                pub const #field_ident: TrackedData = TrackedData {
                    id: TrackedId { #id_fields },
                    r#type: MetaDataType::#type_const_ident,
                };
            });
        }

        // 2. Generate normalized and semantic aliases
        for field in &all_fields {
            let field_upper = field.to_uppercase();
            let field_ident = format_ident!("{}", field_upper);

            let mut candidate_aliases = Vec::new();

            // Strip DATA_ prefix
            if let Some(stripped) = field_upper.strip_prefix("DATA_") {
                candidate_aliases.push(stripped.to_string());
            }

            // Strip _ID suffix
            if let Some(stripped_id) = field_upper.strip_suffix("_ID") {
                candidate_aliases.push(stripped_id.to_string());
                if let Some(norm) = stripped_id.strip_prefix("DATA_") {
                    candidate_aliases.push(norm.to_string());
                }
            }

            // Semantic aliases
            add_semantic_aliases(entity, &field_upper, &mut candidate_aliases);

            for alias in candidate_aliases {
                if !defined_idents.contains(&alias) && is_valid_ident(&alias) {
                    defined_idents.insert(alias.clone());
                    let alias_ident = format_ident!("{}", alias);
                    field_consts.extend(quote! {
                        pub const #alias_ident: TrackedData = #field_ident;
                    });
                }
            }
        }

        modules.extend(quote! {
            pub mod #entity_ident {
                use super::*;

                #field_consts
            }
        });
    }

    for (canonical, aliases) in entity_aliases {
        if !all_entities.contains(canonical) {
            continue;
        }
        let canonical_ident = format_ident!("{canonical}");
        for alias in aliases {
            if all_entities.contains(alias) || !is_valid_ident(alias) {
                continue;
            }
            let alias_ident = format_ident!("{alias}");
            modules.extend(quote! {
                pub mod #alias_ident {
                    pub use super::#canonical_ident::*;
                }
            });
        }
    }

    modules
}

fn canonicalize_entity_name(name: &str, mojang_names: &BTreeSet<String>) -> String {
    if mojang_names.contains(name) {
        return name.to_string();
    }
    let mapped = match name.strip_suffix("_entity") {
        None => return name.to_string(),
        Some("tameable") => "tamable_animal".to_string(),
        Some("tameable_shoulder") => "shoulder_riding_entity".to_string(),
        Some("mooshroom") => "mushroom_cow".to_string(),
        Some("enderman") => "ender_man".to_string(),
        Some("fishing_bobber") => "fishing_hook".to_string(),
        Some("leash_knot") => "leash_fence_knot_entity".to_string(),
        Some("water_creature") => "water_animal".to_string(),
        Some("hostile") => "monster".to_string(),
        Some("illager") => "abstract_illager".to_string(),
        Some("spellcasting_illager") => "spellcaster_illager".to_string(),
        Some("merchant") => "abstract_villager".to_string(),
        Some("golem") => "abstract_golem".to_string(),
        Some("fish") => "abstract_fish".to_string(),
        Some("schooling_fish") => "abstract_schooling_fish".to_string(),
        Some("abstract_donkey") => "abstract_chested_horse".to_string(),
        Some("ambient") => "ambient_creature".to_string(),
        Some("path_aware") => "pathfinder_mob".to_string(),
        Some("patrol") => "patrolling_monster".to_string(),
        Some("abstract_decoration") => "hanging_entity".to_string(),
        Some("player_like") => "avatar".to_string(),
        Some("passive") => "ageable_mob".to_string(),
        Some("lightning") => "lightning_bolt".to_string(),
        Some("thrown_item") => "throwable_item_projectile".to_string(),
        Some("thrown") => "throwable_projectile".to_string(),
        Some("egg") => "thrown_egg".to_string(),
        Some("ender_pearl") => "thrown_enderpearl".to_string(),
        Some("experience_bottle") => "thrown_experience_bottle".to_string(),
        Some("lingering_potion") => "thrown_lingering_potion".to_string(),
        Some("splash_potion") => "thrown_splash_potion".to_string(),
        Some("trident") => "thrown_trident".to_string(),
        Some("potion") => "abstract_thrown_potion".to_string(),
        Some("explosive_projectile" | "abstract_fireball") => {
            "abstract_hurting_projectile".to_string()
        }
        Some("persistent_projectile") => "abstract_arrow".to_string(),
        Some("storage_minecart") => "abstract_minecart_container".to_string(),
        Some(other) => other.to_string(),
    };
    if mojang_names.contains(&mapped) {
        mapped
    } else {
        name.to_string()
    }
}

fn canonicalize_field_type(ty: &str) -> String {
    match ty {
        "integer" | "int" => "int".to_string(),
        "entity_pose" | "pose" => "pose".to_string(),
        "facing" | "direction" => "direction".to_string(),
        "text_component" | "component" => "component".to_string(),
        "optional_text_component" | "optional_component" => "optional_component".to_string(),
        "optional_int" | "optional_unsigned_int" => "optional_unsigned_int".to_string(),
        "vector_3f" | "vector3" => "vector3".to_string(),
        "quaternion_f" | "quaternion" => "quaternion".to_string(),
        "rotation" | "rotations" => "rotations".to_string(),
        "particle_list" | "particles" => "particles".to_string(),
        "lazy_entity_reference" | "optional_living_entity_reference" => {
            "optional_living_entity_reference".to_string()
        }
        "nbt_compound" | "compound_tag" => "nbt_compound".to_string(),
        other => other.to_string(),
    }
}

fn canonicalize_tracked_field_name(name: &str) -> String {
    match name.to_uppercase().as_str() {
        // Yarn (1.21.x assets) vs Mojang (26.x assets) names for the same index.
        "FLAGS" => "DATA_SHARED_FLAGS_ID".to_string(),
        "AIR" => "DATA_AIR_SUPPLY_ID".to_string(),
        "CUSTOM_NAME" => "DATA_CUSTOM_NAME".to_string(),
        "NAME_VISIBLE" => "DATA_CUSTOM_NAME_VISIBLE".to_string(),
        "SILENT" => "DATA_SILENT".to_string(),
        "NO_GRAVITY" => "DATA_NO_GRAVITY".to_string(),
        "POSE" => "DATA_POSE".to_string(),
        "FROZEN_TICKS" => "DATA_TICKS_FROZEN".to_string(),
        "LIVING_FLAGS" => "DATA_LIVING_ENTITY_FLAGS".to_string(),
        "HEALTH" => "DATA_HEALTH_ID".to_string(),
        "POTION_SWIRLS" => "DATA_EFFECT_PARTICLES".to_string(),
        "POTION_SWIRLS_AMBIENT" => "DATA_EFFECT_AMBIENCE_ID".to_string(),
        "STUCK_ARROW_COUNT" => "DATA_ARROW_COUNT_ID".to_string(),
        "STINGER_COUNT" => "DATA_STINGER_COUNT_ID".to_string(),
        "SLEEPING_POSITION" => "SLEEPING_POS_ID".to_string(),
        "MOB_FLAGS" => "DATA_MOB_FLAGS_ID".to_string(),
        "CHILD" | "BABY" => "DATA_BABY_ID".to_string(),
        "DAMAGE_WOBBLE_TICKS" => "DATA_ID_HURT".to_string(),
        "DAMAGE_WOBBLE_SIDE" => "DATA_ID_HURTDIR".to_string(),
        "DAMAGE_WOBBLE_STRENGTH" => "DATA_ID_DAMAGE".to_string(),
        "LEFT_PADDLE_MOVING" => "DATA_ID_PADDLE_LEFT".to_string(),
        "RIGHT_PADDLE_MOVING" => "DATA_ID_PADDLE_RIGHT".to_string(),
        "BUBBLE_WOBBLE_TICKS" => "DATA_ID_BUBBLE_TIME".to_string(),
        "STACK" | "ITEM" => "DATA_ITEM".to_string(),
        "HEAD_ROLLING_TIME_LEFT" | "UNHAPPY_COUNTER" => "DATA_UNHAPPY_COUNTER".to_string(),
        "VILLAGER_DATA" => "DATA_VILLAGER_DATA".to_string(),
        "ABSORPTION_AMOUNT" => "DATA_PLAYER_ABSORPTION_ID".to_string(),
        "SCORE" => "DATA_SCORE_ID".to_string(),
        "PLAYER_MODEL_PARTS" => "DATA_PLAYER_MODE_CUSTOMISATION".to_string(),
        "MAIN_ARM" => "DATA_PLAYER_MAIN_HAND".to_string(),
        "LEFT_SHOULDER_ENTITY" | "LEFT_SHOULDER_PARROT_VARIANT_ID" => {
            "DATA_SHOULDER_PARROT_LEFT".to_string()
        }
        "RIGHT_SHOULDER_ENTITY" | "RIGHT_SHOULDER_PARROT_VARIANT_ID" => {
            "DATA_SHOULDER_PARROT_RIGHT".to_string()
        }
        "TAMEABLE_FLAGS" | "SPIDER_FLAGS" | "BLAZE_FLAGS" => "DATA_FLAGS_ID".to_string(),
        "HORSE_FLAGS" => "DATA_ID_FLAGS".to_string(),
        "OWNER_UUID" => "DATA_OWNERUUID_ID".to_string(),
        "PARTICLE" => "DATA_PARTICLE".to_string(),
        other => other.to_string(),
    }
}

fn lookup_tracked_field<'a>(
    fields: &'a BTreeMap<String, RawTrackedField>,
    canonical: &str,
) -> Option<&'a RawTrackedField> {
    if let Some(info) = fields.get(canonical) {
        return Some(info);
    }
    fields.iter().find_map(|(name, info)| {
        (canonicalize_tracked_field_name(name) == canonical).then_some(info)
    })
}

fn is_valid_ident(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    !matches!(
        name,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

fn add_semantic_aliases(entity: &str, field: &str, aliases: &mut Vec<String>) {
    match (entity, field) {
        (_, "DATA_LIVING_ENTITY_FLAGS") => {
            aliases.push("LIVING_FLAGS".to_string());
        }
        (_, "DATA_ITEM") => {
            aliases.push("STACK".to_string());
            aliases.push("ITEM".to_string());
        }
        (_, "DATA_UNHAPPY_COUNTER") => {
            aliases.push("HEAD_ROLLING_TIME_LEFT".to_string());
            aliases.push("UNHAPPY_COUNTER".to_string());
        }
        (_, "DATA_PLAYER_ABSORPTION_ID") => {
            aliases.push("ABSORPTION_AMOUNT".to_string());
        }
        (_, "DATA_ID_HURT") => {
            aliases.push("DAMAGE_WOBBLE_TICKS".to_string());
        }
        (_, "DATA_ID_HURTDIR") => {
            aliases.push("DAMAGE_WOBBLE_SIDE".to_string());
        }
        (_, "DATA_ID_DAMAGE") => {
            aliases.push("DAMAGE_WOBBLE_STRENGTH".to_string());
        }
        (_, "DATA_AIR_SUPPLY_ID") => {
            aliases.push("AIR".to_string());
        }
        (_, "DATA_TICKS_FROZEN") => {
            aliases.push("FROZEN_TICKS".to_string());
        }
        (_, "DATA_FLAGS_ID") => {
            aliases.push("TAMEABLE_FLAGS".to_string());
            aliases.push("FLAGS".to_string());
        }
        (_, "DATA_OWNERUUID_ID") => {
            aliases.push("OWNER_UUID".to_string());
        }
        ("creeper", "DATA_IS_POWERED") => {
            aliases.push("CHARGED".to_string());
        }
        ("creeper", "DATA_SWELL_DIR") => {
            aliases.push("FUSE_ID".to_string());
        }
        ("tnt" | "primed_tnt", "DATA_FUSE_ID") => {
            aliases.push("FUSE_ID".to_string());
        }
        ("sheep", "DATA_WOOL_ID") => {
            aliases.push("WOOL_ID".to_string());
        }
        ("cat", "IS_LYING") => {
            aliases.push("IN_SLEEPING_POSE".to_string());
        }
        ("cat", "RELAX_STATE_ONE") => {
            aliases.push("HEAD_DOWN".to_string());
        }
        ("cat", "DATA_SOUND_VARIANT_ID") | ("wolf", "DATA_SOUND_VARIANT_ID") => {
            aliases.push("SOUND_VARIANT".to_string());
            aliases.push("SOUND_VARIANT_ID".to_string());
        }
        ("cat", "DATA_VARIANT_ID") => {
            aliases.push("CAT_VARIANT".to_string());
            aliases.push("CAT_VARIANT_ID".to_string());
            aliases.push("VARIANT".to_string());
        }
        ("wolf", "DATA_VARIANT_ID") => {
            aliases.push("WOLF_VARIANT_ID".to_string());
            aliases.push("VARIANT".to_string());
        }
        ("cat", "DATA_COLLAR_COLOR") => {
            aliases.push("CAT_COLLAR_COLOR".to_string());
            aliases.push("COLLAR_COLOR".to_string());
        }
        ("wolf", "DATA_COLLAR_COLOR") => {
            aliases.push("WOLF_COLLAR_COLOR".to_string());
            aliases.push("COLLAR_COLOR".to_string());
        }
        ("player" | "avatar" | "mannequin", "DATA_PLAYER_MODE_CUSTOMISATION") => {
            aliases.push("PLAYER_MODE_CUSTOMIZATION_ID".to_string());
        }
        ("player" | "avatar" | "mannequin", "DATA_PLAYER_MAIN_HAND") => {
            aliases.push("MAIN_ARM_ID".to_string());
        }
        ("display" | "block_display" | "item_display" | "text_display", _) => match field {
            "DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID" => {
                aliases.push("START_INTERPOLATION".to_string());
            }
            "DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID" => {
                aliases.push("INTERPOLATION_DURATION".to_string());
            }
            "DATA_POS_ROT_INTERPOLATION_DURATION_ID" => {
                aliases.push("TELEPORT_DURATION".to_string());
            }
            "DATA_TRANSLATION_ID" => {
                aliases.push("TRANSLATION".to_string());
            }
            "DATA_SCALE_ID" => {
                aliases.push("SCALE".to_string());
            }
            "DATA_LEFT_ROTATION_ID" => {
                aliases.push("LEFT_ROTATION".to_string());
            }
            "DATA_RIGHT_ROTATION_ID" => {
                aliases.push("RIGHT_ROTATION".to_string());
            }
            "DATA_BILLBOARD_RENDER_CONSTRAINTS_ID" => {
                aliases.push("BILLBOARD".to_string());
            }
            "DATA_BRIGHTNESS_OVERRIDE_ID" => {
                aliases.push("BRIGHTNESS".to_string());
            }
            "DATA_VIEW_RANGE_ID" => {
                aliases.push("VIEW_RANGE".to_string());
            }
            "DATA_SHADOW_RADIUS_ID" => {
                aliases.push("SHADOW_RADIUS".to_string());
            }
            "DATA_SHADOW_STRENGTH_ID" => {
                aliases.push("SHADOW_STRENGTH".to_string());
            }
            "DATA_WIDTH_ID" => {
                aliases.push("WIDTH".to_string());
            }
            "DATA_HEIGHT_ID" => {
                aliases.push("HEIGHT".to_string());
            }
            "DATA_GLOW_COLOR_OVERRIDE_ID" => {
                aliases.push("GLOW_COLOR_OVERRIDE".to_string());
            }
            "DATA_BLOCK_STATE_ID" => {
                aliases.push("BLOCK_STATE".to_string());
            }
            "DATA_ITEM_STACK_ID" => {
                aliases.push("ITEM".to_string());
                aliases.push("ITEM_STACK".to_string());
            }
            "DATA_ITEM_DISPLAY_ID" => {
                aliases.push("ITEM_DISPLAY".to_string());
            }
            "DATA_TEXT_ID" => {
                aliases.push("TEXT".to_string());
            }
            "DATA_LINE_WIDTH_ID" => {
                aliases.push("LINE_WIDTH".to_string());
            }
            "DATA_BACKGROUND_COLOR_ID" => {
                aliases.push("BACKGROUND".to_string());
            }
            "DATA_TEXT_OPACITY_ID" => {
                aliases.push("TEXT_OPACITY".to_string());
            }
            "DATA_STYLE_FLAGS_ID" => {
                aliases.push("TEXT_DISPLAY_FLAGS".to_string());
            }
            _ => {}
        },
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::build;

    #[test]
    fn living_entity_flags_merge_yarn_and_mojang_names() {
        let generated = build().to_string();
        let living = generated
            .split("pub mod living_entity")
            .nth(1)
            .expect("living_entity module");
        let flags = living
            .split("DATA_LIVING_ENTITY_FLAGS")
            .nth(1)
            .expect("DATA_LIVING_ENTITY_FLAGS constant")
            .split("pub const")
            .next()
            .expect("constant body");
        assert!(
            !flags.contains("v1_21 : 255u8"),
            "1.21.x Yarn LIVING_FLAGS (index 8) should merge into DATA_LIVING_ENTITY_FLAGS, got {flags}"
        );
        assert!(flags.contains("v1_21 : 8u8"));
        assert!(flags.contains("v1_21_11 : 8u8"));
        assert!(flags.contains("v26_1 : 8u8"));
    }

    fn field_body(generated: &str, module: &str, field: &str) -> String {
        let module_src = generated
            .split(&format!("pub mod {module}"))
            .nth(1)
            .unwrap_or_else(|| panic!("{module} module"));
        module_src
            .split(field)
            .nth(1)
            .unwrap_or_else(|| panic!("{module}::{field}"))
            .split("pub const")
            .next()
            .expect("constant body")
            .to_string()
    }

    #[test]
    fn living_health_and_air_merge_yarn_names() {
        let generated = build().to_string();
        let health = field_body(&generated, "living_entity", "DATA_HEALTH_ID");
        assert!(
            !health.contains("v1_21 : 255u8"),
            "1.21.x Yarn HEALTH should merge into DATA_HEALTH_ID, got {health}"
        );
        assert!(health.contains("v1_21 : 9u8"));

        let air = field_body(&generated, "entity", "DATA_AIR_SUPPLY_ID");
        assert!(
            !air.contains("v1_21 : 255u8"),
            "1.21.x Yarn AIR should merge into DATA_AIR_SUPPLY_ID, got {air}"
        );
    }

    #[test]
    fn yarn_entity_modules_merge_into_mojang_names() {
        let generated = build().to_string();

        let boat_hurt = field_body(&generated, "boat", "DATA_ID_HURT");
        assert!(
            !boat_hurt.contains("v1_21_11 : 255u8"),
            "boat_entity DAMAGE_WOBBLE_TICKS should merge into boat::DATA_ID_HURT, got {boat_hurt}"
        );
        assert!(boat_hurt.contains("v1_21_11 : 8u8"));
        assert!(boat_hurt.contains("v26_2 : 8u8"));

        let baby = field_body(&generated, "ageable_mob", "DATA_BABY_ID");
        assert!(
            !baby.contains("v1_21 : 255u8"),
            "passive_entity CHILD should merge into ageable_mob::DATA_BABY_ID, got {baby}"
        );

        let item = field_body(&generated, "item", "DATA_ITEM");
        assert!(
            !item.contains("v1_21_11 : 255u8"),
            "Yarn STACK/ITEM should merge into item::DATA_ITEM, got {item}"
        );
    }

    #[test]
    fn wolf_and_cat_have_correct_entity_specific_tracker_constants() {
        let generated = build().to_string();

        assert!(generated.contains("mod wolf"));
        assert!(generated.contains("mod cat"));
        assert!(generated.contains("DATA_COLLAR_COLOR"));
    }
}
