use core::f32;
use std::sync::atomic::Ordering;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};
use arc_swap::ArcSwap;
use pumpkin_data::damage::DamageType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;

/// Painting variant information (width and height in quads / 16px units)
/// Based on vanilla PaintingVariants.java (MC 1.26.2)
pub struct PaintingVariantInfo {
    pub name: &'static str,
    pub width_quads: i32,
    pub height_quads: i32,
}

/// All vanilla painting variants with their dimensions
/// Source: net.minecraft.world.entity.decoration.painting.PaintingVariants (MC 1.26.2)
pub const PAINTING_VARIANTS: &[PaintingVariantInfo] = &[
    PaintingVariantInfo {
        name: "minecraft:kebab",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:aztec",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:alban",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:aztec2",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:bomb",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:plant",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:wasteland",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:pool",
        width_quads: 2,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:courbet",
        width_quads: 2,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:sea",
        width_quads: 2,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:sunset",
        width_quads: 2,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:creebet",
        width_quads: 2,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:wanderer",
        width_quads: 1,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:graham",
        width_quads: 1,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:match",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:bust",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:stage",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:void",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:skull_and_roses",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:wither",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:fighters",
        width_quads: 4,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:pointer",
        width_quads: 4,
        height_quads: 4,
    },
    PaintingVariantInfo {
        name: "minecraft:pigscene",
        width_quads: 4,
        height_quads: 4,
    },
    PaintingVariantInfo {
        name: "minecraft:burning_skull",
        width_quads: 4,
        height_quads: 4,
    },
    PaintingVariantInfo {
        name: "minecraft:skeleton",
        width_quads: 4,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:earth",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:wind",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:water",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:fire",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:donkey_kong",
        width_quads: 4,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:baroque",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:humble",
        width_quads: 2,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:meditative",
        width_quads: 1,
        height_quads: 1,
    },
    PaintingVariantInfo {
        name: "minecraft:prairie_ride",
        width_quads: 1,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:unpacked",
        width_quads: 4,
        height_quads: 4,
    },
    PaintingVariantInfo {
        name: "minecraft:backyard",
        width_quads: 3,
        height_quads: 4,
    },
    PaintingVariantInfo {
        name: "minecraft:bouquet",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:cavebird",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:changing",
        width_quads: 4,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:cotan",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:endboss",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:fern",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:finding",
        width_quads: 4,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:lowmist",
        width_quads: 4,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:orb",
        width_quads: 4,
        height_quads: 4,
    },
    PaintingVariantInfo {
        name: "minecraft:owlemons",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:passage",
        width_quads: 4,
        height_quads: 2,
    },
    PaintingVariantInfo {
        name: "minecraft:pond",
        width_quads: 3,
        height_quads: 4,
    },
    PaintingVariantInfo {
        name: "minecraft:sunflowers",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:tides",
        width_quads: 3,
        height_quads: 3,
    },
    PaintingVariantInfo {
        name: "minecraft:dennis",
        width_quads: 3,
        height_quads: 3,
    },
];

/// Placeable painting variants (from pumpkin-data tag.rs - `MINECRAFT_PLACEABLE`)
/// All variants except: earth, wind, water, fire
pub const PLACEABLE_VARIANTS: &[&str] = &[
    "minecraft:kebab",
    "minecraft:aztec",
    "minecraft:alban",
    "minecraft:aztec2",
    "minecraft:bomb",
    "minecraft:plant",
    "minecraft:wasteland",
    "minecraft:pool",
    "minecraft:courbet",
    "minecraft:sea",
    "minecraft:sunset",
    "minecraft:creebet",
    "minecraft:wanderer",
    "minecraft:graham",
    "minecraft:match",
    "minecraft:bust",
    "minecraft:stage",
    "minecraft:void",
    "minecraft:skull_and_roses",
    "minecraft:wither",
    "minecraft:fighters",
    "minecraft:pointer",
    "minecraft:pigscene",
    "minecraft:burning_skull",
    "minecraft:skeleton",
    "minecraft:donkey_kong",
    "minecraft:baroque",
    "minecraft:humble",
    "minecraft:meditative",
    "minecraft:prairie_ride",
    "minecraft:unpacked",
    "minecraft:backyard",
    "minecraft:bouquet",
    "minecraft:cavebird",
    "minecraft:changing",
    "minecraft:cotan",
    "minecraft:endboss",
    "minecraft:fern",
    "minecraft:finding",
    "minecraft:lowmist",
    "minecraft:orb",
    "minecraft:owlemons",
    "minecraft:passage",
    "minecraft:pond",
    "minecraft:sunflowers",
    "minecraft:tides",
    "minecraft:dennis",
];

pub struct PaintingEntity {
    entity: Entity,
    /// The painting variant (registry name like "kebab", "aztec", etc.)
    variant: ArcSwap<String>,
}

impl PaintingEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            variant: ArcSwap::new(std::sync::Arc::new("minecraft:kebab".to_string())),
        }
    }

    pub fn get_variant(&self) -> String {
        self.variant.load().to_string()
    }

    pub fn set_variant(&self, variant: String) {
        self.variant.store(std::sync::Arc::new(variant));
    }

    /// Get the dimensions of a painting variant in quads
    #[must_use]
    pub fn get_variant_dimensions(variant: &str) -> Option<(i32, i32)> {
        PAINTING_VARIANTS
            .iter()
            .find(|v| v.name == variant)
            .map(|v| (v.width_quads, v.height_quads))
    }

    /// Get the dimensions of the current variant in quads
    pub fn get_current_dimensions(&self) -> (i32, i32) {
        let variant = self.get_variant();
        Self::get_variant_dimensions(&variant).unwrap_or((1, 1))
    }

    /// Pick a random placeable variant
    /// This is a simplified selection that doesn't check fit - it just picks
    /// a random variant that can be placed.
    #[must_use]
    pub fn pick_random_variant() -> String {
        use rand::seq::IndexedRandom;
        let mut rng = rand::rng();
        PLACEABLE_VARIANTS
            .choose(&mut rng)
            .map_or_else(|| "minecraft:kebab".to_string(), ToString::to_string)
    }

    /// Set the variant and optionally update the entity dimensions.
    /// Painting dimensions in pixels are: `width_quads` * 16, `height_quads` * 16
    pub fn set_variant_with_dimensions(&self, variant: &str) {
        self.set_variant(variant.to_string());
        if let Some((width_quads, height_quads)) = Self::get_variant_dimensions(variant) {
            // Convert quads to block units (each quad is 1/16 block)
            let width = width_quads as f32 / 16.0;
            let height = height_quads as f32 / 16.0;

            let dimensions = pumpkin_util::math::boundingbox::EntityDimensions {
                width,
                height,
                eye_height: 0.0,
            };
            self.entity.entity_dimension.store(dimensions);
        }
    }
}

impl NBTStorage for PaintingEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.write_nbt(nbt).await;
            nbt.put_byte("facing", self.entity.data.load(Ordering::Relaxed) as i8);
            nbt.put_string("variant", self.get_variant());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.read_nbt_non_mut(nbt).await;
            let facing = nbt.get_byte("facing").unwrap_or(3);
            self.entity.data.store(facing as i32, Ordering::Relaxed);
            if let Some(variant) = nbt.get_string("variant") {
                self.set_variant(variant.to_string());
            }
        })
    }
}

impl EntityBase for PaintingEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            let entity = self.get_entity();
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::PAINTING_VARIANT_ID,
                    MetaDataType::PAINTING_VARIANT,
                    self.get_variant(),
                )],
                None,
            );
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async {
            // TODO
            self.entity.remove().await;
            true
        })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
