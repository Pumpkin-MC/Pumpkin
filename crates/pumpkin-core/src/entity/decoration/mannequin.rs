use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use pumpkin_data::{
    attributes::Attributes,
    data_component::DataComponent,
    data_component_impl::{DataComponentImpl, ProfileImpl},
    entity::EntityPose,
    tracked_data,
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

use crate::entity::{Entity, EntityBase, living::LivingEntity};

pub const ALL_LAYERS: u8 = 0x7F;
const MAIN_HAND_LEFT: u8 = 0;
const MAIN_HAND_RIGHT: u8 = 1;

const PLAYER_MODEL_PARTS: &[(&str, u8)] = &[
    ("cape", 1 << 0),
    ("jacket", 1 << 1),
    ("left_sleeve", 1 << 2),
    ("right_sleeve", 1 << 3),
    ("left_pants_leg", 1 << 4),
    ("right_pants_leg", 1 << 5),
    ("hat", 1 << 6),
];

fn default_description() -> TextComponent {
    TextComponent::translate("entity.minecraft.mannequin.label", [])
}

fn parse_valid_pose(name: &str) -> Option<EntityPose> {
    match name {
        "standing" => Some(EntityPose::Standing),
        "crouching" => Some(EntityPose::Crouching),
        "swimming" => Some(EntityPose::Swimming),
        "fall_flying" => Some(EntityPose::FallFlying),
        "sleeping" => Some(EntityPose::Sleeping),
        _ => None,
    }
}

const fn pose_to_name(pose: EntityPose) -> &'static str {
    match pose {
        EntityPose::Crouching => "crouching",
        EntityPose::Swimming => "swimming",
        EntityPose::FallFlying => "fall_flying",
        EntityPose::Sleeping => "sleeping",
        _ => "standing",
    }
}

pub struct MannequinEntity {
    pub living_entity: LivingEntity,
    profile: Mutex<ProfileImpl>,
    layers_mask: AtomicU8,
    main_hand: AtomicU8,
    immovable: AtomicBool,
    description: Mutex<TextComponent>,
    hide_description: AtomicBool,
}

impl MannequinEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            living_entity: LivingEntity::new(entity),
            profile: Mutex::new(ProfileImpl::default()),
            layers_mask: AtomicU8::new(ALL_LAYERS),
            main_hand: AtomicU8::new(MAIN_HAND_RIGHT),
            immovable: AtomicBool::new(false),
            description: Mutex::new(default_description()),
            hide_description: AtomicBool::new(false),
        })
    }

    #[must_use]
    pub fn get_profile(&self) -> ProfileImpl {
        self.profile
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_profile(&self, profile: ProfileImpl) {
        self.living_entity
            .entity
            .set_synced_data(tracked_data::mannequin::DATA_PROFILE, profile.clone());
        *self
            .profile
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = profile;
    }

    #[must_use]
    pub fn get_layers_mask(&self) -> u8 {
        self.layers_mask.load(Ordering::Relaxed)
    }

    pub fn set_layers_mask(&self, mask: u8) {
        let clamped = mask & ALL_LAYERS;
        self.layers_mask.store(clamped, Ordering::Relaxed);
        self.living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MODE_CUSTOMISATION,
            clamped as i8,
        );
    }

    #[must_use]
    pub fn get_main_hand_id(&self) -> u8 {
        self.main_hand.load(Ordering::Relaxed)
    }

    pub fn set_main_hand_id(&self, hand: u8) {
        let valid = if hand == MAIN_HAND_LEFT {
            MAIN_HAND_LEFT
        } else {
            MAIN_HAND_RIGHT
        };
        self.main_hand.store(valid, Ordering::Relaxed);
        self.living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MAIN_HAND,
            VarInt(i32::from(valid)),
        );
    }

    #[must_use]
    pub fn get_immovable(&self) -> bool {
        self.immovable.load(Ordering::Relaxed)
    }

    pub fn set_immovable(&self, immovable: bool) {
        self.immovable.store(immovable, Ordering::Relaxed);
        self.living_entity
            .entity
            .set_synced_data(tracked_data::mannequin::DATA_IMMOVABLE, immovable);
    }

    #[must_use]
    pub fn get_description(&self) -> Option<TextComponent> {
        if self.hide_description.load(Ordering::Relaxed) {
            None
        } else {
            Some(
                self.description
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            )
        }
    }

    pub fn set_description_state(&self, description: TextComponent, hide: bool) {
        self.hide_description.store(hide, Ordering::Relaxed);
        *self
            .description
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = description;
        self.sync_description();
    }

    fn sync_description(&self) {
        self.living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_DESCRIPTION,
            self.get_description(),
        );
    }
}

impl EntityBase for MannequinEntity {
    fn init_data_tracker(&self) {
        let entity = &self.living_entity.entity;
        entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MAIN_HAND,
            VarInt(i32::from(self.get_main_hand_id())),
        );
        entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MODE_CUSTOMISATION,
            self.get_layers_mask() as i8,
        );
        entity.set_synced_data(tracked_data::mannequin::DATA_PROFILE, self.get_profile());
        entity.set_synced_data(
            tracked_data::mannequin::DATA_IMMOVABLE,
            self.get_immovable(),
        );
        self.sync_description();
    }

    fn is_immobile(&self) -> bool {
        self.get_immovable() || self.living_entity.is_dead_or_dying()
    }

    fn is_effective_ai(&self) -> bool {
        !self.get_immovable()
    }

    fn get_gravity(&self) -> f64 {
        self.living_entity.get_attribute_value(&Attributes::GRAVITY)
    }

    fn apply_implicit_component(&self, component: DataComponent, value: &str) {
        if component == DataComponent::Profile {
            self.set_profile(ProfileImpl {
                name: Some(value.to_string()),
                ..Default::default()
            });
        }
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put("profile", self.get_profile().write_data());

        let mask = self.get_layers_mask();
        let hidden_layers: Vec<NbtTag> = PLAYER_MODEL_PARTS
            .iter()
            .filter(|(_, bit)| (mask & bit) == 0)
            .map(|(name, _)| NbtTag::String((*name).into()))
            .collect();
        nbt.put_list("hidden_layers", hidden_layers);

        let hand_str = if self.get_main_hand_id() == MAIN_HAND_LEFT {
            "left"
        } else {
            "right"
        };
        nbt.put_string("main_hand", hand_str.to_string());
        nbt.put_string(
            "pose",
            pose_to_name(self.living_entity.entity.pose.load()).to_string(),
        );
        nbt.put_bool("immovable", self.get_immovable());

        if let Some(description) = self.get_description() {
            if description != default_description() {
                nbt.put(
                    "description",
                    description.to_nbt_tag_for_version(&JavaMinecraftVersion::V_26_3),
                );
            }
        } else {
            nbt.put_bool("hide_description", true);
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(profile) = nbt.get("profile").and_then(ProfileImpl::read_data) {
            self.set_profile(profile);
        }

        let mask = nbt.get_list("hidden_layers").map_or(ALL_LAYERS, |list| {
            let mut m = ALL_LAYERS;
            for entry in list {
                if let Some(name) = entry.extract_string()
                    && let Some((_, bit)) = PLAYER_MODEL_PARTS.iter().find(|(n, _)| *n == name)
                {
                    m &= !bit;
                }
            }
            m
        });
        self.set_layers_mask(mask);

        let hand = match nbt.get_string("main_hand") {
            Some("left") => MAIN_HAND_LEFT,
            _ => MAIN_HAND_RIGHT,
        };
        self.set_main_hand_id(hand);

        let pose = nbt
            .get_string("pose")
            .and_then(parse_valid_pose)
            .unwrap_or(EntityPose::Standing);
        self.living_entity.entity.set_pose(pose);

        self.set_immovable(nbt.get_bool("immovable").unwrap_or(false));

        let hide_description = nbt.get_bool("hide_description").unwrap_or(false);
        let description = nbt
            .get("description")
            .map_or_else(default_description, TextComponent::from_nbt);
        self.set_description_state(description, hide_description);
    }

    fn remove_custom_nbt(&self, removed: &[Box<str>]) {
        for key in removed {
            match key.as_ref() {
                "profile" => self.set_profile(ProfileImpl::default()),
                "hidden_layers" => self.set_layers_mask(ALL_LAYERS),
                "main_hand" => self.set_main_hand_id(MAIN_HAND_RIGHT),
                "pose" => self.living_entity.entity.set_pose(EntityPose::Standing),
                "immovable" => self.set_immovable(false),
                "description" => {
                    let hide = self.hide_description.load(Ordering::Relaxed);
                    self.set_description_state(default_description(), hide);
                }
                "hide_description" => {
                    let desc = self
                        .description
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone();
                    self.set_description_state(desc, false);
                }
                _ => {}
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::{data_component_impl::ProfileProperty, entity::EntityType};
    use pumpkin_util::math::vector3::Vector3;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn mannequin_nbt_roundtrip_and_immovable_ai() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let mannequin = MannequinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::MANNEQUIN,
        ));

        assert!(mannequin.is_effective_ai());
        assert_eq!(mannequin.get_layers_mask(), ALL_LAYERS);

        let mut nbt = NbtCompound::new();
        let profile = ProfileImpl {
            name: Some("Alex".to_string()),
            id: Some([1, 2, 3, 4]),
            properties: vec![ProfileProperty {
                name: "textures".to_string(),
                value: "eyJ0ZXh0dXJlcyI6e319".to_string(),
                signature: None,
            }],
            ..Default::default()
        };
        nbt.put("profile", profile.write_data());
        nbt.put_list(
            "hidden_layers",
            vec![NbtTag::String("cape".into()), NbtTag::String("hat".into())],
        );
        nbt.put_string("main_hand", "left".to_string());
        nbt.put_string("pose", "crouching".to_string());
        nbt.put_bool("immovable", true);
        nbt.put_string("description", "Custom NPC".to_string());

        EntityBase::read_nbt_non_mut(mannequin.as_ref(), &nbt);

        assert!(!mannequin.is_effective_ai());
        assert!(mannequin.is_immobile());
        assert_eq!(mannequin.get_profile(), profile);
        assert_eq!(
            mannequin.get_layers_mask(),
            ALL_LAYERS & !(1 << 0) & !(1 << 6)
        );
        assert_eq!(mannequin.get_main_hand_id(), MAIN_HAND_LEFT);
        assert_eq!(mannequin.get_entity().pose.load(), EntityPose::Crouching);
        assert_eq!(
            mannequin.get_description(),
            Some(TextComponent::text("Custom NPC"))
        );

        let mut saved = NbtCompound::new();
        EntityBase::write_nbt(mannequin.as_ref(), &mut saved);
        assert_eq!(saved.get_string("main_hand"), Some("left"));
        assert_eq!(saved.get_string("pose"), Some("crouching"));
        assert_eq!(saved.get_bool("immovable"), Some(true));
        assert_eq!(
            saved.get_list("hidden_layers"),
            Some(&[NbtTag::String("cape".into()), NbtTag::String("hat".into()),][..])
        );

        world.level.shutdown().await;
        Ok(())
    }
}
