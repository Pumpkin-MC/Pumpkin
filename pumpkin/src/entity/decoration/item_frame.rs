use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockDirection;
use pumpkin_data::damage::DamageType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;

/// An item frame or glow item frame.
///
/// Holds the displayed item and its rotation so that comparators can read the
/// frame's analog output and so frames from vanilla worlds keep their data
/// across save cycles.
pub struct ItemFrameEntity {
    entity: Entity,
    item_stack: Mutex<ItemStack>,
    /// Rotation of the displayed item, always in `0..8`.
    rotation: AtomicU8,
    /// The direction the frame faces, i.e. the axis pointing away from the
    /// block it hangs on. Stored as the vanilla 3D direction index
    /// (0 = down, 1 = up, 2 = north, 3 = south, 4 = west, 5 = east).
    facing: AtomicU8,
    item_drop_chance: AtomicCell<f32>,
    invisible: AtomicBool,
    fixed: AtomicBool,
}

impl ItemFrameEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            item_stack: Mutex::new(ItemStack::EMPTY.clone()),
            rotation: AtomicU8::new(0),
            // Vanilla frames default to south when no NBT is present.
            facing: AtomicU8::new(BlockDirection::South.to_index()),
            item_drop_chance: AtomicCell::new(1.0),
            invisible: AtomicBool::new(false),
            fixed: AtomicBool::new(false),
        }
    }

    pub fn get_facing(&self) -> BlockDirection {
        BlockDirection::from_index(self.facing.load(Ordering::Relaxed))
            .unwrap_or(BlockDirection::South)
    }

    /// The comparator signal this frame produces.
    ///
    /// Vanilla: `getItem().isEmpty() ? 0 : getRotation() % 8 + 1`.
    pub async fn get_analog_output(&self) -> u8 {
        if self.item_stack.lock().await.is_empty() {
            0
        } else {
            self.rotation.load(Ordering::Relaxed) % 8 + 1
        }
    }
}

impl NBTStorage for ItemFrameEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;

            let item = self.item_stack.lock().await;
            if !item.is_empty() {
                let mut item_compound = NbtCompound::new();
                item.write_item_stack(&mut item_compound);
                nbt.put_compound("Item", item_compound);
                nbt.put_float("ItemDropChance", self.item_drop_chance.load());
            }
            nbt.put_byte("ItemRotation", self.rotation.load(Ordering::Relaxed) as i8);
            nbt.put_byte("Facing", self.facing.load(Ordering::Relaxed) as i8);
            nbt.put_bool("Invisible", self.invisible.load(Ordering::Relaxed));
            nbt.put_bool("Fixed", self.fixed.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.read_nbt_non_mut(nbt).await;

            if let Some(item_compound) = nbt.get_compound("Item")
                && let Some(stack) = ItemStack::read_item_stack(item_compound)
            {
                *self.item_stack.lock().await = stack;
            }
            self.rotation.store(
                (nbt.get_byte("ItemRotation").unwrap_or(0) as u8) % 8,
                Ordering::Relaxed,
            );
            let facing = nbt.get_byte("Facing").unwrap_or(0) as u8 % 6;
            self.facing.store(facing, Ordering::Relaxed);
            // The spawn packet's data field carries the frame's direction.
            self.entity.data.store(i32::from(facing), Ordering::Relaxed);
            self.item_drop_chance
                .store(nbt.get_float("ItemDropChance").unwrap_or(1.0));
            self.invisible.store(
                nbt.get_bool("Invisible").unwrap_or(false),
                Ordering::Relaxed,
            );
            self.fixed
                .store(nbt.get_bool("Fixed").unwrap_or(false), Ordering::Relaxed);
        })
    }
}

impl EntityBase for ItemFrameEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
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
            // TODO: vanilla pops the displayed item first and only removes the
            // frame itself when hit while empty; both should drop their items.
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
