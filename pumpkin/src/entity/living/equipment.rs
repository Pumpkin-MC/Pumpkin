use super::LivingEntity;
use crate::entity::{Entity, EntityBase};
use pumpkin_data::data_component_impl::{BlocksAttacksImpl, EquipmentSlot};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_protocol::bedrock::client::take_item_actor::CTakeItemActor;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{CSetEquipment, CTakeItemEntity, Metadata};
use pumpkin_util::Hand;
use std::sync::Arc;
use std::sync::atomic::Ordering::{self, SeqCst};
use tokio::sync::Mutex;

impl LivingEntity {
    pub fn send_equipment_changes(&self, equipment: &[(EquipmentSlot, ItemStack)]) {
        if equipment.is_empty() {
            return;
        }
        let equipment: Vec<(i8, ItemStackSerializer)> = equipment
            .iter()
            .map(|(slot, stack)| {
                (
                    slot.discriminant(),
                    ItemStackSerializer::from(stack.clone()),
                )
            })
            .collect();
        self.entity.world.load().broadcast_packet_except(
            &[self.entity.entity_uuid],
            &CSetEquipment::new(self.entity_id().into(), equipment),
        );
    }

    /// Snapshot of non-empty equipment for spawn packets (sync `try_lock`).
    #[must_use]
    pub fn equipment_packet_if_any(&self) -> Option<CSetEquipment> {
        let Ok(eq) = self.entity_equipment.try_lock() else {
            return None;
        };
        let mut list = Vec::new();
        for (slot, stack_arc) in &eq.equipment {
            let Ok(stack) = stack_arc.try_lock() else {
                continue;
            };
            if stack.is_empty() {
                continue;
            }
            list.push((
                slot.discriminant(),
                ItemStackSerializer::from(stack.clone()),
            ));
        }
        if list.is_empty() {
            None
        } else {
            Some(CSetEquipment::new(self.entity_id().into(), list))
        }
    }

    /// Picks up and Item entity or XP Orb
    pub fn pickup(&self, item: &Entity, stack_amount: u32) {
        let chunk_pos = self.entity.chunk_pos.load();
        self.entity.world.load().broadcast_to_chunk_editioned_sync(
            chunk_pos,
            &CTakeItemEntity::new(
                item.entity_id.into(),
                self.entity.entity_id.into(),
                VarInt(stack_amount as i32),
            ),
            &CTakeItemActor::new(
                VarULong(item.entity_id as u64),
                VarULong(self.entity.entity_id as u64),
            ),
        );
    }

    /// Sends the Hand animation to all others, used when Eating for example
    pub async fn set_active_hand(&self, hand: Hand, stack: ItemStack, duration: i32) {
        let mut item_in_use = self.item_in_use.lock().await;
        let mut active_hand = self.active_hand.lock().await;

        // Vanilla `startUsingItem` ignores empty or already-active uses.
        if stack.is_empty() || active_hand.is_some() {
            return;
        }

        self.item_use_time.store(duration, Ordering::Relaxed);
        *item_in_use = Some(stack);
        *active_hand = Some(hand);

        // Emit the completed USING_ITEM/OFF_HAND state in one update.
        self.sync_active_hand_flags(Some(hand), true);
    }

    pub(super) const fn with_active_hand_flags(flags: u8, hand: Option<Hand>) -> u8 {
        let mut flags = flags & !Self::ACTIVE_HAND_FLAGS;
        if let Some(hand) = hand {
            flags |= Self::USING_ITEM_FLAG;
            if matches!(hand, Hand::Left) {
                flags |= Self::OFF_HAND_ACTIVE_FLAG;
            }
        }
        flags
    }

    pub(super) fn living_flags_metadata(flags: u8) -> [Metadata<u8>; 2] {
        // `LIVING_FLAGS` = index 8 on 1.21.x; `LIVING_ENTITY_FLAGS` = index 8 on 26.x.
        // Metadata write skips TrackedId entries that resolve to 255 for the client
        // version, so send both and the correct one is applied.
        // Without this, skeleton bow-draw (using-item bit) never reaches 1.21 clients.
        [
            Metadata::new(TrackedData::LIVING_ENTITY_FLAGS, MetaDataType::BYTE, flags),
            Metadata::new(TrackedData::LIVING_FLAGS, MetaDataType::BYTE, flags),
        ]
    }

    fn sync_active_hand_flags(&self, hand: Option<Hand>, force_sync: bool) {
        let mut current = self.livings_flags.load(SeqCst);
        let (flags, changed) = loop {
            let next = Self::with_active_hand_flags(current, hand);
            match self
                .livings_flags
                .compare_exchange_weak(current, next, SeqCst, SeqCst)
            {
                Ok(_) => break (next, next != current),
                Err(actual) => current = actual,
            }
        };

        if !force_sync && !changed {
            return;
        }

        let mut bedrock_meta =
            pumpkin_protocol::bedrock::client::set_actor_data::EntityMetadata::new();
        bedrock_meta.set_flag(
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_key::FLAGS,
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::USING_ITEM as u8,
            flags & Self::USING_ITEM_FLAG != 0,
        );

        self.entity
            .send_meta_data(&Self::living_flags_metadata(flags), Some(&bedrock_meta));
    }

    pub async fn clear_active_hand(&self) {
        let mut item_in_use = self.item_in_use.lock().await;
        let mut active_hand = self.active_hand.lock().await;
        let had_item_in_use = item_in_use.take().is_some();
        let had_active_hand = active_hand.take().is_some();
        let was_using_item = had_item_in_use || had_active_hand;
        self.item_use_time.store(0, Ordering::Relaxed);

        self.sync_active_hand_flags(None, was_using_item);
    }

    pub async fn is_blocking(&self) -> bool {
        let item_in_use = self.item_in_use.lock().await;
        if let Some(item) = item_in_use.as_ref()
            && item.get_data_component::<BlocksAttacksImpl>().is_some()
        {
            let use_time = self.item_use_time.load(Ordering::Relaxed);
            return item.get_max_use_time() - use_time >= 5;
        }
        false
    }

    pub async fn swing_hand(&self) {
        let world = self.entity.world.load();
        let entity_id = self.entity_id();

        let je_packet = pumpkin_protocol::java::client::play::CEntityAnimation::new(
            entity_id.into(),
            pumpkin_protocol::java::client::play::Animation::SwingMainArm,
        );
        let be_packet = pumpkin_protocol::bedrock::server::animate::SAnimate {
            action: pumpkin_protocol::bedrock::server::animate::AnimateAction::SwingArm,
            runtime_entity_id: pumpkin_protocol::codec::var_ulong::VarULong(entity_id as u64),
            data: 0.0,
            swing_source: None,
        };

        world.broadcast_editioned(&je_packet, &be_packet).await;
    }

    pub async fn held_item(&self, caller: &dyn EntityBase) -> Arc<Mutex<ItemStack>> {
        if let Some(player) = caller.get_player() {
            return player.inventory.held_item();
        }
        self.entity_equipment
            .lock()
            .await
            .get(&EquipmentSlot::MAIN_HAND)
    }

    pub async fn get_stack_in_hand(
        &self,
        caller: &dyn EntityBase,
        hand: Hand,
    ) -> Arc<Mutex<ItemStack>> {
        match hand {
            Hand::Left => self.off_hand_item().await,
            Hand::Right => self.held_item(caller).await,
        }
    }

    /// getOffHandStack in source
    pub async fn off_hand_item(&self) -> Arc<Mutex<ItemStack>> {
        let slot = self
            .equipment_slots
            .get(&PlayerInventory::OFF_HAND_SLOT)
            .unwrap();
        self.entity_equipment.lock().await.get(slot)
    }
}
