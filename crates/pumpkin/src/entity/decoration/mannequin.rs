use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use tokio::sync::Mutex;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityPose;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

/// `PlayerModelPart` name/bit pairs (`PlayerModelPart.java`), in declaration order.
const PLAYER_MODEL_PARTS: [(&str, u8); 7] = [
    ("cape", 0x01),
    ("jacket", 0x02),
    ("left_sleeve", 0x04),
    ("right_sleeve", 0x08),
    ("left_pants_leg", 0x10),
    ("right_pants_leg", 0x20),
    ("hat", 0x40),
];
const ALL_LAYERS: u8 = 0x7F;
const DEFAULT_MAIN_HAND_RIGHT: bool = true;

fn default_description() -> TextComponent {
    TextComponent::translate("entity.minecraft.mannequin.label", [])
}

const fn is_valid_pose(pose: EntityPose) -> bool {
    matches!(
        pose,
        EntityPose::Standing
            | EntityPose::Crouching
            | EntityPose::Swimming
            | EntityPose::FallFlying
            | EntityPose::Sleeping
    )
}

/// Vanilla's `Pose.CODEC` is `StringRepresentable.fromEnum`, so poses (de)serialize by
/// their lowercase name, not their ordinal.
const fn pose_name(pose: EntityPose) -> Option<&'static str> {
    Some(match pose {
        EntityPose::Standing => "standing",
        EntityPose::Crouching => "crouching",
        EntityPose::Swimming => "swimming",
        EntityPose::FallFlying => "fall_flying",
        EntityPose::Sleeping => "sleeping",
        _ => return None,
    })
}

fn pose_from_name(name: &str) -> Option<EntityPose> {
    let pose = match name {
        "standing" => EntityPose::Standing,
        "crouching" => EntityPose::Crouching,
        "swimming" => EntityPose::Swimming,
        "fall_flying" => EntityPose::FallFlying,
        "sleeping" => EntityPose::Sleeping,
        _ => return None,
    };
    is_valid_pose(pose).then_some(pose)
}

/// `HumanoidArm.CODEC` is likewise `StringRepresentable.fromEnum` ("left"/"right").
const fn main_hand_name(right: bool) -> &'static str {
    if right { "right" } else { "left" }
}

/// `Mannequin.LAYERS_CODEC` stores the *hidden* parts as a list of `PlayerModelPart`
/// names; a mask bit of `1` here means the part is shown, matching
/// `DATA_PLAYER_MODE_CUSTOMISATION`'s all-shown default.
fn hidden_layers_list(shown_mask: u8) -> Vec<NbtTag> {
    PLAYER_MODEL_PARTS
        .iter()
        .filter(|(_, bit)| shown_mask & bit == 0)
        .map(|(name, _)| NbtTag::from(*name))
        .collect()
}

fn shown_mask_from_hidden_layers(hidden: &[NbtTag]) -> u8 {
    let mut mask = ALL_LAYERS;
    for tag in hidden {
        if let Some(name) = tag.extract_string()
            && let Some((_, bit)) = PLAYER_MODEL_PARTS.iter().find(|(n, _)| *n == name)
        {
            mask &= !bit;
        }
    }
    mask
}

/// Player-shaped decoration entity used to display a (possibly customized) skin in a
/// fixed pose.
///
/// Vanilla's `Mannequin` extends `Avatar`, an abstract `LivingEntity` subclass shared
/// with the real player entity that provides humanoid main-hand/model-customization
/// data. This codebase has no such humanoid-avatar base: real players are a distinct,
/// fully separate `Player`/`LivingEntity` type with no extraction point for a
/// "player-shaped but not a player" entity, and there is no AI/goal-selector concept
/// attached to plain (non-`Mob`) `LivingEntity` values (see `entity::mob::Mob`) for
/// `isEffectiveAi` to gate against. Rendering a skin, resolving a `ResolvableProfile`
/// into textures, and any pose-driven animation are therefore all out of scope here;
/// `isEffectiveAi`/`isImmobile` (which in vanilla only gate self-movement in
/// `aiStep`/`travel`, not gravity or pushability) have nothing to attach to and are
/// not implemented.
///
/// What is implemented is the NBT-persistence shell vanilla defines on top of that
/// base: the profile (stored as an opaque compound, the same pass-through pattern
/// `SkullBlockEntity` uses for the same data shape), hidden layer list, main hand,
/// pose, immovable flag, and name-tag description text - all using vanilla's actual
/// wire format (string-named poses/hand, list of hidden-part names) rather than an
/// invented encoding.
pub struct MannequinEntity {
    living_entity: LivingEntity,
    profile: Mutex<Option<NbtCompound>>,
    shown_layers: AtomicU8,
    main_hand_right: AtomicBool,
    immovable: AtomicBool,
    description: Mutex<TextComponent>,
    hide_description: AtomicBool,
}

impl MannequinEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            living_entity: LivingEntity::new(entity),
            profile: Mutex::new(None),
            shown_layers: AtomicU8::new(ALL_LAYERS),
            main_hand_right: AtomicBool::new(DEFAULT_MAIN_HAND_RIGHT),
            immovable: AtomicBool::new(false),
            description: Mutex::new(default_description()),
            hide_description: AtomicBool::new(false),
        }
    }

    pub async fn get_profile(&self) -> Option<NbtCompound> {
        self.profile.lock().await.clone()
    }

    pub async fn set_profile(&self, profile: NbtCompound) {
        *self.profile.lock().await = Some(profile);
    }

    pub fn is_immovable(&self) -> bool {
        self.immovable.load(Ordering::Relaxed)
    }

    pub fn set_immovable(&self, immovable: bool) {
        self.immovable.store(immovable, Ordering::Relaxed);
    }

    fn apply_pose(entity: &Entity, pose: EntityPose) {
        let dimensions = Entity::get_entity_dimensions(pose);
        let pos = entity.pos.load();
        entity.pose.store(pose);
        entity
            .bounding_box
            .store(BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &dimensions));
        entity.entity_dimension.store(dimensions);
    }
}

impl NBTStorage for MannequinEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.living_entity.write_nbt(nbt).await;

            if let Some(profile) = self.profile.lock().await.as_ref() {
                nbt.put_compound("profile", profile.clone());
            }
            nbt.put_list(
                "hidden_layers",
                hidden_layers_list(self.shown_layers.load(Ordering::Relaxed)),
            );
            nbt.put_string(
                "main_hand",
                main_hand_name(self.main_hand_right.load(Ordering::Relaxed)).to_string(),
            );
            if let Some(name) = pose_name(self.living_entity.entity.pose.load()) {
                nbt.put_string("pose", name.to_string());
            }
            nbt.put_bool("immovable", self.is_immovable());

            if self.hide_description.load(Ordering::Relaxed) {
                nbt.put_bool("hide_description", true);
            } else {
                let description = self.description.lock().await;
                if *description != default_description()
                    && let Ok(json) = pumpkin_util::serde_json::to_string(&*description)
                {
                    nbt.put_string("description", json);
                }
            }
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.living_entity.read_nbt_non_mut(nbt).await;

            *self.profile.lock().await = nbt.get_compound("profile").cloned();

            self.shown_layers.store(
                nbt.get_list("hidden_layers")
                    .map_or(ALL_LAYERS, shown_mask_from_hidden_layers),
                Ordering::Relaxed,
            );
            self.main_hand_right.store(
                nbt.get_string("main_hand")
                    .map_or(DEFAULT_MAIN_HAND_RIGHT, |name| name != "left"),
                Ordering::Relaxed,
            );

            let pose = nbt
                .get_string("pose")
                .and_then(pose_from_name)
                .unwrap_or(EntityPose::Standing);
            Self::apply_pose(&self.living_entity.entity, pose);

            self.set_immovable(nbt.get_bool("immovable").unwrap_or(false));

            let hide_description = nbt.get_bool("hide_description").unwrap_or(false);
            self.hide_description
                .store(hide_description, Ordering::Relaxed);
            let description = if hide_description {
                default_description()
            } else {
                nbt.get_string("description")
                    .and_then(|json| pumpkin_util::serde_json::from_str(json).ok())
                    .unwrap_or_else(default_description)
            };
            *self.description.lock().await = description;
        })
    }
}

impl EntityBase for MannequinEntity {
    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a crate::server::Server,
    ) -> EntityBaseFuture<'a, ()> {
        self.living_entity.tick(caller, server)
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        self.living_entity
            .damage_with_context(caller, amount, damage_type, position, source, cause)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ALL_LAYERS, hidden_layers_list, is_valid_pose, pose_from_name, pose_name,
        shown_mask_from_hidden_layers,
    };
    use pumpkin_data::entity::EntityPose;

    #[test]
    fn all_layers_mask_covers_seven_bits() {
        assert_eq!(ALL_LAYERS, 0x7F);
    }

    #[test]
    fn only_vanilla_allowed_poses_are_valid() {
        assert!(is_valid_pose(EntityPose::Standing));
        assert!(is_valid_pose(EntityPose::Crouching));
        assert!(is_valid_pose(EntityPose::Swimming));
        assert!(is_valid_pose(EntityPose::FallFlying));
        assert!(is_valid_pose(EntityPose::Sleeping));
        assert!(!is_valid_pose(EntityPose::Dying));
        assert!(!is_valid_pose(EntityPose::SpinAttack));
        assert!(!is_valid_pose(EntityPose::Sitting));
    }

    #[test]
    fn pose_name_roundtrips_through_vanilla_strings() {
        for pose in [
            EntityPose::Standing,
            EntityPose::Crouching,
            EntityPose::Swimming,
            EntityPose::FallFlying,
            EntityPose::Sleeping,
        ] {
            let name = pose_name(pose).unwrap();
            assert!(matches!(pose_from_name(name), Some(p) if p == pose));
        }
        assert!(pose_name(EntityPose::Dying).is_none());
        assert!(pose_from_name("dying").is_none());
        assert!(pose_from_name("not_a_pose").is_none());
    }

    #[test]
    fn all_shown_serializes_to_empty_hidden_list() {
        assert!(hidden_layers_list(ALL_LAYERS).is_empty());
        assert_eq!(shown_mask_from_hidden_layers(&[]), ALL_LAYERS);
    }

    #[test]
    fn hidden_layers_roundtrip() {
        let shown = ALL_LAYERS & !0x01 & !0x40; // cape and hat hidden
        let list = hidden_layers_list(shown);
        assert_eq!(list.len(), 2);
        assert_eq!(shown_mask_from_hidden_layers(&list), shown);
    }
}
