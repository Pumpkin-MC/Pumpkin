use std::sync::atomic::{AtomicU8, Ordering};

use pumpkin_data::damage::DamageType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;

use crate::entity::decoration::display::DisplayEntity;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};

/// `ItemDisplayContext` ids from `net.minecraft.world.item.ItemDisplayContext`.
const CONTEXT_NONE: u8 = 0;

pub struct ItemDisplayEntity {
    display: DisplayEntity,
    item_stack: Mutex<ItemStack>,
    item_display_context: AtomicU8,
}

impl ItemDisplayEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            display: DisplayEntity::new(entity),
            item_stack: Mutex::new(ItemStack::EMPTY.clone()),
            item_display_context: AtomicU8::new(CONTEXT_NONE),
        }
    }
}

impl NBTStorage for ItemDisplayEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.write_nbt(nbt).await;

            let item = self.item_stack.lock().await;
            if !item.is_empty() {
                let mut item_compound = NbtCompound::new();
                item.write_item_stack(&mut item_compound);
                nbt.put_compound("item", item_compound);
            }
            drop(item);

            let context_name = match self.item_display_context.load(Ordering::Relaxed) {
                1 => "thirdperson_lefthand",
                2 => "thirdperson_righthand",
                3 => "firstperson_lefthand",
                4 => "firstperson_righthand",
                5 => "head",
                6 => "gui",
                7 => "ground",
                8 => "fixed",
                9 => "on_shelf",
                _ => "none",
            };
            nbt.put_string("item_display", context_name.to_string());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.read_nbt_non_mut(nbt).await;

            if let Some(item_compound) = nbt.get_compound("item")
                && let Some(stack) = ItemStack::read_item_stack(item_compound)
            {
                *self.item_stack.lock().await = stack;
            }

            let context = match nbt.get_string("item_display") {
                Some("thirdperson_lefthand") => 1,
                Some("thirdperson_righthand") => 2,
                Some("firstperson_lefthand") => 3,
                Some("firstperson_righthand") => 4,
                Some("head") => 5,
                Some("gui") => 6,
                Some("ground") => 7,
                Some("fixed") => 8,
                Some("on_shelf") => 9,
                _ => CONTEXT_NONE,
            };
            self.item_display_context.store(context, Ordering::Relaxed);
        })
    }
}

impl EntityBase for ItemDisplayEntity {
    fn get_entity(&self) -> &Entity {
        &self.display.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.display.send_metadata();

            let item_stack = self.item_stack.lock().await.clone();
            self.display.entity.send_meta_data(
                &[
                    Metadata::new(
                        TrackedData::ITEM_STACK,
                        MetaDataType::ITEM_STACK,
                        &ItemStackSerializer::from(item_stack.clone()),
                    ),
                    Metadata::new(
                        TrackedData::ITEM_STACK_ID,
                        MetaDataType::ITEM_STACK,
                        &ItemStackSerializer::from(item_stack),
                    ),
                ],
                None,
            );

            let context = self.item_display_context.load(Ordering::Relaxed);
            self.display.entity.send_meta_data(
                &[
                    Metadata::new(TrackedData::ITEM_DISPLAY, MetaDataType::BYTE, context),
                    Metadata::new(TrackedData::ITEM_DISPLAY_ID, MetaDataType::BYTE, context),
                ],
                None,
            );
        })
    }

    /// `Display.hurtServer` always returns false; display entities cannot be damaged.
    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async { false })
    }
}
