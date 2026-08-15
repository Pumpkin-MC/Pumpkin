use std::{
    any::{Any, TypeId, type_name},
    sync::Arc,
};

use pumpkin_codecs::{Decode, Encode};
use pumpkin_registry::{Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider};
use pumpkin_util::identifier::Identifier;

use crate::attributes::{
    ArgbColor, EnvAttribute, EnvAttributeBuilder, FloatRange, RgbColor,
    builtin_types::{
        AttributeTypeEntry, activity_type, ambient_particles_type, ambient_sounds_type, angle_type,
        argb_type, background_music_type, bed_rule_type, boolean_type, float_type, moon_phase_type,
        particle_type, rgb_type, tri_state_type,
    },
    value_types::{
        Activity, AmbientParticles, AmbientSounds, BackgroundMusic, BedRule, MoonPhase,
        ParticleOptions, TriState,
    },
};

static UNIT_FLOAT_RANGE: FloatRange = FloatRange::UNIT;
static NON_NEGATIVE_FLOAT_RANGE: FloatRange = FloatRange::NON_NEGATIVE;

pub struct EnvironmentAttributeEntry {
    value_type_id: TypeId,
    value_type_name: &'static str,
    value: Arc<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for EnvironmentAttributeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvironmentAttributeEntry")
            .field("value_type_name", &self.value_type_name)
            .finish_non_exhaustive()
    }
}

impl EnvironmentAttributeEntry {
    #[must_use]
    pub fn new<T: Encode + Decode + Send + Sync + 'static>(value: Arc<EnvAttribute<T>>) -> Self {
        Self {
            value_type_id: TypeId::of::<T>(),
            value_type_name: type_name::<T>(),
            value,
        }
    }

    #[must_use]
    pub const fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }

    #[must_use]
    pub const fn value_type_name(&self) -> &'static str {
        self.value_type_name
    }

    #[must_use]
    pub fn downcast<T: Encode + Decode + Send + Sync + 'static>(&self) -> Option<&EnvAttribute<T>> {
        (self.value_type_id == TypeId::of::<T>())
            .then(|| self.value.downcast_ref::<EnvAttribute<T>>())
            .flatten()
    }
}

fn build_attribute<T: Encode + Decode + Send + Sync + 'static>(
    builder: EnvAttributeBuilder<T>,
) -> Arc<EnvAttribute<T>> {
    match builder.build() {
        Ok(attribute) => Arc::new(attribute),
        Err(error) => unreachable!("{error}"),
    }
}

fn typed_attribute<T: Encode + Decode + Send + Sync + 'static>(
    value_type: Arc<crate::attributes::AttributeType<T>>,
    default_value: T,
    syncable: bool,
    positional: bool,
    interpolated: bool,
) -> Arc<EnvAttribute<T>> {
    let mut builder = EnvAttribute::builder(value_type).default_value(default_value);
    if syncable {
        builder = builder.syncable();
    }
    if !positional {
        builder = builder.not_positional();
    }
    if interpolated {
        builder = builder.spatially_interpolated();
    }
    build_attribute(builder)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn ranged_float_attribute(
    default_value: f32,
    range: &'static FloatRange,
    syncable: bool,
    positional: bool,
    interpolated: bool,
) -> Arc<EnvAttribute<f32>> {
    let mut builder = EnvAttribute::builder(float_type())
        .default_value(default_value)
        .value_range(range);
    if syncable {
        builder = builder.syncable();
    }
    if !positional {
        builder = builder.not_positional();
    }
    if interpolated {
        builder = builder.spatially_interpolated();
    }
    build_attribute(builder)
}

bootstrap_provider! {
    ATTRIBUTE_TYPES: AttributeTypeEntry => "minecraft:attribute_type",
    || {
        vec![
            RegistryEntry::new(Identifier::vanilla_static("boolean"), AttributeTypeEntry::new(boolean_type())),
            RegistryEntry::new(Identifier::vanilla_static("tri_state"), AttributeTypeEntry::new(tri_state_type())),
            RegistryEntry::new(Identifier::vanilla_static("float"), AttributeTypeEntry::new(float_type())),
            RegistryEntry::new(Identifier::vanilla_static("angle_degrees"), AttributeTypeEntry::new(angle_type())),
            RegistryEntry::new(Identifier::vanilla_static("rgb_color"), AttributeTypeEntry::new(rgb_type())),
            RegistryEntry::new(Identifier::vanilla_static("argb_color"), AttributeTypeEntry::new(argb_type())),
            RegistryEntry::new(Identifier::vanilla_static("moon_phase"), AttributeTypeEntry::new(moon_phase_type())),
            RegistryEntry::new(Identifier::vanilla_static("activity"), AttributeTypeEntry::new(activity_type())),
            RegistryEntry::new(Identifier::vanilla_static("bed_rule"), AttributeTypeEntry::new(bed_rule_type())),
            RegistryEntry::new(Identifier::vanilla_static("particle"), AttributeTypeEntry::new(particle_type())),
            RegistryEntry::new(Identifier::vanilla_static("ambient_particles"), AttributeTypeEntry::new(ambient_particles_type())),
            RegistryEntry::new(Identifier::vanilla_static("background_music"), AttributeTypeEntry::new(background_music_type())),
            RegistryEntry::new(Identifier::vanilla_static("ambient_sounds"), AttributeTypeEntry::new(ambient_sounds_type())),
        ]
    }
}

bootstrap_provider! {
    ATTRIBUTE_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    || {
        let Ok(registry) = RegistryBuilder::<AttributeTypeEntry>::frozen(
            &Identifier::vanilla_static("attribute_type"),
        ) else {
            return Vec::new();
        };
        vec![RegistryEntry::new(
            Identifier::vanilla_static("attribute_type"),
            registry.arc_dyn(),
        )]
    }
}

bootstrap_provider! {
    ENVIRONMENT_ATTRIBUTES: EnvironmentAttributeEntry => "minecraft:environment_attribute",
    || {
        let mut entries = Vec::new();

        macro_rules! add {
            ($id:literal, $value:expr) => {
                entries.push(RegistryEntry::new(
                    Identifier::parse_static(concat!("minecraft:", $id)),
                    EnvironmentAttributeEntry::new($value),
                ));
            };
        }

        // Visual attributes from 1.21.11.
        add!("visual/fog_color", typed_attribute(rgb_type(), RgbColor::new(0x000000), true, true, true));
        add!("visual/fog_start_distance", typed_attribute(float_type(), 0.0, true, true, true));
        add!("visual/fog_end_distance", ranged_float_attribute(1024.0, &NON_NEGATIVE_FLOAT_RANGE, true, true, true));
        add!("visual/sky_fog_end_distance", ranged_float_attribute(512.0, &NON_NEGATIVE_FLOAT_RANGE, true, true, true));
        add!("visual/cloud_fog_end_distance", ranged_float_attribute(2048.0, &NON_NEGATIVE_FLOAT_RANGE, true, true, true));
        add!("visual/water_fog_color", typed_attribute(rgb_type(), RgbColor::new(0x050533), true, true, true));
        add!("visual/water_fog_start_distance", typed_attribute(float_type(), -8.0, true, true, true));
        add!("visual/water_fog_end_distance", ranged_float_attribute(96.0, &NON_NEGATIVE_FLOAT_RANGE, true, true, true));
        add!("visual/sky_color", typed_attribute(rgb_type(), RgbColor::new(0x000000), true, true, true));
        add!("visual/cloud_color", typed_attribute(argb_type(), ArgbColor::new(0x00000000), true, true, true));
        add!("visual/cloud_height", typed_attribute(float_type(), 192.33, true, true, true));
        add!("visual/default_dripstone_particle", typed_attribute(
            particle_type(),
            ParticleOptions::simple(Identifier::vanilla_static("dripping_dripstone_water")),
            true,
            true,
            false,
        ));
        add!("visual/ambient_particles", typed_attribute(
            ambient_particles_type(),
            AmbientParticles::default(),
            true,
            true,
            false,
        ));
        add!("visual/sunrise_sunset_color", typed_attribute(argb_type(), ArgbColor::new(0x00000000), true, true, true));
        add!("visual/sun_angle", typed_attribute(angle_type(), 0.0, true, true, true));
        add!("visual/moon_angle", typed_attribute(angle_type(), 0.0, true, true, true));
        add!("visual/star_angle", typed_attribute(angle_type(), 0.0, true, true, true));
        add!("visual/moon_phase", typed_attribute(moon_phase_type(), MoonPhase::FullMoon, true, true, false));
        add!("visual/star_brightness", ranged_float_attribute(0.0, &UNIT_FLOAT_RANGE, true, true, true));
        add!("visual/sky_light_color", typed_attribute(rgb_type(), RgbColor::new(0xFFFFFF), true, true, true));
        add!("visual/sky_light_factor", typed_attribute(float_type(), 1.0, true, true, true));

        // Added in 26.1 and therefore part of Pumpkin's 26.2 target catalog.
        add!("visual/block_light_tint", typed_attribute(rgb_type(), RgbColor::new(0xFFD88C), true, true, true));
        add!("visual/ambient_light_color", typed_attribute(rgb_type(), RgbColor::new(0x0A0A0A), true, true, true));
        add!("visual/night_vision_color", typed_attribute(rgb_type(), RgbColor::new(0x999999), true, true, true));

        // Audio.
        add!("audio/background_music", typed_attribute(
            background_music_type(),
            BackgroundMusic::default(),
            true,
            true,
            false,
        ));
        add!("audio/music_volume", ranged_float_attribute(1.0, &UNIT_FLOAT_RANGE, true, true, false));
        add!("audio/ambient_sounds", typed_attribute(
            ambient_sounds_type(),
            AmbientSounds::default(),
            true,
            true,
            false,
        ));
        add!("audio/firefly_bush_sounds", typed_attribute(boolean_type(), false, true, true, false));

        // Gameplay.
        add!("gameplay/can_start_raid", typed_attribute(boolean_type(), true, false, true, false));
        add!("gameplay/water_evaporates", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/bed_rule", typed_attribute(
            bed_rule_type(),
            BedRule::overworld(),
            false,
            true,
            false,
        ));
        add!("gameplay/respawn_anchor_works", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/nether_portal_spawns_piglin", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/fast_lava", typed_attribute(boolean_type(), false, false, false, false));
        add!("gameplay/increased_fire_burnout", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/piglins_zombify", typed_attribute(boolean_type(), true, false, true, false));
        add!("gameplay/snow_golem_melts", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/sky_light_level", typed_attribute(float_type(), 15.0, false, false, true));
        add!("gameplay/eyeblossom_open", typed_attribute(tri_state_type(), TriState::Default, false, true, false));
        add!("gameplay/turtle_egg_hatch_chance", ranged_float_attribute(0.002, &UNIT_FLOAT_RANGE, false, true, true));
        add!("gameplay/creaking_active", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/surface_slime_spawn_chance", ranged_float_attribute(0.0, &UNIT_FLOAT_RANGE, false, true, true));
        add!("gameplay/cat_waking_up_gift_chance", typed_attribute(float_type(), 0.0, false, true, true));
        add!("gameplay/bees_stay_in_hive", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/monsters_burn", typed_attribute(boolean_type(), false, false, true, false));
        add!("gameplay/can_pillager_patrol_spawn", typed_attribute(boolean_type(), true, false, true, false));
        add!("gameplay/villager_activity", typed_attribute(
            activity_type(),
            Activity(Identifier::vanilla_static("idle")),
            false,
            true,
            false,
        ));
        add!("gameplay/baby_villager_activity", typed_attribute(
            activity_type(),
            Activity(Identifier::vanilla_static("idle")),
            false,
            true,
            false,
        ));

        entries
    }
}

bootstrap_provider! {
    ENVIRONMENT_ATTRIBUTE_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    || {
        let Ok(registry) = RegistryBuilder::<EnvironmentAttributeEntry>::frozen(
            &Identifier::vanilla_static("environment_attribute"),
        ) else {
            return Vec::new();
        };
        vec![RegistryEntry::new(
            Identifier::vanilla_static("environment_attribute"),
            registry.arc_dyn(),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::attribute_modifier::{AttributeOperation, FloatWithAlpha};
    use pumpkin_registry::{ROOT, Registry};

    fn child_registry(name: &'static str) -> Arc<dyn Registry> {
        crate::init_test_registries();
        let root = ROOT.get().expect("root registry must be initialized");
        let id = root
            .get_id(&Identifier::vanilla_static(name))
            .expect("child registry must be present");
        let value = root.by_id_erased(id).expect("child registry must resolve");
        Arc::clone(
            value
                .downcast_ref::<Arc<dyn Registry>>()
                .expect("root entry must be a registry"),
        )
    }

    #[test]
    fn attribute_type_registry_contains_all_vanilla_types() {
        let registry = child_registry("attribute_type");
        assert_eq!(registry.iter_erased().count(), 13);

        for identifier in [
            "boolean",
            "tri_state",
            "float",
            "angle_degrees",
            "rgb_color",
            "argb_color",
            "moon_phase",
            "activity",
            "bed_rule",
            "particle",
            "ambient_particles",
            "background_music",
            "ambient_sounds",
        ] {
            assert!(
                registry
                    .get_id(&Identifier::vanilla_static(identifier))
                    .is_some(),
                "missing minecraft:{identifier}"
            );
        }
    }

    #[test]
    fn environment_attribute_registry_contains_26_2_catalog() {
        let registry = child_registry("environment_attribute");
        assert_eq!(registry.iter_erased().count(), 48);

        for identifier in [
            "visual/fog_color",
            "visual/block_light_tint",
            "visual/ambient_light_color",
            "visual/night_vision_color",
            "audio/background_music",
            "gameplay/fast_lava",
            "gameplay/turtle_egg_hatch_chance",
            "gameplay/baby_villager_activity",
        ] {
            assert!(
                registry
                    .get_id(&Identifier::vanilla_static(identifier))
                    .is_some(),
                "missing minecraft:{identifier}"
            );
        }
    }

    #[test]
    fn float_attribute_type_has_all_float_modifiers() {
        let value_type = float_type();
        let operations = value_type.modifier_library();

        for operation in [
            AttributeOperation::Override,
            AttributeOperation::Add,
            AttributeOperation::Subtract,
            AttributeOperation::Multiply,
            AttributeOperation::Minimum,
            AttributeOperation::Maximum,
            AttributeOperation::AlphaBlend,
        ] {
            assert!(operations.contains_key(&operation), "missing {operation:?}");
        }

        let alpha_blend = operations
            .get(&AttributeOperation::AlphaBlend)
            .expect("alpha_blend must exist");
        assert_eq!(
            alpha_blend.argument_type_id(),
            TypeId::of::<FloatWithAlpha>()
        );
    }

    #[test]
    fn important_environment_attribute_metadata_matches_vanilla() {
        let registry = child_registry("environment_attribute");

        let fast_lava_id = registry
            .get_id(&Identifier::parse_static("minecraft:gameplay/fast_lava"))
            .expect("fast_lava must exist");
        let fast_lava = registry
            .by_id_erased(fast_lava_id)
            .expect("fast_lava must resolve");
        let fast_lava = fast_lava
            .downcast_ref::<EnvironmentAttributeEntry>()
            .expect("environment entry type must match")
            .downcast::<bool>()
            .expect("fast_lava must be boolean");
        assert!(!fast_lava.is_positional());
        assert!(!fast_lava.is_spatially_interpolated());

        let turtle_id = registry
            .get_id(&Identifier::parse_static(
                "minecraft:gameplay/turtle_egg_hatch_chance",
            ))
            .expect("turtle_egg_hatch_chance must exist");
        let turtle = registry
            .by_id_erased(turtle_id)
            .expect("turtle_egg_hatch_chance must resolve");
        let turtle = turtle
            .downcast_ref::<EnvironmentAttributeEntry>()
            .expect("environment entry type must match")
            .downcast::<f32>()
            .expect("turtle_egg_hatch_chance must be float");
        assert_eq!(*turtle.default_value(), 0.002);
        assert!(turtle.is_spatially_interpolated());
    }
}
