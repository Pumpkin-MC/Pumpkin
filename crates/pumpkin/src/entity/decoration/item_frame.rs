use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockDirection;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::{CSetEntityMetadata, Metadata};
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
    /// Facing used when a frame is created without NBT, matching vanilla.
    const DEFAULT_FACING: BlockDirection = BlockDirection::South;

    pub fn new(entity: Entity) -> Self {
        let facing = Self::DEFAULT_FACING.to_index();
        // The spawn packet reads the direction from the entity data field, so
        // it has to agree with `facing` or the frame spawns facing elsewhere.
        entity.data.store(i32::from(facing), Ordering::Relaxed);
        Self {
            entity,
            item_stack: Mutex::new(ItemStack::EMPTY.clone()),
            rotation: AtomicU8::new(0),
            facing: AtomicU8::new(facing),
            item_drop_chance: AtomicCell::new(1.0),
            invisible: AtomicBool::new(false),
            fixed: AtomicBool::new(false),
        }
    }

    pub const fn is_glow(&self) -> bool {
        self.entity.entity_type.id == EntityType::GLOW_ITEM_FRAME.id
    }

    pub const fn get_add_item_sound(&self) -> Sound {
        if self.is_glow() {
            Sound::EntityGlowItemFrameAddItem
        } else {
            Sound::EntityItemFrameAddItem
        }
    }

    pub const fn get_remove_item_sound(&self) -> Sound {
        if self.is_glow() {
            Sound::EntityGlowItemFrameRemoveItem
        } else {
            Sound::EntityItemFrameRemoveItem
        }
    }

    pub const fn get_rotate_item_sound(&self) -> Sound {
        if self.is_glow() {
            Sound::EntityGlowItemFrameRotateItem
        } else {
            Sound::EntityItemFrameRotateItem
        }
    }

    pub const fn get_break_sound(&self) -> Sound {
        if self.is_glow() {
            Sound::EntityGlowItemFrameBreak
        } else {
            Sound::EntityItemFrameBreak
        }
    }

    pub const fn get_place_sound(&self) -> Sound {
        if self.is_glow() {
            Sound::EntityGlowItemFramePlace
        } else {
            Sound::EntityItemFramePlace
        }
    }

    pub fn get_facing(&self) -> BlockDirection {
        BlockDirection::from_index(self.facing.load(Ordering::Relaxed))
            .unwrap_or(Self::DEFAULT_FACING)
    }

    pub fn set_facing(&self, facing: BlockDirection) {
        let index = facing.to_index();
        self.facing.store(index, Ordering::Relaxed);
        self.entity.data.store(i32::from(index), Ordering::Relaxed);
    }

    pub async fn get_item(&self) -> ItemStack {
        self.item_stack.lock().await.clone()
    }

    pub async fn set_item(&self, mut item_stack: ItemStack, update_neighbours: bool) {
        if !item_stack.is_empty() {
            item_stack.item_count = 1;
        }

        let play_sound = !item_stack.is_empty();
        let item_serializer = ItemStackSerializer::from(item_stack.clone());
        *self.item_stack.lock().await = item_stack;

        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::item_frame::ITEM,
                &item_serializer,
            )],
            None,
        );

        if play_sound {
            self.entity.play_sound(self.get_add_item_sound());
        }

        if update_neighbours {
            let world = self.entity.world.load();
            let pos = self.entity.block_pos.load();
            world.update_neighbors(&pos, None).await;
        }
    }

    pub fn get_rotation(&self) -> u8 {
        self.rotation.load(Ordering::Relaxed) % 8
    }

    pub async fn set_rotation(&self, rotation: u8, update_neighbours: bool) {
        let rot = rotation % 8;
        self.rotation.store(rot, Ordering::Relaxed);

        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::item_frame::ROTATION,
                rot as i32,
            )],
            None,
        );

        if update_neighbours {
            let world = self.entity.world.load();
            let pos = self.entity.block_pos.load();
            world.update_neighbors(&pos, None).await;
        }
    }

    pub fn get_drop_chance(&self) -> f32 {
        self.item_drop_chance.load()
    }

    pub fn set_drop_chance(&self, chance: f32) {
        self.item_drop_chance.store(chance);
    }

    pub fn is_fixed(&self) -> bool {
        self.fixed.load(Ordering::Relaxed)
    }

    pub fn set_fixed(&self, fixed: bool) {
        self.fixed.store(fixed, Ordering::Relaxed);
    }

    pub fn is_invisible(&self) -> bool {
        self.invisible.load(Ordering::Relaxed)
    }

    pub fn set_invisible(&self, invisible: bool) {
        self.invisible.store(invisible, Ordering::Relaxed);
    }

    pub fn get_frame_item_stack(&self) -> ItemStack {
        if self.is_glow() {
            ItemStack::new(1, &Item::GLOW_ITEM_FRAME)
        } else {
            ItemStack::new(1, &Item::ITEM_FRAME)
        }
    }

    pub fn get_frame_item_stack_with_data(&self) -> ItemStack {
        let mut stack = self.get_frame_item_stack();
        if let Some(custom_name) = self.entity.custom_name.load().as_ref().clone() {
            stack.set_custom_name(custom_name.to_pretty_console());
        }
        stack
    }

    pub async fn get_pick_result(&self) -> ItemStack {
        let framed_stack = self.get_item().await;
        if framed_stack.is_empty() {
            self.get_frame_item_stack_with_data()
        } else {
            framed_stack
        }
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

    pub async fn drop_item(&self, caused_by: Option<&dyn EntityBase>, with_frame: bool) {
        if self.is_fixed() {
            return;
        }

        let item_stack = self.get_item().await;
        self.set_item(ItemStack::EMPTY.clone(), true).await;

        let is_creative_player = caused_by.is_some_and(|s| {
            s.cast_any()
                .downcast_ref::<Player>()
                .is_some_and(Player::is_creative)
        });

        if is_creative_player {
            return;
        }

        let world = self.entity.world.load();
        let pos = self.entity.block_pos.load();

        if with_frame {
            world
                .drop_stack(&pos, self.get_frame_item_stack_with_data())
                .await;
        }

        if !item_stack.is_empty() {
            let drop_chance = self.item_drop_chance.load();
            if rand::random::<f32>() < drop_chance {
                world.drop_stack(&pos, item_stack).await;
            }
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
            }
            nbt.put_float("ItemDropChance", self.item_drop_chance.load());
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

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            let item_serializer = ItemStackSerializer::from(self.item_stack.lock().await.clone());
            let rotation = self.get_rotation() as i32;

            self.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::item_frame::ITEM,
                    &item_serializer,
                )],
                None,
            );
            self.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::item_frame::ROTATION,
                    rotation,
                )],
                None,
            );
        })
    }

    fn send_java_spawn_packet<'a>(
        &'a self,
        client: &'a crate::net::java::JavaClient,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            client
                .enqueue_packet(&self.entity.create_spawn_packet())
                .await;

            let ver = client.version.load();
            if ver >= CURRENT_MC_VERSION {
                let item_serializer =
                    ItemStackSerializer::from(self.item_stack.lock().await.clone());
                let rotation = self.get_rotation() as i32;

                let mut data = Vec::new();
                let meta_item = Metadata::new(
                    pumpkin_data::tracked_data::item_frame::ITEM,
                    item_serializer,
                );
                let meta_rot =
                    Metadata::new(pumpkin_data::tracked_data::item_frame::ROTATION, rotation);

                if meta_item.write(&mut data, &ver).is_ok()
                    && meta_rot.write(&mut data, &ver).is_ok()
                {
                    data.push(255);
                    client
                        .enqueue_packet(&CSetEntityMetadata::new(
                            self.entity.entity_id.into(),
                            data.into(),
                        ))
                        .await;
                }
            }
        })
    }

    fn interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.is_fixed() {
                return false;
            }

            let frame_has_item = !self.get_item().await.is_empty();
            let has_held_item = !item_stack.is_empty();

            if frame_has_item {
                let new_rot = self.get_rotation() + 1;
                self.set_rotation(new_rot, true).await;
                self.entity.play_sound(self.get_rotate_item_sound());
                true
            } else if has_held_item && !self.entity.removed.load(Ordering::Relaxed) {
                let mut new_stack = item_stack.clone();
                new_stack.item_count = 1;
                self.set_item(new_stack, true).await;

                if !player.is_creative() {
                    item_stack.decrement(1);
                }
                true
            } else {
                false
            }
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let fixed = self.is_fixed();
            let is_creative_player = source.is_some_and(|s| {
                s.cast_any()
                    .downcast_ref::<Player>()
                    .is_some_and(Player::is_creative)
            });
            let bypasses_invuln =
                damage_type == DamageType::OUT_OF_WORLD || damage_type == DamageType::GENERIC_KILL;

            if fixed {
                if !bypasses_invuln && !is_creative_player {
                    return false;
                }
                self.drop_item(source, true).await;
                self.entity.remove().await;
                return true;
            }

            let has_item = !self.get_item().await.is_empty();
            let is_explosion =
                damage_type == DamageType::EXPLOSION || damage_type == DamageType::PLAYER_EXPLOSION;

            if !is_explosion && has_item {
                self.drop_item(source, false).await;
                self.entity.play_sound(self.get_remove_item_sound());
            } else {
                self.drop_item(source, true).await;
                self.entity.play_sound(self.get_break_sound());
                self.entity.remove().await;
            }
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
