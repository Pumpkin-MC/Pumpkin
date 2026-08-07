use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
    player::Player,
};
use crate::server::Server;
use crate::world::game_event::{GameEventContext, emit_game_event};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedId;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
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
    fixed: AtomicBool,
    /// `BlockAttachedEntity.checkInterval`: the support block is only
    /// revalidated once every 100 ticks.
    check_interval: AtomicU32,
}

impl ItemFrameEntity {
    /// Facing used when a frame is created without NBT, matching vanilla.
    const DEFAULT_FACING: BlockDirection = BlockDirection::South;

    /// `ItemFrame.DATA_ITEM`. `HangingEntity` gained a synched `DATA_DIRECTION`
    /// field in 26.1 (generated `TrackedData::DIRECTION` is absent before then
    /// and 8 from 26.1), which pushed both item-frame fields up by one. The
    /// pre-26.1 values follow from `Entity` occupying 0..=7 in every supported
    /// version; the generated `ROTATION` entry is a cross-entity name collision
    /// and is deliberately not used here.
    const DATA_ITEM: TrackedId = TrackedId {
        v1_21: 8,
        v1_21_2: 8,
        v1_21_4: 8,
        v1_21_5: 8,
        v1_21_6: 8,
        v1_21_7: 8,
        v1_21_9: 8,
        v1_21_11: 8,
        v26_1: 9,
        v26_2: 9,
    };

    /// `ItemFrame.DATA_ROTATION`.
    const DATA_ROTATION: TrackedId = TrackedId {
        v1_21: 9,
        v1_21_2: 9,
        v1_21_4: 9,
        v1_21_5: 9,
        v1_21_6: 9,
        v1_21_7: 9,
        v1_21_9: 9,
        v1_21_11: 9,
        v26_1: 10,
        v26_2: 10,
    };

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
            fixed: AtomicBool::new(false),
            check_interval: AtomicU32::new(0),
        }
    }

    pub fn get_facing(&self) -> BlockDirection {
        BlockDirection::from_index(self.facing.load(Ordering::Relaxed))
            .unwrap_or(Self::DEFAULT_FACING)
    }

    /// `GlowItemFrame` overrides every sound and the dropped frame item.
    fn is_glow(&self) -> bool {
        self.entity.entity_type == &EntityType::GLOW_ITEM_FRAME
    }

    fn play_frame_sound(&self, plain: Sound, glow: Sound) {
        self.entity.world.load().play_sound(
            if self.is_glow() { glow } else { plain },
            SoundCategory::Blocks,
            &self.entity.pos.load(),
        );
    }

    /// Vanilla `level.updateNeighbourForOutputSignal(pos, Blocks.AIR)`, run by
    /// `setItem`/`setRotation` so an attached comparator re-reads the frame.
    async fn update_output_signal(&self) {
        let world = self.entity.world.load();
        world
            .update_comparators(&self.entity.block_pos.load(), &Block::AIR)
            .await;
    }

    fn sync_item(&self, stack: &ItemStack) {
        self.entity.send_meta_data(
            &[Metadata::new(
                Self::DATA_ITEM,
                MetaDataType::ITEM_STACK,
                &ItemStackSerializer::from(stack.clone()),
            )],
            None,
        );
    }

    fn sync_rotation(&self, rotation: u8) {
        self.entity.send_meta_data(
            &[Metadata::new(
                Self::DATA_ROTATION,
                MetaDataType::INT,
                VarInt(i32::from(rotation)),
            )],
            None,
        );
    }

    /// Vanilla `ItemFrame.setItem`: the frame only ever holds a single item.
    async fn set_item(&self, stack: ItemStack) {
        let stack = if stack.is_empty() {
            ItemStack::EMPTY.clone()
        } else {
            stack.copy_with_count(1)
        };
        *self.item_stack.lock().await = stack.clone();
        self.sync_item(&stack);
        if !stack.is_empty() {
            self.play_frame_sound(
                Sound::EntityItemFrameAddItem,
                Sound::EntityGlowItemFrameAddItem,
            );
        }
        self.update_output_signal().await;
    }

    /// Vanilla `ItemFrame.setRotation`, always reduced modulo 8.
    async fn set_rotation(&self, rotation: u8) {
        let rotation = rotation % 8;
        self.rotation.store(rotation, Ordering::Relaxed);
        self.sync_rotation(rotation);
        self.update_output_signal().await;
    }

    /// Vanilla `ItemFrame.survives`. The collision and `canCoexist` checks are
    /// not modelled; only the support block is tested.
    fn survives(&self) -> bool {
        if self.fixed.load(Ordering::Relaxed) {
            return true;
        }
        let facing = self.get_facing();
        let support = self
            .entity
            .block_pos
            .load()
            .offset(facing.opposite().to_offset());
        let world = self.entity.world.load();
        // An unloaded support block must not read as air, or the frame would be
        // destroyed for a chunk that simply is not there yet.
        let Some(state_id) = world.get_block_state_id_if_loaded(&support) else {
            return true;
        };
        let (block, state) = BlockState::from_id_with_block(state_id);
        state.is_solid()
            || (facing.is_horizontal()
                && (block == &Block::REPEATER || block == &Block::COMPARATOR))
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

    /// Vanilla `ItemFrame.dropItem`; spawning depends on the `entity_drops`
    /// game rule and whether the causer is a creative-mode player.
    async fn drop_item(&self, causer: Option<&dyn EntityBase>, with_frame: bool) {
        if self.fixed.load(Ordering::Relaxed) {
            return;
        }

        let item_stack =
            std::mem::replace(&mut *self.item_stack.lock().await, ItemStack::EMPTY.clone());
        if !item_stack.is_empty() {
            // Vanilla clears the slot through setItem, so the emptied frame is
            // both re-rendered and re-read by any attached comparator.
            self.sync_item(ItemStack::EMPTY);
            self.update_output_signal().await;
        }

        let world = self.entity.world.load();
        if !world.level_info.load().game_rules.entity_drops {
            return;
        }
        let creative_causer = causer
            .and_then(EntityBase::get_player)
            .is_some_and(|player| player.gamemode.load() == GameMode::Creative);
        if creative_causer {
            return;
        }

        let pos = self.entity.block_pos.load();
        if with_frame {
            // GlowItemFrame.getFrameItemStack
            let frame_item = if self.is_glow() {
                &Item::GLOW_ITEM_FRAME
            } else {
                &Item::ITEM_FRAME
            };
            world.drop_stack(&pos, ItemStack::new(1, frame_item)).await;
        }
        if !item_stack.is_empty() && rand::rng().random::<f32>() < self.item_drop_chance.load() {
            world.drop_stack(&pos, item_stack).await;
        }
    }

    /// Vanilla `ItemFrame.hurtServer`/`dropItem`/`interact` all fire
    /// `GameEvent.BLOCK_CHANGE`, attributed to whoever caused the change.
    async fn emit_block_change(&self, causer: Option<Arc<dyn EntityBase>>) {
        emit_game_event(
            &self.entity.world.load(),
            GameEvent::BlockChange,
            self.entity.pos.load(),
            causer.map_or_else(GameEventContext::none, GameEventContext::of_entity),
        )
        .await;
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
            nbt.put_bool("Invisible", self.entity.invisible.load(Ordering::Relaxed));
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
            // `setInvisible` is `Entity.setSharedFlag(FLAG_INVISIBLE)`, so this has
            // to go through the shared-flags metadata or the frame still renders.
            self.entity
                .set_invisible(nbt.get_bool("Invisible").unwrap_or(false))
                .await;
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
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let causer = cause.or(source);

            // ItemFrame.canHurtWhenFixed: a fixed frame can only be hit by
            // damage that bypasses invulnerability, or a creative player.
            let can_hurt_when_fixed = damage_type
                .has_tag(&tag::DamageType::MINECRAFT_BYPASSES_INVULNERABILITY)
                || causer
                    .and_then(EntityBase::get_player)
                    .is_some_and(|player| player.gamemode.load() == GameMode::Creative);
            if self.fixed.load(Ordering::Relaxed) && !can_hurt_when_fixed {
                return false;
            }

            if !self.fixed.load(Ordering::Relaxed)
                && self.entity.is_invulnerable_to(&damage_type).await
            {
                return false;
            }

            // ItemFrame.shouldDamageDropItem: non-explosion damage against a
            // frame currently holding an item only pops the item -- the frame
            // itself survives.
            let holds_item = !self.item_stack.lock().await.is_empty();
            let is_explosion = damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION);
            if !self.fixed.load(Ordering::Relaxed) && !is_explosion && holds_item {
                self.drop_item(causer, false).await;
                self.emit_block_change(None).await;
                self.play_frame_sound(
                    Sound::EntityItemFrameRemoveItem,
                    Sound::EntityGlowItemFrameRemoveItem,
                );
                return true;
            }

            // Otherwise the frame itself breaks: drop the frame item (and the
            // displayed item, if any), matching ItemFrame.dropItem.
            self.drop_item(causer, true).await;
            self.emit_block_change(None).await;
            self.play_frame_sound(Sound::EntityItemFrameBreak, Sound::EntityGlowItemFrameBreak);
            self.entity.remove().await;
            true
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            let stack = self.item_stack.lock().await.clone();
            self.sync_item(&stack);
            self.sync_rotation(self.rotation.load(Ordering::Relaxed));
        })
    }

    /// Vanilla `ItemFrame.interact`: an empty frame takes the held item, an
    /// occupied one steps the displayed rotation through its eight positions.
    fn interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.fixed.load(Ordering::Relaxed) {
                return false;
            }

            let causer = Some(player.clone() as Arc<dyn EntityBase>);
            if !self.item_stack.lock().await.is_empty() {
                self.play_frame_sound(
                    Sound::EntityItemFrameRotateItem,
                    Sound::EntityGlowItemFrameRotateItem,
                );
                self.set_rotation(self.rotation.load(Ordering::Relaxed).wrapping_add(1))
                    .await;
                self.emit_block_change(causer).await;
                return true;
            }

            if item_stack.is_empty() || self.entity.removed.load(Ordering::Relaxed) {
                return false;
            }

            self.set_item(item_stack.clone()).await;
            self.emit_block_change(causer).await;
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            true
        })
    }

    /// `BlockAttachedEntity.tick`: drop the frame once its support is gone.
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.entity.tick(caller, server).await;

            if self.check_interval.fetch_add(1, Ordering::Relaxed) < 100 {
                return;
            }
            self.check_interval.store(0, Ordering::Relaxed);

            if self.entity.removed.load(Ordering::Relaxed) || self.survives() {
                return;
            }

            self.entity.remove().await;
            self.play_frame_sound(Sound::EntityItemFrameBreak, Sound::EntityGlowItemFrameBreak);
            self.drop_item(None, true).await;
            self.emit_block_change(None).await;
        })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
