use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::BlockEntity;
use crate::block::entities::sign::Text;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct HangingSignBlockEntity {
    pub front_text: Text,
    pub back_text: Text,
    pub is_waxed: AtomicBool,
    pub allow_op_features: AtomicBool,
    position: BlockPos,
    pub currently_editing_player: Arc<Mutex<Option<uuid::Uuid>>>,
}

impl BlockEntity for HangingSignBlockEntity {
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
        let allow_op_features = nbt.get_bool("allow_op_features").unwrap_or(false);
        let front_text = nbt
            .get("front_text")
            .map(|tag| Text::from_nbt_tag(tag, allow_op_features))
            .unwrap_or_default();
        let back_text = nbt
            .get("back_text")
            .map(|tag| Text::from_nbt_tag(tag, allow_op_features))
            .unwrap_or_default();
        let is_waxed = nbt.get_bool("is_waxed").unwrap_or(false);
        Self {
            position,
            front_text,
            back_text,
            is_waxed: AtomicBool::new(is_waxed),
            allow_op_features: AtomicBool::new(allow_op_features),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        crate::block::entities::sign::write_allow_op_features(nbt, &self.allow_op_features);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        crate::block::entities::sign::write_allow_op_features(&mut nbt, &self.allow_op_features);
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        Some(crate::block::entities::sign::build_sign_actor(
            self.position,
            true,
            &self.front_text,
            &self.back_text,
            self.is_waxed.load(Ordering::Relaxed),
        ))
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        crate::block::entities::sign::collect_sign_item_components(
            &self.front_text,
            &self.back_text,
            &self.is_waxed,
            stack,
        );
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
        crate::block::entities::sign::apply_sign_item_components(
            &self.front_text,
            &self.back_text,
            &self.is_waxed,
            stack,
        );
        // Item data must not be able to enable op-level click commands.
        self.allow_op_features.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl HangingSignBlockEntity {
    pub const ID: &'static str = "minecraft:hanging_sign";
    #[must_use]
    pub fn new(position: BlockPos, is_front: bool, messages: [Box<str>; 4]) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            allow_op_features: AtomicBool::new(false),
            front_text: if is_front {
                Text::from_messages(messages.clone())
            } else {
                Text::default()
            },
            back_text: if is_front {
                Text::default()
            } else {
                Text::from_messages(messages)
            },
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
    #[must_use]
    pub fn empty(position: BlockPos) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            allow_op_features: AtomicBool::new(false),
            front_text: Text::default(),
            back_text: Text::default(),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
}
