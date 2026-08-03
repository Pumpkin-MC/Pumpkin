use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};
use crate::world::game_event::{GameEventContext, emit_game_event};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockDirection;
use pumpkin_data::damage::DamageType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
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

    pub fn get_facing(&self) -> BlockDirection {
        BlockDirection::from_index(self.facing.load(Ordering::Relaxed))
            .unwrap_or(Self::DEFAULT_FACING)
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

    /// Vanilla `ItemFrame.dropItem(level, causedBy, withFrame)`. Clears the
    /// displayed item unconditionally; whether anything actually spawns in
    /// the world depends on the `entity_drops` game rule and whether the
    /// causer is a creative-mode player (vanilla: `hasInfiniteMaterials`).
    async fn drop_item(&self, causer: Option<&dyn EntityBase>, with_frame: bool) {
        if self.fixed.load(Ordering::Relaxed) {
            return;
        }

        let item_stack =
            std::mem::replace(&mut *self.item_stack.lock().await, ItemStack::EMPTY.clone());

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
            world
                .drop_stack(&pos, ItemStack::new(1, &Item::ITEM_FRAME))
                .await;
        }
        if !item_stack.is_empty() && rand::rng().random::<f32>() < self.item_drop_chance.load() {
            world.drop_stack(&pos, item_stack).await;
        }
    }

    /// Vanilla fires `GameEvent.BLOCK_CHANGE` from both the item-pop and the
    /// full-break paths of `ItemFrame.hurtServer`/`dropItem`. No `Arc<dyn
    /// EntityBase>` is available for the causer here (only `&dyn
    /// EntityBase`), so this uses `GameEventContext::none()` like other
    /// position-only emission sites this session -- only source-entity-based
    /// listener suppression loses fidelity, not the emission itself.
    async fn emit_block_change(&self) {
        emit_game_event(
            &self.entity.world.load(),
            GameEvent::BlockChange,
            self.entity.pos.load(),
            GameEventContext::none(),
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
                self.emit_block_change().await;
                self.entity.world.load().play_sound(
                    pumpkin_data::sound::Sound::EntityItemFrameRemoveItem,
                    pumpkin_data::sound::SoundCategory::Blocks,
                    &self.entity.pos.load(),
                );
                return true;
            }

            // Otherwise the frame itself breaks: drop the frame item (and the
            // displayed item, if any), matching ItemFrame.dropItem.
            self.drop_item(causer, true).await;
            self.emit_block_change().await;
            self.entity.world.load().play_sound(
                pumpkin_data::sound::Sound::EntityItemFrameBreak,
                pumpkin_data::sound::SoundCategory::Blocks,
                &self.entity.pos.load(),
            );
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
