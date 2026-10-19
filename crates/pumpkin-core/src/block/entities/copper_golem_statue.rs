use crate::entity::passive::copper_golem::CopperGolemEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::BlockState;
use pumpkin_data::block_properties::{
    CopperGolemPose, CopperGolemStatueLikeProperties, HorizontalFacing,
};
use pumpkin_data::data_component_impl::{BlockStateImpl, CustomNameImpl};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::BlockEntity;

pub struct CopperGolemStatueBlockEntity {
    pub position: BlockPos,
    pub custom_name: Mutex<Option<TextComponent>>,
    dirty: AtomicBool,
}

impl BlockEntity for CopperGolemStatueBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let custom_name = nbt
            .get_compound("components")
            .and_then(|components| components.get("minecraft:custom_name"))
            .or_else(|| nbt.get("custom_name"))
            .or_else(|| nbt.get("CustomName"))
            .map(TextComponent::from_nbt);
        Self {
            position,
            custom_name: Mutex::new(custom_name),
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            let mut components = NbtCompound::new();
            components.put(
                "minecraft:custom_name",
                name.to_nbt_tag_for_version(&CURRENT_MC_VERSION),
            );
            nbt.put_compound("components", components);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn collect_item_components(&self, stack: &mut ItemStack) {
        if let Some(name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            stack.set_data_component(CustomNameImpl { name: name.clone() });
        }
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        *self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stack
            .get_data_component::<CustomNameImpl>()
            .map(|component| component.name.clone());
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CopperGolemStatueBlockEntity {
    pub const ID: &'static str = "minecraft:copper_golem_statue";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            custom_name: Mutex::new(None),
            dirty: AtomicBool::new(false),
        }
    }

    pub fn create_statue(&self, copper_golem: &CopperGolemEntity) {
        self.custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone_from(&copper_golem.get_entity().custom_name.load());
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn remove_statue(&self, world: &Arc<World>, state: &BlockState) -> Arc<CopperGolemEntity> {
        let position = Vector3::new(
            f64::from(self.position.0.x) + 0.5,
            f64::from(self.position.0.y),
            f64::from(self.position.0.z) + 0.5,
        );
        let golem = CopperGolemEntity::new(Entity::new(
            world.clone(),
            position,
            &EntityType::COPPER_GOLEM,
        ));
        let entity = golem.get_entity();
        if let Some(name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            entity.set_custom_name(name.clone());
        }
        let yaw = match CopperGolemStatueLikeProperties::from_state_id(state.id).facing {
            HorizontalFacing::South => 0.0,
            HorizontalFacing::West => 90.0,
            HorizontalFacing::North => 180.0,
            HorizontalFacing::East => 270.0,
        };
        entity.set_rotation(yaw, 0.0);
        entity.body_yaw.store(yaw);
        golem
    }

    pub fn get_item(&self, stack: &mut ItemStack, pose: CopperGolemPose) {
        self.collect_item_components(stack);
        stack.set_data_component(BlockStateImpl {
            properties: Cow::Owned(vec![(
                Cow::Borrowed("copper_golem_pose"),
                Cow::Borrowed(pose.to_value()),
            )]),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;
    use pumpkin_util::text::color::NamedColor;

    #[test]
    fn vanilla_custom_name_survives_save_and_item_components() {
        let mut name = NbtCompound::new();
        name.put_string("text", "Rusty".to_string());
        name.put_string("color", "gold".to_string());
        name.put_bool("bold", true);
        let mut components = NbtCompound::new();
        components.put_compound("minecraft:custom_name", name);
        let mut vanilla_nbt = NbtCompound::new();
        vanilla_nbt.put_compound("components", components);

        let statue = CopperGolemStatueBlockEntity::from_nbt(&vanilla_nbt, BlockPos::new(0, 0, 0));
        let mut saved = NbtCompound::new();
        statue.write_nbt(&mut saved);
        assert_eq!(saved, vanilla_nbt);

        let mut item = ItemStack::new(1, &Item::COPPER_GOLEM_STATUE);
        statue.collect_item_components(&mut item);
        let placed = CopperGolemStatueBlockEntity::new(BlockPos::new(1, 0, 0));
        placed.apply_item_components(&item);
        assert_eq!(
            placed
                .custom_name
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref(),
            Some(
                &TextComponent::text("Rusty")
                    .color_named(NamedColor::Gold)
                    .bold()
            ),
        );
    }
}
